//! User service
//!
//! Provides business logic for user operations including
//! user management, validation, and account operations.

use crate::store::user_store::{UserStore};
use crate::utils::errors::AuthError;
use database::utils::DbId;

/// User service for handling user business logic
pub struct UserService<T: UserStore> {
    store: T,
}

impl<T: UserStore> UserService<T> {
    /// Create a new user service
    pub fn new(store: T) -> Self {
        Self { store }
    }

    /// Check if a user exists by email
    pub fn email_exists(&self, email: &str) -> Result<bool, AuthError> {
        let user = self.store.find_by_email(email)?;
        Ok(user.is_some())
    }

    /// Check if a user exists by phone
    pub fn phone_exists(&self, phone: &str) -> Result<bool, AuthError> {
        let user = self.store.find_by_phone(phone)?;
        Ok(user.is_some())
    }

    /// Check if a user exists by username
    pub fn username_exists(&self, username: &str) -> Result<bool, AuthError> {
        let user = self.store.find_by_username(username)?;
        Ok(user.is_some())
    }

    /// Check if a user exists by any identifier (email, phone, or username)
    pub fn user_exists(&self, identifier: &str) -> Result<bool, AuthError> {
        let user = self.store.find_by_identifier(identifier)?;
        Ok(user.is_some())
    }

    /// Check if a user exists by ID
    pub fn user_exists_by_id(&self, user_id: &DbId) -> Result<bool, AuthError> {
        let user = self.store.find_by_id(user_id)?;
        Ok(user.is_some())
    }

}

impl<T: UserStore> Default for UserService<T> {
    fn default() -> Self {
        // This is a placeholder - in practice, you'd need to inject the store
        panic!("UserService cannot be created without a store. Use UserService::new(store) instead.")
    }
}