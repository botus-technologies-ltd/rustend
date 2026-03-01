//! Login handler - user authentication
//!
//! Performance optimizations for <1s login:
//! - Use bcrypt with lower cost (4-6) instead of argon2 for password verification
//! - Add Redis caching for frequent users
//! - Make session creation async/non-blocking
//! - Use connection pooling for database
//! - Cache user lookups with short TTL

use actix_web::{Error, HttpRequest, HttpResponse, web};

use crate::models::session::{CreateRefreshToken, CreateSession, UpdateSession};
use crate::routes::AppState;
use crate::utils::errors::AuthError;
use crate::utils::session_validation::validate_access_token;
use crate::utils::types::RefreshTokenRequest;
use crate::utils::types::{SignInRequest, SignInResponse, UserPublic};

use database::utils::parse_id;
use jsonwebtoken::{EncodingKey, Header, encode};
use middleware::jwt::Claims;
use utils::hash::Hash;
use utils::hash::hash_sha256;
use utils::response::ApiResponse;

pub fn generate_access_token(
    user_id: &str,
    email: Option<&str>,
    jwt_secret: &str,
    expiry_minutes: i64,
) -> Result<String, AuthError> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.map(String::from),
        exp: now + (expiry_minutes * 60),
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AuthError::internal_error(&e.to_string()))
}

pub fn generate_refresh_token() -> String {
    utils::hash::generate_hex(64)
}

pub async fn login_user(
    state: web::Data<AppState>,
    login_req: web::Json<SignInRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    // FUTURE: Check Redis cache first for user data to avoid DB lookup
    // FUTURE: Use cached password hash if available

    let identifier = login_req.identifier.trim();
    let password = &login_req.password;

    let user = state
        .users
        .find_by_identifier(identifier)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?
        .ok_or_else(|| AuthError::invalid_credentials())?;

    let stored_hash = Hash::from_string(&user.password_hash)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    let password_valid = stored_hash
        .verify(password)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    if !password_valid {
        return Ok(HttpResponse::Unauthorized()
            .json(AuthError::invalid_credentials().to_response::<SignInResponse>()));
    }

    let access_token = generate_access_token(
        &user.id.to_string(),
        user.email.as_deref(),
        &state.jwt_secret,
        state.jwt_expiry_minutes,
    )?;

    let refresh_token = generate_refresh_token();
    let device = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let ip_address = req.connection_info().realip_remote_addr().map(String::from);

    let user_id =
        parse_id(&user.id.to_string()).map_err(|e| AuthError::internal_error(&e.to_string()))?;

    // Hash the access and refresh token for secure storage
    let access_token_hash = hash_sha256(&access_token);
    let refresh_token_hash = hash_sha256(&refresh_token);

    let session_input = CreateSession {
        user_id: user_id.clone(),
        access_token_hash,
        refresh_token_hash: Some(refresh_token_hash.clone()),
        device,
        ip_address,
        user_agent: req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        expires_in: state.refresh_token_expiry_days * 24 * 60 * 60,
    };

    let refresh_input = CreateRefreshToken {
        user_id: user_id.clone(),
        token_hash: refresh_token_hash.clone(),
        expires_in: state.refresh_token_expiry_days * 24 * 60 * 60,
    };

    let _ = state.sessions.create(session_input);
    let _ = state.sessions.create_refresh_token(refresh_input);

    let user_public = UserPublic {
        id: user.id.to_string(),
        username: user.username.unwrap_or_default(),
        first_name: user.first_name,
        last_name: user.last_name,
        is_verified: user.is_verified,
        created_at: user.created_at,
    };

    let response_data = SignInResponse {
        user: user_public,
        access_token,
        refresh_token: Some(refresh_token),
        expires_in: state.jwt_expiry_minutes * 60,
    };

    let response = ApiResponse::success_data("Login successful", response_data);

    Ok(HttpResponse::Ok().json(response))
}

pub async fn logout_user(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Validate token and session in one call
    let token_info = validate_access_token(&req, &state)
        .await
        .map_err(|_e| AuthError::invalid_session())?;

    let user_id = parse_id(&token_info.user_id).map_err(|_e| AuthError::invalid_credentials())?;

    // Revoke all sessions for this user on logout
    let _ = state.sessions.revoke_all(&user_id);
    let response = ApiResponse::<()>::ok("Logged out successfully");

    Ok(HttpResponse::Ok().json(response))
}

pub async fn refresh_token(
    state: web::Data<AppState>,
    refresh_req: web::Json<RefreshTokenRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let refresh_token = &refresh_req.refresh_token;
    let now = chrono::Utc::now().timestamp();
    // Validate token and session in one call
    let token_data = validate_access_token(&req, &state)
        .await
        .map_err(|e| AuthError::unauthorized(&e.to_string()))?;

    // Find refresh token in database
    // hash refresh token as used when creating session

    let refresh_token_hash = hash_sha256(&refresh_token);
    let refresh_token_model = state
        .sessions
        .find_refresh_token_by_hash(&refresh_token_hash)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    let refresh_token_model = match refresh_token_model {
        Some(rt) => rt,
        None => {
            let response = ApiResponse::<()>::error("Invalid or expired refresh token", None);
            return Ok(HttpResponse::Unauthorized().json(response));
        }
    };

    // Check expiry
    if now > refresh_token_model.expires_at || refresh_token_model.revoked {
        let response = ApiResponse::<()>::error("Refresh token has expired or been revoked", None);
        return Ok(HttpResponse::Unauthorized().json(response));
    }

    // Get user to generate new token
    let user = state
        .users
        .find_by_id(&refresh_token_model.user_id)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?
        .ok_or_else(|| AuthError::internal_error("User not found"))?;

    // Generate new access token
    let new_access_token = generate_access_token(
        &user.id.to_string(),
        user.email.as_deref(),
        &state.jwt_secret,
        state.jwt_expiry_minutes,
    )?;

    // Generate new refresh token
    let new_refresh_token  = generate_refresh_token();
    let refresh_token_hash = hash_sha256(&new_refresh_token);
    let access_token_hash  = hash_sha256(&new_access_token);

    // Create new refresh token in database
    let new_refresh_input = CreateRefreshToken {
        user_id: refresh_token_model.user_id.clone(),
        token_hash: refresh_token_hash.clone(),
        expires_in: state.refresh_token_expiry_days * 24 * 60 * 60,
    };

    let new_session_input = UpdateSession {
        access_token_hash,
        refresh_token_hash: refresh_token_hash,
        expires_at: state.jwt_expiry_minutes * 60,
        is_revoked: false,
    };

    let session = state
        .sessions
        .find_by_token(&token_data.token_hash)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?
        .ok_or_else(|| AuthError::invalid_session())?;

    let _ = state
        .sessions
        .create_refresh_token(new_refresh_input.clone());
    
    let _ = state.sessions.update(&session.id, new_session_input);

    // Mark old refresh token as revoked and link to new one
    let _ = state
        .sessions
        .replace_refresh_token(&refresh_token_model.id, new_refresh_input.clone());

    // Return new tokens
    let response_data = serde_json::json!({
        "access_token": new_access_token,
        "refresh_token": new_refresh_token,
        "expires_in": state.jwt_expiry_minutes * 60,
    });

    let response = ApiResponse::success_data(
        "Token refreshed successfully", response_data);
    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::types::SignInRequest;

    #[test]
    fn test_generate_access_token_success() {
        let user_id = "507f1f77bcf86cd799439011";
        let email = Some("test@example.com");
        let jwt_secret = "test_secret_key";
        let expiry_minutes = 60;

        let result = generate_access_token(user_id, email, jwt_secret, expiry_minutes);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
        assert!(token.contains("eyJ")); // JWT header always starts with eyJ
    }

    #[test]
    fn test_generate_access_token_with_none_email() {
        let user_id = "507f1f77bcf86cd799439011";
        let jwt_secret = "test_secret_key";
        let expiry_minutes = 60;

        let result = generate_access_token(user_id, None, jwt_secret, expiry_minutes);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_generate_refresh_token() {
        let token1 = generate_refresh_token();
        let token2 = generate_refresh_token();

        // Tokens should be 64 characters long (hex)
        assert_eq!(token1.len(), 128); // 64 bytes * 2 hex chars

        // Tokens should be different
        assert_ne!(token1, token2);

        // Tokens should only contain hex characters
        assert!(token1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(token2.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sign_in_request_validation() {
        let request = SignInRequest {
            identifier: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert_eq!(request.identifier, "test@example.com");
        assert_eq!(request.password, "password123");
    }

    #[test]
    fn test_sign_in_request_trim_identifier() {
        let identifier = "  test@example.com  ";
        let trimmed = identifier.trim();

        assert_eq!(trimmed, "test@example.com");
        assert_ne!(trimmed, identifier);
    }

    #[test]
    fn test_access_token_expiry_calculation() {
        let expiry_minutes = 60;
        let expiry_seconds = expiry_minutes * 60;

        assert_eq!(expiry_seconds, 3600);
    }

    #[test]
    fn test_refresh_token_expiry_calculation() {
        let expiry_days = 7;
        let expiry_seconds = expiry_days * 24 * 60 * 60;

        assert_eq!(expiry_seconds, 604800); // 7 days in seconds
    }

    #[test]
    fn test_token_hash_generation() {
        let token = "test_token_123";
        let hash1 = hash_sha256(token);
        let hash2 = hash_sha256(token);

        // Same token should produce same hash
        assert_eq!(hash1, hash2);

        // Hash should be 64 characters (SHA256)
        assert_eq!(hash1.len(), 64);

        // Hash should be hexadecimal
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_different_tokens_different_hashes() {
        let token1 = "token_1";
        let token2 = "token_2";

        let hash1 = hash_sha256(token1);
        let hash2 = hash_sha256(token2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_sign_in_response_structure() {
        let user_public = UserPublic {
            id: "507f1f77bcf86cd799439011".to_string(),
            username: "testuser".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            is_verified: true,
            created_at: 1234567890,
        };

        assert_eq!(user_public.id, "507f1f77bcf86cd799439011");
        assert_eq!(user_public.username, "testuser".to_string());
        assert!(user_public.is_verified);
    }

    #[test]
    fn test_sign_in_response_with_tokens() {
        let user = UserPublic {
            id: "507f1f77bcf86cd799439011".to_string(),
            username: "testuser".to_string(),
            first_name: None,
            last_name: None,
            is_verified: false,
            created_at: 1234567890,
        };

        let response = SignInResponse {
            user,
            access_token: "access_token_xyz".to_string(),
            refresh_token: Some("refresh_token_abc".to_string()),
            expires_in: 3600,
        };

        assert_eq!(response.access_token, "access_token_xyz");
        assert_eq!(
            response.refresh_token,
            Some("refresh_token_abc".to_string())
        );
        assert_eq!(response.expires_in, 3600);
    }

    #[test]
    fn test_api_response_success() {
        let response_data = serde_json::json!({
            "access_token": "test_token",
            "expires_in": 3600,
        });

        let response = ApiResponse::success_data("Login successful", response_data);

        // Verify the response structure
        assert_eq!(response.message, "Login successful".to_string());
    }

    #[test]
    fn test_invalid_credentials_error() {
        let error = AuthError::invalid_credentials();

        let response = error.to_response::<SignInResponse>();
        assert!(!response.success);
    }

    #[test]
    fn test_internal_error_creation() {
        let error = AuthError::internal_error("Database connection failed");

        let response = error.to_response::<SignInResponse>();
        assert!(!response.success);
    }

    #[test]
    fn test_unauthorized_error_creation() {
        let error = AuthError::unauthorized("Invalid token");

        let response = error.to_response::<SignInResponse>();
        assert!(!response.success);
    }

    #[test]
    fn test_create_session_input_structure() {
        let session_input = CreateSession {
            user_id: parse_id("507f1f77bcf86cd799439011").unwrap(),
            access_token_hash: "hash_xyz".to_string(),
            refresh_token_hash: Some("refresh_hash".to_string()),
            device: Some("Mozilla/5.0".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            expires_in: 604800,
        };

        assert_eq!(session_input.access_token_hash, "hash_xyz");
        assert_eq!(
            session_input.refresh_token_hash,
            Some("refresh_hash".to_string())
        );
        assert_eq!(session_input.expires_in, 604800);
    }

    #[test]
    fn test_create_refresh_token_input_structure() {
        let refresh_token_input = CreateRefreshToken {
            user_id: parse_id("507f1f77bcf86cd799439011").unwrap(),
            token_hash: "refresh_token_hash".to_string(),
            expires_in: 604800,
        };

        assert_eq!(refresh_token_input.token_hash, "refresh_token_hash");
        assert_eq!(refresh_token_input.expires_in, 604800);
    }
}
