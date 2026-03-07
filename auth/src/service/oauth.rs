//! OAuth Service
//!
//! Handles OAuth flows: authorization, token exchange, user linking/unlinking
//! Delegates to providers and manages OAuth accounts

use crate::config::oauth::OAuthConfig;
use crate::models::oauth::OAuthProvider;
use crate::utils::errors::AuthError;
use serde::{Deserialize, Serialize};

/// OAuth user info returned from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// OAuth error response from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

/// OAuth token response from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
}

/// OAuth service for handling OAuth flows
#[derive(Clone)]
pub struct OAuthService {
    config: OAuthConfig,
    http_client: reqwest::Client,
}

impl OAuthService {
    /// Create new OAuth service with configuration
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get authorization redirect URL for a provider
    pub fn get_auth_redirect_url(
        &self,
        provider: &OAuthProvider,
        state: &str,
    ) -> Result<String, AuthError> {
        let provider_config = self.config.get_provider(provider).ok_or_else(|| {
            AuthError::unauthorized(&format!("OAuth provider {:?} not configured", provider))
        })?;

        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            provider_config.auth_url,
            urlencoding::encode(&provider_config.client_id),
            urlencoding::encode(&provider_config.redirect_uri),
            urlencoding::encode(&self.get_default_scope(provider)),
            urlencoding::encode(state)
        );

        Ok(auth_url)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(
        &self,
        provider: &OAuthProvider,
        code: &str,
    ) -> Result<OAuthTokenResponse, AuthError> {
        let provider_config = self.config.get_provider(provider).ok_or_else(|| {
            AuthError::unauthorized(&format!("OAuth provider {:?} not configured", provider))
        })?;

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &provider_config.redirect_uri),
            ("client_id", &provider_config.client_id),
            ("client_secret", &provider_config.client_secret),
        ];

        let response = self
            .http_client
            .post(&provider_config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::internal_error(&format!("Failed to exchange code: {}", e)))?;

        // Check response status
        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<OAuthErrorResponse>(&response_text) {
                let error_msg = error_response.error_description.unwrap_or(error_response.error);
                return Err(AuthError::internal_error(&format!(
                    "OAuth provider error: {} - {}",
                    status, error_msg
                )));
            }
            return Err(AuthError::internal_error(&format!(
                "OAuth provider returned error {}: {}",
                status, response_text
            )));
        }

        // Parse successful token response
        let token_response: OAuthTokenResponse = serde_json::from_str(&response_text)
            .map_err(|e| AuthError::internal_error(&format!(
                "Failed to parse token response: {}. Response: {}",
                e, response_text
            )))?;

        Ok(token_response)
    }

    /// Get user info from OAuth provider
    pub async fn get_user_info(
        &self,
        provider: &OAuthProvider,
        access_token: &str,
    ) -> Result<OAuthUserInfo, AuthError> {
        let provider_config = self.config.get_provider(provider).ok_or_else(|| {
            AuthError::unauthorized(&format!("OAuth provider {:?} not configured", provider))
        })?;

        let response = self
            .http_client
            .get(&provider_config.userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::internal_error(&format!("Failed to fetch user info: {}", e)))?;

        // Parse provider-specific response
        let user_info = match provider {
            OAuthProvider::Google => self.parse_google_user_info(response).await?,
            OAuthProvider::GitHub => self.parse_github_user_info(response).await?,
            OAuthProvider::Facebook => self.parse_facebook_user_info(response).await?,
            _ => {
                // Generic parsing for other providers
                let json: serde_json::Value = response.json().await.map_err(|e| {
                    AuthError::internal_error(&format!("Failed to parse user info: {}", e))
                })?;

                OAuthUserInfo {
                    provider_user_id: json["id"].as_str().unwrap_or("").to_string(),
                    email: json["email"].as_str().map(String::from),
                    name: json["name"].as_str().map(String::from),
                    picture: json["picture"].as_str().map(String::from),
                }
            }
        };

        Ok(user_info)
    }

    /// Parse Google user info response
    async fn parse_google_user_info(
        &self,
        response: reqwest::Response,
    ) -> Result<OAuthUserInfo, AuthError> {
        #[derive(Deserialize)]
        struct GoogleUserInfo {
            id: String,
            email: Option<String>,
            name: Option<String>,
            picture: Option<String>,
        }

        let google_user: GoogleUserInfo = response.json().await.map_err(|e| {
            AuthError::internal_error(&format!("Failed to parse Google response: {}", e))
        })?;

        Ok(OAuthUserInfo {
            provider_user_id: google_user.id,
            email: google_user.email,
            name: google_user.name,
            picture: google_user.picture,
        })
    }

    /// Parse GitHub user info response
    async fn parse_github_user_info(
        &self,
        response: reqwest::Response,
    ) -> Result<OAuthUserInfo, AuthError> {
        #[derive(Deserialize)]
        struct GitHubUserInfo {
            id: u64,
            login: String,
            email: Option<String>,
            name: Option<String>,
            avatar_url: Option<String>,
        }

        let github_user: GitHubUserInfo = response.json().await.map_err(|e| {
            AuthError::internal_error(&format!("Failed to parse GitHub response: {}", e))
        })?;

        Ok(OAuthUserInfo {
            provider_user_id: github_user.id.to_string(),
            email: github_user.email,
            name: github_user.name.or(Some(github_user.login)),
            picture: github_user.avatar_url,
        })
    }

    /// Parse Facebook user info response
    async fn parse_facebook_user_info(
        &self,
        response: reqwest::Response,
    ) -> Result<OAuthUserInfo, AuthError> {
        #[derive(Deserialize)]
        struct FacebookUserInfo {
            id: String,
            email: Option<String>,
            name: Option<String>,
            picture: Option<FacebookPicture>,
        }

        #[derive(Deserialize)]
        struct FacebookPicture {
            data: Option<FacebookPictureData>,
        }

        #[derive(Deserialize)]
        struct FacebookPictureData {
            url: Option<String>,
        }

        let fb_user: FacebookUserInfo = response.json().await.map_err(|e| {
            AuthError::internal_error(&format!("Failed to parse Facebook response: {}", e))
        })?;

        let picture = fb_user.picture.and_then(|p| p.data).and_then(|d| d.url);

        Ok(OAuthUserInfo {
            provider_user_id: fb_user.id,
            email: fb_user.email,
            name: fb_user.name,
            picture,
        })
    }

    /// Get default scope for a provider
    fn get_default_scope(&self, provider: &OAuthProvider) -> &'static str {
        match provider {
            OAuthProvider::Google    => "openid email profile",
            OAuthProvider::GitHub    => "user:email",
            OAuthProvider::Facebook  => "email public_profile",
            OAuthProvider::Twitter   => "tweet.read users.read",
            OAuthProvider::LinkedIn  => "openid profile email",
            OAuthProvider::Microsoft => "openid profile email",
            OAuthProvider::Apple     => "openid email profile",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_user_info_creation() {
        let user_info = OAuthUserInfo {
            provider_user_id: "123456".to_string(),
            email: Some("user@example.com".to_string()),
            name: Some("John Doe".to_string()),
            picture: None,
        };

        assert_eq!(user_info.provider_user_id, "123456");
        assert_eq!(user_info.email, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_oauth_token_response() {
        let token_response = OAuthTokenResponse {
            access_token: "token123".to_string(),
            refresh_token: Some("refresh123".to_string()),
            expires_in: Some(3600),
            scope: Some("openid email profile".to_string()),
        };

        assert_eq!(token_response.access_token, "token123");
        assert!(token_response.refresh_token.is_some());
    }
}
