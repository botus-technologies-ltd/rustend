//! OAuth Configuration
//!
//! Loads OAuth provider credentials from environment variables.
//! Each provider needs: CLIENT_ID, CLIENT_SECRET, and REDIRECT_URI

use crate::models::oauth::OAuthProvider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub google:   Option<OAuthProviderConfig>,
    pub facebook: Option<OAuthProviderConfig>,
    pub github:   Option<OAuthProviderConfig>,
    pub twitter:  Option<OAuthProviderConfig>,
    pub linkedin: Option<OAuthProviderConfig>,
    pub microsoft:Option<OAuthProviderConfig>,
    pub apple:    Option<OAuthProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    pub client_id:     String,
    pub client_secret: String,
    pub redirect_uri:  String,
    pub auth_url:      String,
    pub token_url:     String,
    pub userinfo_url:  String,
}

impl OAuthConfig {
    /// Load OAuth configuration from environment variables
    ///
    /// Expected env vars format:
    /// - OAUTH_GOOGLE_CLIENT_ID
    /// - OAUTH_GOOGLE_CLIENT_SECRET
    /// - OAUTH_GOOGLE_REDIRECT_URI
    /// etc for each provider
    pub fn from_env() -> Self {
        Self {
            google:    Self::load_provider_config("GOOGLE"),
            facebook:  Self::load_provider_config("FACEBOOK"),
            github:    Self::load_provider_config("GITHUB"),
            twitter:   Self::load_provider_config("TWITTER"),
            linkedin:  Self::load_provider_config("LINKEDIN"),
            microsoft: Self::load_provider_config("MICROSOFT"),
            apple:     Self::load_provider_config("APPLE"),
        }
    }

    /// Load configuration for a specific provider
    pub fn load_provider_config(provider: &str) -> Option<OAuthProviderConfig> {
        let client_id      = std::env::var(&format!("OAUTH_{}_CLIENT_ID", provider)).ok()?;
        let client_secret  = std::env::var(&format!("OAUTH_{}_CLIENT_SECRET", provider)).ok()?;
        let redirect_uri   = std::env::var(&format!("OAUTH_{}_REDIRECT_URI", provider)).ok()?;

        // Use standard OAuth endpoints for each provider
        let (auth_url, token_url, userinfo_url) = match provider {
            "GOOGLE" => (
                "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                "https://oauth2.googleapis.com/token".to_string(),
                "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            ),

            "FACEBOOK" => (
                "https://www.facebook.com/v18.0/dialog/oauth".to_string(),
                "https://graph.facebook.com/v18.0/oauth/access_token".to_string(),
                "https://graph.facebook.com/v18.0/me".to_string(),
            ),

            "GITHUB" => (
                "https://github.com/login/oauth/authorize".to_string(),
                "https://github.com/login/oauth/access_token".to_string(),
                "https://api.github.com/user".to_string(),
            ),

            "TWITTER" => (
                "https://twitter.com/i/oauth2/authorize".to_string(),
                "https://api.twitter.com/2/oauth2/token".to_string(),
                "https://api.twitter.com/2/users/me".to_string(),
            ),

            "LINKEDIN" => (
                "https://www.linkedin.com/oauth/v2/authorization".to_string(),
                "https://www.linkedin.com/oauth/v2/accessToken".to_string(),
                "https://api.linkedin.com/v2/me".to_string(),
            ),

            "MICROSOFT" => (
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
                "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
                "https://graph.microsoft.com/v1.0/me".to_string(),
            ),

            "APPLE" => (
                "https://appleid.apple.com/auth/authorize".to_string(),
                "https://appleid.apple.com/auth/token".to_string(),
                "https://appleid.apple.com/auth/keys".to_string(),
            ),
            _ => return None,
        };

        Some(OAuthProviderConfig {
            client_id,
            client_secret,
            redirect_uri,
            auth_url,
            token_url,
            userinfo_url,
        })
    }

    /// Get configuration for a specific provider
    pub fn get_provider(&self, provider: &OAuthProvider) -> Option<&OAuthProviderConfig> {
        match provider {
            OAuthProvider::Google    => self.google.as_ref(),
            OAuthProvider::Facebook  => self.facebook.as_ref(),
            OAuthProvider::GitHub    => self.github.as_ref(),
            OAuthProvider::Twitter   => self.twitter.as_ref(),
            OAuthProvider::LinkedIn  => self.linkedin.as_ref(),
            OAuthProvider::Microsoft => self.microsoft.as_ref(),
            OAuthProvider::Apple     => self.apple.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_provider_config_creation() {
        let config = OAuthProviderConfig {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "http://localhost:3000/callback".to_string(),
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            userinfo_url: "https://example.com/userinfo".to_string(),
        };

        assert_eq!(config.client_id, "test_id");
        assert_eq!(config.client_secret, "test_secret");
    }

    #[test]
    fn test_oauth_config_get_provider() {
        let config = OAuthConfig {
            google: Some(OAuthProviderConfig {
                client_id: "google_id".to_string(),
                client_secret: "google_secret".to_string(),
                redirect_uri: "http://localhost:3000/google/callback".to_string(),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            }),
            facebook: None,
            github: None,
            twitter: None,
            linkedin: None,
            microsoft: None,
            apple: None,
        };

        assert!(config.get_provider(&OAuthProvider::Google).is_some());
        assert!(config.get_provider(&OAuthProvider::Facebook).is_none());
    }
}
