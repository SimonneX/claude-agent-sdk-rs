//! Full Query implementation with bidirectional control protocol

use dashmap::DashMap;
use futures::stream::StreamExt;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::oneshot;

use crate::errors::{ClaudeError, Result};
use crate::types::hooks::{HookCallback, HookContext, HookInput, HookMatcher};
use crate::types::mcp::McpSdkServerConfig;

use super::transport::Transport;

/// Control request from SDK to CLI
#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
struct ControlRequest {
    #[serde(rename = "type")]
    type_: String,
    request_id: String,
    request: serde_json::Value,
}

/// Control response from CLI to SDK
#[derive(Debug, serde::Deserialize)]
struct ControlResponse {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    type_: String,
    response: ControlResponseData,
}

#[derive(Debug, serde::Deserialize)]
struct ControlResponseData {
    #[allow(dead_code)]
    subtype: String,
    request_id: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

/// Control request from CLI to SDK
#[derive(Debug, serde::Deserialize)]
struct IncomingControlRequest {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    type_: String,
    request_id: String,
    request: serde_json::Value,
}

/// Full Query implementation with bidirectional control protocol
pub struct QueryFull {
    /// Transport for communication - uses &self methods via internal sync
    pub(crate) transport: Arc<dyn Transport>,
    /// Hook callbacks - concurrent access via DashMap
    hook_callbacks: Arc<DashMap<String, HookCallback>>,
    /// SDK MCP servers - concurrent access via DashMap
    sdk_mcp_servers: Arc<DashMap<String, McpSdkServerConfig>>,
    next_callback_id: Arc<AtomicU64>,
    request_counter: Arc<AtomicU64>,
    /// Pending control request responses - concurrent access via DashMap
    pending_responses: Arc<DashMap<String, oneshot::Sender<serde_json::Value>>>,
    message_tx: flume::Sender<serde_json::Value>,
    /// Message receiver - cloneable without mutex thanks to flume
    pub(crate) message_rx: flume::Receiver<serde_json::Value>,
    /// Initialization result - set once during initialize(), read many times
    initialization_result: OnceLock<serde_json::Value>,
}

impl QueryFull {
    /// Create a new Query
    pub fn new(transport: Box<dyn Transport>) -> Self {
        let (message_tx, message_rx) = flume::unbounded();

        Self {
            transport: Arc::from(transport),
            hook_callbacks: Arc::new(DashMap::new()),
            sdk_mcp_servers: Arc::new(DashMap::new()),
            next_callback_id: Arc::new(AtomicU64::new(0)),
            request_counter: Arc::new(AtomicU64::new(0)),
            pending_responses: Arc::new(DashMap::new()),
            message_tx,
            message_rx,
            initialization_result: OnceLock::new(),
        }
    }

    /// Create a new Query with a pre-existing Arc transport (for testing)
    #[cfg(feature = "testing")]
    pub fn new_with_transport(transport: Arc<dyn Transport>) -> Self {
        let (message_tx, message_rx) = flume::unbounded();

        Self {
            transport,
            hook_callbacks: Arc::new(DashMap::new()),
            sdk_mcp_servers: Arc::new(DashMap::new()),
            next_callback_id: Arc::new(AtomicU64::new(0)),
            request_counter: Arc::new(AtomicU64::new(0)),
            pending_responses: Arc::new(DashMap::new()),
            message_tx,
            message_rx,
            initialization_result: OnceLock::new(),
        }
    }

    /// Set SDK MCP servers
    pub fn set_sdk_mcp_servers(&mut self, servers: HashMap<String, McpSdkServerConfig>) {
        self.sdk_mcp_servers.clear();
        for (name, config) in servers {
            self.sdk_mcp_servers.insert(name, config);
        }
    }

    /// Initialize with hooks (no preset-prompt extras). Thin wrapper around
    /// [`Self::initialize_with`]; kept for API compatibility and used by
    /// existing tests.
    #[allow(dead_code)]
    pub async fn initialize(
        &self,
        hooks: Option<HashMap<String, Vec<HookMatcher>>>,
    ) -> Result<serde_json::Value> {
        self.initialize_with(hooks, None).await
    }

    /// Initialize with hooks plus optional preset-prompt knobs that travel
    /// inside the initialize control request rather than the CLI argv.
    /// Currently the only such knob is `exclude_dynamic_sections`, mirroring
    /// Python `_internal/query.py:initialize` which forwards it as
    /// `excludeDynamicSections`.
    pub async fn initialize_with(
        &self,
        hooks: Option<HashMap<String, Vec<HookMatcher>>>,
        exclude_dynamic_sections: Option<bool>,
    ) -> Result<serde_json::Value> {
        // Build hooks configuration
        let mut hooks_config: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

        if let Some(hooks_map) = hooks {
            for (event, matchers) in hooks_map {
                let mut event_matchers = Vec::new();

                for matcher in matchers {
                    let mut callback_ids = Vec::new();

                    for callback in matcher.hooks {
                        let callback_id = format!(
                            "hook_{}",
                            self.next_callback_id.fetch_add(1, Ordering::SeqCst)
                        );
                        self.hook_callbacks.insert(callback_id.clone(), callback);
                        callback_ids.push(callback_id);
                    }

                    let mut matcher_json = json!({
                        "matcher": matcher.matcher,
                        "hookCallbackIds": callback_ids
                    });

                    // Add timeout if specified
                    if let Some(timeout) = matcher.timeout {
                        matcher_json["timeout"] = json!(timeout);
                    }

                    event_matchers.push(matcher_json);
                }

                hooks_config.insert(event, event_matchers);
            }
        }

        // Send initialize request
        let mut request = json!({
            "subtype": "initialize",
            "hooks": if hooks_config.is_empty() { json!(null) } else { json!(hooks_config) }
        });

        if let Some(eds) = exclude_dynamic_sections {
            request["excludeDynamicSections"] = json!(eds);
        }

        let response = self.send_control_request(request).await?;

        // Store initialization result for get_server_info() (set once, read many)
        let _ = self.initialization_result.set(response.clone());

        Ok(response)
    }

    /// Start reading messages in background
    ///
    /// Returns a receiver that signals when the background task completes.
    /// The caller should store this and await it during disconnect.
    pub async fn start(&self) -> Result<oneshot::Receiver<()>> {
        let transport = Arc::clone(&self.transport);
        let transport_for_hooks = Arc::clone(&self.transport);
        let hook_callbacks = Arc::clone(&self.hook_callbacks);
        let sdk_mcp_servers = Arc::clone(&self.sdk_mcp_servers);
        let pending_responses = Arc::clone(&self.pending_responses);
        let message_tx = self.message_tx.clone();

        // Create a channel to signal when background task is ready
        let (ready_tx, ready_rx) = oneshot::channel();

        // Create a channel to signal when background task completes
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(async move {
            // No lock needed - Transport uses &self methods with internal sync
            let mut stream = transport.read_messages();

            // Signal that we're ready to receive messages
            let _ = ready_tx.send(());

            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => {
                        let msg_type = message.get("type").and_then(|v| v.as_str());

                        match msg_type {
                            Some("control_response") => {
                                // Handle control response.
                                //
                                // Wire format from CLI (see Python `_internal/query.py`
                                // `_send_control_request` and `tools/capture_control_protocol.py`):
                                //   {"type":"control_response",
                                //    "response":{"subtype":"success","request_id":"...","response":<actual>}}
                                // The inner `.response` holds the real payload that callers want
                                // (e.g. McpStatusResponse, ContextUsageResponse, the initialize
                                // result). Strip it once here so every caller receives the
                                // unwrapped value, matching Python's behavior.
                                if let Ok(response) =
                                    serde_json::from_value::<ControlResponse>(message.clone())
                                {
                                    // DashMap remove returns Option<(K, V)>
                                    if let Some((_, tx)) =
                                        pending_responses.remove(&response.response.request_id)
                                    {
                                        let inner = response
                                            .response
                                            .data
                                            .get("response")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null);
                                        let _ = tx.send(inner);
                                    }
                                }
                            }
                            Some("control_request") => {
                                // Handle incoming control request (e.g., hook callback, MCP message)
                                if let Ok(request) = serde_json::from_value::<IncomingControlRequest>(
                                    message.clone(),
                                ) {
                                    let transport_clone = Arc::clone(&transport_for_hooks);
                                    let hook_callbacks_clone = Arc::clone(&hook_callbacks);
                                    let sdk_mcp_servers_clone = Arc::clone(&sdk_mcp_servers);

                                    tokio::spawn(async move {
                                        if let Err(e) = Self::handle_control_request(
                                            request,
                                            transport_clone,
                                            hook_callbacks_clone,
                                            sdk_mcp_servers_clone,
                                        )
                                        .await
                                        {
                                            eprintln!("Error handling control request: {}", e);
                                        }
                                    });
                                }
                            }
                            _ => {
                                // Regular message - send to stream
                                let _ = message_tx.send(message);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }

            // Signal that background task has completed
            let _ = shutdown_tx.send(());
        });

        // Wait for background task to be ready before returning
        ready_rx
            .await
            .map_err(|_| ClaudeError::Transport("Background task failed to start".to_string()))?;

        Ok(shutdown_rx)
    }

    /// Handle incoming control request from CLI
    async fn handle_control_request(
        request: IncomingControlRequest,
        transport: Arc<dyn Transport>,
        hook_callbacks: Arc<DashMap<String, HookCallback>>,
        sdk_mcp_servers: Arc<DashMap<String, McpSdkServerConfig>>,
    ) -> Result<()> {
        let request_id = request.request_id;
        let request_data = request.request;

        let subtype = request_data
            .get("subtype")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClaudeError::ControlProtocol("Missing subtype".to_string()))?;

        let response_data: serde_json::Value = match subtype {
            "hook_callback" => {
                // Execute hook callback
                let callback_id = request_data
                    .get("callback_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ClaudeError::ControlProtocol("Missing callback_id".to_string())
                    })?;

                // Clone the callback Arc to release the DashMap guard before async call
                let callback = hook_callbacks
                    .get(callback_id)
                    .map(|r| r.clone())
                    .ok_or_else(|| {
                        ClaudeError::ControlProtocol(format!(
                            "Hook callback not found: {}",
                            callback_id
                        ))
                    })?;

                // Parse hook input
                let input_json = request_data.get("input").cloned().unwrap_or(json!({}));
                let hook_input: HookInput = serde_json::from_value(input_json).map_err(|e| {
                    ClaudeError::ControlProtocol(format!("Failed to parse hook input: {}", e))
                })?;

                let tool_use_id = request_data
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let context = HookContext::default();

                // Call the hook
                let hook_output = callback(hook_input, tool_use_id, context).await;

                // Convert to JSON
                serde_json::to_value(&hook_output).map_err(|e| {
                    ClaudeError::ControlProtocol(format!("Failed to serialize hook output: {}", e))
                })?
            }
            "mcp_message" => {
                // Handle SDK MCP message
                let server_name = request_data
                    .get("server_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ClaudeError::ControlProtocol(
                            "Missing server_name for mcp_message".to_string(),
                        )
                    })?;

                let mcp_message = request_data.get("message").ok_or_else(|| {
                    ClaudeError::ControlProtocol("Missing message for mcp_message".to_string())
                })?;

                let mcp_response =
                    Self::handle_sdk_mcp_request(sdk_mcp_servers, server_name, mcp_message.clone())
                        .await?;

                json!({"mcp_response": mcp_response})
            }
            _ => {
                return Err(ClaudeError::ControlProtocol(format!(
                    "Unsupported control request subtype: {}",
                    subtype
                )));
            }
        };

        // Send success response
        let response = json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": request_id,
                "response": response_data
            }
        });

        let response_str = serde_json::to_string(&response)
            .map_err(|e| ClaudeError::Transport(format!("Failed to serialize response: {}", e)))?;

        // Write via transport - stdin/stdout have separate locks, no deadlock
        transport.write(&response_str).await?;

        Ok(())
    }

    /// Send control request to CLI
    async fn send_control_request(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        let request_id = format!(
            "req_{}_{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst),
            uuid::Uuid::new_v4().simple()
        );

        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();
        self.pending_responses.insert(request_id.clone(), tx);

        // Build and send request
        let control_request = json!({
            "type": "control_request",
            "request_id": request_id,
            "request": request
        });

        let request_str = serde_json::to_string(&control_request)
            .map_err(|e| ClaudeError::Transport(format!("Failed to serialize request: {}", e)))?;

        // Write via transport - stdin/stdout have separate locks, no deadlock
        self.transport.write(&request_str).await?;

        // Wait for response
        let response = rx.await.map_err(|_| {
            ClaudeError::ControlProtocol("Control request response channel closed".to_string())
        })?;

        Ok(response)
    }

    /// Receive messages
    #[allow(dead_code)]
    pub async fn receive_messages(&self) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();
        let rx = self.message_rx.clone();

        while let Ok(message) = rx.recv_async().await {
            messages.push(message);
        }

        messages
    }

    /// Send interrupt signal to Claude
    pub async fn interrupt(&self) -> Result<()> {
        let request = json!({
            "subtype": "interrupt"
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Change permission mode dynamically
    pub async fn set_permission_mode(
        &self,
        mode: crate::types::config::PermissionMode,
    ) -> Result<()> {
        let mode_str = match mode {
            crate::types::config::PermissionMode::Default => "default",
            crate::types::config::PermissionMode::AcceptEdits => "acceptEdits",
            crate::types::config::PermissionMode::Plan => "plan",
            crate::types::config::PermissionMode::BypassPermissions => "bypassPermissions",
            crate::types::config::PermissionMode::DontAsk => "dontAsk",
            crate::types::config::PermissionMode::Auto => "auto",
        };

        let request = json!({
            "subtype": "set_permission_mode",
            "mode": mode_str
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Change AI model dynamically
    pub async fn set_model(&self, model: Option<&str>) -> Result<()> {
        let request = json!({
            "subtype": "set_model",
            "model": model
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Rewind tracked files to their state at a specific user message.
    ///
    /// Requires:
    /// - `enable_file_checkpointing=true` to track file changes
    /// - `extra_args={"replay-user-messages": None}` to receive UserMessage
    ///   objects with `uuid` in the response stream
    ///
    /// # Arguments
    /// * `user_message_id` - UUID of the user message to rewind to. This should be
    ///   the `uuid` field from a `UserMessage` received during the conversation.
    pub async fn rewind_files(&self, user_message_id: &str) -> Result<()> {
        let request = json!({
            "subtype": "rewind_files",
            "user_message_id": user_message_id
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Get server initialization info
    ///
    /// Returns the initialization result that was obtained during connect().
    /// This includes information about available commands, output styles, and server capabilities.
    /// This is lock-free since initialization_result uses OnceLock.
    pub fn get_initialization_result(&self) -> Option<serde_json::Value> {
        self.initialization_result.get().cloned()
    }

    /// Handle SDK MCP request by routing to the appropriate server
    async fn handle_sdk_mcp_request(
        sdk_mcp_servers: Arc<DashMap<String, McpSdkServerConfig>>,
        server_name: &str,
        message: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Clone the server config to release the DashMap guard before async call
        let server_config = sdk_mcp_servers
            .get(server_name)
            .map(|r| r.clone())
            .ok_or_else(|| {
                ClaudeError::ControlProtocol(format!("SDK MCP server not found: {}", server_name))
            })?;

        // Call the server's handle_message method
        server_config
            .instance
            .handle_message(message)
            .await
            .map_err(|e| ClaudeError::ControlProtocol(format!("MCP server error: {}", e)))
    }

    /// Get MCP server status for all configured servers
    pub async fn get_mcp_status(&self) -> Result<crate::types::mcp::McpStatusResponse> {
        let request = json!({
            "subtype": "mcp_status"
        });

        let response = self.send_control_request(request).await?;

        // Parse the response into McpStatusResponse
        let status_response: crate::types::mcp::McpStatusResponse =
            serde_json::from_value(response).map_err(|e| {
                ClaudeError::ControlProtocol(format!("Failed to parse MCP status response: {}", e))
            })?;

        Ok(status_response)
    }

    /// Reconnect a failed MCP server
    pub async fn reconnect_mcp_server(&self, server_name: &str) -> Result<()> {
        let request = json!({
            "subtype": "mcp_reconnect",
            "serverName": server_name
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Toggle an MCP server (enable/disable)
    pub async fn toggle_mcp_server(&self, server_name: &str, enabled: bool) -> Result<()> {
        let request = json!({
            "subtype": "mcp_toggle",
            "serverName": server_name,
            "enabled": enabled
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Stop a running task
    pub async fn stop_task(&self, task_id: &str) -> Result<()> {
        let request = json!({
            "subtype": "stop_task",
            "task_id": task_id
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Get context usage information
    pub async fn get_context_usage(
        &self,
    ) -> Result<crate::types::context::ContextUsageResponse> {
        let request = json!({
            "subtype": "context_usage"
        });

        let response = self.send_control_request(request).await?;

        // Parse the response into ContextUsageResponse
        let context_response: crate::types::context::ContextUsageResponse =
            serde_json::from_value(response).map_err(|e| {
                ClaudeError::ControlProtocol(format!(
                    "Failed to parse context usage response: {}",
                    e
                ))
            })?;

        Ok(context_response)
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::MockTransport;
    use crate::types::config::PermissionMode;
    use crate::types::hooks::{
        HookCallback, HookInput, HookJsonOutput, HookMatcher, SyncHookJsonOutput,
    };
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::sync::Notify;

    /// Wait up to `timeout` for the mock to capture at least `n` writes.
    async fn wait_for_writes(mock: &MockTransport, n: usize, timeout: Duration) -> Vec<String> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let writes: Vec<String> = mock
                .written_messages_async()
                .await
                .into_iter()
                .map(|w| w.data)
                .collect();
            if writes.len() >= n {
                return writes;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("timed out waiting for {} write(s); have {}", n, writes.len());
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    fn extract_request_id(written: &str) -> String {
        let v: serde_json::Value = serde_json::from_str(written).expect("written json");
        v["request_id"]
            .as_str()
            .expect("request_id missing")
            .to_string()
    }

    /// Build a CLI-shaped successful control_response envelope.
    ///
    /// Wire format mirrors what the Python tool capture
    /// (`tools/capture_control_protocol.py`) records: the actual payload sits
    /// under a nested `response` field, NOT flattened at the top level.
    fn success_response(request_id: &str, payload: serde_json::Value) -> serde_json::Value {
        json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": request_id,
                "response": payload,
            },
        })
    }

    async fn setup() -> (Arc<MockTransport>, Arc<QueryFull>, oneshot::Receiver<()>) {
        let mock = Arc::new(MockTransport::builder().build());
        mock.connect().await.unwrap();
        let query = Arc::new(QueryFull::new_with_transport(
            Arc::clone(&mock) as Arc<dyn Transport>
        ));
        let shutdown = query.start().await.unwrap();
        (mock, query, shutdown)
    }

    // ==================== P0-2: Control protocol ====================

    #[tokio::test]
    async fn interrupt_round_trip() {
        let (mock, query, _shutdown) = setup().await;

        let q = Arc::clone(&query);
        let handle = tokio::spawn(async move { q.interrupt().await });

        let writes = wait_for_writes(&mock, 1, Duration::from_secs(2)).await;
        let req_id = extract_request_id(&writes[0]);
        mock.inject(success_response(&req_id, json!({})));

        handle.await.unwrap().unwrap();
        mock.close().await.unwrap();
    }

    #[tokio::test]
    async fn out_of_order_control_responses_are_matched_by_request_id() {
        let (mock, query, _shutdown) = setup().await;

        // Fire two concurrent control requests
        let q1 = Arc::clone(&query);
        let q2 = Arc::clone(&query);
        let h1 = tokio::spawn(async move { q1.interrupt().await });
        let h2 = tokio::spawn(async move { q2.set_permission_mode(PermissionMode::Plan).await });

        let writes = wait_for_writes(&mock, 2, Duration::from_secs(2)).await;
        let id1 = extract_request_id(&writes[0]);
        let id2 = extract_request_id(&writes[1]);
        assert_ne!(id1, id2, "request ids must be unique");

        // Reply in reverse order
        mock.inject(success_response(&id2, json!({})));
        mock.inject(success_response(&id1, json!({})));

        h1.await.unwrap().unwrap();
        h2.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn orphan_control_response_is_ignored_and_pipeline_keeps_running() {
        let (mock, query, _shutdown) = setup().await;

        // Inject a response for a nonexistent request — must not panic
        mock.inject(success_response("req_does_not_exist", json!({})));

        // Then a regular message — should arrive on message_rx
        mock.inject(json!({"type": "system", "subtype": "init"}));

        // Receive the regular message via the public API
        let rx = query.message_rx.clone();
        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv_async())
            .await
            .expect("timeout waiting for message")
            .expect("channel closed");
        assert_eq!(msg["type"], "system");

        mock.close().await.unwrap();
    }

    #[tokio::test]
    async fn request_ids_are_unique_under_concurrency() {
        let (mock, query, _shutdown) = setup().await;

        let n = 20usize;
        let mut handles = Vec::with_capacity(n);
        for _ in 0..n {
            let q = Arc::clone(&query);
            handles.push(tokio::spawn(async move { q.interrupt().await }));
        }

        let writes = wait_for_writes(&mock, n, Duration::from_secs(3)).await;
        let mut ids: Vec<String> = writes.iter().map(|w| extract_request_id(w)).collect();
        ids.sort();
        let before = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), before, "duplicate request_id detected");

        // Reply to each so handles can complete
        for id in &ids {
            mock.inject(success_response(id, json!({})));
        }
        for h in handles {
            h.await.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn closing_transport_terminates_background_task() {
        let (mock, _query, shutdown) = setup().await;

        // Closing causes read_messages to break out of its loop
        mock.close().await.unwrap();

        tokio::time::timeout(Duration::from_secs(2), shutdown)
            .await
            .expect("background task did not shut down")
            .expect("shutdown channel dropped");
    }

    #[tokio::test]
    async fn regular_message_routes_to_message_rx() {
        let (mock, query, _shutdown) = setup().await;

        mock.inject(json!({
            "type": "assistant",
            "message": {"content": []}
        }));

        let rx = query.message_rx.clone();
        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv_async())
            .await
            .expect("timeout")
            .expect("closed");
        assert_eq!(msg["type"], "assistant");
    }

    #[tokio::test]
    async fn initialization_result_is_set_after_initialize() {
        let (mock, query, _shutdown) = setup().await;

        // initialize() sends a control_request with subtype=initialize
        let q = Arc::clone(&query);
        let handle = tokio::spawn(async move { q.initialize(None).await });

        let writes = wait_for_writes(&mock, 1, Duration::from_secs(2)).await;
        let req_id = extract_request_id(&writes[0]);
        mock.inject(success_response(
            &req_id,
            json!({"commands": ["a", "b"], "outputStyles": []}),
        ));

        let result = handle.await.unwrap().unwrap();
        assert_eq!(result["commands"][0], "a");

        // OnceLock is now populated
        let cached = query.get_initialization_result().expect("cached");
        assert_eq!(cached["commands"][1], "b");
    }

    // ==================== P0-3: Hook callback dispatch ====================

    fn make_hook_matcher(notify: Arc<Notify>, captured: Arc<Mutex<Option<HookInput>>>) -> HookMatcher {
        let cb: HookCallback = Arc::new(move |input, _tool_use_id, _ctx| {
            let notify = Arc::clone(&notify);
            let captured = Arc::clone(&captured);
            Box::pin(async move {
                *captured.lock().await = Some(input);
                notify.notify_one();
                HookJsonOutput::Sync(SyncHookJsonOutput {
                    continue_: Some(true),
                    ..Default::default()
                })
            })
        });
        HookMatcher {
            matcher: Some("Bash".to_string()),
            hooks: vec![cb],
            timeout: None,
        }
    }

    use tokio::sync::Mutex;

    async fn install_hook_and_get_callback_id(
        mock: &MockTransport,
        query: &Arc<QueryFull>,
        matcher: HookMatcher,
    ) -> String {
        let mut hooks = HashMap::new();
        hooks.insert("PreToolUse".to_string(), vec![matcher]);

        // Drive initialize() so callbacks register and we can find their ID
        let q = Arc::clone(query);
        let handle = tokio::spawn(async move { q.initialize(Some(hooks)).await });

        let writes = wait_for_writes(mock, 1, Duration::from_secs(2)).await;
        let init_req: serde_json::Value = serde_json::from_str(&writes[0]).unwrap();
        let req_id = init_req["request_id"].as_str().unwrap().to_string();

        // Pull the registered callback id out of the request payload
        let callback_id = init_req["request"]["hooks"]["PreToolUse"][0]["hookCallbackIds"][0]
            .as_str()
            .expect("callback id present in initialize request")
            .to_string();

        mock.inject(success_response(&req_id, json!({})));
        handle.await.unwrap().unwrap();

        callback_id
    }

    #[tokio::test]
    async fn hook_callback_is_invoked_when_cli_dispatches() {
        let (mock, query, _shutdown) = setup().await;
        let notify = Arc::new(Notify::new());
        let captured: Arc<Mutex<Option<HookInput>>> = Arc::new(Mutex::new(None));
        let matcher = make_hook_matcher(Arc::clone(&notify), Arc::clone(&captured));
        let callback_id = install_hook_and_get_callback_id(&mock, &query, matcher).await;

        // CLI -> SDK control_request to fire the hook
        let cli_req_id = "req_from_cli_1";
        mock.inject(json!({
            "type": "control_request",
            "request_id": cli_req_id,
            "request": {
                "subtype": "hook_callback",
                "callback_id": callback_id,
                "tool_use_id": "tool_1",
                "input": {
                    "hook_event_name": "PreToolUse",
                    "session_id": "s1",
                    "transcript_path": "/tmp/x",
                    "cwd": "/tmp",
                    "tool_name": "Bash",
                    "tool_input": {"command": "echo hi"}
                }
            }
        }));

        // Wait for the callback to fire
        tokio::time::timeout(Duration::from_secs(2), notify.notified())
            .await
            .expect("callback never fired");

        let captured = captured.lock().await;
        match captured.as_ref().expect("input not captured") {
            HookInput::PreToolUse(p) => {
                assert_eq!(p.tool_name, "Bash");
                assert_eq!(p.tool_input["command"], "echo hi");
            }
            other => panic!("wrong input variant: {:?}", other),
        }

        // SDK should have written a control_response back to the CLI.
        // Skip the initialize write (index 0); look for the hook response.
        let writes = wait_for_writes(&mock, 2, Duration::from_secs(2)).await;
        let response_write = writes
            .iter()
            .find(|w| w.contains("control_response") && w.contains(cli_req_id))
            .expect("response not written");
        let v: serde_json::Value = serde_json::from_str(response_write).unwrap();
        assert_eq!(v["response"]["subtype"], "success");
        assert_eq!(v["response"]["request_id"], cli_req_id);
        assert_eq!(v["response"]["response"]["continue"], true);
    }

    #[tokio::test]
    async fn unknown_hook_callback_id_is_handled_without_panic() {
        let (mock, query, _shutdown) = setup().await;

        // Inject a hook_callback control_request with a callback_id that was never registered
        mock.inject(json!({
            "type": "control_request",
            "request_id": "req_bogus_1",
            "request": {
                "subtype": "hook_callback",
                "callback_id": "hook_does_not_exist",
                "input": {
                    "hook_event_name": "Stop",
                    "session_id": "s1",
                    "transcript_path": "/tmp/x",
                    "cwd": "/tmp",
                    "stop_hook_active": false
                }
            }
        }));

        // Pipeline should still process normal traffic afterwards
        mock.inject(json!({"type": "system", "subtype": "init"}));
        let rx = query.message_rx.clone();
        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv_async())
            .await
            .expect("timeout")
            .expect("channel closed");
        assert_eq!(msg["type"], "system");
    }

    #[tokio::test]
    async fn missing_subtype_in_control_request_is_handled() {
        let (mock, query, _shutdown) = setup().await;

        mock.inject(json!({
            "type": "control_request",
            "request_id": "req_no_subtype",
            "request": {"some": "garbage"}
        }));

        // Pipeline keeps working
        mock.inject(json!({"type": "system", "subtype": "init"}));
        let rx = query.message_rx.clone();
        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv_async())
            .await
            .expect("timeout")
            .expect("channel closed");
        assert_eq!(msg["type"], "system");
    }

    #[tokio::test]
    async fn mcp_message_for_unknown_server_is_handled() {
        let (mock, query, _shutdown) = setup().await;

        mock.inject(json!({
            "type": "control_request",
            "request_id": "req_mcp_unknown",
            "request": {
                "subtype": "mcp_message",
                "server_name": "ghost_server",
                "message": {"jsonrpc": "2.0", "method": "tools/list", "id": 1}
            }
        }));

        // Pipeline still processes regular messages
        mock.inject(json!({"type": "system", "subtype": "init"}));
        let rx = query.message_rx.clone();
        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv_async())
            .await
            .expect("timeout")
            .expect("channel closed");
        assert_eq!(msg["type"], "system");
    }
}
