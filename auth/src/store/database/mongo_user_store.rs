//! MongoDB User Store Implementation

use mongodb::bson::{doc, oid};
use mongodb::sync::Collection;

use crate::models::user::{CreateUserInput, UpdateUserInput, User};
use crate::store::user_store::{IdentifierType, UserStore, identify_user};
use crate::utils::errors::{AuthError, AuthResult};
use database::utils::{DbId, generate_id};

/// MongoDB implementation of UserStore
pub struct MongoUserStore {
    collection: Collection<User>,
}

impl MongoUserStore {
    /// Create a new MongoUserStore
    pub fn new(collection: Collection<User>) -> Self {
        Self { collection }
    }

    /// Get the MongoDB collection
    pub fn collection(&self) -> &Collection<User> {
        &self.collection
    }
}

impl UserStore for MongoUserStore {
    /// Create a new user
    fn create(&self, input: CreateUserInput) -> AuthResult<User> {
        // Generate ID
        let id = generate_id();
        let now = chrono::Utc::now().timestamp();

        // Create user document
        let user = User {
            id: id.clone(),
            email: input.email.clone(),
            password_hash: input.password,
            phone: input.phone.clone(),
            username: input.username.clone(),
            first_name: input.first_name.clone(),
            last_name: input.last_name.clone(),
            is_active: true,
            is_verified: false,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: None,
            last_login_at: None,
        };

        // Insert into database
        self.collection
            .insert_one(&user, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to create user: {}", e)))?;

        // Return created user
        Ok(user)
    }

    /// Find user by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<User>> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        let result = self
            .collection
            .find_one(filter, None)
            .map_err(|_e| AuthError::internal_error(&format!("Failed to find user")))?;

        Ok(result)
    }

    /// Find user by email
    fn find_by_email(&self, email: &str) -> AuthResult<Option<User>> {
        let filter = doc! { "email": email };

        let result = self
            .collection
            .find_one(filter, None)
            .map_err(|_e| AuthError::internal_error(&format!("Failed to find user by email")))?;

        Ok(result)
    }

    /// Find user by phone
    fn find_by_phone(&self, phone: &str) -> AuthResult<Option<User>> {
        let filter = doc! { "phone": phone };

        let result = self
            .collection
            .find_one(filter, None)
            .map_err(|_e| AuthError::internal_error(&format!("Failed to find user by phone")))?;

        Ok(result)
    }

    /// Find user by username
    fn find_by_username(&self, username: &str) -> AuthResult<Option<User>> {
        let filter = doc! { "username": username };

        let result = self
            .collection
            .find_one(filter, None)
            .map_err(|_e| AuthError::internal_error(&format!("Failed to find user by username")))?;

        Ok(result)
    }

    /// Find user by any identifier (email, phone, or username)
    fn find_by_identifier(&self, identifier: &str) -> AuthResult<Option<User>> {
        match identify_user(identifier) {
            IdentifierType::Email => self.find_by_email(identifier),
            IdentifierType::Phone => self.find_by_phone(identifier),
            IdentifierType::Username => self.find_by_username(identifier),
        }
    }

    /// Update user
    fn update(&self, id: &DbId, input: UpdateUserInput) -> AuthResult<User> {
        // Build update document
        let mut update_doc = doc! { "$set": {} };
        let set = update_doc.get_document_mut("$set").unwrap();

        if let Some(ref email) = input.email {
            set.insert("email", email);
        }
        if let Some(ref phone) = input.phone {
            set.insert("phone", phone);
        }
        if let Some(ref username) = input.username {
            set.insert("username", username);
        }
        if let Some(ref first_name) = input.first_name {
            set.insert("first_name", first_name);
        }
        if let Some(ref last_name) = input.last_name {
            set.insert("last_name", last_name);
        }
        if let Some(is_active) = input.is_active {
            set.insert("is_active", is_active);
        }
        if let Some(is_verified) = input.is_verified {
            set.insert("is_verified", is_verified);
        }

        // Add updated_at timestamp
        set.insert("updated_at", chrono::Utc::now().timestamp());

        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        let result = self
            .collection
            .find_one_and_update(filter, update_doc, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to update user: {}", e)))?;

        result.ok_or_else(|| AuthError::not_found("User not found"))
    }

    /// Update user password
    fn update_password(&self, id: &DbId, password_hash: &str) -> AuthResult<()> {
        let now = chrono::Utc::now().timestamp();
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };
        let update = doc! {
            "$set": {
                "password_hash": password_hash,
                "updated_at": now
            }
        };

        let result = self
            .collection
            .update_one(filter, update, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to update password: {}", e)))?;

        if result.modified_count == 0 {
            return Err(AuthError::not_found("User not found"));
        }

        Ok(())
    }

    /// Delete user
    fn delete(&self, id: &DbId) -> AuthResult<()> {
        let bson_oid = oid::ObjectId::from_bytes(id.as_bytes().clone());
        let filter = doc! { "id": bson_oid };

        self.collection
            .delete_one(filter, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to delete user: {}", e)))?;

        Ok(())
    }

    /// List all users (with pagination)
    fn list(&self, page: u32, limit: u32) -> AuthResult<Vec<User>> {
        let skip = (page * limit) as usize;
        let limit = limit as usize;
        let filter = doc! {};

        let cursor = self
            .collection
            .find(filter, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to list users: {}", e)))?;

        let users: Vec<User> = cursor
            .skip(skip)
            .take(limit)
            .filter_map(|r| r.ok())
            .collect();

        Ok(users)
    }

    /// Count total users
    fn count(&self) -> AuthResult<u64> {
        let count = self
            .collection
            .count_documents(doc! {}, None)
            .map_err(|e| AuthError::internal_error(&format!("Failed to count users: {}", e)))?;

        Ok(count)
    }
}
