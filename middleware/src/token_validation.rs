//! Token validation utilities
//!
//! Provides helpers for validating JWT tokens and extracting token information from requests.
//! This is middleware-specific functionality for handling token validation across handlers.

use crate::jwt::JwtClaims;
use serde_json::json;
use utils::hash::hash_sha256;

/// Response structure for invalid token errors
#[derive(Debug)]
pub struct TokenValidationError {
    pub message: String,
    pub status_code: u16,
}

impl std::fmt::Display for TokenValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TokenValidationError {}

impl TokenValidationError {
    pub fn unauthorized(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            status_code: 401,
        }
    }

    pub fn bad_request(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            status_code: 400,
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
            status_code: 500,
        }
    }

    pub fn to_response(&self) -> serde_json::Value {
        json!({
            "success": false,
            "message": self.message,
            "error": self.message
        })
    }
}

/// Token info extracted from request
#[derive(Debug, Clone)]
pub struct ExtractedTokenInfo {
    pub raw_token: String,
    pub token_hash: String,
    pub user_id: String,
}

impl ExtractedTokenInfo {
    /// Create from raw token and user_id
    pub fn new(raw_token: String, user_id: String) -> Self {
        let token_hash = hash_sha256(&raw_token);
        Self {
            raw_token,
            token_hash,
            user_id,
        }
    }
}

pub fn validate_token_extraction(
    req: &actix_web::HttpRequest,
) -> Result<ExtractedTokenInfo, TokenValidationError> {
    // Get token info from request
    let token_info = req
        .token_info()
        .ok_or_else(|| TokenValidationError::unauthorized("Invalid or missing token"))?;

    Ok(ExtractedTokenInfo::new(
        token_info.token.clone(),
        token_info.claims.sub.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extracted_token_info_creation() {
        let token = "test_token_123".to_string();
        let user_id = "user_456".to_string();

        let info = ExtractedTokenInfo::new(token.clone(), user_id.clone());

        assert_eq!(info.raw_token, token);
        assert_eq!(info.user_id, user_id);
        assert_eq!(info.token_hash.len(), 64); // SHA256 produces 64 char hex
    }

    #[test]
    fn test_token_validation_error_unauthorized() {
        let err = TokenValidationError::unauthorized("test message");
        assert_eq!(err.status_code, 401);
        assert_eq!(err.message, "test message");
    }

    #[test]
    fn test_token_validation_error_display() {
        let err = TokenValidationError::bad_request("bad request");
        assert_eq!(format!("{}", err), "bad request");
    }
}
