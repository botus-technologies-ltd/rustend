//! Authentication routes

use crate::store::oauth_account_store::OAuthAccountStore;
use crate::store::password_reset_store::PasswordResetStore;
use crate::store::session_store::SessionStore;
use crate::store::user_store::UserStore;
use crate::store::verification_store::VerificationStore;
use actix_web::web;

use middleware::jwt::JwtMiddleware;
use std::sync::Arc;
use utils::email::EmailService;

// Internal imports for store initialization
use crate::store::database::MongoPasswordResetStore;
use crate::store::database::MongoSessionStore;
use crate::store::database::MongoUserStore;
use crate::store::database::MongoVerificationStore;
use crate::store::database::MongoOAuthAccountStore;

/// Application state for authentication handlers
pub struct AppState {
    pub users:           Arc<dyn UserStore>,
    pub sessions:        Arc<dyn SessionStore>,
    pub password_resets: Option<Arc<dyn PasswordResetStore>>,
    pub verifications:   Option<Arc<dyn VerificationStore>>,
    pub oauth_accounts:  Option<Arc<dyn OAuthAccountStore>>,
    pub jwt_secret:      String,
    pub jwt_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
    pub email:           Arc<EmailService>,
    pub email_from:      String,
    pub app_name:        String,
    pub frontend_url:    String,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            users: self.users.clone(),
            sessions: self.sessions.clone(),
            password_resets: self.password_resets.clone(),
            verifications: self.verifications.clone(),
            oauth_accounts: self.oauth_accounts.clone(),
            jwt_secret: self.jwt_secret.clone(),
            jwt_expiry_minutes: self.jwt_expiry_minutes,
            refresh_token_expiry_days: self.refresh_token_expiry_days,
            email: self.email.clone(),
            email_from: self.email_from.clone(),
            app_name: self.app_name.clone(),
            frontend_url: self.frontend_url.clone(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        panic!("AppState must be initialized with store implementations")
    }

    /// Create AppState with all required fields
    pub fn with_jwt(
        users:           Arc<dyn UserStore>,
        sessions:        Arc<dyn SessionStore>,
        password_resets: Option<Arc<dyn PasswordResetStore>>,
        verifications:   Option<Arc<dyn VerificationStore>>,
        oauth_accounts:  Option<Arc<dyn OAuthAccountStore>>,
        jwt_secret:      String,
        jwt_expiry_minutes: i64,
        refresh_token_expiry_days: i64,
        email:           Arc<EmailService>,
        email_from:      String,
        app_name:        String,
        frontend_url:    String,
    ) -> Self {
        Self {
            users,
            sessions,
            password_resets,
            verifications,
            oauth_accounts,
            jwt_secret,
            jwt_expiry_minutes,
            refresh_token_expiry_days,
            email,
            email_from,
            app_name,
            frontend_url,
        }
    }
}

/// Initialize auth module with database and configuration
/// 
/// This function creates all auth-specific stores internally and returns
/// the auth AppState, encapsulating all auth dependencies.
pub fn init(
    mongo_db: &mongodb::sync::Database,
    jwt_secret: String,
    jwt_expiry_minutes: i64,
    refresh_token_expiry_days: i64,
    email: Arc<EmailService>,
    email_from: String,
    app_name: String,
    frontend_url: String,
) -> AppState {
    // Create auth store instances
    let users = Arc::new(MongoUserStore::new(mongo_db.collection("users"))) as Arc<dyn UserStore>;

    let sessions = Arc::new(MongoSessionStore::new(
        mongo_db.collection("sessions"),
        mongo_db.collection("refresh_tokens"),
    )) as Arc<dyn SessionStore>;

    let verifications = Arc::new(MongoVerificationStore::new(
        mongo_db.collection("verification_codes"),
    )) as Arc<dyn VerificationStore>;

    let password_resets = Arc::new(MongoPasswordResetStore::new(
        mongo_db.collection("password_reset_tokens"),
    )) as Arc<dyn PasswordResetStore>;

    let oauth_accounts = Arc::new(MongoOAuthAccountStore::new(
        mongo_db.collection("oauth_accounts"),
    )) as Arc<dyn OAuthAccountStore>;

    // Create auth AppState with provided dependencies
    AppState::with_jwt(
        users,
        sessions,
        Some(password_resets),
        Some(verifications),
        Some(oauth_accounts),
        jwt_secret,
        jwt_expiry_minutes,
        refresh_token_expiry_days,
        email,
        email_from,
        app_name,
        frontend_url,
    )
}

/// Configure all authentication routes
pub fn configure(cfg: &mut web::ServiceConfig, jwt_secret: String) {
    let secret = jwt_secret;

    cfg.service(
        web::scope("/auth")
            // Public routes (no auth required)
            .route("/login", web::post().to(crate::handler::login_user))
            .route("/register", web::post().to(crate::handler::signup_user))
            .route(
                "/verify/{user_id}",
                web::post().to(crate::handler::verify_email),
            )
            .route(
                "/verification/send",
                web::post().to(crate::handler::send_verification_code),
            )
            .route(
                "/forgot-password",
                web::post().to(crate::handler::forgot_password),
            )
            .route(
                "/reset-password",
                web::post().to(crate::handler::reset_password),
            )
            .route(
                "/oauth/{provider}",
                web::get().to(crate::handler::oauth_redirect),
            )
            .route(
                "/oauth/{provider}/callback",
                web::get().to(crate::handler::oauth_callback),
            )
            
            // Protected routes (auth required) - different path prefix
            .service(
                web::scope("/pt")
                    .wrap(JwtMiddleware::from_secret(secret))
                    .route(
                        "/refresh-token",
                        web::post().to(crate::handler::refresh_token),
                    )
                    .route("/logout", web::post().to(crate::handler::logout_user))
                    .route(
                        "/change-password",
                        web::post().to(crate::handler::change_password),
                    )
                    .route(
                        "/oauth/link/{user_id}",
                        web::post().to(crate::handler::link_oauth),
                    )
                    .route(
                        "/oauth/unlink/{user_id}",
                        web::post().to(crate::handler::unlink_oauth),
                    )
                    .route(
                        "/oauth/connections/{user_id}",
                        web::get().to(crate::handler::list_oauth_connections),
                    )
                    .route(
                        "/2fa/status/{user_id}",
                        web::get().to(crate::handler::get_2fa_status),
                    )
                    
                    // Unimplimented Routes - these are for future features and may not be fully implemented yet
                    .route(
                        "/magic/link",
                        web::post().to(crate::handler::request_magic_link),
                    )
                    .route(
                        "/magic/verify",
                        web::post().to(crate::handler::verify_magic_link),
                    )
                    .route(
                        "/2fa/enable/{user_id}",
                        web::post().to(crate::handler::enable_2fa),
                    )
                    .route(
                        "/2fa/disable/{user_id}",
                        web::post().to(crate::handler::disable_2fa),
                    )
                    .route(
                        "/2fa/verify/{user_id}",
                        web::post().to(crate::handler::verify_2fa),
                    )
                    .route(
                        "/2fa/setup/{user_id}",
                        web::get().to(crate::handler::generate_2fa_setup),
                    )
                    .route(
                        "/2fa/backup/{user_id}",
                        web::get().to(crate::handler::get_backup_codes),
                    )
                    .route(
                        "/2fa/backup/{user_id}/regenerate",
                        web::post().to(crate::handler::regenerate_backup_codes),
                    )
                    .route(
                        "/sessions/{user_id}",
                        web::get().to(crate::handler::list_sessions),
                    )
                    .route(
                        "/sessions/{session_id}",
                        web::get().to(crate::handler::get_session),
                    )
                    .route(
                        "/sessions/{session_id}/revoke",
                        web::post().to(crate::handler::revoke_session),
                    )
                    .route(
                        "/sessions/{user_id}/revoke-all",
                        web::post().to(crate::handler::revoke_all_sessions),
                    )
                    .route(
                        "/sessions/{user_id}/revoke-others",
                        web::post().to(crate::handler::revoke_other_sessions),
                    )
                    .route(
                        "/sessions/cleanup",
                        web::post().to(crate::handler::cleanup_expired_sessions),
                    )
                    .route(
                        "/devices/{user_id}",
                        web::get().to(crate::handler::list_devices),
                    )
                    .route(
                        "/devices/{session_id}",
                        web::get().to(crate::handler::get_device),
                    )
                    .route(
                        "/devices/{session_id}/trust",
                        web::post().to(crate::handler::trust_device),
                    )
                    .route(
                        "/devices/{session_id}/untrust",
                        web::post().to(crate::handler::untrust_device),
                    )
                    .route(
                        "/devices/{session_id}/revoke",
                        web::post().to(crate::handler::revoke_device),
                    )
                    .route(
                        "/devices/{user_id}/revoke-all",
                        web::post().to(crate::handler::revoke_all_devices),
                    )
                    .route("/users/{user_id}", web::get().to(crate::handler::get_user))
                    .route(
                        "/users/{user_id}",
                        web::put().to(crate::handler::update_user),
                    )
                    .route(
                        "/users/{user_id}",
                        web::delete().to(crate::handler::delete_user),
                    )
                    .route("/users", web::get().to(crate::handler::list_users))
                    .route(
                        "/users/{user_id}/activate",
                        web::post().to(crate::handler::activate_user),
                    )
                    .route(
                        "/users/{user_id}/deactivate",
                        web::post().to(crate::handler::deactivate_user),
                    ),
            ),
    );
}
