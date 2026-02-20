# Middleware Crate

Web middleware components for authentication, CORS, rate limiting, and session management.

## Modules

```
middleware/
├── lib.rs         # Main entry point
├── jwt.rs        # JWT authentication
├── sessions.rs   # Session management
├── cors.rs       # CORS configuration
├── rate_limit.rs # Rate limiting
└── README.md     # This file
```

---

## 1. JWT Module (`jwt.rs`)

JWT (JSON Web Token) authentication for securing endpoints.

### Key Types

| Type | Description |
|------|-------------|
| `JwtConfig` | JWT configuration (secret, algorithm, expiry) |
| `Claims` | JWT claims (sub, email, exp, iat) |
| `TokenType` | Enum: Access, Refresh |
| `JwtService` | JWT generation and validation |

### Usage

```rust
use middleware::{JwtConfig, JwtService};

// Create JWT config
let config = JwtConfig::new("your-secret-key")
    .with_access_token_expire_minutes(60); // 1 hour default

// Create JWT service
let jwt_service = JwtService::new(config);

// Generate access token
let token = jwt_service
    .generate_access_token("user_123", Some("user@example.com"))
    .unwrap();

// Validate token
let claims = jwt_service.validate_token(&token).unwrap();
println!("User: {}", claims.sub);
println!("Email: {:?}", claims.email);
```

### Configuration Options

```rust
let config = JwtConfig::new("secret")
    .with_algorithm(Algorithm::HS256)  // HS256, HS384, HS512
    .with_access_token_expire_minutes(60);
```

---

## 2. Sessions Module (`sessions.rs`)

Session management for stateful authentication.

### Key Types

| Type | Description |
|------|-------------|
| `SessionConfig` | Session configuration (cookie name, expiry, security) |
| `SessionData` | Session data (user_id, email, timestamps, custom data) |
| `SessionStore` | In-memory session storage |

### Usage

```rust
use middleware::{SessionConfig, SessionData, SessionStore};

// Create session config
let config = SessionConfig::new()
    .with_cookie_name("session_id")
    .with_expire_seconds(3600)
    .with_secure(true)
    .with_http_only(true)
    .with_path("/");

// Create session store
let store = SessionStore::new(config);

// Create session
let (session_id, session) = store.create("user_123");
println!("Session ID: {}", session_id);

// Get session
let session = store.get(&session_id);
if let Some(s) = session {
    println!("User: {}", s.user_id);
}

// Delete session
store.delete(&session_id);
```

### Session Data Customization

```rust
let mut session = SessionData::new("user_123");
session.email = Some("user@example.com".to_string());
session.data.insert("role".to_string(), "admin".to_string());
```

---

## 3. CORS Module (`cors.rs`)

Cross-Origin Resource Sharing configuration.

### Key Types

| Type | Description |
|------|-------------|
| `CorsConfig` | CORS configuration (origins, methods) |

### Usage

```rust
use middleware::CorsConfig;
use actix_web::http::Method;

// Create CORS config
let cors = CorsConfig::new()
    .with_allowed_origins(vec![
        "https://example.com".to_string(),
        "http://localhost:3000".to_string(),
    ])
    .with_allowed_methods(vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
    ]);

// Apply to app
App::new()
    .wrap(Cors::default() /* or custom cors config */)
```

---

## 4. Rate Limit Module (`rate_limit.rs`)

Request rate limiting to prevent abuse.

### Key Types

| Type | Description |
|------|-------------|
| `RateLimitConfig` | Rate limit configuration (max requests, time window) |
| `RateLimiter` | Rate limiter storage and checking |

### Usage

```rust
use middleware::{RateLimitConfig, RateLimiter};

// Create rate limit config - 100 requests per minute
let config = RateLimitConfig::new(100, 60);

// Create rate limiter
let limiter = RateLimiter::new(config);

// Check if request is allowed
if limiter.check("user_123") {
    // Allow request
} else {
    // Rate limit exceeded
}
```

### Advanced Usage

```rust
// Different limits for different endpoints
let strict_limiter = RateLimiter::new(RateLimitConfig::new(10, 60));   // 10/min for auth
let loose_limiter = RateLimiter::new(RateLimitConfig::new(1000, 60)); // 1000/min for reads

// Check with different keys
limiter.check("ip:192.168.1.1");     // By IP
limiter.check("user:123");            // By user ID
limiter.check("endpoint:/api/items"); // By endpoint
```

---

## Integration with Actix-web

### Full Example

```rust
use actix_web::{App, HttpServer, web, HttpResponse};
use middleware::{JwtConfig, JwtService, SessionConfig, SessionStore, CorsConfig, RateLimitConfig, RateLimiter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize middleware
    let jwt_config = JwtConfig::new("secret-key");
    let jwt_service = JwtService::new(jwt_config);
    
    let session_config = SessionConfig::new();
    let session_store = SessionStore::new(session_config);
    
    let rate_limit_config = RateLimitConfig::new(100, 60);
    let rate_limiter = RateLimiter::new(rate_limit_config);
    
    HttpServer::new(move || {
        App::new()
            // CORS
            .wrap(Cors::default())
            // Rate limiting
            .app_data(web::Data::new(rate_limiter.clone()))
            // Session store
            .app_data(web::Data::new(session_store.clone()))
            // JWT service
            .app_data(web::Data::new(jwt_service.clone()))
            .service(
                web::scope("/api")
                    .route("/items", web::get().to(get_items))
                    .route("/items", web::post().to(create_item))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn get_items() -> HttpResponse {
    HttpResponse::Ok().json(vec!["item1", "item2"])
}

async fn create_item() -> HttpResponse {
    HttpResponse::Created().json("created")
}
```

---

## Protected Routes with JWT

```rust
use actix_web::{web, HttpRequest, HttpResponse, Result};
use middleware::jwt::Claims;

// Middleware to extract JWT
async fn jwt_auth(req: HttpRequest, jwt_service: web::Data<JwtService>) -> Result<HttpResponse> {
    // Get token from header
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());
    
    if let Some(header) = auth_header {
        if let Some(token) = header.strip_prefix("Bearer ") {
            match jwt_service.validate_token(token) {
                Ok(claims) => {
                    // Store claims in request for handler access
                    return Ok(HttpResponse::Ok().json(claims));
                }
                Err(_) => {
                    return Ok(HttpResponse::Unauthorized().json("Invalid token"));
                }
            }
        }
    }
    
    Ok(HttpResponse::Unauthorized().json("No token provided"))
}

// Protected endpoint
#[web::get("/protected")]
async fn protected(claims: web::ReqData<Claims>) -> HttpResponse {
    HttpResponse::Ok().json(format!("Hello, {}!", claims.sub))
}
```

---

## Session-based Authentication

```rust
use actix_web::{web, HttpResponse, cookie::Cookie};

// Login handler - create session
async fn login(
    store: web::Data<SessionStore>,
) -> HttpResponse {
    let (session_id, _) = store.create("user_123");
    
    HttpResponse::Ok()
        .cookie(
            Cookie::build("session_id", session_id)
                .path("/")
                .http_only(true)
                .secure(true)
                .finish()
        )
        .json("logged in")
}

// Logout handler - delete session
async fn logout(
    req: HttpRequest,
    store: web::Data<SessionStore>,
) -> HttpResponse {
    if let Some(session_id) = req.cookie("session_id") {
        store.delete(session_id.value());
    }
    
    HttpResponse::Ok()
        .cookie(
            Cookie::build("session_id", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::ZERO)
                .finish()
        )
        .json("logged out")
}
```

---

## Environment Variables

| Variable | Module | Description |
|----------|--------|-------------|
| `JWT_SECRET` | jwt | Secret key for signing tokens |
| `JWT_ALGORITHM` | jwt | Algorithm (HS256, HS384, HS512) |
| `JWT_EXPIRE_MINUTES` | jwt | Token expiry time |
| `SESSION_COOKIE_NAME` | sessions | Session cookie name |
| `SESSION_EXPIRE_SECONDS` | sessions | Session expiry |
| `CORS_ALLOWED_ORIGINS` | cors | Comma-separated origins |
| `RATE_LIMIT_MAX_REQUESTS` | rate_limit | Max requests per window |
| `RATE_LIMIT_WINDOW_SECONDS` | rate_limit | Time window in seconds |

---

## Dependencies

```toml
[dependencies]
jsonwebtoken = "8"
chrono = "0.4"
parking_lot = "0.12"
actix-web = "4"
actix-cors = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Testing

```bash
cargo test -p middleware
```

---

## License

MIT
