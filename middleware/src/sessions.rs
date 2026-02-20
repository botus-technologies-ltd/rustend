//! Session Management types

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub cookie_name: String,
    pub expire_seconds: i64,
    pub secure: bool,
    pub http_only: bool,
    pub path: String,
}

impl Default for SessionConfig {
    fn default() -> Self { Self { cookie_name: "session_id".to_string(), expire_seconds: 3600, secure: true, http_only: true, path: "/".to_string() } }
}

impl SessionConfig { pub fn new() -> Self { Self::default() } }

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub email: Option<String>,
    pub created_at: i64,
    pub last_accessed: i64,
    #[serde(default)] pub data: HashMap<String, String>,
}

impl SessionData {
    pub fn new(user_id: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self { user_id: user_id.into(), email: None, created_at: now, last_accessed: now, data: HashMap::new() }
    }
}

/// Session store
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl SessionStore {
    pub fn new(_config: SessionConfig) -> Self { Self { sessions: Arc::new(RwLock::new(HashMap::new())) } }
    pub fn create(&self, user_id: impl Into<String>) -> (String, SessionData) {
        let id = format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let session = SessionData::new(user_id);
        let clone = session.clone();
        self.sessions.write().insert(id.clone(), session);
        (id, clone)
    }
    pub fn get(&self, id: &str) -> Option<SessionData> { self.sessions.read().get(id).cloned() }
    pub fn delete(&self, id: &str) -> bool { self.sessions.write().remove(id).is_some() }
}
