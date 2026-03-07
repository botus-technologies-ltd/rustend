//! OAuth handler
//!
//! Handles OAuth flows: authorization redirect, callback handling, account linking/unlinking
//! Delegates business logic to OAuthService

use actix_web::{Error, HttpResponse, web, HttpRequest};
use serde::{Deserialize, Serialize};
use utils::hash::hash_sha256;
use utils::response::{ApiResponse, ApiError };

use crate::config::oauth::{OAuthConfig};
use crate::service::OAuthService;
use crate::handler::login_user::{generate_access_token, generate_refresh_token};

use crate::models::oauth::{
    CreateOAuthAccount, 
    OAuthProvider, 
    UnlinkOAuthRequest,
    LinkOAuthRequest,
};
use crate::models::user::CreateUserInput;
use crate::models::session::CreateRefreshToken;   
use crate::models::session::CreateSession;
use crate::utils::errors::AuthError;
use crate::routes::AppState;
use database::utils::{generate_id, parse_id};

// Import logging utilities
use middleware::tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthConnectionResponse {
    pub provider: String,
    pub provider_user_id: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

/// Helper function to load and configure OAuth for a specific provider
fn load_oauth_config_for_provider(
    provider: &OAuthProvider,
    provider_str: &str,
) -> Result<OAuthConfig, AuthError> {
    let config = OAuthConfig::load_provider_config(&provider_str.to_uppercase())
        .ok_or_else(|| {
            AuthError::internal_error(&format!("OAuth provider {} not configured", provider_str))
        })?;

    Ok(OAuthConfig {
        google:   if *provider == OAuthProvider::Google { Some(config.clone()) } else { None },
        facebook: if *provider == OAuthProvider::Facebook { Some(config.clone()) } else { None },
        github:   if *provider == OAuthProvider::GitHub { Some(config.clone()) } else { None },
        twitter:  if *provider == OAuthProvider::Twitter { Some(config.clone()) } else { None },
        linkedin: if *provider == OAuthProvider::LinkedIn { Some(config.clone()) } else { None },
        microsoft:if *provider == OAuthProvider::Microsoft { Some(config.clone()) } else { None },
        apple:    if *provider == OAuthProvider::Apple { Some(config.clone()) } else { None },
    })
}

/// OAuth redirect - initiates OAuth flow
pub async fn oauth_redirect(
    _state: web::Data<AppState>,
    provider: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let provider_str = provider.into_inner();
    
    info!("OAuth redirect requested for provider: {}", provider_str);
    
    let provider = match provider_str.to_lowercase().as_str() {
        "google"    => OAuthProvider::Google,
        "facebook"  => OAuthProvider::Facebook,
        "github"    => OAuthProvider::GitHub,
        "twitter"   => OAuthProvider::Twitter,
        "linkedin"  => OAuthProvider::LinkedIn,
        "microsoft" => OAuthProvider::Microsoft,
        "apple"     => OAuthProvider::Apple,
        _ => {
            warn!("Unsupported OAuth provider requested: {}", provider_str);
            let response: ApiResponse<ApiError> = ApiResponse::error(
                &format!("Unsupported OAuth provider: {}", provider_str),
                Some(ApiError {
                    code: "UNSUPPORTED_PROVIDER".into(),
                    details: None,
                }),
            );
            return Ok(HttpResponse::BadRequest().json(response));
        }
           
    };

    // Get OAuth service
    let oauth_config = load_oauth_config_for_provider(&provider, &provider_str)?;
    let oauth_service = OAuthService::new(oauth_config);

    // Generate state token - this will be validated in callback
    let state_token = format!("{}.{}", generate_id().to_string(),
        chrono::Utc::now().timestamp()
    );

    // Get authorization URL with actual state token
    let auth_url = oauth_service
        .get_auth_redirect_url(&provider, &state_token)
        .map_err(|_| AuthError::internal_error("Failed to generate OAuth URL"))?;

    info!("OAuth redirect URL generated for provider: {}", provider_str);
    
    // Return redirect response with state token
    let response = ApiResponse::success_data(
        "OAuth redirect initiated",
        serde_json::json!({
            "auth_url": auth_url,
            "state": state_token,  // Client should store this and send back in callback
        }),
    );
    
    info!("OAuth redirect response sent for provider: {}", provider_str);
    Ok(HttpResponse::Ok().json(response))
}

/// OAuth callback - handles OAuth provider callback
pub async fn oauth_callback(
    state: web::Data<AppState>,
    provider: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    
    let provider_str = provider.into_inner();
    let provider = match provider_str.to_lowercase().as_str() {
        "google"    => OAuthProvider::Google,
        "facebook"  => OAuthProvider::Facebook,
        "github"    => OAuthProvider::GitHub,
        "twitter"   => OAuthProvider::Twitter,
        "linkedin"  => OAuthProvider::LinkedIn,
        "microsoft" => OAuthProvider::Microsoft,
        "apple"     => OAuthProvider::Apple,
        _ => {
           let response: ApiResponse<ApiError> = ApiResponse::error(
                &format!("Unsupported OAuth provider: {}", provider_str),
                Some(ApiError {
                    code: "UNSUPPORTED_PROVIDER".into(),
                    details: None,
                }),
            );
            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    // Get authorization code
    let code = query
        .get("code")
        .ok_or_else(|| AuthError::invalid_request("Missing authorization code"))?
        .clone();

    // Validate state parameter (CSRF protection)
    let _state_param = query
        .get("state")
        .ok_or_else(|| AuthError::invalid_request("Missing state parameter"))?
        .clone();
    // TODO: Implement proper state store to validate state matches what was sent

    // Get OAuth service
    let oauth_config = load_oauth_config_for_provider(&provider, &provider_str)?;
    let oauth_service = OAuthService::new(oauth_config);

    let oauth_accounts = state
        .oauth_accounts
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("OAuth accounts store not configured"))?;

    // Exchange code for token
    let token_response = oauth_service
        .exchange_code_for_token(&provider, &code)
        .await
        .map_err(|_| AuthError::internal_error("Failed to exchange code for token"))?;

    // Get user info from provider
    let user_info = oauth_service
        .get_user_info(&provider, &token_response.access_token)
        .await
        .map_err(|_| AuthError::internal_error("Failed to retrieve user info from provider"))?;

    let device = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let ip_address = req.connection_info().realip_remote_addr().map(String::from);
    let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

    // Check if OAuth account exists
    match oauth_accounts.find_by_provider_user_id(&provider, &user_info.provider_user_id) {
        Ok(Some(oauth_account)) => {
            // User already linked - create new session
            let access_token   = generate_access_token(
                &oauth_account.user_id.to_string(), 
                user_info.email.as_deref(), 
                &state.jwt_secret, 
                state.jwt_expiry_minutes
            )?;

            let refresh_token      = generate_refresh_token();
            let access_token_hash  = hash_sha256(&access_token);
            let refresh_token_hash = hash_sha256(&refresh_token);

            let session_input = CreateSession {
                user_id: oauth_account.user_id.clone(),
                access_token_hash,
                refresh_token_hash: Some(refresh_token_hash.clone()),
                device: device.clone(),
                ip_address: ip_address.clone(),
                user_agent: user_agent.clone(),
                expires_in: state.jwt_expiry_minutes * 24 * 60 * 60,
            };

            let refresh_input = CreateRefreshToken {
                user_id: oauth_account.user_id.clone(),
                token_hash: refresh_token_hash.clone(),
                expires_in: state.refresh_token_expiry_days * 24 * 60 * 60,
            };

            state
                .sessions
                .create(session_input)
                .map_err(|_| AuthError::internal_error("Failed to create session"))?;

            state
                .sessions
                .create_refresh_token(refresh_input)
                .map_err(|_| AuthError::internal_error("Failed to create refresh token"))?;

            let response = ApiResponse::success_data(
                "OAuth login successful",
                serde_json::json!({
                    "access_token": access_token,
                    "refresh_token": refresh_token,
                    "user_id": oauth_account.user_id,
                }),
            );

            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => {
            // New OAuth user - create account
            let user_email = user_info.email.clone().ok_or_else(|| {
                AuthError::invalid_request("Email not provided by OAuth provider")
            })?;

            // Check if user with this email exists
            let existing_user = state
                .users
                .find_by_email(&user_email)
                .map_err(|_| AuthError::internal_error("Database error while checking email"))?;

            let user = if let Some(existing) = existing_user {
                // User exists - link OAuth account
                existing
            } else {
                // Create new user
                let user_input = CreateUserInput {
                    email: Some(user_email.clone()),
                    password: generate_id().to_string(), // Random password for OAuth users
                    phone: None,
                    username: None,
                    first_name: user_info.name.clone(),
                    last_name: None,
                };

                state
                    .users
                    .create(user_input)
                    .map_err(|_| AuthError::internal_error("Failed to create user"))?
            };

            // Create OAuth account (storing tokens as-is for now, TODO: add encryption)
            let oauth_account_input = CreateOAuthAccount {
                user_id: user.id.clone(),
                provider,
                provider_user_id: user_info.provider_user_id,
                access_token: Some(token_response.access_token),
                refresh_token: token_response.refresh_token,
                expires_in: token_response.expires_in,
                scope: token_response.scope,
            };

            oauth_accounts
                .create(oauth_account_input)
                .map_err(|_| AuthError::internal_error("Failed to create OAuth account"))?;

            // Create session
            let access_token   = generate_access_token(
                &user.id.to_string(), 
                user_info.email.as_deref(), 
                &state.jwt_secret, 
                state.jwt_expiry_minutes
            )?;

            let refresh_token      = generate_refresh_token();
            let access_token_hash  = hash_sha256(&access_token);
            let refresh_token_hash = hash_sha256(&refresh_token);

            let session_input = CreateSession {
                user_id: user.id.clone(),
                access_token_hash,
                refresh_token_hash: Some(refresh_token_hash.clone()),
                device: device.clone(),
                ip_address: ip_address.clone(),
                user_agent: user_agent.clone(),
                expires_in: state.jwt_expiry_minutes * 24 * 60 * 60,
            };

            let refresh_input = CreateRefreshToken {
                user_id: user.id.clone(),
                token_hash: refresh_token_hash.clone(),
                expires_in: state.refresh_token_expiry_days * 24 * 60 * 60,
            };

            state
                .sessions
                .create_refresh_token(refresh_input)
                .map_err(|_| AuthError::internal_error("Failed to create refresh token"))?;

            state
                .sessions
                .create(session_input)
                .map_err(|_| AuthError::internal_error("Failed to create session"))?;

            let response = ApiResponse::success_data(
                "OAuth account created",
                serde_json::json!({
                    "access_token": access_token,
                    "refresh_token": refresh_token,
                    "user_id": user.id,
                }),
            );

            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => Err(e.into()),
    }
}

/// Link OAuth account to existing user
pub async fn link_oauth(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
    req: web::Json<LinkOAuthRequest>,
) -> Result<HttpResponse, Error> {
    let user_id_str = user_id.into_inner();
    let user_id =
        parse_id(&user_id_str).map_err(|_| AuthError::invalid_request("Invalid user ID"))?;

    // Verify user exists
    let user = state
        .users
        .find_by_id(&user_id)
        .map_err(|_| AuthError::internal_error("Database error while finding user"))?
        .ok_or_else(|| AuthError::not_found("User not found"))?;

    let oauth_accounts = state
        .oauth_accounts
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("OAuth accounts store not configured"))?;

    // Check if OAuth account already linked for this user and provider
    if let Ok(Some(_)) = oauth_accounts.find_by_user_and_provider(&user_id, &req.provider) {
        return Err(AuthError::invalid_request("OAuth account already linked for this provider").into());
    }

    // Get OAuth service
    let provider_str = format!("{:?}", req.provider);
    let oauth_config = load_oauth_config_for_provider(&req.provider, &provider_str)?;
    let oauth_service = OAuthService::new(oauth_config);

    // Exchange code for token
    let token_response = oauth_service
        .exchange_code_for_token(&req.provider, &req.code)
        .await
        .map_err(|_| AuthError::internal_error("Failed to exchange code for token"))?;

    // Get user info from provider
    let user_info = oauth_service
        .get_user_info(&req.provider, &token_response.access_token)
        .await
        .map_err(|_| AuthError::internal_error("Failed to retrieve user info from provider"))?;

    // Check if this OAuth account is already linked to another user
    if let Ok(Some(existing_account)) = oauth_accounts.find_by_provider_user_id(&req.provider, &user_info.provider_user_id) {
        if existing_account.user_id != user_id {
            return Err(AuthError::conflict("This OAuth account is already linked to another user").into());
        }
    }

    // Create OAuth account
    let oauth_account_input = CreateOAuthAccount {
        user_id: user.id.clone(),
        provider: req.provider.clone(),
        provider_user_id: user_info.provider_user_id.clone(),
        access_token: Some(token_response.access_token),
        refresh_token: token_response.refresh_token,
        expires_in: token_response.expires_in,
        scope: token_response.scope,
    };

    oauth_accounts
        .create(oauth_account_input)
        .map_err(|_| AuthError::internal_error("Failed to create OAuth account"))?;

    let response = ApiResponse::success_data(
        "OAuth account linked successfully",
        serde_json::json!({
            "provider": format!("{:?}", &req.provider),
            "provider_user_id": &user_info.provider_user_id,
            "user_id": &user.id,
        }),
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Unlink OAuth account from user
pub async fn unlink_oauth(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
    req: web::Json<UnlinkOAuthRequest>,
) -> Result<HttpResponse, Error> {
    let user_id_str = user_id.into_inner();
    let user_id =
        parse_id(&user_id_str).map_err(|_| AuthError::invalid_request("Invalid user ID"))?;

    let oauth_accounts = state
        .oauth_accounts
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("OAuth accounts store not configured"))?;

    oauth_accounts
        .delete_by_user_and_provider(&user_id, &req.provider)
        .map_err(|_| AuthError::internal_error("Failed to delete OAuth account"))?;

    let response =
        ApiResponse::success_data("OAuth account unlinked successfully", serde_json::json!({}));

    Ok(HttpResponse::Ok().json(response))
}

/// List OAuth connections for user
pub async fn list_oauth_connections(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let user_id_str = user_id.into_inner();
    let user_id =
        parse_id(&user_id_str).map_err(|_| AuthError::invalid_request("Invalid user ID"))?;

    let oauth_accounts = state
        .oauth_accounts
        .as_ref()
        .ok_or_else(|| AuthError::internal_error("OAuth accounts store not configured"))?;

    let accounts = oauth_accounts
        .list_by_user(&user_id)
        .map_err(|_| AuthError::internal_error("Failed to retrieve OAuth connections"))?;

    let connections: Vec<OAuthConnectionResponse> = accounts
        .iter()
        .map(|acc| OAuthConnectionResponse {
            provider: format!("{:?}", acc.provider),
            provider_user_id: acc.provider_user_id.clone(),
            created_at: acc.created_at,
            expires_at: acc.expires_at,
        })
        .collect();

    let response = ApiResponse::success_data("OAuth connections retrieved", connections);

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_oauth_connection_response() {
        let response = OAuthConnectionResponse {
            provider: "google".to_string(),
            provider_user_id: "123456789".to_string(),
            created_at: Utc::now().timestamp(),
            expires_at: Some(Utc::now().timestamp() + 3600),
        };

        assert_eq!(response.provider, "google");
        assert_eq!(response.provider_user_id, "123456789");
        assert!(response.expires_at.is_some());
    }

    #[test]
    fn test_oauth_state_token_generation() {
        let state_token = format!(
            "{}.{}",
            generate_id().to_string(),
            Utc::now().timestamp()
        );

        // Should have format: id.timestamp
        let parts: Vec<&str> = state_token.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert!(!parts[0].is_empty());
        assert!(!parts[1].is_empty());
    }

    #[test]
    fn test_oauth_provider_serialization() {
        let provider = OAuthProvider::Google;
        let json_str = serde_json::to_string(&provider).unwrap();
        assert_eq!(json_str, "\"google\"");
    }

    #[test]
    fn test_link_oauth_request_validation() {
        let request = LinkOAuthRequest {
            provider: OAuthProvider::GitHub,
            code: "auth_code_123".to_string(),
            redirect_uri: Some("https://example.com/callback".to_string()),
        };

        assert_eq!(request.provider, OAuthProvider::GitHub);
        assert_eq!(request.code, "auth_code_123");
        assert!(request.redirect_uri.is_some());
    }

    #[test]
    fn test_unlink_oauth_request_validation() {
        let request = UnlinkOAuthRequest {
            provider: OAuthProvider::Facebook,
        };

        assert_eq!(request.provider, OAuthProvider::Facebook);
    }
}
