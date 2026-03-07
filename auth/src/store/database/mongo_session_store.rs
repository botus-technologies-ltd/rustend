//! MongoDB Session Store Implementation

use mongodb::bson::{doc, oid};
use mongodb::sync::Collection;

use crate::models::session::{
    CreateRefreshToken, CreateSession, RefreshTokenModel, SessionModel, UpdateSession,
};
use crate::store::session_store::SessionStore;
use crate::utils::errors::{AuthError, AuthResult};
use database::utils::{DbId, generate_id};

/// MongoDB implementation of SessionStore
pub struct MongoSessionStore {
    session_collection: Collection<SessionModel>,
    refresh_token_collection: Collection<RefreshTokenModel>,
}

impl MongoSessionStore {
    /// Create a new MongoSessionStore
    pub fn new(
        session_collection: Collection<SessionModel>,
        refresh_token_collection: Collection<RefreshTokenModel>,
    ) -> Self {
        Self {
            session_collection,
            refresh_token_collection,
        }
    }
}

impl SessionStore for MongoSessionStore {
    /// Create a new session
    fn create(&self, input: CreateSession) -> AuthResult<SessionModel> {
        let id = generate_id();
        let now = chrono::Utc::now().timestamp();

        let session = SessionModel {
            id: id.clone(),
            user_id: input.user_id.clone(),
            access_token_hash: input.access_token_hash,
            refresh_token_hash: input.refresh_token_hash,
            device: input.device,
            ip_address: input.ip_address,
            user_agent: input.user_agent,
            created_at: now,
            expires_at: now + input.expires_in,
            last_used_at: now,
            is_revoked: false,
        };

        self.session_collection
            .insert_one(&session, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to create session: {}", e)))?;

        Ok(session)
    }

    /// Find session by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<SessionModel>> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        let result = self
            .session_collection
            .find_one(filter, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to find session: {}", e)))?;

        Ok(result)
    }

    /// Find session by access token hash
    fn find_by_token(&self, token_hash: &str) -> AuthResult<Option<SessionModel>> {
        let filter = doc! { "access_token_hash": token_hash };

        let result = self
            .session_collection
            .find_one(filter, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to find session by token: {}", e))
            })?;

        Ok(result)
    }

    /// Find all sessions for a user
    fn find_by_user_id(&self, user_id: &DbId) -> AuthResult<Vec<SessionModel>> {
        let filter = doc! { "user_id": user_id.to_string() };

        let cursor = self
            .session_collection
            .find(filter, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to find sessions: {}", e)))?;

        let sessions: Vec<SessionModel> = cursor.filter_map(|r| r.ok()).collect();

        Ok(sessions)
    }

    /// Update session
    fn update(&self, id: &DbId, session: UpdateSession) -> AuthResult<UpdateSession> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };
        let update = doc! { "$set": {
            "access_token_hash": session.access_token_hash.clone(),
            "refresh_token_hash": session.refresh_token_hash.clone(),
            "expires_at": session.expires_at,
            "is_revoked": session.is_revoked
        }};

        let _ = self
            .session_collection
            .find_one_and_update(filter, update, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to update session: {}", e)))?
            .ok_or_else(|| AuthError::not_found("Session not found"));

        Ok(session)
    }

    /// Revoke a session
    fn revoke(&self, id: &DbId) -> AuthResult<()> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "_id": bson_oid };
        let update = doc! { "$set": { "is_revoked": true }};

        self.session_collection
            .update_one(filter, update, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to revoke session: {}", e)))?;

        Ok(())
    }

    /// Revoke all sessions for a user
    fn revoke_all(&self, user_id: &DbId) -> AuthResult<u64> {
        let filter = doc! { "user_id": user_id.to_string() };
        let update = doc! { "$set": { "is_revoked": true }};

        let result = self
            .session_collection
            .update_many(filter, update, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to revoke sessions: {}", e)))?;

        Ok(result.modified_count)
    }

    /// Delete expired sessions
    fn cleanup_expired(&self) -> AuthResult<u64> {
        let now = chrono::Utc::now().timestamp();
        let filter = doc! { "expires_at": { "$lt": now }};

        let result = self
            .session_collection
            .delete_many(filter, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to cleanup sessions: {}", e))
            })?;

        Ok(result.deleted_count)
    }

    /// Create refresh token
    fn create_refresh_token(&self, input: CreateRefreshToken) -> AuthResult<RefreshTokenModel> {
        let id = generate_id();
        let now = chrono::Utc::now().timestamp();

        let token = RefreshTokenModel {
            id: id.clone(),
            user_id: input.user_id.clone(),
            token_hash: input.token_hash,
            expires_at: now + input.expires_in,
            created_at: now,
            revoked: false,
            revoked_at: None,
            replaced_by: None,
        };

        self.refresh_token_collection
            .insert_one(&token, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to create refresh token: {}", e))
            })?;

        Ok(token)
    }

    /// Find refresh token by ID
    fn find_refresh_token(&self, id: &DbId) -> AuthResult<Option<RefreshTokenModel>> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        let result = self
            .refresh_token_collection
            .find_one(filter, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to find refresh token: {}", e))
            })?;

        Ok(result)
    }

    /// Find refresh token by hash
    fn find_refresh_token_by_hash(
        &self,
        token_hash: &str,
    ) -> AuthResult<Option<RefreshTokenModel>> {
        let filter = doc! { "token_hash": token_hash };

        let result = self
            .refresh_token_collection
            .find_one(filter, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to find refresh token: {}", e))
            })?;

        Ok(result)
    }

    /// Revoke refresh token
    fn revoke_refresh_token(&self, id: &DbId) -> AuthResult<()> {
        let now = chrono::Utc::now().timestamp();
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };
        let update = doc! { "$set": { "revoked": true, "revoked_at": now }};

        self.refresh_token_collection
            .update_one(filter, update, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to revoke refresh token: {}", e))
            })?;

        Ok(())
    }

    /// Replace refresh token (for rotation)
    fn replace_refresh_token(
        &self,
        old_id: &DbId,
        new_token: CreateRefreshToken,
    ) -> AuthResult<()> {
        // Revoke old token
        self.revoke_refresh_token(old_id)?;

        // Create new token
        let input = CreateRefreshToken {
            user_id: new_token.user_id.clone(),
            token_hash: new_token.token_hash.clone(),
            expires_in: new_token.expires_in - chrono::Utc::now().timestamp(),
        };

        self.create_refresh_token(input)?;

        Ok(())
    }
}
