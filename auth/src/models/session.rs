//! Session model

use serde::{Deserialize, Serialize};
use database::utils::DbId;

/// Session model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: DbId,
    pub user_id: DbId,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub device: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_used_at: i64,
    pub is_revoked: bool,
}

impl SessionModel {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked
    }

    pub fn update_last_used(&mut self) {
        self.last_used_at = chrono::Utc::now().timestamp();
    }
}

/// Create session input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSession {
    pub user_id: DbId,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub device: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_in: i64,
}

/// Refresh token model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenModel {
    pub id: DbId,
    pub user_id: DbId,
    pub token_hash: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub revoked: bool,
    pub revoked_at: Option<i64>,
    pub replaced_by: Option<String>,
}

impl RefreshTokenModel {
    pub fn is_valid(&self) -> bool {
        !self.revoked && chrono::Utc::now().timestamp() < self.expires_at
    }
}

/// Create refresh token input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRefreshToken {
    pub user_id: DbId,
    pub token_hash: String,
    pub expires_in: i64,
}

/// Login attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttempt {
    pub id: DbId,
    pub user_id: Option<DbId>,
    pub identifier: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub created_at: i64,
}

/// Rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub identifier: String,
    pub action: String,
    pub count: i32,
    pub window_start: i64,
    pub window_duration: i64,
}

impl RateLimit {
    pub fn new(identifier: &str, action: &str, window_duration: i64) -> Self {
        Self {
            identifier: identifier.to_string(),
            action: action.to_string(),
            count: 1,
            window_start: chrono::Utc::now().timestamp(),
            window_duration,
        }
    }

    pub fn is_exceeded(&self, max_attempts: i32) -> bool {
        self.count >= max_attempts
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn should_reset(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.window_start > self.window_duration
    }
}
