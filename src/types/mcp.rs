//! MCP (Model Context Protocol) types for Claude Agent SDK

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::errors::Result;

/// MCP servers configuration
#[derive(Clone, Default)]
pub enum McpServers {
    /// No MCP servers
    #[default]
    Empty,
    /// Dictionary of server configurations
    Dict(HashMap<String, McpServerConfig>),
    /// Path to MCP servers configuration file
    Path(PathBuf),
}

/// MCP server configuration
#[derive(Clone)]
pub enum McpServerConfig {
    /// Stdio-based MCP server
    Stdio(McpStdioServerConfig),
    /// SSE-based MCP server
    Sse(McpSseServerConfig),
    /// HTTP-based MCP server
    Http(McpHttpServerConfig),
    /// SDK (in-process) MCP server
    Sdk(McpSdkServerConfig),
}

/// Stdio MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStdioServerConfig {
    /// Command to execute
    pub command: String,
    /// Command arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// SSE MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSseServerConfig {
    /// Server URL
    pub url: String,
    /// HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// HTTP MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHttpServerConfig {
    /// Server URL
    pub url: String,
    /// HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// SDK (in-process) MCP server configuration
#[derive(Clone)]
pub struct McpSdkServerConfig {
    /// Server name
    pub name: String,
    /// Server instance
    pub instance: Arc<dyn SdkMcpServer>,
}

/// Trait for SDK MCP server implementations
#[async_trait]
pub trait SdkMcpServer: Send + Sync {
    /// Handle an MCP message
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value>;
}

/// Tool handler trait
pub trait ToolHandler: Send + Sync {
    /// Handle a tool invocation
    fn handle(&self, args: serde_json::Value) -> BoxFuture<'static, Result<ToolResult>>;
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Result content
    pub content: Vec<ToolResultContent>,
    /// Whether this is an error
    #[serde(default)]
    pub is_error: bool,
}

/// Tool result content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolResultContent {
    /// Text content
    Text {
        /// Text content
        text: String,
    },
    /// Image content
    Image {
        /// Base64-encoded image data
        data: String,
        /// MIME type
        mime_type: String,
    },
}

/// SDK MCP tool definition
pub struct SdkMcpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for tool input
    pub input_schema: serde_json::Value,
    /// Tool handler
    pub handler: Arc<dyn ToolHandler>,
}

// ============================================================================
// MCP Status Types (for get_mcp_status)
// ============================================================================

/// MCP server connection status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum McpServerConnectionStatus {
    /// Server is connected
    Connected,
    /// Server connection failed
    Failed,
    /// Server needs authentication
    NeedsAuth,
    /// Server is pending connection
    Pending,
    /// Server is disabled
    Disabled,
}

/// MCP server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// MCP tool info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tool annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<McpToolAnnotations>,
}

/// MCP tool annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolAnnotations {
    /// Whether the tool is read-only
    #[serde(skip_serializing_if = "Option::is_none", rename = "readOnly")]
    pub read_only: Option<bool>,
    /// Whether the tool is destructive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive: Option<bool>,
    /// Whether the tool has open world access
    #[serde(skip_serializing_if = "Option::is_none", rename = "openWorld")]
    pub open_world: Option<bool>,
}

/// MCP server status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerStatus {
    /// Server name
    pub name: String,
    /// Connection status
    pub status: McpServerConnectionStatus,
    /// Server info (if connected)
    #[serde(skip_serializing_if = "Option::is_none", rename = "serverInfo")]
    pub server_info: Option<McpServerInfo>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Server configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Server scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Available tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<McpToolInfo>>,
}

/// MCP status response (for get_mcp_status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStatusResponse {
    /// MCP servers status
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Vec<McpServerStatus>,
}

/// Create an in-process MCP server
pub fn create_sdk_mcp_server(
    name: impl Into<String>,
    version: impl Into<String>,
    tools: Vec<SdkMcpTool>,
) -> McpSdkServerConfig {
    let server = DefaultSdkMcpServer {
        name: name.into(),
        version: version.into(),
        tools: tools.into_iter().map(|t| (t.name.clone(), t)).collect(),
    };

    McpSdkServerConfig {
        name: server.name.clone(),
        instance: Arc::new(server),
    }
}

/// Default implementation of SDK MCP server
struct DefaultSdkMcpServer {
    name: String,
    version: String,
    tools: HashMap<String, SdkMcpTool>,
}

#[async_trait]
impl SdkMcpServer for DefaultSdkMcpServer {
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value> {
        // Parse the MCP message
        let method = message["method"]
            .as_str()
            .ok_or_else(|| crate::errors::ClaudeError::Transport("Missing method".to_string()))?;

        let message_id = message.get("id").cloned();

        match method {
            "initialize" => {
                // Return JSONRPC response for initialize
                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": message_id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": self.name,
                            "version": self.version
                        }
                    }
                }))
            }
            "tools/list" => {
                // Return list of tools in JSONRPC format
                let tools: Vec<_> = self
                    .tools
                    .values()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema
                        })
                    })
                    .collect();

                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": message_id,
                    "result": {
                        "tools": tools
                    }
                }))
            }
            "tools/call" => {
                // Execute a tool
                let params = &message["params"];
                let tool_name = params["name"].as_str().ok_or_else(|| {
                    crate::errors::ClaudeError::Transport("Missing tool name".to_string())
                })?;
                let arguments = params["arguments"].clone();

                let tool = self.tools.get(tool_name).ok_or_else(|| {
                    crate::errors::ClaudeError::Transport(format!("Tool not found: {}", tool_name))
                })?;

                let result = tool.handler.handle(arguments).await?;

                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": message_id,
                    "result": {
                        "content": result.content,
                        "isError": result.is_error
                    }
                }))
            }
            "notifications/initialized" | "notifications/cancelled" => {
                // Notifications don't require a response
                // Return null for notifications
                Ok(serde_json::json!(null))
            }
            _ => Err(crate::errors::ClaudeError::Transport(format!(
                "Unknown method: {}",
                method
            ))),
        }
    }
}

/// Macro to create a tool
#[macro_export]
macro_rules! tool {
    ($name:expr, $desc:expr, $schema:expr, $handler:expr) => {{
        struct Handler<F>(F);

        impl<F, Fut> $crate::types::mcp::ToolHandler for Handler<F>
        where
            F: Fn(serde_json::Value) -> Fut + Send + Sync,
            Fut: std::future::Future<Output = anyhow::Result<$crate::types::mcp::ToolResult>>
                + Send
                + 'static,
        {
            fn handle(
                &self,
                args: serde_json::Value,
            ) -> futures::future::BoxFuture<
                'static,
                $crate::errors::Result<$crate::types::mcp::ToolResult>,
            > {
                use futures::FutureExt;
                let f = &self.0;
                let fut = f(args);
                async move { fut.await.map_err(|e| e.into()) }.boxed()
            }
        }

        $crate::types::mcp::SdkMcpTool {
            name: $name.to_string(),
            description: $desc.to_string(),
            input_schema: $schema,
            handler: std::sync::Arc::new(Handler($handler)),
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_connection_status_serialization() {
        let status = McpServerConnectionStatus::Connected;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json, "connected");
    }

    #[test]
    fn test_mcp_server_status_serialization() {
        let status = McpServerStatus {
            name: "my-server".to_string(),
            status: McpServerConnectionStatus::Connected,
            server_info: Some(McpServerInfo {
                name: "my-server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            error: None,
            config: None,
            scope: None,
            tools: Some(vec![McpToolInfo {
                name: "greet".to_string(),
                description: Some("Greet a user".to_string()),
                annotations: Some(McpToolAnnotations {
                    read_only: Some(true),
                    destructive: Some(false),
                    open_world: None,
                }),
            }]),
        };

        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["name"], "my-server");
        assert_eq!(json["status"], "connected");
        assert_eq!(json["serverInfo"]["version"], "1.0.0");
        assert_eq!(json["tools"][0]["name"], "greet");
    }

    #[test]
    fn test_mcp_status_response_serialization() {
        let response = McpStatusResponse {
            mcp_servers: vec![McpServerStatus {
                name: "server-1".to_string(),
                status: McpServerConnectionStatus::Pending,
                server_info: None,
                error: None,
                config: None,
                scope: None,
                tools: None,
            }],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["mcpServers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_mcp_tool_annotations_serialization() {
        let annotations = McpToolAnnotations {
            read_only: Some(true),
            destructive: Some(false),
            open_world: Some(true),
        };

        let json = serde_json::to_value(&annotations).unwrap();
        assert_eq!(json["readOnly"], true);
        assert_eq!(json["destructive"], false);
        assert_eq!(json["openWorld"], true);
    }
}
