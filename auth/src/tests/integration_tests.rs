//! Integration tests for authentication functionality
//!
//! These tests verify the complete authentication flow including:
//! - User registration (signup)
//! - Email/phone validation
//! - Password hashing
//! - Database operations
//! - Error handling

use actix_web::{
    test,
    web::{self},
    App, Result,
};
use chrono::Utc;
use mongodb::{
    bson::{doc, oid::ObjectId},
    sync::{Client, Database},
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    handler::signup_user,
    models::user::{CreateUserInput, User},
    routes::AppState,
    store::database::MongoUserStore,
    store::user_store::UserStore,
};
use database::utils::DbId;
use utils::response::ApiResponse;
use utils::email::EmailService;
use crate::utils::types::{SignInResponse, SignUpResponseData};

/// Test user data for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUser {
    pub email: String,
    pub phone: String,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

impl TestUser {
    pub fn new() -> Self {
        let timestamp = Utc::now().timestamp();
        let mut rng = rand::thread_rng();
        Self {
            email: format!("test_user_{}@example.com", timestamp),
            phone: format!("+2547{}{}{}{}{}{}{}", 
                rng.gen_range(0..10),
                rng.gen_range(0..10),
                rng.gen_range(0..10),
                rng.gen_range(0..10),
                rng.gen_range(0..10),
                rng.gen_range(0..10),
                rng.gen_range(0..10),
            ),
            username: format!("testuser_{}", timestamp),
            password: "SecurePassword123!".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
        }
    }
}

/// Test setup for integration tests
pub struct TestSetup {
    pub db: Database,
    pub user_store: Arc<MongoUserStore>,
    pub app_state: AppState,
}

impl TestSetup {
    /// Create a new test setup with a test database
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Connect to test database
        let client = Client::with_uri_str("mongodb://localhost:27017")?;
        let db = client.database("nyumberni_test");
        
        // Create user collection
        let user_collection = db.collection::<User>("users_test");
        let user_store = Arc::new(MongoUserStore::new(user_collection));

        // Create app state
        let app_state = AppState {
            users: user_store.clone(),
            sessions: Arc::new(crate::store::database::MongoSessionStore::new(
                db.collection("sessions_test"),
                db.collection("refresh_tokens_test"),
            )),
            password_resets: Some(Arc::new(crate::store::database::MongoPasswordResetStore::new(
                db.collection("password_reset_tokens_test"),
            ))),
            verifications: Some(Arc::new(crate::store::database::MongoVerificationStore::new(
                db.collection("verification_codes_test"),
            ))),
            oauth_accounts: Some(Arc::new(crate::store::database::MongoOAuthAccountStore::new(
                db.collection("oauth_accounts_test"),
            ))),
            jwt_secret: "test_jwt_secret_key_for_integration_tests".to_string(),
            jwt_expiry_minutes: 60,
            refresh_token_expiry_days: 30,
            email: Arc::new(EmailService::smtp(
                utils::email::SmtpConfig::new(
                    "smtp.example.com".to_string(),
                    587,
                    "test@example.com".to_string(),
                    "test_password".to_string(),
                )
            )),
            email_from: "noreply@example.com".to_string(),
            app_name: "Test App".to_string(),
            frontend_url: "http://localhost:3000".to_string(),
        };

        Ok(Self {
            db,
            user_store,
            app_state,
        })
    }

    /// Clean up test data
    pub fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Drop test collections
        self.db.collection::<User>("users_test").drop(None)?;
        self.db.collection::<User>("sessions_test").drop(None)?;
        self.db.collection::<User>("refresh_tokens_test").drop(None)?;
        self.db.collection::<User>("password_reset_tokens_test").drop(None)?;
        self.db.collection::<User>("verification_codes_test").drop(None)?;
        self.db.collection::<User>("oauth_accounts_test").drop(None)?;
        Ok(())
    }
}

/// Integration test for successful user registration
#[actix_web::test]
async fn test_successful_user_registration() -> Result<(), Box<dyn std::error::Error>> {
    let setup = TestSetup::new()?;
    let _cleanup_guard = CleanupGuard(&setup);

    let test_user = TestUser::new();
    let signup_request = crate::utils::types::SignUpRequest {
        email: Some(test_user.email.clone()),
        phone: Some(test_user.phone.clone()),
        username: Some(test_user.username.clone()),
        password: test_user.password.clone(),
        first_name: Some(test_user.first_name.clone()),
        last_name: Some(test_user.last_name.clone()),
    };

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(setup.app_state.clone()))
            .route("/signup", web::post().to(signup_user)),
    )
    .await;

    // Send signup request
    let req = test::TestRequest::post()
        .uri("/signup")
        .set_json(&signup_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify response
    assert_eq!(resp.status(), 200);
    let response: ApiResponse<SignUpResponseData> = test::read_body_json(resp).await;
    
    assert!(response.success);
    assert_eq!(response.message, "User registered successfully");
    assert!(response.data.is_some());
    
    let user_data = response.data.unwrap();
    assert!(!user_data.user_id.is_empty());
    assert_eq!(user_data.email, Some(test_user.email.clone()));

    // Verify user was created in database
    let user = setup.user_store.find_by_email(&test_user.email)?;
    assert!(user.is_some());
    
    let user = user.unwrap();
    assert_eq!(user.email, Some(test_user.email));
    assert_eq!(user.phone, Some(test_user.phone));
    assert_eq!(user.username, Some(test_user.username));
    assert_eq!(user.first_name, Some(test_user.first_name));
    assert_eq!(user.last_name, Some(test_user.last_name));
    assert_eq!(user.is_active, true);
    assert_eq!(user.is_verified, false);
    assert_eq!(user.failed_login_attempts, 0);
    assert!(user.created_at > 0);

    // Verify password was hashed
    assert_ne!(user.password_hash, test_user.password);
    assert!(user.password_hash.starts_with("$argon2"));

    Ok(())
}

/// Integration test for email already exists error
#[actix_web::test]
async fn test_email_already_exists() -> Result<(), Box<dyn std::error::Error>> {
    let setup = TestSetup::new()?;
    let _cleanup_guard = CleanupGuard(&setup);

    let test_user = TestUser::new();
    
    // Create first user
    let create_input = CreateUserInput {
        email: Some(test_user.email.clone()),
        phone: Some(test_user.phone.clone()),
        username: Some(test_user.username.clone()),
        password: test_user.password.clone(),
        first_name: Some(test_user.first_name.clone()),
        last_name: Some(test_user.last_name.clone()),
    };
    
    setup.user_store.create(create_input)?;

    // Try to register with same email
    let signup_request = crate::utils::types::SignUpRequest {
        email: Some(test_user.email.clone()),
        phone: Some("different_phone@example.com".to_string()),
        username: Some("different_username".to_string()),
        password: "DifferentPassword123!".to_string(),
        first_name: Some("Different".to_string()),
        last_name: Some("User".to_string()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(setup.app_state.clone()))
            .route("/signup", web::post().to(signup_user)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/signup")
        .set_json(&signup_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify conflict response
    assert_eq!(resp.status(), 409);
    let response: ApiResponse<SignUpResponseData> = test::read_body_json(resp).await;
    
    assert!(!response.success);
    assert!(response.message.contains("already registered"));

    Ok(())
}

/// Integration test for phone already exists error
#[actix_web::test]
async fn test_phone_already_exists() -> Result<(), Box<dyn std::error::Error>> {
    let setup = TestSetup::new()?;
    let _cleanup_guard = CleanupGuard(&setup);

    let test_user = TestUser::new();
    
    // Create first user
    let create_input = CreateUserInput {
        email: Some("different_email@example.com".to_string()),
        phone: Some(test_user.phone.clone()),
        username: Some(test_user.username.clone()),
        password: test_user.password.clone(),
        first_name: Some(test_user.first_name.clone()),
        last_name: Some(test_user.last_name.clone()),
    };
    
    setup.user_store.create(create_input)?;

    // Try to register with same phone
    let signup_request = crate::utils::types::SignUpRequest {
        email: Some("new_email@example.com".to_string()),
        phone: Some(test_user.phone.clone()),
        username: Some("different_username".to_string()),
        password: "DifferentPassword123!".to_string(),
        first_name: Some("Different".to_string()),
        last_name: Some("User".to_string()),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(setup.app_state.clone()))
            .route("/signup", web::post().to(signup_user)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/signup")
        .set_json(&signup_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify conflict response
    assert_eq!(resp.status(), 409);
    let response: ApiResponse<SignInResponse> = test::read_body_json(resp).await;
    
    assert!(!response.success);
    assert!(response.message.contains("already registered"));

    Ok(())
}

/// Integration test for minimal user registration (password only)
#[actix_web::test]
async fn test_minimal_user_registration() -> Result<(), Box<dyn std::error::Error>> {
    let setup = TestSetup::new()?;
    let _cleanup_guard = CleanupGuard(&setup);

    let signup_request = crate::utils::types::SignUpRequest {
        email: None,
        phone: None,
        username: None,
        password: "MinimalPassword123!".to_string(),
        first_name: None,
        last_name: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(setup.app_state.clone()))
            .route("/signup", web::post().to(signup_user)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/signup")
        .set_json(&signup_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify response
    assert_eq!(resp.status(), 200);
    let response: ApiResponse<SignUpResponseData> = test::read_body_json(resp).await;
    
    assert!(response.success);
    assert_eq!(response.message, "User registered successfully");
    assert!(response.data.is_some());
    
    let user_data = response.data.unwrap();
    assert!(!user_data.user_id.is_empty());
    assert_eq!(user_data.email, None);

    // Verify user was created in database
    let user = setup.user_store.find_by_id(&DbId::from_bytes(
        ObjectId::parse_str(&user_data.user_id)?.bytes()
    ))?;
    assert!(user.is_some());
    
    let user = user.unwrap();
    assert_eq!(user.email, None);
    assert_eq!(user.phone, None);
    assert_eq!(user.username, None);
    assert_eq!(user.first_name, None);
    assert_eq!(user.last_name, None);
    assert_eq!(user.is_active, true);
    assert_eq!(user.is_verified, false);

    // Verify password was hashed
    assert_ne!(user.password_hash, signup_request.password);
    assert!(user.password_hash.starts_with("$argon2"));

    Ok(())
}

/// Integration test for user registration with only email
#[actix_web::test]
async fn test_user_registration_with_email_only() -> Result<(), Box<dyn std::error::Error>> {
    let setup = TestSetup::new()?;
    let _cleanup_guard = CleanupGuard(&setup);

    let test_user = TestUser::new();
    let signup_request = crate::utils::types::SignUpRequest {
        email: Some(test_user.email.clone()),
        phone: None,
        username: None,
        password: test_user.password.clone(),
        first_name: None,
        last_name: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(setup.app_state.clone()))
            .route("/signup", web::post().to(signup_user)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/signup")
        .set_json(&signup_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify response
    assert_eq!(resp.status(), 200);
    let response: ApiResponse<SignUpResponseData> = test::read_body_json(resp).await;
    
    assert!(response.success);
    assert_eq!(response.message, "User registered successfully");
    assert!(response.data.is_some());
    
    let user_data = response.data.unwrap();
    assert!(!user_data.user_id.is_empty());
    assert_eq!(user_data.email, Some(test_user.email.clone()));

    // Verify user was created in database
    let user = setup.user_store.find_by_email(&test_user.email)?;
    assert!(user.is_some());
    
    let user = user.unwrap();
    assert_eq!(user.email, Some(test_user.email));
    assert_eq!(user.phone, None);
    assert_eq!(user.username, None);
    assert_eq!(user.first_name, None);
    assert_eq!(user.last_name, None);
    assert_eq!(user.is_active, true);
    assert_eq!(user.is_verified, false);

    Ok(())
}

/// Integration test for user registration with only phone
#[actix_web::test]
async fn test_user_registration_with_phone_only() -> Result<(), Box<dyn std::error::Error>> {
    let setup = TestSetup::new()?;
    let _cleanup_guard = CleanupGuard(&setup);

    let test_user = TestUser::new();
    let signup_request = crate::utils::types::SignUpRequest {
        email: None,
        phone: Some(test_user.phone.clone()),
        username: None,
        password: test_user.password.clone(),
        first_name: None,
        last_name: None,
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(setup.app_state.clone()))
            .route("/signup", web::post().to(signup_user)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/signup")
        .set_json(&signup_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify response
    assert_eq!(resp.status(), 200);
    let response: ApiResponse<SignUpResponseData> = test::read_body_json(resp).await;
    
    assert!(response.success);
    assert_eq!(response.message, "User registered successfully");
    assert!(response.data.is_some());
    
    let user_data = response.data.unwrap();
    assert!(!user_data.user_id.is_empty());
    assert_eq!(user_data.email, None);

    // Verify user was created in database
    let user = setup.user_store.find_by_phone(&test_user.phone)?;
    assert!(user.is_some());
    
    let user = user.unwrap();
    assert_eq!(user.email, None);
    assert_eq!(user.phone, Some(test_user.phone));
    assert_eq!(user.username, None);
    assert_eq!(user.first_name, None);
    assert_eq!(user.last_name, None);
    assert_eq!(user.is_active, true);
    assert_eq!(user.is_verified, false);

    Ok(())
}

/// Cleanup guard to ensure test data is cleaned up
struct CleanupGuard<'a>(&'a TestSetup);

impl<'a> Drop for CleanupGuard<'a> {
    fn drop(&mut self) {
        let _ = self.0.cleanup();
    }
}

/// Helper function to create a test user in the database
pub async fn create_test_user(
    setup: &TestSetup,
    test_user: &TestUser,
) -> Result<User, Box<dyn std::error::Error>> {
    let create_input = CreateUserInput {
        email: Some(test_user.email.clone()),
        phone: Some(test_user.phone.clone()),
        username: Some(test_user.username.clone()),
        password: test_user.password.clone(),
        first_name: Some(test_user.first_name.clone()),
        last_name: Some(test_user.last_name.clone()),
    };
    
    let user = setup.user_store.create(create_input)?;
    Ok(user)
}

/// Helper function to verify user exists in database
pub async fn verify_user_exists(
    setup: &TestSetup,
    email: &str,
) -> Result<User, Box<dyn std::error::Error>> {
    let user = setup.user_store.find_by_email(email)?;
    assert!(user.is_some(), "User should exist in database");
    Ok(user.unwrap())
}

/// Helper function to verify password is properly hashed
pub fn verify_password_hashed(user: &User, original_password: &str) {
    assert_ne!(user.password_hash, original_password, "Password should be hashed");
    assert!(user.password_hash.starts_with("$argon2"), "Password should use Argon2 hashing");
}

/// Helper function to verify user fields
pub fn verify_user_fields(
    user: &User,
    expected_email: Option<String>,
    expected_phone: Option<String>,
    expected_username: Option<String>,
    expected_first_name: Option<String>,
    expected_last_name: Option<String>,
) {
    assert_eq!(user.email, expected_email);
    assert_eq!(user.phone, expected_phone);
    assert_eq!(user.username, expected_username);
    assert_eq!(user.first_name, expected_first_name);
    assert_eq!(user.last_name, expected_last_name);
    assert_eq!(user.is_active, true);
    assert_eq!(user.is_verified, false);
    assert_eq!(user.failed_login_attempts, 0);
    assert!(user.created_at > 0);
}