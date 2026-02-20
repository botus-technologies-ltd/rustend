//! Two-Factor Authentication (2FA) models

use serde::{Deserialize, Serialize};
use database::utils::DbId;

/// Two-factor authentication method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TwoFactorMethod {
    Totp,      // Time-based One-Time Password (Google Authenticator, etc.)
    Sms,       // SMS verification codes
    Email,     // Email verification codes
    Backup,    // Backup codes
}

/// Two-factor authentication method - stores 2FA configuration for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorConfig {
    pub id: DbId,
    pub user_id: DbId,
    pub method: TwoFactorMethod,
    pub secret: Option<String>,  // Encrypted TOTP secret
    pub phone: Option<String>,  // For SMS method
    pub enabled: bool,
    pub verified_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl TwoFactorConfig {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Backup codes for 2FA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCode {
    pub id: DbId,
    pub user_id: DbId,
    pub code_hash: String,
    pub used: bool,
    pub used_at: Option<i64>,
    pub created_at: i64,
}

impl BackupCode {
    pub fn is_used(&self) -> bool {
        self.used
    }
}

/// Create 2FA configuration input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTwoFactorConfig {
    pub user_id: DbId,
    pub method: TwoFactorMethod,
    pub secret: Option<String>,
    pub phone: Option<String>,
}

/// Enable 2FA request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableTwoFactorRequest {
    pub method: TwoFactorMethod,
    pub code: String,  // Verification code
    pub phone: Option<String>,  // For SMS
}

/// Disable 2FA request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisableTwoFactorRequest {
    pub password: String,
    pub code: Option<String>,  // Optional 2FA code if enabled
}

/// Verify 2FA code request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyTwoFactorRequest {
    pub code: String,
}

/// 2FA challenge - temporary session for 2FA flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorChallenge {
    pub id: DbId,
    pub user_id: DbId,
    pub method: TwoFactorMethod,
    pub code_hash: String,
    pub expires_at: i64,
    pub created_at: i64,
}

impl TwoFactorChallenge {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }
}

/// Create 2FA challenge input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTwoFactorChallenge {
    pub user_id: DbId,
    pub method: TwoFactorMethod,
    pub code_hash: String,
    pub expires_in: i64,  // Usually 5 minutes
}
