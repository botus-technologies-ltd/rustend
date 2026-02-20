//! OAuth/Social login models

use serde::{Deserialize, Serialize};
use database::utils::DbId;

/// OAuth provider enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Facebook,
    Apple,
    GitHub,
    Twitter,
    LinkedIn,
    Microsoft,
}

/// OAuth account model - links external OAuth accounts to users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAccount {
    pub id: DbId,
    pub user_id: DbId,
    pub provider: OAuthProvider,
    pub provider_user_id: String,  // ID from the OAuth provider
    pub access_token: Option<String>,  // Encrypted
    pub refresh_token: Option<String>,  // Encrypted
    pub expires_at: Option<i64>,
    pub scope: Option<String>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

impl OAuthAccount {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            return chrono::Utc::now().timestamp() > expires_at;
        }
        false
    }

    pub fn needs_refresh(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            // Refresh if less than 5 minutes remaining
            return chrono::Utc::now().timestamp() > expires_at - 300;
        }
        false
    }
}

/// Create OAuth account input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOAuthAccount {
    pub user_id: DbId,
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
}

/// OAuth callback state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub provider: OAuthProvider,
    pub redirect_uri: Option<String>,
    pub nonce: String,
    pub created_at: i64,
}

impl OAuthState {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() - self.created_at > 600 // 10 minutes
    }
}

/// OAuth link request - link existing account with OAuth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkOAuthRequest {
    pub provider: OAuthProvider,
    pub code: String,
    pub redirect_uri: Option<String>,
}

/// OAuth unlink request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlinkOAuthRequest {
    pub provider: OAuthProvider,
}
