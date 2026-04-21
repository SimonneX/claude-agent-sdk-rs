//! Session management functions for Claude Agent SDK
//!
//! This module provides functions for managing Claude Code sessions,
//! including listing, retrieving, renaming, tagging, and deleting sessions.
//!
//! ## Implementation status
//!
//! These helpers correspond to the Python SDK's session management API
//! (`list_sessions`, `get_session_info`, `tag_session`, …) but the actual
//! transcript I/O against `~/.claude/projects/<project_hash>/sessions/*.jsonl`
//! has not yet been ported to Rust. To avoid silently returning empty results
//! or fabricated records, every helper currently returns
//! [`ClaudeError::InvalidConfig`] with a `not yet implemented` message. Once
//! the real implementation lands the error returns will be replaced with the
//! actual session data.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::errors::ClaudeError;

fn unimplemented_err(name: &str) -> ClaudeError {
    ClaudeError::InvalidConfig(format!(
        "sessions::{name} is not yet implemented in the Rust SDK; \
         transcript I/O against ~/.claude/projects/.../sessions/*.jsonl has \
         not been ported from the Python SDK yet"
    ))
}

/// Session information returned by list_sessions and get_session_info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKSessionInfo {
    /// Session ID
    pub session_id: String,
    /// Session summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    /// Custom title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    /// First prompt in the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_prompt: Option<String>,
    /// Git branch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    /// Current working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Session tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Created at timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// A message in a session transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    /// Message type (user, assistant, etc.)
    #[serde(rename = "type")]
    pub type_: String,
    /// Message UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// The message content
    pub message: serde_json::Value,
    /// Parent tool use ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

/// Result of fork_session operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkSessionResult {
    /// New session ID
    pub new_session_id: String,
    /// Path to the new session file
    pub session_file: PathBuf,
}

/// Sort order for session listing
#[derive(Debug, Clone, Copy, Default)]
pub enum SessionSortOrder {
    /// Sort by last modified (most recent first)
    #[default]
    LastModified,
    /// Sort by creation time
    CreatedAt,
    /// Sort alphabetically by session ID
    SessionId,
}

/// Options for listing sessions
#[derive(Debug, Clone, Default)]
pub struct ListSessionsOptions {
    /// Maximum number of sessions to return
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Sort order
    pub sort: SessionSortOrder,
}

// NOT-YET-IMPLEMENTED helpers — see module-level docs.
// Each function returns ClaudeError::InvalidConfig with a "not yet
// implemented" message rather than fabricating empty data, so callers can
// detect the absence of real session I/O.

/// List all sessions in a project directory.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn list_sessions(
    _directory: Option<&str>,
    _options: Option<ListSessionsOptions>,
) -> crate::Result<Vec<SDKSessionInfo>> {
    Err(unimplemented_err("list_sessions"))
}

/// Get information about a specific session.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn get_session_info(
    _session_id: &str,
    _directory: Option<&str>,
) -> crate::Result<SDKSessionInfo> {
    Err(unimplemented_err("get_session_info"))
}

/// Get messages from a session.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn get_session_messages(
    _session_id: &str,
    _directory: Option<&str>,
    _limit: Option<usize>,
    _offset: Option<usize>,
) -> crate::Result<Vec<SessionMessage>> {
    Err(unimplemented_err("get_session_messages"))
}

/// List subagent IDs for a session.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn list_subagents(
    _session_id: &str,
    _directory: Option<&str>,
) -> crate::Result<Vec<String>> {
    Err(unimplemented_err("list_subagents"))
}

/// Get messages from a subagent.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn get_subagent_messages(
    _session_id: &str,
    _agent_id: &str,
    _directory: Option<&str>,
) -> crate::Result<Vec<SessionMessage>> {
    Err(unimplemented_err("get_subagent_messages"))
}

/// Rename a session with a custom title.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn rename_session(
    _session_id: &str,
    _title: &str,
    _directory: Option<&str>,
) -> crate::Result<()> {
    Err(unimplemented_err("rename_session"))
}

/// Tag or clear tag from a session.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn tag_session(
    _session_id: &str,
    _tag: Option<&str>,
    _directory: Option<&str>,
) -> crate::Result<()> {
    Err(unimplemented_err("tag_session"))
}

/// Delete a session.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn delete_session(
    _session_id: &str,
    _directory: Option<&str>,
) -> crate::Result<()> {
    Err(unimplemented_err("delete_session"))
}

/// Fork a session to create a new branch.
///
/// Returns [`ClaudeError::InvalidConfig`] until transcript I/O is implemented;
/// see module docs.
pub async fn fork_session(
    _session_id: &str,
    _directory: Option<&str>,
    _up_to_message_id: Option<&str>,
    _title: Option<&str>,
) -> crate::Result<ForkSessionResult> {
    Err(unimplemented_err("fork_session"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_serialization() {
        let info = SDKSessionInfo {
            session_id: "test-session".to_string(),
            summary: Some("Test summary".to_string()),
            last_modified: Some("2024-01-01T00:00:00Z".to_string()),
            file_size: Some(1024),
            custom_title: Some("My Session".to_string()),
            first_prompt: None,
            git_branch: None,
            cwd: None,
            tag: None,
            created_at: None,
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["session_id"], "test-session");
        assert_eq!(json["summary"], "Test summary");
        assert_eq!(json["custom_title"], "My Session");
    }

    #[test]
    fn test_session_message_serialization() {
        let msg = SessionMessage {
            type_: "user".to_string(),
            uuid: "msg-123".to_string(),
            session_id: "session-1".to_string(),
            message: serde_json::json!({"content": "Hello"}),
            parent_tool_use_id: None,
        };

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "user");
        assert_eq!(json["uuid"], "msg-123");
    }

    #[tokio::test]
    async fn test_list_sessions_returns_not_implemented() {
        let err = list_sessions(None, None).await.expect_err("must error");
        match err {
            ClaudeError::InvalidConfig(msg) => {
                assert!(msg.contains("not yet implemented"), "got: {msg}");
                assert!(msg.contains("list_sessions"), "got: {msg}");
            }
            other => panic!("expected InvalidConfig, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_get_session_info_returns_not_implemented() {
        let err = get_session_info("test-id", None)
            .await
            .expect_err("must error");
        match err {
            ClaudeError::InvalidConfig(msg) => {
                assert!(msg.contains("not yet implemented"), "got: {msg}")
            }
            other => panic!("expected InvalidConfig, got {other:?}"),
        }
    }
}