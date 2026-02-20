//! Password reset models

use serde::{Deserialize, Serialize};
use database::utils::DbId;

/// Password reset token model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetTokenModel {
    pub id: DbId,
    pub user_id: DbId,
    pub token_hash: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub used_at: Option<i64>,
}

impl PasswordResetTokenModel {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }

    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used()
    }
}

/// Create password reset token input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePasswordResetToken {
    pub user_id: DbId,
    pub token_hash: String,
    pub expires_in: i64,
}
