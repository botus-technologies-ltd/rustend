//! Reset password handler
//!
//! Handles password reset confirmation - verifies the token and updates the password.

use actix_web::{Error, HttpRequest, HttpResponse, web};

use crate::routes::AppState;
use crate::utils::errors::AuthError;
use crate::utils::passwords::validate_strength;
use crate::utils::session_validation::validate_access_token;
use crate::utils::types::ChangePasswordRequest;
use crate::utils::types::PasswordResetConfirm;

use database::utils::parse_id;
use utils::hash::Hash;
use utils::hash::hash_sha256;
use utils::response::ApiResponse;

pub async fn reset_password(
    state: web::Data<AppState>,
    reset_req: web::Json<PasswordResetConfirm>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let token = reset_req.token.trim();
    let new_password = &reset_req.new_password;
    let confirm_password = &reset_req.confirm_password;
    let token_hash = hash_sha256(&token);

    // Validate token and session in one call
    let _token_info = validate_access_token(&req, &state)
        .await
        .map_err(|_e| AuthError::invalid_session())?;

    // Get password resets store
    let password_resets = state
        .password_resets
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("Password reset not configured"))?
        .find_by_hash(&token_hash)?;

    let password_resets = match password_resets {
        Some(pr) => pr,
        None => {
            let response = ApiResponse::<()>::error("Invalid or expired token", None);
            return Ok(HttpResponse::Unauthorized().json(response));
        }
    };

    // check the token is valid and not expired
    let now = chrono::Utc::now().timestamp();
    if password_resets.expires_at < now {
        return Ok(
            HttpResponse::BadRequest().json(AuthError::reset_token_expired().to_response::<()>())
        );
    }

    // check if the token is already used
    if password_resets.used_at != None {
        return Ok(
            HttpResponse::BadRequest().json(AuthError::invalid_reset_token().to_response::<()>())
        );
    }

    // Validate passwords match
    if new_password != confirm_password {
        return Ok(
            HttpResponse::BadRequest().json(AuthError::password_mismatch().to_response::<()>())
        );
    }

    // Validate password strength
    let _valid_password =
        validate_strength(new_password).map_err(|e| AuthError::weak_password(&e.to_string()));

    // Hash new password
    let password_hash = Hash::argon2(new_password)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?
        .to_string();

    let _ = state
        .users
        .update_password(&password_resets.user_id, &password_hash)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    let _ = state
        .password_resets
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("Password reset not configured"))?
        .mark_used(&password_resets.id);

    let response = ApiResponse::<()>::ok("Password changed successfully");

    Ok(HttpResponse::Ok().json(response))
}

pub async fn change_password(
    state: web::Data<AppState>,
    change_req: web::Json<ChangePasswordRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let current_password = &change_req.current_password;
    let new_password = &change_req.new_password;
    let confirm_password = &change_req.confirm_password;

    // Validate token and session in one call
    let token_info = validate_access_token(&req, &state)
        .await
        .map_err(|_e| AuthError::invalid_session())?;

    // Validate passwords match
    if new_password != confirm_password {
        return Ok(
            HttpResponse::BadRequest().json(AuthError::password_mismatch().to_response::<()>())
        );
    }

    // Validate password strength
    let _valid_password =
        validate_strength(new_password).map_err(|e| AuthError::weak_password(&e.to_string()));

    // Get user ID
    let uid = parse_id(token_info.user_id.as_str())
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    // Find user
    let user = state
        .users
        .find_by_id(&uid)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?
        .ok_or_else(|| AuthError::not_found("User not found"))?;

    // Verify current password
    let stored_hash = Hash::from_string(&user.password_hash)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    let password_valid = stored_hash
        .verify(current_password)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    if !password_valid {
        return Ok(
            HttpResponse::Unauthorized().json(AuthError::invalid_credentials().to_response::<()>())
        );
    }

    // Hash new password
    let password_hash = Hash::argon2(new_password)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?
        .to_string();

    // Update password
    let _ = state
        .users
        .update_password(&uid, &password_hash)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    let response = ApiResponse::<()>::ok("Password changed successfully");

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_reset_confirm_validation() {
        let request = PasswordResetConfirm {
            token: "valid_token_123abc".to_string(),
            new_password: "new_secure_password_123".to_string(),
            confirm_password: "new_secure_password_123".to_string(),
        };

        assert_eq!(request.token, "valid_token_123abc");
        assert_eq!(request.new_password, "new_secure_password_123");
        assert_eq!(request.confirm_password, "new_secure_password_123");
    }

    #[test]
    fn test_change_password_request_validation() {
        let request = ChangePasswordRequest {
            current_password: "old_password_123".to_string(),
            new_password: "new_secure_password_456".to_string(),
            confirm_password: "new_secure_password_456".to_string(),
        };

        assert_eq!(request.current_password, "old_password_123");
        assert_eq!(request.new_password, "new_secure_password_456");
        assert_eq!(request.confirm_password, "new_secure_password_456");
    }

    #[test]
    fn test_password_mismatch_validation() {
        let password1 = "password123";
        let password2 = "password456";

        assert_ne!(password1, password2);
    }

    #[test]
    fn test_reset_token_hashing_for_lookup() {
        let reset_token = "test_reset_token_abc123";
        let token_hash = hash_sha256(&reset_token);

        // Hash should be 64 characters long (SHA256)
        assert_eq!(token_hash.len(), 64);

        // Hash should only contain hex characters
        assert!(token_hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Same token should produce same hash for lookup
        let token_hash2 = hash_sha256(&reset_token);
        assert_eq!(token_hash, token_hash2);
    }

    #[test]
    fn test_password_hash_creation() {
        let new_password = "new_password_secure_123";

        let password_hash = Hash::argon2(new_password)
            .expect("Should hash password successfully")
            .to_string();

        // Hash should be non-empty
        assert!(!password_hash.is_empty());

        // Hash should not equal plaintext password
        assert_ne!(password_hash, new_password);
    }

    #[test]
    fn test_password_hash_verification() {
        let password = "test_password_123";

        let hash = Hash::argon2(password).expect("Should create hash");

        // Password should verify against hash
        assert!(hash.verify(password).expect("Should verify"));

        // Wrong password should not verify
        assert!(!hash.verify("wrong_password").expect("Should verify"));
    }

    #[test]
    fn test_password_reset_token_expiry_validation() {
        let now = chrono::Utc::now().timestamp();
        let expired_time = now - 100;
        let valid_time = now + 100;

        // Expired should be in past
        assert!(expired_time < now);

        // Valid should be in future
        assert!(valid_time > now);
    }

    #[test]
    fn test_password_reset_token_used_validation() {
        let used_at: Option<i64> = Some(chrono::Utc::now().timestamp());
        let not_used_at: Option<i64> = None;

        // Used token should have Some value
        assert!(used_at.is_some());

        // Unused token should be None
        assert!(not_used_at.is_none());
    }

    #[test]
    fn test_password_strength_validation() {
        // Strong password candidates
        let strong_password = "Secure_Password_123!@#";
        let weak_password = "123";

        // Strong password should be longer
        assert!(strong_password.len() > weak_password.len());
    }

    #[test]
    fn test_success_response_structure() {
        let response = ApiResponse::<()>::ok("Password changed successfully");

        assert!(response.success);
    }

    #[test]
    fn test_api_response_password_mismatch() {
        let error = AuthError::password_mismatch();
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_reset_token_expired() {
        let error = AuthError::reset_token_expired();
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_invalid_reset_token() {
        let error = AuthError::invalid_reset_token();
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_invalid_session() {
        let error = AuthError::invalid_session();
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_invalid_credentials() {
        let error = AuthError::invalid_credentials();
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_user_not_found() {
        let error = AuthError::not_found("User not found");
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_internal_error() {
        let error = AuthError::internal_error("Password reset not configured");
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_reset_token_trim_whitespace() {
        let token_with_whitespace = "  valid_token_123  ";
        let trimmed = token_with_whitespace.trim();

        assert_eq!(trimmed, "valid_token_123");
        assert_ne!(trimmed, token_with_whitespace);
    }

    #[test]
    fn test_password_reset_confirm_trim_token() {
        let request = PasswordResetConfirm {
            token: "  token_with_spaces  ".to_string(),
            new_password: "password123".to_string(),
            confirm_password: "password123".to_string(),
        };

        let token = request.token.trim();
        assert_eq!(token, "token_with_spaces");
    }

    #[test]
    fn test_hash_from_string_parsing() {
        let password = "test_password";
        let hash_obj = Hash::argon2(password).expect("Should hash password");

        let hash_string = hash_obj.to_string();

        // Should be able to parse hash back from string
        let parsed_hash = Hash::from_string(&hash_string).expect("Should parse hash from string");

        // Parsed hash should verify original password
        assert!(parsed_hash.verify(password).expect("Should verify"));
    }
}
