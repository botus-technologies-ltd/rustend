//! MongoDB Password Reset Store Implementation

use mongodb::bson::{doc, oid};
use mongodb::sync::Collection;

use crate::models::reset_password::{CreatePasswordResetToken, PasswordResetTokenModel};
use crate::store::password_reset_store::PasswordResetStore;
use crate::utils::errors::{AuthError, AuthResult};
use database::utils::{DbId, generate_id};

/// MongoDB implementation of PasswordResetStore
pub struct MongoPasswordResetStore {
    collection: Collection<PasswordResetTokenModel>,
}

impl MongoPasswordResetStore {
    /// Create a new MongoPasswordResetStore
    pub fn new(collection: Collection<PasswordResetTokenModel>) -> Self {
        Self { collection }
    }
}

impl PasswordResetStore for MongoPasswordResetStore {
    /// Create a new password reset token
    fn create(&self, input: CreatePasswordResetToken) -> AuthResult<PasswordResetTokenModel> {
        let id = generate_id();
        let now = chrono::Utc::now().timestamp();

        let token = PasswordResetTokenModel {
            id: id.clone(),
            user_id: input.user_id.clone(),
            token_hash: input.token_hash,
            created_at: now,
            expires_at: now + input.expires_in,
            used_at: None,
        };

        self.collection.insert_one(&token, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to create password reset token: {}", e))
        })?;

        Ok(token)
    }

    /// Find password reset token by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<PasswordResetTokenModel>> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        let result = self.collection.find_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to find password reset token: {}", e))
        })?;

        Ok(result)
    }

    /// Find valid password reset token by user_id
    fn find_valid_token(&self, user_id: &DbId) -> AuthResult<Option<PasswordResetTokenModel>> {
        let now = chrono::Utc::now().timestamp();
        let filter = doc! {
            "user_id": user_id.to_string(),
            "expires_at": { "$gt": now },
            "used_at": null
        };

        let result = self.collection.find_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to find valid password reset token: {}", e))
        })?;

        Ok(result)
    }

    /// Find token by hash
    fn find_by_hash(&self, token_hash: &str) -> AuthResult<Option<PasswordResetTokenModel>> {
        let filter = doc! { "token_hash": token_hash };

        let result = self.collection.find_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to find password reset token: {}", e))
        })?;

        Ok(result)
    }

    /// Mark token as used
    fn mark_used(&self, id: &DbId) -> AuthResult<()> {
        let now = chrono::Utc::now().timestamp();
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };
        let update = doc! { "$set": { "used_at": now }};

        self.collection
            .update_one(filter, update, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to mark token as used: {}", e))
            })?;

        Ok(())
    }

    /// Delete/expire a token
    fn delete(&self, id: &DbId) -> AuthResult<()> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        self.collection.delete_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to delete password reset token: {}", e))
        })?;

        Ok(())
    }

    /// Delete all tokens for a user
    fn delete_all_for_user(&self, user_id: &DbId) -> AuthResult<u64> {
        let filter = doc! { "user_id": user_id.to_string() };

        let result = self.collection.delete_many(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to delete password reset tokens: {}", e))
        })?;

        Ok(result.deleted_count)
    }

    /// Cleanup expired tokens
    fn cleanup_expired(&self) -> AuthResult<u64> {
        let now = chrono::Utc::now().timestamp();
        let filter = doc! { "expires_at": { "$lt": now }};

        let result = self.collection.delete_many(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to cleanup expired tokens: {}", e))
        })?;

        Ok(result.deleted_count)
    }
}
