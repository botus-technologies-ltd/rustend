//! Authentication types module
//! 
//! Provides API types for authentication operations including
//! user data, requests, responses, and tokens.
//! 
//! These are the types used in HTTP requests/responses (API layer).
//! For database models, see `crate::models`.

use serde::{Deserialize, Serialize};
use utils::response::{ApiResponse, ResponseMeta};

use crate::models::verification::VerificationMedium;

/// User type - for API responses (excludes sensitive fields like password)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl User {
    /// Get display name (prefer first_name or username)
    pub fn display_name(&self) -> &str {
        self.first_name
            .as_deref()
            .unwrap_or(self.username.as_str())
    }

    /// Get primary identifier (email, phone, or username)
    pub fn primary_id(&self) -> &str {
        self.email
            .as_deref()
            .or(self.phone.as_deref())
            .unwrap_or(&self.username)
    }
}

/// User without sensitive data (for public API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_verified: bool,
    pub created_at: i64,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
            is_verified: user.is_verified,
            created_at: user.created_at,
        }
    }
}

// ============================================
// Sign Up Types
// ============================================

/// Sign up request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub username: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Sign up response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpResponse {
    pub user: UserPublic,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_required: Option<bool>,
}

impl SignUpResponse {
    pub fn success(user: UserPublic) -> ApiResponse<Self> {
        ApiResponse::success_data(
            "Account created successfully. Please verify your email.",
            Self {
                user,
                message: "Account created successfully".to_string(),
                verification_required: Some(true),
            },
        )
    }

    pub fn no_verification(user: UserPublic) -> ApiResponse<Self> {
        ApiResponse::success_data(
            "Account created successfully",
            Self {
                user,
                message: "Account created successfully".to_string(),
                verification_required: Some(false),
            },
        )
    }
}

// ============================================
// Sign In Types
// ============================================

/// Sign in request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInRequest {
    pub identifier: String,  // email, phone, or username
    pub password: String,
}

/// Sign in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInResponse {
    pub user: UserPublic,
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub expires_in: i64,
}

impl SignInResponse {
    pub fn success(user: UserPublic, access_token: String) -> ApiResponse<Self> {
        ApiResponse::success_data(
            "Signed in successfully",
            Self {
                user,
                access_token,
                refresh_token: None,
                expires_in: 3600, // 1 hour
            },
        )
    }

    pub fn with_refresh(user: UserPublic, access_token: String, refresh_token: String) -> ApiResponse<Self> {
        ApiResponse::success_data(
            "Signed in successfully",
            Self {
                user,
                access_token,
                refresh_token: Some(refresh_token),
                expires_in: 3600,
            },
        )
    }
}

// ============================================
// Password Reset Types
// ============================================

/// Request password reset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub identifier: String,  // email or phone
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
    pub medium: VerificationMedium,  // email or phone
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
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
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
    pub id: String,
    pub user_id: String,
    pub device: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
}

/// Active sessions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsResponse {
    pub sessions: Vec<Session>,
    pub current_session_id: String,
}

// ============================================
// Validation Types
// ============================================

/// Field validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

/// Validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrors {
    pub errors: Vec<FieldError>,
}

impl ValidationErrors {
    pub fn single(field: &str, message: &str) -> Self {
        Self {
            errors: vec![FieldError {
                field: field.to_string(),
                message: message.to_string(),
            }],
        }
    }
}

// ============================================
// Request/Response Helpers
// ============================================

/// Paginated users response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersListResponse {
    pub users: Vec<UserPublic>,
    pub meta: ResponseMeta,
}

/// Generic auth response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> AuthResponse<T> {
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }
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
