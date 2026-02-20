//! Authentication errors module
//! 
//! Provides comprehensive error types for authentication operations including
//! sign up, sign in, password reset, and general auth errors.
//! 
//! These errors are designed to work with utils::response::ApiResponse for
//! consistent API error responses.

use serde::{Deserialize, Serialize};

/// Main authentication error enum
/// All auth operations should return this or wrap it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthError {
    pub code: AuthErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AuthError {
    /// Create a new auth error with code and message
    pub fn new(code: AuthErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// Create with additional details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Convert to utils response format
    pub fn to_response<T>(&self) -> utils::response::ApiResponse<T> {
        let api_error = utils::response::ApiError {
            code: self.code.to_string(),
            details: self.details.clone(),
        };
        
        utils::response::ApiResponse::error(&self.message, Some(api_error))
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AuthError {}

impl actix_web::ResponseError for AuthError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code())
            .json(self.to_response::<()>())
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.code {
            AuthErrorCode::InternalError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AuthErrorCode::InvalidRequest => actix_web::http::StatusCode::BAD_REQUEST,
            AuthErrorCode::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            AuthErrorCode::Forbidden => actix_web::http::StatusCode::FORBIDDEN,
            AuthErrorCode::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            AuthErrorCode::Conflict => actix_web::http::StatusCode::CONFLICT,
            AuthErrorCode::EmailAlreadyExists 
            | AuthErrorCode::PhoneAlreadyExists 
            | AuthErrorCode::UsernameAlreadyExists => actix_web::http::StatusCode::CONFLICT,
            AuthErrorCode::InvalidCredentials 
            | AuthErrorCode::AccountLocked 
            | AuthErrorCode::AccountNotVerified 
            | AuthErrorCode::TooManyAttempts => actix_web::http::StatusCode::UNAUTHORIZED,
            AuthErrorCode::InvalidResetToken 
            | AuthErrorCode::ResetTokenExpired => actix_web::http::StatusCode::BAD_REQUEST,
            AuthErrorCode::InvalidVerificationCode 
            | AuthErrorCode::VerificationCodeExpired => actix_web::http::StatusCode::BAD_REQUEST,
            AuthErrorCode::SessionExpired 
            | AuthErrorCode::InvalidSession 
            | AuthErrorCode::SessionRevoked => actix_web::http::StatusCode::UNAUTHORIZED,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Error codes for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthErrorCode {
    // General errors
    InternalError,
    InvalidRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    
    // Sign up errors
    EmailAlreadyExists,
    PhoneAlreadyExists,
    UsernameAlreadyExists,
    InvalidEmail,
    InvalidPhoneNumber,
    InvalidUsername,
    WeakPassword,
    
    // Sign in errors
    InvalidCredentials,
    AccountLocked,
    AccountNotVerified,
    TooManyAttempts,
    
    // Password reset errors
    InvalidResetToken,
    ResetTokenExpired,
    InvalidPassword,
    PasswordMismatch,
    
    // Verification errors
    InvalidVerificationCode,
    VerificationCodeExpired,
    AlreadyVerified,
    
    // Session errors
    SessionExpired,
    InvalidSession,
    SessionRevoked,
}

impl std::fmt::Display for AuthErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AuthErrorCode::InternalError => "INTERNAL_ERROR",
            AuthErrorCode::InvalidRequest => "INVALID_REQUEST",
            AuthErrorCode::Unauthorized => "UNAUTHORIZED",
            AuthErrorCode::Forbidden => "FORBIDDEN",
            AuthErrorCode::NotFound => "NOT_FOUND",
            AuthErrorCode::Conflict => "CONFLICT",
            AuthErrorCode::EmailAlreadyExists => "EMAIL_ALREADY_EXISTS",
            AuthErrorCode::PhoneAlreadyExists => "PHONE_ALREADY_EXISTS",
            AuthErrorCode::UsernameAlreadyExists => "USERNAME_ALREADY_EXISTS",
            AuthErrorCode::InvalidEmail => "INVALID_EMAIL",
            AuthErrorCode::InvalidPhoneNumber => "INVALID_PHONE_NUMBER",
            AuthErrorCode::InvalidUsername => "INVALID_USERNAME",
            AuthErrorCode::WeakPassword => "WEAK_PASSWORD",
            AuthErrorCode::InvalidCredentials => "INVALID_CREDENTIALS",
            AuthErrorCode::AccountLocked => "ACCOUNT_LOCKED",
            AuthErrorCode::AccountNotVerified => "ACCOUNT_NOT_VERIFIED",
            AuthErrorCode::TooManyAttempts => "TOO_MANY_ATTEMPTS",
            AuthErrorCode::InvalidResetToken => "INVALID_RESET_TOKEN",
            AuthErrorCode::ResetTokenExpired => "RESET_TOKEN_EXPIRED",
            AuthErrorCode::InvalidPassword => "INVALID_PASSWORD",
            AuthErrorCode::PasswordMismatch => "PASSWORD_MISMATCH",
            AuthErrorCode::InvalidVerificationCode => "INVALID_VERIFICATION_CODE",
            AuthErrorCode::VerificationCodeExpired => "VERIFICATION_CODE_EXPIRED",
            AuthErrorCode::AlreadyVerified => "ALREADY_VERIFIED",
            AuthErrorCode::SessionExpired => "SESSION_EXPIRED",
            AuthErrorCode::InvalidSession => "INVALID_SESSION",
            AuthErrorCode::SessionRevoked => "SESSION_REVOKED",
        };
        write!(f, "{}", s)
    }
}

/// Helper functions to create common auth errors
impl AuthError {
    // General errors
    pub fn internal_error(msg: &str) -> Self {
        Self::new(AuthErrorCode::InternalError, msg)
    }
    
    pub fn unauthorized(msg: &str) -> Self {
        Self::new(AuthErrorCode::Unauthorized, msg)
    }
    
    pub fn forbidden(msg: &str) -> Self {
        Self::new(AuthErrorCode::Forbidden, msg)
    }
    
    pub fn not_found(msg: &str) -> Self {
        Self::new(AuthErrorCode::NotFound, msg)
    }
    
    pub fn conflict(msg: &str) -> Self {
        Self::new(AuthErrorCode::Conflict, msg)
    }
    
    pub fn invalid_request(msg: &str) -> Self {
        Self::new(AuthErrorCode::InvalidRequest, msg)
    }

    // Sign up errors
    pub fn email_already_exists(email: &str) -> Self {
        Self::new(
            AuthErrorCode::EmailAlreadyExists, 
            format!("Email '{}' is already registered", email)
        )
    }
    
    pub fn phone_already_exists(phone: &str) -> Self {
        Self::new(
            AuthErrorCode::PhoneAlreadyExists,
            format!("Phone number '{}' is already registered", phone)
        )
    }
    
    pub fn username_already_exists(username: &str) -> Self {
        Self::new(
            AuthErrorCode::UsernameAlreadyExists,
            format!("Username '{}' is already taken", username)
        )
    }
    
    pub fn invalid_email(email: &str) -> Self {
        Self::new(
            AuthErrorCode::InvalidEmail,
            format!("Invalid email format: {}", email)
        )
    }
    
    pub fn invalid_username(username: &str) -> Self {
        Self::new(
            AuthErrorCode::InvalidUsername,
            format!("Invalid username: {}", username)
        )
    }
    
    pub fn weak_password() -> Self {
        Self::new(
            AuthErrorCode::WeakPassword,
            "Password must be at least 8 characters with uppercase, lowercase, and numbers"
        )
    }

    // Sign in errors
    pub fn invalid_credentials() -> Self {
        Self::new(
            AuthErrorCode::InvalidCredentials,
            "Invalid email/username or password"
        )
    }
    
    pub fn account_locked() -> Self {
        Self::new(
            AuthErrorCode::AccountLocked,
            "Your account has been locked. Please try again later or contact support."
        )
    }
    
    pub fn account_not_verified() -> Self {
        Self::new(
            AuthErrorCode::AccountNotVerified,
            "Please verify your email address to continue"
        )
    }
    
    pub fn too_many_attempts() -> Self {
        Self::new(
            AuthErrorCode::TooManyAttempts,
            "Too many login attempts. Please try again later."
        )
    }

    // Password reset errors
    pub fn invalid_reset_token() -> Self {
        Self::new(
            AuthErrorCode::InvalidResetToken,
            "Invalid password reset token"
        )
    }
    
    pub fn reset_token_expired() -> Self {
        Self::new(
            AuthErrorCode::ResetTokenExpired,
            "Password reset token has expired. Please request a new one."
        )
    }
    
    pub fn password_mismatch() -> Self {
        Self::new(
            AuthErrorCode::PasswordMismatch,
            "Password and confirmation password do not match"
        )
    }

    // Verification errors
    pub fn invalid_verification_code() -> Self {
        Self::new(
            AuthErrorCode::InvalidVerificationCode,
            "Invalid verification code"
        )
    }
    
    pub fn verification_code_expired() -> Self {
        Self::new(
            AuthErrorCode::VerificationCodeExpired,
            "Verification code has expired. Please request a new one."
        )
    }
    
    pub fn already_verified() -> Self {
        Self::new(
            AuthErrorCode::AlreadyVerified,
            "Account is already verified"
        )
    }

    // Session errors
    pub fn session_expired() -> Self {
        Self::new(
            AuthErrorCode::SessionExpired,
            "Your session has expired. Please log in again."
        )
    }
    
    pub fn invalid_session() -> Self {
        Self::new(
            AuthErrorCode::InvalidSession,
            "Invalid session"
        )
    }
}

/// Result type for auth operations
pub type AuthResult<T> = Result<T, AuthError>;
