//! Handler module
//!
//! Contains all HTTP request handlers for authentication operations.

pub mod devices;
pub mod forgot_password;
pub mod login_user;
pub mod magic_link;
pub mod oauth;
pub mod reset_password;
pub mod sessions;
pub mod signup_user;
pub mod two_factor;
pub mod users;

// Re-export handlers for easier use
pub use devices::*;
pub use forgot_password::*;
pub use login_user::*;
pub use magic_link::*;
pub use oauth::*;
pub use reset_password::*;
pub use sessions::*;
pub use signup_user::*;
pub use two_factor::*;
pub use users::*;
