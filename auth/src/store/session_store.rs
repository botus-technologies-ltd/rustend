//! Session store module
//! 
//! Provides a generic session store that works with any database.

use crate::models::session::{SessionModel, CreateSession, RefreshTokenModel, CreateRefreshToken};
use crate::utils::errors::{AuthResult};
use database::utils::DbId;

/// Session store trait - implement this for each database
pub trait SessionStore: Send + Sync {
    /// Create a new session
    fn create(&self, input: CreateSession) -> AuthResult<SessionModel>;
    
    /// Find session by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<SessionModel>>;
    
    /// Find session by access token hash
    fn find_by_token(&self, token_hash: &str) -> AuthResult<Option<SessionModel>>;
    
    /// Find all sessions for a user
    fn find_by_user_id(&self, user_id: &DbId) -> AuthResult<Vec<SessionModel>>;
    
    /// Update session (e.g., update last_used_at, extend expiry)
    fn update(&self, id: &DbId, session: SessionModel) -> AuthResult<SessionModel>;
    
    /// Revoke a session
    fn revoke(&self, id: &DbId) -> AuthResult<()>;
    
    /// Revoke all sessions for a user
    fn revoke_all(&self, user_id: &DbId) -> AuthResult<u64>;
    
    /// Delete expired sessions
    fn cleanup_expired(&self) -> AuthResult<u64>;
    
    // Refresh Token methods
    
    /// Create refresh token
    fn create_refresh_token(&self, input: CreateRefreshToken) -> AuthResult<RefreshTokenModel>;
    
    /// Find refresh token by ID
    fn find_refresh_token(&self, id: &DbId) -> AuthResult<Option<RefreshTokenModel>>;
    
    /// Find refresh token by hash
    fn find_refresh_token_by_hash(&self, token_hash: &str) -> AuthResult<Option<RefreshTokenModel>>;
    
    /// Revoke refresh token
    fn revoke_refresh_token(&self, id: &DbId) -> AuthResult<()>;
    
    /// Replace refresh token (for rotation)
    fn replace_refresh_token(&self, old_id: &DbId, new_token: RefreshTokenModel) -> AuthResult<()>;
}
