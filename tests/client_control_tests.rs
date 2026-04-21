//! P2: ClaudeClient dynamic control method tests.
//!
//! Drive each control method through the bidirectional protocol with
//! `MockTransport`: capture the outgoing control_request, inject a matching
//! control_response, and assert the call resolves correctly.

use claude_agent_sdk_rs::testing::{MockTransport, SystemMessageBuilder, Transport};
use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ClaudeError, PermissionMode, UserContentBlock,
    query_stream_with_content, query_with_content,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

// ============================================================================
// Helpers
// ============================================================================

/// Build a connected ClaudeClient backed by a MockTransport. The mock is
/// returned alongside so tests can inspect writes and inject responses.
async fn make_connected() -> (ClaudeClient, Arc<MockTransport>) {
    // Pre-seed an on_connect system message so receive_messages() has something
    // to drain if a test asks for it (most don't).
    let _ = SystemMessageBuilder::default();
    let mock = Arc::new(MockTransport::builder().build());
    let mut client = ClaudeClient::with_transport(
        Arc::clone(&mock) as Arc<dyn claude_agent_sdk_rs::testing::Transport>,
        ClaudeAgentOptions::default(),
    );
    client.connect_with_transport().await.expect("connect ok");
    (client, mock)
}

/// Wait for `n` writes on the mock, then return their raw payloads.
async fn wait_for_writes(mock: &MockTransport, n: usize) -> Vec<String> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
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
            panic!("timeout waiting for {n} write(s); have {}", writes.len());
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

fn parse(written: &str) -> serde_json::Value {
    serde_json::from_str(written).expect("written is JSON")
}

fn request_id_of(written: &str) -> String {
    parse(written)["request_id"]
        .as_str()
        .expect("request_id present")
        .to_string()
}

/// Build a `control_response` envelope. `extra` is merged into the response
/// alongside `subtype` and `request_id` (these are extracted by the SDK and
/// the rest is delivered to the awaiting caller via `#[serde(flatten)]`).
fn success_response(request_id: &str, extra: serde_json::Value) -> serde_json::Value {
    let mut response = json!({
        "subtype": "success",
        "request_id": request_id,
    });
    if let Some(map) = extra.as_object() {
        for (k, v) in map {
            response[k] = v.clone();
        }
    }
    json!({"type": "control_response", "response": response})
}

/// Drive a control-method round-trip: spawn the call, wait for the
/// outgoing write, inject the response, and return the parsed write.
async fn round_trip<F, Fut, T>(
    mock: Arc<MockTransport>,
    call: F,
    response_extra: serde_json::Value,
) -> (serde_json::Value, T)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let mock_for_inject = Arc::clone(&mock);
    let initial_writes = mock.written_messages_async().await.len();
    let handle = tokio::spawn(call());

    // Wait for one *additional* write on top of any prior traffic.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    let new_write = loop {
        let writes = mock.written_messages_async().await;
        if writes.len() > initial_writes {
            break writes[initial_writes].data.clone();
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("timeout waiting for control request write");
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    };

    let req_id = request_id_of(&new_write);
    mock_for_inject.inject(success_response(&req_id, response_extra));

    let result = timeout(Duration::from_secs(2), handle)
        .await
        .expect("call timed out")
        .expect("task panicked");

    (parse(&new_write), result)
}

// ============================================================================
// interrupt
// ============================================================================

#[tokio::test]
async fn interrupt_sends_control_request() {
    let (client, mock) = make_connected().await;
    let (req, result) =
        round_trip(Arc::clone(&mock), || async move { client.interrupt().await }, json!({})).await;
    result.expect("interrupt ok");
    assert_eq!(req["type"], "control_request");
    assert_eq!(req["request"]["subtype"], "interrupt");
    let _ = mock.close().await;
}

// ============================================================================
// set_permission_mode (all 6 modes)
// ============================================================================

#[tokio::test]
async fn set_permission_mode_emits_correct_mode_string() {
    let cases = [
        (PermissionMode::Default, "default"),
        (PermissionMode::AcceptEdits, "acceptEdits"),
        (PermissionMode::Plan, "plan"),
        (PermissionMode::BypassPermissions, "bypassPermissions"),
        (PermissionMode::DontAsk, "dontAsk"),
        (PermissionMode::Auto, "auto"),
    ];
    for (mode, expected) in cases {
        let (client, mock) = make_connected().await;
        let (req, result) = round_trip(
            Arc::clone(&mock),
            || async move { client.set_permission_mode(mode).await },
            json!({}),
        )
        .await;
        result.expect("set_permission_mode ok");
        assert_eq!(req["request"]["subtype"], "set_permission_mode");
        assert_eq!(
            req["request"]["mode"], expected,
            "mode {mode:?} → {expected}"
        );
        let _ = mock.close().await;
    }
}

// ============================================================================
// set_model
// ============================================================================

#[tokio::test]
async fn set_model_with_explicit_name() {
    let (client, mock) = make_connected().await;
    let (req, result) = round_trip(
        Arc::clone(&mock),
        || async move { client.set_model(Some("claude-opus-4")).await },
        json!({}),
    )
    .await;
    result.unwrap();
    assert_eq!(req["request"]["subtype"], "set_model");
    assert_eq!(req["request"]["model"], "claude-opus-4");
    let _ = mock.close().await;
}

#[tokio::test]
async fn set_model_with_none_clears_to_default() {
    let (client, mock) = make_connected().await;
    let (req, result) = round_trip(
        Arc::clone(&mock),
        || async move { client.set_model(None).await },
        json!({}),
    )
    .await;
    result.unwrap();
    assert_eq!(req["request"]["model"], serde_json::Value::Null);
    let _ = mock.close().await;
}

// ============================================================================
// rewind_files
// ============================================================================

#[tokio::test]
async fn rewind_files_passes_uuid() {
    let (client, mock) = make_connected().await;
    let (req, result) = round_trip(
        Arc::clone(&mock),
        || async move { client.rewind_files("uuid-checkpoint-1").await },
        json!({}),
    )
    .await;
    result.unwrap();
    assert_eq!(req["request"]["subtype"], "rewind_files");
    assert_eq!(req["request"]["user_message_id"], "uuid-checkpoint-1");
    let _ = mock.close().await;
}

// ============================================================================
// MCP control methods
// ============================================================================

#[tokio::test]
async fn get_mcp_status_parses_response() {
    let (client, mock) = make_connected().await;
    let payload = json!({
        "mcpServers": [
            {
                "name": "fs",
                "status": "connected",
                "serverInfo": {"name": "fs-server", "version": "1.0.0"}
            },
            {
                "name": "broken",
                "status": "failed",
                "error": "connection refused"
            }
        ]
    });
    let (req, result) = round_trip(
        Arc::clone(&mock),
        || async move { client.get_mcp_status().await },
        payload,
    )
    .await;
    let status = result.expect("get_mcp_status ok");
    assert_eq!(req["request"]["subtype"], "mcp_status");
    assert_eq!(status.mcp_servers.len(), 2);
    assert_eq!(status.mcp_servers[0].name, "fs");
    assert_eq!(status.mcp_servers[1].error.as_deref(), Some("connection refused"));
    let _ = mock.close().await;
}

#[tokio::test]
async fn reconnect_mcp_server_passes_server_name() {
    let (client, mock) = make_connected().await;
    let (req, result) = round_trip(
        Arc::clone(&mock),
        || async move { client.reconnect_mcp_server("fs").await },
        json!({}),
    )
    .await;
    result.unwrap();
    assert_eq!(req["request"]["subtype"], "mcp_reconnect");
    assert_eq!(req["request"]["serverName"], "fs");
    let _ = mock.close().await;
}

#[tokio::test]
async fn toggle_mcp_server_emits_enabled_flag() {
    for enabled in [true, false] {
        let (client, mock) = make_connected().await;
        let (req, result) = round_trip(
            Arc::clone(&mock),
            || async move { client.toggle_mcp_server("fs", enabled).await },
            json!({}),
        )
        .await;
        result.unwrap();
        assert_eq!(req["request"]["subtype"], "mcp_toggle");
        assert_eq!(req["request"]["serverName"], "fs");
        assert_eq!(req["request"]["enabled"], enabled);
        let _ = mock.close().await;
    }
}

// ============================================================================
// stop_task / get_context_usage
// ============================================================================

#[tokio::test]
async fn stop_task_passes_task_id() {
    let (client, mock) = make_connected().await;
    let (req, result) = round_trip(
        Arc::clone(&mock),
        || async move { client.stop_task("task-42").await },
        json!({}),
    )
    .await;
    result.unwrap();
    assert_eq!(req["request"]["subtype"], "stop_task");
    assert_eq!(req["request"]["task_id"], "task-42");
    let _ = mock.close().await;
}

#[tokio::test]
async fn get_context_usage_parses_response() {
    let (client, mock) = make_connected().await;
    let payload = json!({
        "categories": [
            {"name": "system_prompt", "tokens": 1200},
            {"name": "history", "tokens": 800}
        ],
        "totalTokens": 2000,
        "maxTokens": 200000,
        "percentage": 1.0,
        "model": "claude-sonnet-4",
        "isAutoCompactEnabled": true
    });
    let (req, result) = round_trip(
        Arc::clone(&mock),
        || async move { client.get_context_usage().await },
        payload,
    )
    .await;
    let usage = result.expect("context_usage ok");
    assert_eq!(req["request"]["subtype"], "context_usage");
    assert_eq!(usage.total_tokens, 2000);
    assert_eq!(usage.max_tokens, 200_000);
    assert_eq!(usage.model, "claude-sonnet-4");
    assert!(usage.is_auto_compact_enabled);
    assert_eq!(usage.categories.len(), 2);
    let _ = mock.close().await;
}

// ============================================================================
// Pre-connect guards: every control method should fail with InvalidConfig
// before connect_with_transport() runs.
// ============================================================================

fn assert_invalid_config(err: ClaudeError) {
    match err {
        ClaudeError::InvalidConfig(msg) => assert!(msg.contains("not connected"), "{msg}"),
        other => panic!("expected InvalidConfig, got {other:?}"),
    }
}

#[tokio::test]
async fn control_methods_reject_before_connect() {
    let mock = Arc::new(MockTransport::builder().build());
    let client = ClaudeClient::with_transport(
        Arc::clone(&mock) as Arc<dyn claude_agent_sdk_rs::testing::Transport>,
        ClaudeAgentOptions::default(),
    );
    // No connect_with_transport() — every call should immediately error.
    assert_invalid_config(client.interrupt().await.unwrap_err());
    assert_invalid_config(
        client
            .set_permission_mode(PermissionMode::Plan)
            .await
            .unwrap_err(),
    );
    assert_invalid_config(client.set_model(Some("x")).await.unwrap_err());
    assert_invalid_config(client.rewind_files("uuid").await.unwrap_err());
    assert_invalid_config(client.get_mcp_status().await.unwrap_err());
    assert_invalid_config(client.reconnect_mcp_server("a").await.unwrap_err());
    assert_invalid_config(client.toggle_mcp_server("a", true).await.unwrap_err());
    assert_invalid_config(client.stop_task("t").await.unwrap_err());
    assert_invalid_config(client.get_context_usage().await.unwrap_err());

    // get_server_info returns Option, so it should be None instead of erroring.
    assert!(client.get_server_info().is_none());
}

// ============================================================================
// Disconnect lifecycle
// ============================================================================

#[tokio::test]
async fn disconnect_is_idempotent() {
    let (mut client, mock) = make_connected().await;
    client.disconnect().await.unwrap();
    // Second call on an already-disconnected client must be a no-op
    client.disconnect().await.unwrap();
    let _ = mock.close().await;
}

#[tokio::test]
async fn disconnect_writes_no_extra_traffic_after_close() {
    let (mut client, mock) = make_connected().await;
    let before = mock.written_messages_async().await.len();
    client.disconnect().await.unwrap();
    let after = mock.written_messages_async().await.len();
    // disconnect() may shut stdin via end_input(), but it must not write
    // any new control_request payloads.
    assert_eq!(before, after, "disconnect emitted unexpected writes");
}

#[tokio::test]
async fn get_server_info_is_none_when_init_skipped() {
    // connect_with_transport intentionally skips initialize() (no real CLI),
    // so get_server_info must be None.
    let (client, mock) = make_connected().await;
    assert!(client.get_server_info().is_none());
    let _ = mock.close().await;
}

// ============================================================================
// query.rs validation paths (no subprocess required)
// ============================================================================

#[tokio::test]
async fn query_with_content_rejects_empty_blocks() {
    let err = query_with_content(Vec::<UserContentBlock>::new(), None)
        .await
        .unwrap_err();
    match err {
        ClaudeError::InvalidConfig(msg) => assert!(msg.contains("at least one block"), "{msg}"),
        other => panic!("expected InvalidConfig, got {other:?}"),
    }
}

#[tokio::test]
async fn query_stream_with_content_rejects_empty_blocks() {
    let result = query_stream_with_content(Vec::<UserContentBlock>::new(), None).await;
    match result {
        Err(ClaudeError::InvalidConfig(msg)) => {
            assert!(msg.contains("at least one block"), "{msg}")
        }
        Err(other) => panic!("expected InvalidConfig, got {other:?}"),
        Ok(_) => panic!("expected error, got Ok stream"),
    }
}

#[tokio::test]
async fn client_query_with_content_rejects_empty_blocks() {
    let (mut client, mock) = make_connected().await;
    let err = client
        .query_with_content(Vec::<UserContentBlock>::new())
        .await
        .unwrap_err();
    match err {
        ClaudeError::InvalidConfig(msg) => assert!(msg.contains("at least one block"), "{msg}"),
        other => panic!("expected InvalidConfig, got {other:?}"),
    }
    // No write should have been emitted for an empty content payload.
    let writes = wait_for_writes(&mock, 0).await;
    assert!(writes.is_empty(), "no payload should have been written");
    let _ = mock.close().await;
}
