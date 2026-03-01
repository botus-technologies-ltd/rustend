//! OAuth Account Store trait
//!
//! Provides interface for storing and retrieving OAuth accounts

use crate::models::oauth::{CreateOAuthAccount, OAuthAccount, OAuthProvider};
use crate::utils::errors::AuthError;
use database::utils::DbId;

/// OAuth Account store trait - handles OAuth account persistence
pub trait OAuthAccountStore: Send + Sync {
    /// Create a new OAuth account
    fn create(&self, account: CreateOAuthAccount) -> Result<OAuthAccount, AuthError>;

    /// Find OAuth account by ID
    fn find_by_id(&self, id: &DbId) -> Result<Option<OAuthAccount>, AuthError>;

    /// Find OAuth account by user ID and provider
    fn find_by_user_and_provider(
        &self,
        user_id: &DbId,
        provider: &OAuthProvider,
    ) -> Result<Option<OAuthAccount>, AuthError>;

    /// Find OAuth account by provider and provider user ID
    fn find_by_provider_user_id(
        &self,
        provider: &OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, AuthError>;

    /// List all OAuth accounts for a user
    fn list_by_user(&self, user_id: &DbId) -> Result<Vec<OAuthAccount>, AuthError>;

    /// Update OAuth account
    fn update(&self, id: &DbId, account: &OAuthAccount) -> Result<OAuthAccount, AuthError>;

    /// Delete OAuth account
    fn delete(&self, id: &DbId) -> Result<(), AuthError>;

    /// Delete all OAuth accounts for a user
    fn delete_by_user(&self, user_id: &DbId) -> Result<(), AuthError>;

    /// Delete OAuth account by provider
    fn delete_by_user_and_provider(
        &self,
        user_id: &DbId,
        provider: &OAuthProvider,
    ) -> Result<(), AuthError>;
}
