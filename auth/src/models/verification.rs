//! Verification models

use serde::{Deserialize, Serialize};
use database::utils::DbId;

/// Verification code model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCodeModel {
    pub id: DbId,
    pub user_id: DbId,
    pub code_hash: String,
    pub medium: VerificationMedium,
    pub purpose: VerificationPurpose,
    pub attempts: i32,
    pub created_at: i64,
    pub expires_at: i64,
    pub verified_at: Option<i64>,
}

impl VerificationCodeModel {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }

    pub fn is_verified(&self) -> bool {
        self.verified_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_verified()
    }
}

/// Verification medium
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VerificationMedium {
    Email,
    Phone,
}

/// Verification purpose
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationPurpose {
    SignUp,
    EmailChange,
    PhoneChange,
    PasswordReset,
    TwoFactor,
}

/// Create verification code input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVerificationCode {
    pub user_id: DbId,
    pub code_hash: String,
    pub medium: VerificationMedium,
    pub purpose: VerificationPurpose,
    pub expires_in: i64,
}
