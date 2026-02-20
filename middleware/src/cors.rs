//! CORS Configuration

use actix_web::http::Method;

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![],
            allowed_methods: vec![Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS],
        }
    }
}

impl CorsConfig { pub fn new() -> Self { Self::default() } }
