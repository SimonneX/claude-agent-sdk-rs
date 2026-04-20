//! Message types for Claude Agent SDK

use serde::{Deserialize, Serialize};

/// Supported image MIME types for Claude API
const SUPPORTED_IMAGE_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

/// Maximum base64 data size (15MB results in ~20MB decoded, within Claude's limits)
const MAX_BASE64_SIZE: usize = 15_728_640;

/// Error types for assistant messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssistantMessageError {
    /// Authentication failed
    AuthenticationFailed,
    /// Billing error
    BillingError,
    /// Rate limit exceeded
    RateLimit,
    /// Invalid request
    InvalidRequest,
    /// Server error
    ServerError,
    /// Unknown error
    Unknown,
}

/// Main message enum containing all message types from CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    /// Assistant message
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    /// System message
    #[serde(rename = "system")]
    System(SystemMessage),
    /// Result message
    #[serde(rename = "result")]
    Result(ResultMessage),
    /// Stream event
    #[serde(rename = "stream_event")]
    StreamEvent(StreamEvent),
    /// User message (rarely used in stream output)
    #[serde(rename = "user")]
    User(UserMessage),
    /// Control cancel request (ignore this - it's internal control protocol)
    #[serde(rename = "control_cancel_request")]
    ControlCancelRequest(serde_json::Value),
    /// Task started message
    #[serde(rename = "task_started")]
    TaskStarted(TaskStartedMessage),
    /// Task progress message
    #[serde(rename = "task_progress")]
    TaskProgress(TaskProgressMessage),
    /// Task notification message
    #[serde(rename = "task_notification")]
    TaskNotification(TaskNotificationMessage),
    /// Rate limit event
    #[serde(rename = "rate_limit")]
    RateLimit(RateLimitEvent),
}

/// User message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    /// Message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Message content blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentBlock>>,
    /// UUID for file checkpointing (used with enable_file_checkpointing and rewind_files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Parent tool use ID (if this is a tool result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    /// Additional fields
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Message content can be text or blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text { text: String },
    /// Structured content blocks
    Blocks { content: Vec<ContentBlock> },
}

impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        MessageContent::Text { text }
    }
}

impl From<&str> for MessageContent {
    fn from(text: &str) -> Self {
        MessageContent::Text {
            text: text.to_string(),
        }
    }
}

impl From<Vec<ContentBlock>> for MessageContent {
    fn from(blocks: Vec<ContentBlock>) -> Self {
        MessageContent::Blocks { content: blocks }
    }
}

/// Assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    /// The actual message content (wrapped)
    pub message: AssistantMessageInner,
    /// Parent tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// Inner assistant message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageInner {
    /// Message content blocks
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    /// Model used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Message ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
    /// Error type (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AssistantMessageError>,
}

/// System message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    /// Message subtype
    pub subtype: String,
    /// Current working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Available tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// MCP servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<serde_json::Value>>,
    /// Model being used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none", rename = "permissionMode")]
    pub permission_mode: Option<String>,
    /// UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Additional data
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Result message indicating query completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMessage {
    /// Result subtype
    pub subtype: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// API duration in milliseconds
    pub duration_api_ms: u64,
    /// Whether this is an error result
    pub is_error: bool,
    /// Number of turns in conversation
    pub num_turns: u32,
    /// Session ID
    pub session_id: String,
    /// Total cost in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
    /// Result text (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Structured output (when output_format is specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<serde_json::Value>,
    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Model usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_usage: Option<serde_json::Value>,
    /// Permission denials list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_denials: Option<Vec<serde_json::Value>>,
    /// Error messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    /// Message UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// Stream event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// Event UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Event data
    pub event: serde_json::Value,
    /// Parent tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

/// Task started message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStartedMessage {
    /// Task ID
    pub task_id: String,
    /// Task description
    pub description: String,
    /// UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Task type (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
}

/// Task progress message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressMessage {
    /// Task ID
    pub task_id: String,
    /// Progress description
    pub description: String,
    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TaskUsage>,
    /// UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Last tool name used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
}

/// Task usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUsage {
    /// Total tokens used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    /// Number of tool uses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_uses: Option<u32>,
    /// Duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Task notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotificationMessage {
    /// Task ID
    pub task_id: String,
    /// Task status (completed, failed, stopped)
    pub status: TaskNotificationStatus,
    /// Output file path (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,
    /// Summary of the task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TaskUsage>,
}

/// Task notification status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskNotificationStatus {
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was stopped
    Stopped,
}

/// Rate limit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEvent {
    /// Rate limit information
    pub rate_limit_info: RateLimitInfo,
    /// UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Rate limit status (allowed, allowed_warning, rejected)
    pub status: RateLimitStatus,
    /// When the limit resets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<String>,
    /// Rate limit type (five_hour, seven_day, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_type: Option<RateLimitType>,
    /// Utilization percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utilization: Option<f64>,
    /// Overage status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overage_status: Option<RateLimitStatus>,
    /// When overage resets
    #[serde(skip_serializing_if = "Option::is_none", rename = "overage_resets_at")]
    pub overage_resets_at: Option<String>,
    /// Reason for overage being disabled
    #[serde(skip_serializing_if = "Option::is_none", rename = "overage_disabled_reason")]
    pub overage_disabled_reason: Option<String>,
    /// Raw data
    #[serde(flatten)]
    pub raw: serde_json::Value,
}

/// Rate limit status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitStatus {
    /// Request allowed
    Allowed,
    /// Request allowed but with warning
    AllowedWarning,
    /// Request rejected
    Rejected,
}

/// Rate limit type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitType {
    /// Five hour limit
    FiveHour,
    /// Seven day limit (default)
    SevenDay,
    /// Seven day Opus limit
    SevenDayOpus,
    /// Seven day Sonnet limit
    SevenDaySonnet,
    /// Overage
    Overage,
}

/// Content block types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text block
    Text(TextBlock),
    /// Thinking block (extended thinking)
    Thinking(ThinkingBlock),
    /// Tool use block
    ToolUse(ToolUseBlock),
    /// Tool result block
    ToolResult(ToolResultBlock),
    /// Image block
    Image(ImageBlock),
}

/// Text content block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    /// Text content
    pub text: String,
}

/// Thinking block (extended thinking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    /// Thinking content
    pub thinking: String,
    /// Signature
    pub signature: String,
}

/// Tool use block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// Tool use ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool input parameters
    pub input: serde_json::Value,
}

/// Tool result block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    /// Tool use ID this result corresponds to
    pub tool_use_id: String,
    /// Result content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultContent>,
    /// Whether this is an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Tool result content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Text result
    Text(String),
    /// Structured blocks
    Blocks(Vec<serde_json::Value>),
}

/// Image source for user prompts
///
/// Represents the source of image data that can be included in user messages.
/// Claude supports both base64-encoded images and URL references.
///
/// # Supported Formats
///
/// - JPEG (`image/jpeg`)
/// - PNG (`image/png`)
/// - GIF (`image/gif`)
/// - WebP (`image/webp`)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64-encoded image data
    Base64 {
        /// MIME type (e.g., "image/png", "image/jpeg", "image/gif", "image/webp")
        media_type: String,
        /// Base64-encoded image data (without data URI prefix)
        data: String,
    },
    /// URL reference to an image
    Url {
        /// Publicly accessible image URL
        url: String,
    },
}

/// Image block for user prompts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageBlock {
    /// Image source (base64 or URL)
    pub source: ImageSource,
}

/// Content block for user prompts (input)
///
/// Represents content that can be included in user messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserContentBlock {
    /// Text content
    Text {
        /// Text content string
        text: String,
    },
    /// Image content
    Image {
        /// Image source (base64 or URL)
        source: ImageSource,
    },
}

impl UserContentBlock {
    /// Create a text content block
    pub fn text(text: impl Into<String>) -> Self {
        UserContentBlock::Text { text: text.into() }
    }

    /// Create an image content block from base64 data
    ///
    /// # Arguments
    ///
    /// * `media_type` - MIME type of the image (e.g., "image/png", "image/jpeg")
    /// * `data` - Base64-encoded image data (without data URI prefix)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The MIME type is not supported (valid types: image/jpeg, image/png, image/gif, image/webp)
    /// - The base64 data exceeds the maximum size limit (15MB)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::UserContentBlock;
    /// let block = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=")?;
    /// # Ok::<(), claude_agent_sdk_rs::ClaudeError>(())
    /// ```
    pub fn image_base64(
        media_type: impl Into<String>,
        data: impl Into<String>,
    ) -> crate::errors::Result<Self> {
        let media_type_str = media_type.into();
        let data_str = data.into();

        // Validate MIME type
        if !SUPPORTED_IMAGE_MIME_TYPES.contains(&media_type_str.as_str()) {
            return Err(crate::errors::ImageValidationError::new(format!(
                "Unsupported media type '{}'. Supported types: {:?}",
                media_type_str, SUPPORTED_IMAGE_MIME_TYPES
            ))
            .into());
        }

        // Validate base64 size
        if data_str.len() > MAX_BASE64_SIZE {
            return Err(crate::errors::ImageValidationError::new(format!(
                "Base64 data exceeds maximum size of {} bytes (got {} bytes)",
                MAX_BASE64_SIZE,
                data_str.len()
            ))
            .into());
        }

        Ok(UserContentBlock::Image {
            source: ImageSource::Base64 {
                media_type: media_type_str,
                data: data_str,
            },
        })
    }

    /// Create an image content block from URL
    pub fn image_url(url: impl Into<String>) -> Self {
        UserContentBlock::Image {
            source: ImageSource::Url { url: url.into() },
        }
    }

    /// Validate a collection of content blocks
    ///
    /// Ensures the content is non-empty. This is used internally by query functions
    /// to provide consistent validation.
    ///
    /// # Errors
    ///
    /// Returns an error if the content blocks slice is empty.
    pub fn validate_content(blocks: &[UserContentBlock]) -> crate::Result<()> {
        if blocks.is_empty() {
            return Err(crate::errors::ClaudeError::InvalidConfig(
                "Content must include at least one block (text or image)".to_string(),
            ));
        }
        Ok(())
    }
}

impl From<String> for UserContentBlock {
    fn from(text: String) -> Self {
        UserContentBlock::Text { text }
    }
}

impl From<&str> for UserContentBlock {
    fn from(text: &str) -> Self {
        UserContentBlock::Text {
            text: text.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_content_block_text_serialization() {
        let block = ContentBlock::Text(TextBlock {
            text: "Hello".to_string(),
        });

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello");
    }

    #[test]
    fn test_content_block_tool_use_serialization() {
        let block = ContentBlock::ToolUse(ToolUseBlock {
            id: "tool_123".to_string(),
            name: "Bash".to_string(),
            input: json!({"command": "echo hello"}),
        });

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "tool_use");
        assert_eq!(json["id"], "tool_123");
        assert_eq!(json["name"], "Bash");
        assert_eq!(json["input"]["command"], "echo hello");
    }

    #[test]
    fn test_message_assistant_deserialization() {
        let json_str = r#"{
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Hello"}],
                "model": "claude-sonnet-4"
            },
            "session_id": "test-session"
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::Assistant(assistant) => {
                assert_eq!(assistant.session_id, Some("test-session".to_string()));
                assert_eq!(assistant.message.model, Some("claude-sonnet-4".to_string()));
            }
            _ => panic!("Expected Assistant variant"),
        }
    }

    #[test]
    fn test_message_result_deserialization() {
        let json_str = r#"{
            "type": "result",
            "subtype": "query_complete",
            "duration_ms": 1500,
            "duration_api_ms": 1200,
            "is_error": false,
            "num_turns": 3,
            "session_id": "test-session",
            "total_cost_usd": 0.0042
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::Result(result) => {
                assert_eq!(result.subtype, "query_complete");
                assert_eq!(result.duration_ms, 1500);
                assert_eq!(result.num_turns, 3);
                assert_eq!(result.total_cost_usd, Some(0.0042));
            }
            _ => panic!("Expected Result variant"),
        }
    }

    #[test]
    fn test_message_system_deserialization() {
        let json_str = r#"{
            "type": "system",
            "subtype": "session_start",
            "cwd": "/home/user",
            "session_id": "test-session",
            "tools": ["Bash", "Read", "Write"]
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::System(system) => {
                assert_eq!(system.subtype, "session_start");
                assert_eq!(system.cwd, Some("/home/user".to_string()));
                assert_eq!(system.tools.as_ref().unwrap().len(), 3);
            }
            _ => panic!("Expected System variant"),
        }
    }

    #[test]
    fn test_tool_result_content_text() {
        let content = ToolResultContent::Text("Command output".to_string());
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json, "Command output");
    }

    #[test]
    fn test_tool_result_content_blocks() {
        let content = ToolResultContent::Blocks(vec![json!({"type": "text", "text": "Result"})]);
        let json = serde_json::to_value(&content).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["type"], "text");
    }

    #[test]
    fn test_image_source_base64_serialization() {
        let source = ImageSource::Base64 {
            media_type: "image/png".to_string(),
            data: "iVBORw0KGgo=".to_string(),
        };

        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["type"], "base64");
        assert_eq!(json["media_type"], "image/png");
        assert_eq!(json["data"], "iVBORw0KGgo=");
    }

    #[test]
    fn test_image_source_url_serialization() {
        let source = ImageSource::Url {
            url: "https://example.com/image.png".to_string(),
        };

        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["type"], "url");
        assert_eq!(json["url"], "https://example.com/image.png");
    }

    #[test]
    fn test_image_source_base64_deserialization() {
        let json_str = r#"{
            "type": "base64",
            "media_type": "image/jpeg",
            "data": "base64data=="
        }"#;

        let source: ImageSource = serde_json::from_str(json_str).unwrap();
        match source {
            ImageSource::Base64 { media_type, data } => {
                assert_eq!(media_type, "image/jpeg");
                assert_eq!(data, "base64data==");
            }
            _ => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_image_source_url_deserialization() {
        let json_str = r#"{
            "type": "url",
            "url": "https://example.com/test.gif"
        }"#;

        let source: ImageSource = serde_json::from_str(json_str).unwrap();
        match source {
            ImageSource::Url { url } => {
                assert_eq!(url, "https://example.com/test.gif");
            }
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_user_content_block_text_serialization() {
        let block = UserContentBlock::text("Hello world");

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello world");
    }

    #[test]
    fn test_user_content_block_image_base64_serialization() {
        let block = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=").unwrap();

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["type"], "base64");
        assert_eq!(json["source"]["media_type"], "image/png");
        assert_eq!(json["source"]["data"], "iVBORw0KGgo=");
    }

    #[test]
    fn test_user_content_block_image_url_serialization() {
        let block = UserContentBlock::image_url("https://example.com/image.webp");

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["type"], "url");
        assert_eq!(json["source"]["url"], "https://example.com/image.webp");
    }

    #[test]
    fn test_user_content_block_from_string() {
        let block: UserContentBlock = "Test message".into();

        match block {
            UserContentBlock::Text { text } => {
                assert_eq!(text, "Test message");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_user_content_block_from_owned_string() {
        let block: UserContentBlock = String::from("Owned message").into();

        match block {
            UserContentBlock::Text { text } => {
                assert_eq!(text, "Owned message");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_image_block_serialization() {
        let block = ImageBlock {
            source: ImageSource::Base64 {
                media_type: "image/gif".to_string(),
                data: "R0lGODlh".to_string(),
            },
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["source"]["type"], "base64");
        assert_eq!(json["source"]["media_type"], "image/gif");
        assert_eq!(json["source"]["data"], "R0lGODlh");
    }

    #[test]
    fn test_content_block_image_serialization() {
        let block = ContentBlock::Image(ImageBlock {
            source: ImageSource::Url {
                url: "https://example.com/photo.jpg".to_string(),
            },
        });

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["type"], "url");
        assert_eq!(json["source"]["url"], "https://example.com/photo.jpg");
    }

    #[test]
    fn test_content_block_image_deserialization() {
        let json_str = r#"{
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": "image/webp",
                "data": "UklGR"
            }
        }"#;

        let block: ContentBlock = serde_json::from_str(json_str).unwrap();
        match block {
            ContentBlock::Image(image) => match image.source {
                ImageSource::Base64 { media_type, data } => {
                    assert_eq!(media_type, "image/webp");
                    assert_eq!(data, "UklGR");
                }
                _ => panic!("Expected Base64 source"),
            },
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_user_content_block_deserialization() {
        let json_str = r#"{
            "type": "text",
            "text": "Describe this image"
        }"#;

        let block: UserContentBlock = serde_json::from_str(json_str).unwrap();
        match block {
            UserContentBlock::Text { text } => {
                assert_eq!(text, "Describe this image");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_user_content_block_image_deserialization() {
        let json_str = r#"{
            "type": "image",
            "source": {
                "type": "url",
                "url": "https://example.com/diagram.png"
            }
        }"#;

        let block: UserContentBlock = serde_json::from_str(json_str).unwrap();
        match block {
            UserContentBlock::Image { source } => match source {
                ImageSource::Url { url } => {
                    assert_eq!(url, "https://example.com/diagram.png");
                }
                _ => panic!("Expected Url source"),
            },
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_image_base64_valid() {
        let block = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=");
        assert!(block.is_ok());
    }

    #[test]
    fn test_image_base64_invalid_mime_type() {
        let block = UserContentBlock::image_base64("image/bmp", "data");
        assert!(block.is_err());
        let err = block.unwrap_err().to_string();
        assert!(err.contains("Unsupported media type"));
        assert!(err.contains("image/bmp"));
    }

    #[test]
    fn test_image_base64_exceeds_size_limit() {
        let large_data = "a".repeat(MAX_BASE64_SIZE + 1);
        let block = UserContentBlock::image_base64("image/png", large_data);
        assert!(block.is_err());
        let err = block.unwrap_err().to_string();
        assert!(err.contains("exceeds maximum size"));
    }

    #[test]
    fn test_task_started_message_deserialization() {
        let json_str = r#"{
            "type": "task_started",
            "task_id": "task-123",
            "description": "Building the project",
            "uuid": "uuid-123",
            "session_id": "session-1",
            "tool_use_id": "tool-1",
            "task_type": "build"
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::TaskStarted(task) => {
                assert_eq!(task.task_id, "task-123");
                assert_eq!(task.description, "Building the project");
                assert_eq!(task.task_type, Some("build".to_string()));
            }
            _ => panic!("Expected TaskStarted variant"),
        }
    }

    #[test]
    fn test_task_progress_message_deserialization() {
        let json_str = r#"{
            "type": "task_progress",
            "task_id": "task-123",
            "description": "Progress update",
            "usage": {"total_tokens": 1000, "tool_uses": 5},
            "uuid": "uuid-456",
            "session_id": "session-1",
            "last_tool_name": "Bash"
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::TaskProgress(progress) => {
                assert_eq!(progress.task_id, "task-123");
                assert!(progress.usage.is_some());
                assert_eq!(progress.last_tool_name, Some("Bash".to_string()));
            }
            _ => panic!("Expected TaskProgress variant"),
        }
    }

    #[test]
    fn test_task_notification_message_deserialization() {
        let json_str = r#"{
            "type": "task_notification",
            "task_id": "task-123",
            "status": "completed",
            "output_file": "/tmp/output.txt",
            "summary": "Build completed successfully",
            "uuid": "uuid-789",
            "session_id": "session-1"
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::TaskNotification(notification) => {
                assert_eq!(notification.task_id, "task-123");
                assert_eq!(notification.status, TaskNotificationStatus::Completed);
                assert_eq!(notification.output_file, Some("/tmp/output.txt".to_string()));
            }
            _ => panic!("Expected TaskNotification variant"),
        }
    }

    #[test]
    fn test_rate_limit_event_deserialization() {
        let json_str = r#"{
            "type": "rate_limit",
            "rate_limit_info": {
                "status": "allowed_warning",
                "resets_at": "2024-01-01T00:00:00Z",
                "rate_limit_type": "seven_day",
                "utilization": 80.5
            },
            "uuid": "uuid-limit",
            "session_id": "session-1"
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::RateLimit(rate_limit) => {
                assert_eq!(rate_limit.rate_limit_info.status, RateLimitStatus::AllowedWarning);
                assert_eq!(rate_limit.rate_limit_info.rate_limit_type, Some(RateLimitType::SevenDay));
                assert_eq!(rate_limit.rate_limit_info.utilization, Some(80.5));
            }
            _ => panic!("Expected RateLimit variant"),
        }
    }

    #[test]
    fn test_result_message_with_new_fields() {
        let json_str = r#"{
            "type": "result",
            "subtype": "query_complete",
            "duration_ms": 1500,
            "duration_api_ms": 1200,
            "is_error": false,
            "num_turns": 3,
            "session_id": "test-session",
            "stop_reason": "end_turn",
            "uuid": "result-uuid",
            "errors": []
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::Result(result) => {
                assert_eq!(result.stop_reason, Some("end_turn".to_string()));
                assert_eq!(result.uuid, Some("result-uuid".to_string()));
                assert!(result.errors.is_some());
            }
            _ => panic!("Expected Result variant"),
        }
    }

    #[test]
    fn test_rate_limit_status_serialization() {
        let status = RateLimitStatus::AllowedWarning;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json, "allowed_warning");
    }

    #[test]
    fn test_task_notification_status_serialization() {
        let status = TaskNotificationStatus::Failed;
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json, "failed");
    }
}
