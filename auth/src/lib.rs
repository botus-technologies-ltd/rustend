pub mod config;
pub mod handler;
pub mod models;
pub mod routes;
pub mod service;
pub mod store;
pub mod utils;
pub mod tests;

// Use external utils crate (imported as a crate dependency)
use middleware::logger;

// Initialize logging for auth crate
pub fn init_logging() {
    let _guard = logger::init_with_level("auth", "info");
    middleware::tracing::info!("Auth module initialized...");
}

// Re-export logging utilities for convenience
pub use middleware::tracing::{debug, error, info, trace, warn};
