//! User store module
//! 
//! Provides a generic user store that works with any database.
//! Uses DbId from database crate for flexible database support.

use crate::models::user::{User, CreateUserInput, UpdateUserInput};
use crate::utils::errors::{AuthResult};
use database::utils::DbId;

/// User store trait - implement this for each database
pub trait UserStore: Send + Sync {
    /// Create a new user
    fn create(&self, input: CreateUserInput) -> AuthResult<User>;
    
    /// Find user by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<User>>;
    
    /// Find user by email
    fn find_by_email(&self, email: &str) -> AuthResult<Option<User>>;
    
    /// Find user by phone
    fn find_by_phone(&self, phone: &str) -> AuthResult<Option<User>>;
    
    /// Find user by username
    fn find_by_username(&self, username: &str) -> AuthResult<Option<User>>;
    
    /// Find user by any identifier (email, phone, or username)
    fn find_by_identifier(&self, identifier: &str) -> AuthResult<Option<User>>;
    
    /// Update user
    fn update(&self, id: &DbId, input: UpdateUserInput) -> AuthResult<User>;
    
    /// Delete user
    fn delete(&self, id: &DbId) -> AuthResult<()>;
    
    /// List all users (with pagination)
    fn list(&self, page: u32, limit: u32) -> AuthResult<Vec<User>>;
    
    /// Count total users
    fn count(&self) -> AuthResult<u64>;
}

/// Helper to check if identifier is email, phone, or username
pub fn identify_user(identifier: &str) -> IdentifierType {
    if identifier.contains('@') {
        IdentifierType::Email
    } else if identifier.chars().all(|c| c.is_ascii_digit()) && identifier.len() >= 10 {
        IdentifierType::Phone
    } else {
        IdentifierType::Username
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IdentifierType {
    Email,
    Phone,
    Username,
}
