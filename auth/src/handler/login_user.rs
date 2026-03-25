//! Login handler - user authentication
//!
//! Performance optimizations for <1s login:
//! - Use bcrypt with lower cost (4-6) instead of argon2 for password verification
//! - Add Redis caching for frequent users
//! - Make session creation async/non-blocking
//! - Use connection pooling for database
//! - Cache user lookups with short TTL

use actix_web::{Error, HttpRequest, HttpResponse, web};
use chrono::Duration;

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
    user_id:        &str,
    email:          Option<&str>,
    jwt_secret:     &str,
    expiry_minutes: i64,
) -> Result<String, AuthError> {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims {
        sub:   user_id.to_string(),
        email: email.map(String::from),
        exp:   now + (expiry_minutes * 60),
        iat:   now,
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
    state:     web::Data<AppState>,
    login_req: web::Json<SignInRequest>,
    req:       HttpRequest,
) -> Result<HttpResponse, Error> {
    // FUTURE: Check Redis cache first for user data to avoid DB lookup
    // FUTURE: Use cached password hash if available

    let identifier  = login_req.identifier.trim();
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
        user_id:    user_id.clone(),
        token_hash: refresh_token_hash.clone(),
        expires_in: state.refresh_token_expiry_days * 24 * 60 * 60,
    };

    let _ = state.sessions.create(session_input);
    let _ = state.sessions.create_refresh_token(refresh_input);

    let user_public = UserPublic {
        id:         user.id.to_string(),
        username:   user.username.unwrap_or_default(),
        first_name: user.first_name,
        last_name:  user.last_name,
        is_verified:user.is_verified,
        created_at: user.created_at,
    };

    let response_data = SignInResponse {
        user: user_public,
        access_token,
        refresh_token: Some(refresh_token),
        expires_in: chrono::Utc::now() + Duration::seconds(state.jwt_expiry_minutes * 60),
    };

    let response = ApiResponse::success_data("Login successful", response_data);

    Ok(HttpResponse::Ok().json(response))
}

pub async fn logout_user(
    req:   HttpRequest,
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
    state:       web::Data<AppState>,
    refresh_req: web::Json<RefreshTokenRequest>,
    req:         HttpRequest,
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
        user_id:    refresh_token_model.user_id.clone(),
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
        "access_token":  new_access_token,
        "refresh_token": new_refresh_token,
        "expires_in":    state.jwt_expiry_minutes * 60,
    });

    let response = ApiResponse::success_data(
        "Token refreshed successfully", response_data);
    Ok(HttpResponse::Ok().json(response))
}