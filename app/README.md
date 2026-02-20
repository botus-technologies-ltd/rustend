# App Crate

Main application crate that ties together all other crates (auth, database, middleware, payments).

## Overview

The app crate is the entry point for the application. It configures:
- Database connections
- Authentication
- Middleware (CORS, JWT, sessions, rate limiting)
- Payment gateway
- Routes

## Architecture

```
app/
├── Cargo.toml
├── .env.local           # Environment variables
├── src/
│   ├── lib.rs           # Module exports
│   ├── main.rs          # Application entry point
│   ├── config.rs        # Configuration management
│   ├── state.rs         # Application state
│   └── routes.rs        # Route configuration
└── README.md           # This file
```

---

## 1. Configuration (`config.rs`)

Manages application configuration from environment variables.

### AppConfig

```rust
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db_uri: String,
    pub db_name: String,
    pub server_ip: String,
    pub server_port: u16,
    pub jwt_secret: String,
}
```

### Usage

```rust
use app::config::AppConfig;

let config = AppConfig::from_env();

// Access configuration
println!("Database: {}", config.db_uri);
println!("Server: {}:{}", config.server_ip, config.server_port);
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `DB_URI` | Database connection URI | Yes |
| `DB_NAME` | Database name | Yes |
| `SERVER_IP` | Server IP address | Yes |
| `SERVER_PORT` | Server port | Yes |
| `JWT_SECRET` | JWT secret key | Yes |

### Example .env.local

```
DB_URI=mongodb://localhost:27017
DB_NAME=myapp
SERVER_IP=127.0.0.1
SERVER_PORT=8080
JWT_SECRET=your-secret-key-here
```

---

## 2. Application State (`state.rs`)

Holds shared application state accessible to all request handlers.

### AppState

```rust
#[derive(Clone, Default)]
pub struct AppState {
    // Add your state fields here
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}
```

### Extended Example

```rust
use std::sync::Arc;
use auth::store::{UserStore, SessionStore, VerificationStore, PasswordResetStore};
use payments::PaymentGateway;

#[derive(Clone)]
pub struct AppState {
    // Database stores
    pub users: Arc<dyn UserStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub verifications: Arc<dyn VerificationStore>,
    pub password_resets: Arc<dyn PasswordResetStore>,
    
    // Payment gateway
    pub payments: Arc<dyn PaymentGateway>,
    
    // Configuration
    pub jwt_secret: String,
    pub jwt_expiry_minutes: i64,
}

impl AppState {
    pub fn new(
        users: Arc<dyn UserStore>,
        sessions: Arc<dyn SessionStore>,
        verifications: Arc<dyn VerificationStore>,
        password_resets: Arc<dyn PasswordResetStore>,
        payments: Arc<dyn PaymentGateway>,
        jwt_secret: String,
    ) -> Self {
        Self {
            users,
            sessions,
            verifications,
            password_resets,
            payments,
            jwt_secret,
            jwt_expiry_minutes: 60,
        }
    }
}
```

---

## 3. Routes (`routes.rs`)

Configures all application routes.

### Basic Setup

```rust
use actix_web::web;
use auth::routes::configure as auth_configure;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    // Auth routes
    auth_configure(cfg);
}
```

### Extended Setup with Custom Routes

```rust
use actix_web::web;
use auth::routes::configure as auth_configure;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    // Auth routes
    auth_configure(cfg);
    
    // API routes
    cfg.service(
        web::scope("/api/v1")
            .route("/users", web::get().to(super::handlers::list_users))
            .route("/users/{id}", web::get().to(super::handlers::get_user))
            .route("/items", web::get().to(super::handlers::list_items))
            // Add more routes here
    );
    
    // Public routes
    cfg.service(
        web::scope("/public")
            .route("/info", web::get().to(super::handlers::public_info))
    );
}
```

---

## 4. Main Application (`main.rs`)

Entry point for the application.

### Basic Setup

```rust
use actix_web::{App, HttpServer, web};
use actix_cors::Cors;

use app::config::AppConfig;
use app::state::AppState;
use app::routes::init_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configs
    let config = AppConfig::from_env();

    // Create app state
    let state = AppState::new();

    // Run server
    HttpServer::new(move || {
        App::new()
            // CORS
            .wrap(Cors::default())
            // App state
            .app_data(web::Data::new(state.clone()))
            // Routes
            .configure(init_routes)
            // Health check
            .service(health_check)
    })
    .bind((config.server_ip, config.server_port))?
    .run()
    .await
}

#[actix_web::get("/health")]
async fn health_check() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}
```

### Extended Setup with Full Integration

```rust
use actix_web::{App, HttpServer, web};
use actix_cors::Cors;

use std::sync::Arc;

use app::config::AppConfig;
use app::state::AppState;
use app::routes::init_routes;

// Database
use database::init::{DatabaseConfig, init_database};
use database::utils::DatabaseType;

// Auth stores
use auth::store::database::MongoUserStore;
use auth::store::database::MongoSessionStore;
use auth::store::database::MongoVerificationStore;
use auth::store::database::MongoPasswordResetStore;

// Payments
use payments::config::PaymentConfig;
use payments::providers::VisaGateway;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load config
    let config = AppConfig::from_env();
    
    // Initialize database
    let db_config = DatabaseConfig::new(
        DatabaseType::MongoDB,
        &config.db_uri,
        &config.db_name,
    );
    let _db = init_database(db_config).await
        .expect("Failed to initialize database");
    
    // Initialize auth stores
    let user_store = Arc::new(MongoUserStore::new(&config.db_name).await?);
    let session_store = Arc::new(MongoSessionStore::new(&config.db_name).await?);
    let verification_store = Arc::new(MongoVerificationStore::new(&config.db_name).await?);
    let password_reset_store = Arc::new(MongoPaymentResetStore::new(&config.db_name).await?);
    
    // Initialize payments
    let payment_config = PaymentConfig::new()
        .add_provider(ProviderConfig::visa("api_key", "webhook_secret"))
        .with_default(PaymentProvider::Visa);
    let payments = Arc::new(payment_config.build());
    
    // Create app state
    let state = AppState::new(
        user_store,
        session_store,
        verification_store,
        password_reset_store,
        payments,
        config.jwt_secret,
    );

    // Run server
    HttpServer::new(move || {
        App::new()
            // CORS - configure for your needs
            .wrap(Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec!["Authorization", "Content-Type"])
            )
            // JWT middleware
            .wrap(middleware::jwt::JwtMiddleware::new(config.jwt_secret.clone()))
            // Rate limiting
            .app_data(web::Data::new(rate_limiter.clone()))
            // App state
            .app_data(web::Data::new(state.clone()))
            // Routes
            .configure(init_routes)
            // Health check
            .service(health_check)
    })
    .bind((config.server_ip, config.server_port))?
    .run()
    .await
}
```

---

## 5. Adding Custom Handlers

Create handlers module in `src/`:

```rust
// src/handlers.rs

use actix_web::{web, HttpResponse, Result};
use crate::state::AppState;

pub async fn list_users(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let users = state.users.list(1, 10).await?;
    Ok(HttpResponse::Ok().json(users))
}

pub async fn get_user(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    if let Some(user) = state.users.find_by_id(&user_id).await? {
        Ok(HttpResponse::Ok().json(user))
    } else {
        Ok(HttpResponse::NotFound().json("User not found"))
    }
}
```

Register in `lib.rs`:

```rust
pub mod handlers;
```

---

## 6. Request/Response Types

Create in `src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}
```

---

## 7. Middleware

### JWT Authentication Middleware

```rust
use actix_web::{Error, HttpRequest, dev::ServiceRequest};
use middleware::jwt::{JwtService, JwtConfig, Claims};

pub async fn jwt_auth(
    req: HttpRequest,
    jwt_service: web::Data<JwtService>,
) -> Result<HttpResponse, Error> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());
    
    if let Some(header) = auth_header {
        if let Some(token) = header.strip_prefix("Bearer ") {
            match jwt_service.validate_token(token) {
                Ok(claims) => {
                    // Store claims in request for later use
                    Ok(HttpResponse::Ok().json(claims))
                }
                Err(_) => {
                    Ok(HttpResponse::Unauthorized().json("Invalid token"))
                }
            }
        } else {
            Ok(HttpResponse::Unauthorized().json("Invalid authorization header"))
        }
    } else {
        Ok(HttpResponse::Unauthorized().json("No token provided"))
    }
}
```

### Rate Limiting Middleware

```rust
use middleware::{RateLimitConfig, RateLimiter};

pub struct AppRateLimiter {
    limiter: RateLimiter,
}

impl AppRateLimiter {
    pub fn new() -> Self {
        Self {
            limiter: RateLimiter::new(
                RateLimitConfig::new(100, 60) // 100 requests per minute
            ),
        }
    }
    
    pub fn check(&self, key: &str) -> bool {
        self.limiter.check(key)
    }
}
```

---

## 8. Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_config_from_env() {
        // Set test env vars
        std::env::set_var("DB_URI", "mongodb://localhost:27017");
        std::env::set_var("DB_NAME", "testdb");
        std::env::set_var("SERVER_IP", "127.0.0.1");
        std::env::set_var("SERVER_PORT", "8080");
        std::env::set_var("JWT_SECRET", "test-secret");
        
        let config = AppConfig::from_env();
        
        assert_eq!(config.db_name, "testdb");
    }
}
```

### Integration Tests

```rust
#[actix_web::test]
async fn test_health_check() {
    let app = actix_web::test::init_service!(
        App::new()
            .service(health_check)
    ).await;
    
    let req = actix_web::test::TestRequest::get()
        .uri("/health")
        .to_request();
    
    let resp = actix_web::test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

---

## 9. Project Structure

```
myapp/
├── Cargo.toml
├── .env.local
├── app/                    # Main application
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── state.rs
│   │   ├── routes.rs
│   │   └── handlers.rs
│   └── README.md
├── auth/                   # Authentication crate
├── database/              # Database crate
├── middleware/            # Middleware crate
├── payments/             # Payments crate
└── utils/                # Utility crate
```

---

## 10. Dependencies

```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2021"

[dependencies]
app = { path = "./app" }
auth = { path = "./auth" }
database = { path = "./database", features = ["mongodb"] }
middleware = { path = "./middleware" }
payments = { path = "./payments" }
utils = { path = "./utils" }

actix-web = "4"
actix-cors = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenvy = "0.15"
```

---

## Running the Application

```bash
# Development
cd app
cargo run

# Or from workspace root
cargo run -p app
```

---

## License

MIT
