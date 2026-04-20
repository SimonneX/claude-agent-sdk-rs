//! Session store types for custom session persistence
//!
//! This module provides a trait for custom session storage backends
//! (e.g., PostgreSQL, Redis, S3) instead of the default filesystem storage.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::errors::Result;

/// Session key for identifying a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    /// Project key (derived from directory)
    pub project_key: String,
    /// Session ID
    pub session_id: String,
    /// Subpath (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
}

/// Entry in session store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStoreEntry {
    /// Entry type (required)
    #[serde(rename = "type")]
    pub type_: String,
    /// Entry UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Entry in session list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStoreListEntry {
    /// Session ID
    pub session_id: String,
    /// Last modified time
    pub mtime: String,
}

/// Key for listing subkeys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListSubkeysKey {
    /// Project key
    pub project_key: String,
    /// Session ID
    pub session_id: String,
}

/// Trait for custom session storage backends
///
/// Implement this trait to provide custom session persistence
/// (e.g., PostgreSQL, Redis, S3) instead of the default filesystem storage.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk_rs::types::session_store::{
///     SessionStore, SessionKey, SessionStoreEntry, SessionStoreListEntry
/// };
/// use async_trait::async_trait;
///
/// struct RedisSessionStore {
///     client: redis::Client,
/// }
///
/// #[async_trait]
/// impl SessionStore for RedisSessionStore {
///     async fn append(&self, key: &SessionKey, entries: Vec<SessionStoreEntry>) -> claude_agent_sdk_rs::Result<()> {
///         // Store entries in Redis
///         Ok(())
///     }
///
///     async fn load(&self, key: &SessionKey) -> claude_agent_sdk_rs::Result<Option<Vec<SessionStoreEntry>>> {
///         // Load entries from Redis
///         Ok(None)
///     }
///
///     async fn list_sessions(&self, project_key: &str) -> claude_agent_sdk_rs::Result<Vec<SessionStoreListEntry>> {
///         // List sessions from Redis
///         Ok(Vec::new())
///     }
/// }
/// ```
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Append entries to a session (mirror transcript batch)
    ///
    /// # Arguments
    /// * `key` - Session key identifying the session
    /// * `entries` - Entries to append
    async fn append(&self, key: &SessionKey, entries: Vec<SessionStoreEntry>) -> Result<()>;

    /// Load session entries for resume
    ///
    /// # Arguments
    /// * `key` - Session key identifying the session
    ///
    /// # Returns
    /// The session entries, or None if the session doesn't exist
    async fn load(&self, key: &SessionKey) -> Result<Option<Vec<SessionStoreEntry>>>;

    /// List sessions for a project
    ///
    /// # Arguments
    /// * `project_key` - Project key to list sessions for
    ///
    /// # Returns
    /// List of session entries
    async fn list_sessions(&self, project_key: &str) -> Result<Vec<SessionStoreListEntry>>;

    /// Delete a session (optional)
    ///
    /// # Arguments
    /// * `key` - Session key identifying the session
    async fn delete(&self, key: &SessionKey) -> Result<()> {
        // Default: no-op
        Ok(())
    }

    /// List subkeys for a session (optional)
    ///
    /// # Arguments
    /// * `key` - Key identifying the session
    ///
    /// # Returns
    /// List of subkey names
    async fn list_subkeys(&self, key: &SessionListSubkeysKey) -> Result<Vec<String>> {
        // Default: empty list
        Ok(Vec::new())
    }
}

/// In-memory session store (for testing)
pub struct InMemorySessionStore {
    sessions: std::collections::HashMap<String, Vec<SessionStoreEntry>>,
    session_lists: std::collections::HashMap<String, Vec<SessionStoreListEntry>>,
}

impl InMemorySessionStore {
    /// Create a new in-memory session store
    pub fn new() -> Self {
        Self {
            sessions: std::collections::HashMap::new(),
            session_lists: std::collections::HashMap::new(),
        }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn append(&self, _key: &SessionKey, _entries: Vec<SessionStoreEntry>) -> Result<()> {
        // Note: InMemorySessionStore would need interior mutability for actual use
        // This is a placeholder for demonstration
        Ok(())
    }

    async fn load(&self, key: &SessionKey) -> Result<Option<Vec<SessionStoreEntry>>> {
        let _store_key = format!("{}:{}:{}", key.project_key, key.session_id, key.subpath.as_deref().unwrap_or(""));
        Ok(self.sessions.get(&_store_key).cloned())
    }

    async fn list_sessions(&self, project_key: &str) -> Result<Vec<SessionStoreListEntry>> {
        Ok(self.session_lists.get(project_key).cloned().unwrap_or_default())
    }

    async fn delete(&self, _key: &SessionKey) -> Result<()> {
        // Placeholder
        Ok(())
    }

    async fn list_subkeys(&self, _key: &SessionListSubkeysKey) -> Result<Vec<String>> {
        // Placeholder
        Ok(Vec::new())
    }
}

/// Derive project key from directory path
///
/// This creates a unique identifier for a project directory
/// that can be used as the project_key in SessionKey.
pub fn project_key_for_directory(directory: &PathBuf) -> String {
    // Use canonical path as the project key
    directory
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| directory.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_serialization() {
        let key = SessionKey {
            project_key: "project-1".to_string(),
            session_id: "session-1".to_string(),
            subpath: Some("subagent-1".to_string()),
        };

        let json = serde_json::to_value(&key).unwrap();
        assert_eq!(json["project_key"], "project-1");
        assert_eq!(json["session_id"], "session-1");
        assert_eq!(json["subpath"], "subagent-1");
    }

    #[test]
    fn test_session_store_entry_serialization() {
        let entry = SessionStoreEntry {
            type_: "user".to_string(),
            uuid: Some("uuid-123".to_string()),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["type"], "user");
        assert_eq!(json["uuid"], "uuid-123");
        assert_eq!(json["timestamp"], "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_session_store_list_entry() {
        let entry = SessionStoreListEntry {
            session_id: "session-1".to_string(),
            mtime: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["session_id"], "session-1");
        assert_eq!(json["mtime"], "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_project_key_for_directory() {
        let dir = PathBuf::from("/tmp/test");
        let key = project_key_for_directory(&dir);
        // Should contain the path
        assert!(key.contains("/tmp") || key.contains("tmp"));
    }

    #[tokio::test]
    async fn test_in_memory_session_store() {
        let store = InMemorySessionStore::new();

        let key = SessionKey {
            project_key: "test-project".to_string(),
            session_id: "test-session".to_string(),
            subpath: None,
        };

        // Load should return None initially
        let result = store.load(&key).await.unwrap();
        assert!(result.is_none());

        // List sessions should return empty
        let sessions = store.list_sessions("test-project").await.unwrap();
        assert!(sessions.is_empty());
    }
}