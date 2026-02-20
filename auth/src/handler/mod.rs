//! Handler module
//! 
//! Contains all HTTP request handlers for authentication operations.

pub mod login_user;
pub mod signup_user;
pub mod forgot_password;
pub mod reset_password;
pub mod users;
pub mod sessions;
pub mod oauth;
pub mod two_factor;
pub mod magic_link;
pub mod devices;

// Re-export handlers for easier use
pub use login_user::*;
pub use signup_user::*;
pub use forgot_password::*;
pub use reset_password::*;
pub use users::*;
pub use sessions::*;
pub use oauth::*;
pub use two_factor::*;
pub use magic_link::*;
pub use devices::*;
