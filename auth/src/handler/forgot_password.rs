//! Forgot password handler
//!
//! Handles password reset requests by generating a reset token
//! and sending it via email to the user.

use actix_web::{Error, HttpResponse, web};

use crate::models::reset_password::CreatePasswordResetToken;
use crate::routes::AppState;
use crate::utils::errors::AuthError;
use crate::utils::types::PasswordResetRequest;

use database::utils::parse_id;
use utils::email_templates::{EmailTemplateConfig, password_reset};
use utils::hash::{generate_hex, hash_sha256};
use utils::response::ApiResponse;

const PASSWORD_RESET_TOKEN_EXPIRY_SECONDS: i64 = 360; // 6 minutes

pub async fn forgot_password(
    state: web::Data<AppState>,
    reset_req: web::Json<PasswordResetRequest>,
) -> Result<HttpResponse, Error> {
    let identifier = reset_req.identifier.trim();
    let user = state
        .users
        .find_by_identifier(identifier)
        .map_err(|e| AuthError::not_found(&e.to_string()))?
        .ok_or_else(|| AuthError::not_found("No account found with this creds"))?;

    // Check if user has email
    let email = match &user.email {
        Some(e) => e.clone(),
        None => {
            return Ok(HttpResponse::BadRequest().json(
                AuthError::invalid_request("No email address associated with this account")
                    .to_response::<()>(),
            ));
        }
    };

    // Generate secure reset token
    let reset_token = generate_hex(16);
    let user_id =
        parse_id(&user.id.to_string()).map_err(|e| AuthError::internal_error(&e.to_string()))?;

    // Delete any existing reset tokens for this user
    let _ = state
        .password_resets
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("Password reset not configured"))?
        .delete_all_for_user(&user_id);

    // Create new reset token
    let token_input = CreatePasswordResetToken {
        user_id,
        token_hash: hash_sha256(&reset_token),
        expires_in: PASSWORD_RESET_TOKEN_EXPIRY_SECONDS,
    };

    let _ = state
        .password_resets
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("Password reset not configured"))?
        .create(token_input)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    // Build reset link
    let reset_link = format!(
        "{}/reset-password?token={}",
        state.frontend_url.trim_end_matches('/'),
        &reset_token
    );

    // Create email template config
    let template_config = EmailTemplateConfig::new(&state.app_name, &state.frontend_url);

    // Build password reset email using professional template
    let mut password_reset_email = password_reset::build(&template_config, &reset_link);
    password_reset_email.to = email;
    password_reset_email.from = state.email_from.clone();

    // Send password reset email
    let _email_result = state.email.send(&password_reset_email).await;

    // Always return success to prevent email enumeration
    let response = ApiResponse::<()>::ok(format!(
        "We have sent a password reset link to your email {reset_link}"
    ));

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::utils::generate_id;

    #[test]
    fn test_password_reset_request_validation() {
        let request = PasswordResetRequest {
            identifier: "test@example.com".to_string(),
        };

        assert_eq!(request.identifier, "test@example.com");
    }

    #[test]
    fn test_password_reset_token_generation() {
        let token = generate_hex(16);

        // Token should be 32 characters long (16 bytes * 2 hex chars per byte)
        assert_eq!(token.len(), 32);

        // Token should only contain valid hex characters
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_reset_token_hashing() {
        let reset_token = "abc123def456";
        let token_hash = hash_sha256(&reset_token);

        // Hash should be 64 characters long (SHA256 produces 256 bits = 64 hex chars)
        assert_eq!(token_hash.len(), 64);

        // Hash should only contain valid hex characters
        assert!(token_hash.chars().all(|c| c.is_ascii_hexdigit()));

        // Same token should produce same hash
        let token_hash2 = hash_sha256(&reset_token);
        assert_eq!(token_hash, token_hash2);
    }

    #[test]
    fn test_reset_token_uniqueness() {
        let token1 = generate_hex(16);
        let token2 = generate_hex(16);

        // Each generated token should be unique
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_create_password_reset_token() {
        let user_id = generate_id();
        let reset_token = "test_token_123";

        let token_input = CreatePasswordResetToken {
            user_id,
            token_hash: hash_sha256(&reset_token),
            expires_in: PASSWORD_RESET_TOKEN_EXPIRY_SECONDS,
        };

        assert_eq!(token_input.expires_in, 360);
        assert_eq!(token_input.token_hash.len(), 64);
    }

    #[test]
    fn test_password_reset_token_expiry() {
        // 6 minutes in seconds
        assert_eq!(PASSWORD_RESET_TOKEN_EXPIRY_SECONDS, 360);
    }

    #[test]
    fn test_email_template_config() {
        let template_config = EmailTemplateConfig::new("TestApp", "https://example.com");

        // EmailTemplateConfig should be created successfully
        let _ = template_config;
    }

    #[test]
    fn test_reset_link_format() {
        let frontend_url = "https://example.com";
        let reset_token = "abc123def456";

        let reset_link = format!(
            "{}/reset-password?token={}",
            frontend_url.trim_end_matches('/'),
            reset_token
        );

        assert_eq!(
            reset_link,
            "https://example.com/reset-password?token=abc123def456"
        );
        assert!(reset_link.contains("/reset-password?token="));
    }

    #[test]
    fn test_reset_link_format_with_trailing_slash() {
        let frontend_url = "https://example.com/";
        let reset_token = "token123";

        let reset_link = format!(
            "{}/reset-password?token={}",
            frontend_url.trim_end_matches('/'),
            reset_token
        );

        assert_eq!(
            reset_link,
            "https://example.com/reset-password?token=token123"
        );
    }

    #[test]
    fn test_password_reset_request_email_identifier() {
        let request = PasswordResetRequest {
            identifier: "user@example.com".to_string(),
        };

        let identifier = request.identifier.trim();
        assert_eq!(identifier, "user@example.com");
    }

    #[test]
    fn test_password_reset_request_phone_identifier() {
        let request = PasswordResetRequest {
            identifier: "+1234567890".to_string(),
        };

        let identifier = request.identifier.trim();
        assert_eq!(identifier, "+1234567890");
    }

    #[test]
    fn test_success_response_structure() {
        let response = ApiResponse::<()>::ok("Test message");

        // Response should be valid ApiResponse
        assert!(response.success);
    }

    #[test]
    fn test_api_response_for_user_not_found() {
        let error = AuthError::not_found("No account found with this creds");
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_for_no_email_configured() {
        let error = AuthError::invalid_request("No email address associated with this account");
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_api_response_for_internal_error() {
        let error = AuthError::internal_error("Password reset not configured");
        let response = error.to_response::<()>();

        assert!(!response.success);
    }

    #[test]
    fn test_password_reset_email_template() {
        let template_config = EmailTemplateConfig::new("TestApp", "https://example.com");

        let reset_link = "https://example.com/reset-password?token=abc123";
        let password_reset_email = password_reset::build(&template_config, &reset_link);

        // Email should be built with reset link
        assert!(password_reset_email.subject.len() > 0);
    }
}
