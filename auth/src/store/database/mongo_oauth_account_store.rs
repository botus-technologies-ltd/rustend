//! MongoDB OAuth Account Store Implementation

use mongodb::bson::{doc, oid};
use mongodb::sync::Collection;

use crate::models::oauth::{CreateOAuthAccount, OAuthAccount, OAuthProvider};
use crate::store::oauth_account_store::OAuthAccountStore;
use crate::utils::errors::AuthError;
use database::utils::{DbId, generate_id};

/// MongoDB implementation of OAuthAccountStore
pub struct MongoOAuthAccountStore {
    collection: Collection<OAuthAccount>,
}

impl MongoOAuthAccountStore {
    /// Create a new MongoOAuthAccountStore
    pub fn new(collection: Collection<OAuthAccount>) -> Self {
        Self { collection }
    }

    /// Get the MongoDB collection
    pub fn collection(&self) -> &Collection<OAuthAccount> {
        &self.collection
    }
}

impl OAuthAccountStore for MongoOAuthAccountStore {
    fn create(&self, account: CreateOAuthAccount) -> Result<OAuthAccount, AuthError> {
        let id = generate_id();
        let now = chrono::Utc::now().timestamp();

        let expires_at = account.expires_in.map(|expires_in| now + expires_in);

        let oauth_account = OAuthAccount {
            id: id.clone(),
            user_id: account.user_id,
            provider: account.provider,
            provider_user_id: account.provider_user_id,
            access_token: account.access_token,
            refresh_token: account.refresh_token,
            expires_at,
            scope: account.scope,
            created_at: now,
            updated_at: None,
        };

        self.collection
            .insert_one(&oauth_account, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to create OAuth account: {}", e))
            })?;

        Ok(oauth_account)
    }

    fn find_by_id(&self, id: &DbId) -> Result<Option<OAuthAccount>, AuthError> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "_id": bson_oid };

        self.collection
            .find_one(filter, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to find OAuth account: {}", e)))
    }

    fn find_by_user_and_provider(
        &self,
        user_id: &DbId,
        provider: &OAuthProvider,
    ) -> Result<Option<OAuthAccount>, AuthError> {
        let user_oid = oid::ObjectId::from_bytes(user_id.as_bytes().clone());
        let provider_str = serde_json::to_value(provider)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", provider));

        let filter = doc! {
            "user_id": user_oid,
            "provider": provider_str,
        };

        self.collection
            .find_one(filter, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to find OAuth account: {}", e)))
    }

    fn find_by_provider_user_id(
        &self,
        provider: &OAuthProvider,
        provider_user_id: &str,
    ) -> Result<Option<OAuthAccount>, AuthError> {
        let provider_str = serde_json::to_value(provider)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", provider));

        let filter = doc! {
            "provider": provider_str,
            "provider_user_id": provider_user_id,
        };

        self.collection
            .find_one(filter, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to find OAuth account: {}", e)))
    }

    fn list_by_user(&self, user_id: &DbId) -> Result<Vec<OAuthAccount>, AuthError> {
        let user_oid = oid::ObjectId::from_bytes(user_id.as_bytes().clone());
        let filter = doc! { "user_id": user_oid };

        let mut cursor = self.collection.find(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to list OAuth accounts: {}", e))
        })?;

        let mut accounts = Vec::new();
        while cursor.advance().map_err(|e| {
            AuthError::internal_error(&format!("Failed to iterate OAuth accounts: {}", e))
        })? {
            let account = cursor.deserialize_current().map_err(|e| {
                AuthError::internal_error(&format!("Failed to deserialize OAuth account: {}", e))
            })?;
            accounts.push(account);
        }

        Ok(accounts)
    }

    fn update(&self, id: &DbId, account: &OAuthAccount) -> Result<OAuthAccount, AuthError> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "_id": bson_oid };

        let mut updated_account = account.clone();
        updated_account.updated_at = Some(chrono::Utc::now().timestamp());

        let update = doc! {
            "$set": {
                "access_token": &updated_account.access_token,
                "refresh_token": &updated_account.refresh_token,
                "expires_at": updated_account.expires_at,
                "scope": &updated_account.scope,
                "updated_at": updated_account.updated_at,
            }
        };

        self.collection
            .update_one(filter, update, None)
            .map_err(|e| {
                AuthError::internal_error(&format!("Failed to update OAuth account: {}", e))
            })?;

        Ok(updated_account)
    }

    fn delete(&self, id: &DbId) -> Result<(), AuthError> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "_id": bson_oid };

        self.collection.delete_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to delete OAuth account: {}", e))
        })?;

        Ok(())
    }

    fn delete_by_user(&self, user_id: &DbId) -> Result<(), AuthError> {
        let user_oid = oid::ObjectId::from_bytes(user_id.as_bytes().clone());
        let filter = doc! { "user_id": user_oid };

        self.collection.delete_many(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to delete OAuth accounts: {}", e))
        })?;

        Ok(())
    }

    fn delete_by_user_and_provider(
        &self,
        user_id: &DbId,
        provider: &OAuthProvider,
    ) -> Result<(), AuthError> {
        let user_oid = oid::ObjectId::from_bytes(user_id.as_bytes().clone());
        let provider_str = serde_json::to_value(provider)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| format!("{:?}", provider));

        let filter = doc! {
            "user_id": user_oid,
            "provider": provider_str,
        };

        self.collection.delete_one(filter, None).map_err(|e| {
            AuthError::internal_error(&format!("Failed to delete OAuth account: {}", e))
        })?;

        Ok(())
    }
}
