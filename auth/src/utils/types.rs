//! Authentication types module
//!
//! Provides API types for authentication operations including
//! user data, requests, responses, and tokens.
//!
//! These are the types used in HTTP requests/responses (API layer).
//! For database models, see `crate::models`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utils::response::{ResponseMeta};

use crate::models::verification::VerificationMedium;

/// User without sensitive data (for public API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id:          String,
    pub username:    String,
    pub first_name:  Option<String>,
    pub last_name:   Option<String>,
    pub is_verified: bool,
    pub created_at:  DateTime<Utc>,
}

// ============================================
// Sign Up Types
// ============================================

/// Sign up request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpRequest {
    pub email:      Option<String>,
    pub phone:      Option<String>,
    pub password:   String,
}

/// Sign up response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpResponseData {
    pub user_id: String,
    pub email:   Option<String>,
}

// ============================================
// Sign In Types
// ============================================

/// Sign in request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInRequest {
    pub identifier: String, // email, phone, or username
    pub password:   String,
}

/// Sign in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInResponse {
    pub user:          UserPublic,
    pub access_token:  String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub expires_in:    DateTime<Utc>,
}


// ============================================
// Password Reset Types
// ============================================

/// Request password reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub identifier: String, // email or phone
}

/// Confirm password reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
}

/// Change password (when logged in)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

// ============================================
// Verification Types
// ============================================

/// Send verification code request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVerificationRequest {
    pub medium: VerificationMedium, // email or phone
}

/// Verify code request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCodeRequest {
    pub code: String,
    pub medium: VerificationMedium,
}

// ============================================
// Token Types
// ============================================

/// Token pair response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token:  String,
    pub refresh_token: String,
    pub expires_in:    DateTime<Utc>,
}

/// Token refresh request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

// ============================================
// Session Types (API layer)
// ============================================

/// Session info (for API responses - no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id:         String,
    pub user_id:    String,
    pub device:     Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Active sessions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub sessions: Vec<Session>,
    pub current_session_id: String,
}

// ============================================
// Request/Response Helpers
// ============================================

/// Paginated users response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersListResponse {
    pub users: Vec<UserPublic>,
    pub meta:  ResponseMeta,
}


/// Status response (for simple boolean responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub success: bool,
    pub message: String,
}

impl StatusResponse {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
        }
    }
}
