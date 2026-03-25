//! Service layer for authentication business logic
//!
//! This module contains services that handle the core business logic
//! for authentication flows, OAuth integration, and account management.

pub mod oauth;
pub mod user;

pub use oauth::OAuthService;
pub use user::UserService;
