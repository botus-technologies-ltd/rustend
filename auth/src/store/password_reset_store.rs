//! Password reset store module
//! 
//! Provides a generic password reset store for password reset tokens.

use crate::models::reset_password::{PasswordResetTokenModel, CreatePasswordResetToken};
use crate::utils::errors::{AuthResult};
use database::utils::DbId;

/// Password reset store trait - implement this for each database
pub trait PasswordResetStore: Send + Sync {
    /// Create a new password reset token
    fn create(&self, input: CreatePasswordResetToken) -> AuthResult<PasswordResetTokenModel>;
    
    /// Find password reset token by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<PasswordResetTokenModel>>;
    
    /// Find valid password reset token by user_id
    fn find_valid_token(&self, user_id: &DbId) -> AuthResult<Option<PasswordResetTokenModel>>;
    
    /// Find token by hash
    fn find_by_hash(&self, token_hash: &str) -> AuthResult<Option<PasswordResetTokenModel>>;
    
    /// Mark token as used
    fn mark_used(&self, id: &DbId) -> AuthResult<()>;
    
    /// Delete/expire a token
    fn delete(&self, id: &DbId) -> AuthResult<()>;
    
    /// Delete all tokens for a user
    fn delete_all_for_user(&self, user_id: &DbId) -> AuthResult<u64>;
    
    /// Cleanup expired tokens
    fn cleanup_expired(&self) -> AuthResult<u64>;
}
