# Auth Crate

Authentication and authorization system with multiple authentication methods.

## Features

- User registration and login
- Session management
- JWT tokens
- Password reset
- Email verification
- OAuth support
- Two-factor authentication
- Magic links
- Rate limiting

## Architecture

```
auth/
├── lib.rs              # Main entry point
├── routes.rs           # HTTP routes
├── handler/            # Request handlers
│   ├── login_user.rs
│   ├── forgot_password.rs
│   ├── reset_password.rs
│   └── users.rs
├── models/            # Data models
│   ├── user.rs
│   ├── session.rs
│   ├── verification.rs
│   ├── reset_password.rs
│   ├── oauth.rs
│   ├── two_factor.rs
│   └── magic_link.rs
├── store/             # Data stores
│   ├── user_store.rs
│   ├── session_store.rs
│   ├── verification_store.rs
│   ├── password_reset_store.rs
│   └── database/
│       ├── mongo_user_store.rs
│       ├── mongo_session_store.rs
│       ├── mongo_verification_store.rs
│       └── mongo_password_reset_store.rs
└── utils/             # Utilities
    ├── errors.rs
    └── types.rs
```

---

## 1. Models Module (`models/`)

### User Model (`user.rs`)

```rust
use database::utils::DbId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: DbId,
    pub email: Option<String>,
    pub password_hash: String,
    pub phone: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub failed_login_attempts: i32,
    pub locked_until: Option<i64>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
    pub last_login_at: Option<i64>,
}

impl User {
    pub fn is_locked(&self) -> bool;
    pub fn can_attempt_login(&self) -> bool;
    pub fn record_failed_attempt(&mut self, lock_threshold: i32, lock_duration_secs: i64);
    pub fn reset_failed_attempts(&mut self);
}
```

### Input Types

```rust
// User registration
pub struct CreateUserInput {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

// User update
pub struct UpdateUserInput {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: Option<bool>,
    pub is_verified: Option<bool>,
}
```

---

### Session Model (`session.rs`)

```rust
use database::utils::DbId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionModel {
    pub id: DbId,
    pub user_id: DbId,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub device: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_used_at: i64,
    pub is_revoked: bool,
}

impl SessionModel {
    pub fn is_expired(&self) -> bool;
    pub fn is_valid(&self) -> bool;
    pub fn update_last_used(&mut self);
}

pub struct RefreshTokenModel {
    pub id: DbId,
    pub user_id: DbId,
    pub token_hash: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub revoked: bool,
    pub revoked_at: Option<i64>,
    pub replaced_by: Option<String>,
}
```

### Other Models

| Model | Description |
|-------|-------------|
| `Verification` | Email/phone verification codes |
| `PasswordReset` | Password reset tokens |
| `OAuthUser` | OAuth provider data |
| `TwoFactor` | 2FA configuration |
| `MagicLink` | Passwordless login links |

---

## 2. Store Module (`store/`)

The store module provides trait-based data access that can be implemented for any database.

### UserStore Trait

```rust
use crate::models::user::{User, CreateUserInput, UpdateUserInput};
use crate::utils::errors::AuthResult;
use database::utils::DbId;

pub trait UserStore: Send + Sync {
    fn create(&self, input: CreateUserInput) -> AuthResult<User>;
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<User>>;
    fn find_by_email(&self, email: &str) -> AuthResult<Option<User>>;
    fn find_by_phone(&self, phone: &str) -> AuthResult<Option<User>>;
    fn find_by_username(&self, username: &str) -> AuthResult<Option<User>>;
    fn find_by_identifier(&self, identifier: &str) -> AuthResult<Option<User>>;
    fn update(&self, id: &DbId, input: UpdateUserInput) -> AuthResult<User>;
    fn delete(&self, id: &DbId) -> AuthResult<()>;
    fn list(&self, page: u32, limit: u32) -> AuthResult<Vec<User>>;
    fn count(&self) -> AuthResult<u64>;
}
```

### SessionStore Trait

```rust
use crate::models::session::{SessionModel, RefreshTokenModel, CreateSession, CreateRefreshToken};
use database::utils::DbId;

pub trait SessionStore: Send + Sync {
    fn create_session(&self, input: CreateSession) -> AuthResult<SessionModel>;
    fn get_session(&self, id: &DbId) -> AuthResult<Option<SessionModel>>;
    fn get_user_sessions(&self, user_id: &DbId) -> AuthResult<Vec<SessionModel>>;
    fn revoke_session(&self, id: &DbId) -> AuthResult<()>;
    fn revoke_all_user_sessions(&self, user_id: &DbId) -> AuthResult<()>;
    fn cleanup_expired_sessions(&self) -> AuthResult<u64>;
    
    fn create_refresh_token(&self, input: CreateRefreshToken) -> AuthResult<RefreshTokenModel>;
    fn get_refresh_token(&self, token_hash: &str) -> AuthResult<Option<RefreshTokenModel>>;
    fn revoke_refresh_token(&self, token_hash: &str) -> AuthResult<()>;
}
```

### VerificationStore Trait

```rust
use crate::models::verification::Verification;
use database::utils::DbId;

pub trait VerificationStore: Send + Sync {
    fn create(&self, verification: Verification) -> AuthResult<Verification>;
    fn find_by_token(&self, token: &str) -> AuthResult<Option<Verification>>;
    fn find_by_user(&self, user_id: &DbId, verification_type: &str) -> AuthResult<Option<Verification>>;
    fn mark_used(&self, id: &DbId) -> AuthResult<()>;
    fn delete(&self, id: &DbId) -> AuthResult<()>;
    fn delete_expired(&self) -> AuthResult<u64>;
}
```

### PasswordResetStore Trait

```rust
use crate::models::reset_password::PasswordReset;
use database::utils::DbId;

pub trait PasswordResetStore: Send + Sync {
    fn create(&self, reset: PasswordReset) -> AuthResult<PasswordReset>;
    fn find_by_token(&self, token: &str) -> AuthResult<Option<PasswordReset>>;
    fn mark_used(&self, id: &DbId) -> AuthResult<()>;
    fn delete_expired(&self) -> AuthResult<u64>;
}
```

---

## 3. MongoDB Implementation (`store/database/`)

### Example: MongoUserStore

```rust
use crate::models::user::{User, CreateUserInput, UpdateUserInput};
use crate::store::user_store::UserStore;
use crate::utils::errors::AuthResult;
use database::utils::DbId;

pub struct MongoUserStore {
    collection: Collection<User>,
}

#[async_trait]
impl UserStore for MongoUserStore {
    async fn create(&self, input: CreateUserInput) -> AuthResult<User> {
        let password_hash = hash_password(&input.password)?;
        let user = User {
            id: DbId::from_object_id(&ObjectId::new().to_string()),
            email: input.email,
            password_hash,
            phone: input.phone,
            username: input.username,
            first_name: input.first_name,
            last_name: input.last_name,
            is_active: true,
            is_verified: false,
            failed_login_attempts: 0,
            locked_until: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: None,
            last_login_at: None,
        };
        
        self.collection.insert_one(user.clone()).await?;
        Ok(user)
    }
    
    async fn find_by_email(&self, email: &str) -> AuthResult<Option<User>> {
        let filter = doc! { "email": email };
        Ok(self.collection.find_one(filter).await?)
    }
    // ... implement other methods
}
```

---

## 4. Routes Module (`routes.rs`)

```rust
use actix_web::{web, HttpResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/logout", web::post().to(logout))
            .route("/refresh", web::post().to(refresh_token))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password))
            .route("/verify-email", web::post().to(verify_email))
            .route("/resend-verification", web::post().to(resend_verification))
    );
}
```

---

## 5. Handler Examples

### Login Handler

```rust
use actix_web::{web, HttpRequest, HttpResponse, Error};
use crate::models::user::User;
use crate::store::user_store::UserStore;
use crate::utils::errors::AuthResult;

pub async fn login(
    req: HttpRequest,
    state: web::Data<AppState>,
    login_req: web::Json<LoginRequest>,
) -> Result<HttpResponse, Error> {
    let users = &state.users;
    
    // Find user by email, phone, or username
    let user = users
        .find_by_identifier(&login_req.identifier)
        .await?
        .ok_or_else(|| AuthError::InvalidCredentials)?;
    
    // Check if account is locked
    if user.is_locked() {
        return Err(AuthError::AccountLocked.into());
    }
    
    // Verify password
    if !verify_password(&login_req.password, &user.password_hash)? {
        // Record failed attempt
        return Err(AuthError::InvalidCredentials.into());
    }
    
    // Create session
    let session = create_session(&state.sessions, &user, &req).await?;
    
    Ok(HttpResponse::Ok().json(session))
}
```

### Registration Handler

```rust
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, Error> {
    // Check if user already exists
    if let Some(_) = state.users.find_by_email(&req.email).await? {
        return Err(AuthError::UserAlreadyExists.into());
    }
    
    // Create user
    let input = CreateUserInput {
        email: Some(req.email.clone()),
        phone: req.phone.clone(),
        username: req.username.clone(),
        password: req.password.clone(),
        first_name: req.first_name.clone(),
        last_name: req.last_name.clone(),
    };
    
    let user = state.users.create(input).await?;
    
    // Send verification email
    send_verification_email(&user).await?;
    
    Ok(HttpResponse::Created().json(user))
}
```

---

## 6. App State

```rust
use crate::store::{UserStore, SessionStore, VerificationStore, PasswordResetStore};

pub struct AppState {
    pub users: Arc<dyn UserStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub verifications: Arc<dyn VerificationStore>,
    pub password_resets: Arc<dyn PasswordResetStore>,
    pub jwt_secret: String,
    pub jwt_expiry_minutes: i64,
}

impl AppState {
    pub fn new(
        users: Arc<dyn UserStore>,
        sessions: Arc<dyn SessionStore>,
        verifications: Arc<dyn VerificationStore>,
        password_resets: Arc<dyn PasswordResetStore>,
        jwt_secret: String,
    ) -> Self {
        Self {
            users,
            sessions,
            verifications,
            password_resets,
            jwt_secret,
            jwt_expiry_minutes: 60,
        }
    }
}
```

---

## 7. Password Hashing

Uses bcrypt for secure password hashing:

```rust
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> AuthResult<String> {
    Ok(hash(password, DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> AuthResult<bool> {
    Ok(verify(password, hash)?)
}
```

---

## 8. Rate Limiting

Built-in rate limiting for auth endpoints:

```rust
pub struct RateLimit {
    pub identifier: String,
    pub action: String,
    pub count: i32,
    pub window_start: i64,
    pub window_duration: i64,
}

impl RateLimit {
    pub fn is_exceeded(&self, max_attempts: i32) -> bool;
    pub fn increment(&mut self);
    pub fn should_reset(&self) -> bool;
}
```

---

## Full Example

### Main Application

```rust
use actix_web::{App, HttpServer, web};
use auth::{AppState, routes};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize stores (MongoDB example)
    let user_store = MongoUserStore::new(/* config */).await?;
    let session_store = MongoSessionStore::new(/* config */).await?;
    let verification_store = MongoVerificationStore::new(/* config */).await?;
    let password_reset_store = MongoPasswordResetStore::new(/* config */).await?;
    
    let state = AppState::new(
        Arc::new(user_store),
        Arc::new(session_store),
        Arc::new(verification_store),
        Arc::new(password_reset_store),
        std::env::var("JWT_SECRET").unwrap(),
    );
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `JWT_SECRET` | Secret key for JWT signing |
| `JWT_EXPIRY_MINUTES` | Token expiry time (default: 60) |
| `RATE_LIMIT_MAX_ATTEMPTS` | Max login attempts before lock |
| `RATE_LIMIT_WINDOW_SECONDS` | Rate limit window |
| `LOCK_DURATION_SECONDS` | Account lock duration |
| `VERIFICATION_EXPIRY_HOURS` | Verification link expiry |
| `PASSWORD_RESET_EXPIRY_HOURS` | Reset link expiry |

---

## Error Handling

```rust
pub enum AuthError {
    InvalidCredentials,
    UserAlreadyExists,
    AccountLocked,
    AccountNotVerified,
    TokenExpired,
    TokenInvalid,
    RateLimitExceeded,
    // ... other variants
}

impl actix_web::ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::InvalidCredentials => 
                HttpResponse::Unauthorized().json("Invalid credentials"),
            AuthError::UserAlreadyExists => 
                HttpResponse::BadRequest().json("User already exists"),
            AuthError::AccountLocked => 
                HttpResponse::TooManyRequests().json("Account locked"),
            // ... other responses
        }
    }
}
```

---

## Dependencies

```toml
[dependencies]
auth = { path = "../auth" }
database = { path = "../database", features = ["mongodb"] }
bcrypt = "0.17"
jsonwebtoken = "8"
```

---

## Testing

```bash
cargo test -p auth
```

---

## License

MIT
