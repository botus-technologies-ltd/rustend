//! User model

use serde::{Deserialize, Serialize};
use database::utils::DbId;

/// User model - stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: DbId,
    pub email: Option<String>,
    pub password_hash: String,

    pub phone: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,

    pub is_active: bool,
    pub is_verified: bool,
    pub failed_login_attempts: i32,
    pub locked_until: Option<i64>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
    pub last_login_at: Option<i64>,
}

impl User {
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            return locked_until > chrono::Utc::now().timestamp();
        }
        false
    }

    pub fn can_attempt_login(&self) -> bool {
        !self.is_active || self.is_locked()
    }

    pub fn record_failed_attempt(&mut self, lock_threshold: i32, lock_duration_secs: i64) {
        self.failed_login_attempts += 1;
        
        if self.failed_login_attempts >= lock_threshold {
            self.locked_until = Some(chrono::Utc::now().timestamp() + lock_duration_secs);
        }
    }

    pub fn reset_failed_attempts(&mut self) {
        self.failed_login_attempts = 0;
        self.locked_until = None;
    }
}

/// User creation input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub email:      Option<String>,
    pub phone:      Option<String>,
    pub username:   Option<String>,
    pub password:   String,
    pub first_name: Option<String>,
    pub last_name:  Option<String>,
}

/// User update input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserInput {
    pub email:       Option<String>,
    pub phone:       Option<String>,
    pub username:    Option<String>,
    pub first_name:  Option<String>,
    pub last_name:   Option<String>,
    pub is_active:   Option<bool>,
    pub is_verified: Option<bool>,
}
