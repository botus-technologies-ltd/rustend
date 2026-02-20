//! Magic Link / Passwordless Login models

use serde::{Deserialize, Serialize};
use database::utils::DbId;

/// Magic link model - passwordless authentication via email/SMS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLink {
    pub id: DbId,
    pub user_id: Option<DbId>,  // None until link is used
    pub token_hash: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub purpose: MagicLinkPurpose,
    pub expires_at: i64,
    pub used_at: Option<i64>,
    pub created_at: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl MagicLink {
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

/// Magic link purpose
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MagicLinkPurpose {
    Login,      // Passwordless login
    SignUp,     // Sign up with magic link
    EmailVerify, // Verify email
}

/// Create magic link input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMagicLink {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub purpose: MagicLinkPurpose,
    pub expires_in: i64,  // Usually 15-30 minutes
}

/// Request magic link for login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMagicLink {
    pub identifier: String,  // email or phone
    pub purpose: Option<MagicLinkPurpose>,
}

/// Verify and use magic link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyMagicLink {
    pub token: String,
}

/// Magic link response (contains the token to be sent to user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLinkResponse {
    pub token: String,
    pub expires_in: i64,
    pub purpose: MagicLinkPurpose,
}
