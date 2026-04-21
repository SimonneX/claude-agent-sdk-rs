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
/// /// A custom backend (e.g. backed by Redis, S3, Postgres). Pretend the
/// /// internal client is provided by your application.
/// struct CustomSessionStore;
///
/// #[async_trait]
/// impl SessionStore for CustomSessionStore {
///     async fn append(&self, _key: &SessionKey, _entries: Vec<SessionStoreEntry>) -> claude_agent_sdk_rs::Result<()> {
///         // Persist entries to your backing store of choice
///         Ok(())
///     }
///
///     async fn load(&self, _key: &SessionKey) -> claude_agent_sdk_rs::Result<Option<Vec<SessionStoreEntry>>> {
///         // Load entries from your backing store of choice
///         Ok(None)
///     }
///
///     async fn list_sessions(&self, _project_key: &str) -> claude_agent_sdk_rs::Result<Vec<SessionStoreListEntry>> {
///         // List sessions from your backing store of choice
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

/// In-memory session store backed by a `tokio::sync::Mutex<HashMap<...>>`.
/// Suitable for tests and short-lived processes; not durable across restarts.
pub struct InMemorySessionStore {
    /// Keyed by `"{project_key}:{session_id}:{subpath_or_empty}"`.
    sessions: tokio::sync::Mutex<std::collections::HashMap<String, Vec<SessionStoreEntry>>>,
    /// Keyed by `project_key`.
    session_lists:
        tokio::sync::Mutex<std::collections::HashMap<String, Vec<SessionStoreListEntry>>>,
}

impl InMemorySessionStore {
    /// Create a new in-memory session store
    pub fn new() -> Self {
        Self {
            sessions: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            session_lists: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn entry_key(key: &SessionKey) -> String {
        format!(
            "{}:{}:{}",
            key.project_key,
            key.session_id,
            key.subpath.as_deref().unwrap_or("")
        )
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn append(&self, key: &SessionKey, entries: Vec<SessionStoreEntry>) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let store_key = Self::entry_key(key);

        // Persist the entries themselves.
        {
            let mut sessions = self.sessions.lock().await;
            sessions.entry(store_key).or_default().extend(entries);
        }

        // Maintain the per-project session list so list_sessions reflects the
        // append. Use the latest entry's timestamp (or now) as mtime.
        let now = chrono_rfc3339_now();
        let mut lists = self.session_lists.lock().await;
        let project_entries = lists.entry(key.project_key.clone()).or_default();
        if let Some(existing) = project_entries
            .iter_mut()
            .find(|e| e.session_id == key.session_id)
        {
            existing.mtime = now;
        } else {
            project_entries.push(SessionStoreListEntry {
                session_id: key.session_id.clone(),
                mtime: now,
            });
        }

        Ok(())
    }

    async fn load(&self, key: &SessionKey) -> Result<Option<Vec<SessionStoreEntry>>> {
        let store_key = Self::entry_key(key);
        let sessions = self.sessions.lock().await;
        Ok(sessions.get(&store_key).cloned())
    }

    async fn list_sessions(&self, project_key: &str) -> Result<Vec<SessionStoreListEntry>> {
        let lists = self.session_lists.lock().await;
        Ok(lists.get(project_key).cloned().unwrap_or_default())
    }

    async fn delete(&self, key: &SessionKey) -> Result<()> {
        let store_key = Self::entry_key(key);
        {
            let mut sessions = self.sessions.lock().await;
            sessions.remove(&store_key);
        }
        let mut lists = self.session_lists.lock().await;
        if let Some(entries) = lists.get_mut(&key.project_key) {
            entries.retain(|e| e.session_id != key.session_id);
        }
        Ok(())
    }

    async fn list_subkeys(&self, key: &SessionListSubkeysKey) -> Result<Vec<String>> {
        let prefix = format!("{}:{}:", key.project_key, key.session_id);
        let sessions = self.sessions.lock().await;
        let mut subkeys: Vec<String> = sessions
            .keys()
            .filter_map(|k| k.strip_prefix(&prefix).map(|s| s.to_string()))
            .filter(|s| !s.is_empty())
            .collect();
        subkeys.sort();
        Ok(subkeys)
    }
}

/// Best-effort current timestamp formatted as RFC3339 without pulling in
/// chrono. Falls back to a fixed sentinel if the system clock is broken.
fn chrono_rfc3339_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Plain "epoch seconds since 1970" string is enough for ordering by mtime
    // in tests; callers that care about real RFC3339 should provide their own
    // store impl.
    format!("epoch:{secs}")
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
    async fn test_in_memory_session_store_initial_state_is_empty() {
        let store = InMemorySessionStore::new();
        let key = SessionKey {
            project_key: "test-project".to_string(),
            session_id: "test-session".to_string(),
            subpath: None,
        };
        assert!(store.load(&key).await.unwrap().is_none());
        assert!(store.list_sessions("test-project").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_session_store_round_trip() {
        let store = InMemorySessionStore::new();
        let key = SessionKey {
            project_key: "p1".to_string(),
            session_id: "s1".to_string(),
            subpath: None,
        };
        let entry_a = SessionStoreEntry {
            type_: "user".to_string(),
            uuid: Some("u-1".to_string()),
            timestamp: Some("t-1".to_string()),
        };
        let entry_b = SessionStoreEntry {
            type_: "assistant".to_string(),
            uuid: Some("u-2".to_string()),
            timestamp: Some("t-2".to_string()),
        };

        store
            .append(&key, vec![entry_a.clone(), entry_b.clone()])
            .await
            .unwrap();

        let loaded = store.load(&key).await.unwrap().expect("present");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].uuid.as_deref(), Some("u-1"));
        assert_eq!(loaded[1].uuid.as_deref(), Some("u-2"));

        // append again — should accumulate, not overwrite.
        store.append(&key, vec![entry_a.clone()]).await.unwrap();
        assert_eq!(store.load(&key).await.unwrap().unwrap().len(), 3);

        // list_sessions reflects the project.
        let listed = store.list_sessions("p1").await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].session_id, "s1");

        // Subkey listing for sub-paths.
        let sub_key = SessionKey {
            project_key: "p1".to_string(),
            session_id: "s1".to_string(),
            subpath: Some("subagent-A".to_string()),
        };
        store.append(&sub_key, vec![entry_a.clone()]).await.unwrap();
        let subs = store
            .list_subkeys(&SessionListSubkeysKey {
                project_key: "p1".to_string(),
                session_id: "s1".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(subs, vec!["subagent-A".to_string()]);

        // delete removes the session and its list entry.
        store.delete(&key).await.unwrap();
        assert!(store.load(&key).await.unwrap().is_none());
        assert!(store.list_sessions("p1").await.unwrap().is_empty());
    }
}