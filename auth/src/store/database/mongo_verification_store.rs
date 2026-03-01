//! MongoDB Verification Store Implementation

use mongodb::bson::{doc, oid};
use mongodb::sync::Collection;

use crate::models::verification::{
    CreateVerificationCode, VerificationCodeModel, VerificationMedium, VerificationPurpose,
};
use crate::store::verification_store::VerificationStore;
use crate::utils::errors::{AuthError, AuthResult};
use database::utils::{DbId, generate_id};

/// MongoDB implementation of VerificationStore
pub struct MongoVerificationStore {
    collection: Collection<VerificationCodeModel>,
}

impl MongoVerificationStore {
    /// Create a new MongoVerificationStore
    pub fn new(collection: Collection<VerificationCodeModel>) -> Self {
        Self { collection }
    }
}

impl VerificationStore for MongoVerificationStore {
    /// Create a new verification code
    fn create(&self, input: CreateVerificationCode) -> AuthResult<VerificationCodeModel> {
        let id = generate_id();
        let now = chrono::Utc::now().timestamp();

        let code = VerificationCodeModel {
            id: id.clone(),
            user_id: input.user_id.clone(),
            code_hash: input.code_hash,
            medium: input.medium,
            purpose: input.purpose,
            attempts: 0,
            created_at: now,
            expires_at: now + input.expires_in,
            verified_at: None,
        };

        self.collection.insert_one(&code, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to create verification code: {}", e))
        })?;

        Ok(code)
    }

    /// Find verification code by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<VerificationCodeModel>> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        let result = self.collection.find_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to find verification code: {}", e))
        })?;

        Ok(result)
    }

    /// Find valid verification code by user_id, medium, and purpose
    fn find_valid_code(
        &self,
        user_id: &DbId,
        medium: VerificationMedium,
        purpose: VerificationPurpose,
    ) -> AuthResult<Option<VerificationCodeModel>> {
        let now = chrono::Utc::now().timestamp();
        let filter = doc! {
            "user_id": user_id.to_string(),
            "medium": format!("{:?}", medium).to_lowercase(),
            "purpose": format!("{:?}", purpose).to_lowercase(),
            "expires_at": { "$gt": now },
            "verified_at": null
        };

        let result = self.collection.find_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to find valid verification code: {}", e))
        })?;

        Ok(result)
    }

    /// Verify a code (mark as verified)
    fn verify(&self, id: &DbId) -> AuthResult<()> {
        let now = chrono::Utc::now().timestamp();
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };
        let update = doc! { "$set": { "verified_at": now }};

        self.collection
            .update_one(filter, update, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to verify code: {}", e)))?;

        Ok(())
    }

    /// Increment failed attempts
    fn increment_attempts(&self, id: &DbId) -> AuthResult<()> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };
        let update = doc! { "$inc": { "attempts": 1 }};

        self.collection
            .update_one(filter, update, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to increment attempts: {}", e))
            })?;

        Ok(())
    }

    /// Delete/expire a code
    fn delete(&self, id: &DbId) -> AuthResult<()> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        self.collection.delete_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to delete verification code: {}", e))
        })?;

        Ok(())
    }

    /// Delete all codes for a user
    fn delete_all_for_user(&self, user_id: &DbId) -> AuthResult<u64> {
        let bson_oid = oid::ObjectId::from_bytes(user_id.as_bytes().clone());
        let filter = doc! { "user_id": bson_oid };

        let result = self.collection.delete_many(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to delete verification codes: {}", e))
        })?;

        Ok(result.deleted_count)
    }

    /// Cleanup expired codes
    fn cleanup_expired(&self) -> AuthResult<u64> {
        let now = chrono::Utc::now().timestamp();
        let filter = doc! { "expires_at": { "$lt": now }};

        let result = self.collection.delete_many(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to cleanup expired codes: {}", e))
        })?;

        Ok(result.deleted_count)
    }
}
