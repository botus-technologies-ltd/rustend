//! Authentication routes

use actix_web::web;
use crate::store::user_store::UserStore;
use crate::store::session_store::SessionStore;
use crate::store::password_reset_store::PasswordResetStore;
use crate::store::verification_store::VerificationStore;
use std::sync::Arc;

/// Application state for authentication handlers
pub struct AppState {
    pub users: Arc<dyn UserStore>,
    pub sessions: Arc<dyn SessionStore>,
    pub password_resets: Option<Arc<dyn PasswordResetStore>>,
    pub verifications: Option<Arc<dyn VerificationStore>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            users: self.users.clone(),
            sessions: self.sessions.clone(),
            password_resets: self.password_resets.clone(),
            verifications: self.verifications.clone(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        panic!("AppState must be initialized with store implementations")
    }
}

/// Configure all authentication routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            // Authentication
            .route("/login", web::post().to(crate::handler::login_user))
            .route("/logout", web::post().to(crate::handler::logout_user))
            .route("/refresh", web::post().to(crate::handler::refresh_token))
            
            // Registration
            .route("/signup", web::post().to(crate::handler::signup_user))
            .route("/verify/{user_id}", web::post().to(crate::handler::verify_email))
            .route("/verification/send", web::post().to(crate::handler::send_verification_code))
            
            // Password reset
            .route("/password/forgot", web::post().to(crate::handler::forgot_password))
            .route("/password/reset", web::post().to(crate::handler::reset_password))
            .route("/password/change/{user_id}", web::post().to(crate::handler::change_password))
            
            // Magic link
            .route("/magic/link", web::post().to(crate::handler::request_magic_link))
            .route("/magic/verify", web::post().to(crate::handler::verify_magic_link))
            
            // OAuth
            .route("/oauth/{provider}", web::get().to(crate::handler::oauth_redirect))
            .route("/oauth/{provider}/callback", web::get().to(crate::handler::oauth_callback))
            .route("/oauth/link/{user_id}", web::post().to(crate::handler::link_oauth))
            .route("/oauth/unlink/{user_id}", web::post().to(crate::handler::unlink_oauth))
            .route("/oauth/connections/{user_id}", web::get().to(crate::handler::list_oauth_connections))
            
            // 2FA
            .route("/2fa/status/{user_id}", web::get().to(crate::handler::get_2fa_status))
            .route("/2fa/enable/{user_id}", web::post().to(crate::handler::enable_2fa))
            .route("/2fa/disable/{user_id}", web::post().to(crate::handler::disable_2fa))
            .route("/2fa/verify/{user_id}", web::post().to(crate::handler::verify_2fa))
            .route("/2fa/setup/{user_id}", web::get().to(crate::handler::generate_2fa_setup))
            .route("/2fa/backup/{user_id}", web::get().to(crate::handler::get_backup_codes))
            .route("/2fa/backup/{user_id}/regenerate", web::post().to(crate::handler::regenerate_backup_codes))
            
            // Sessions
            .route("/sessions/{user_id}", web::get().to(crate::handler::list_sessions))
            .route("/sessions/{session_id}", web::get().to(crate::handler::get_session))
            .route("/sessions/{session_id}/revoke", web::post().to(crate::handler::revoke_session))
            .route("/sessions/{user_id}/revoke-all", web::post().to(crate::handler::revoke_all_sessions))
            .route("/sessions/{user_id}/revoke-others", web::post().to(crate::handler::revoke_other_sessions))
            .route("/sessions/cleanup", web::post().to(crate::handler::cleanup_expired_sessions))
            
            // Devices
            .route("/devices/{user_id}", web::get().to(crate::handler::list_devices))
            .route("/devices/{session_id}", web::get().to(crate::handler::get_device))
            .route("/devices/{session_id}/trust", web::post().to(crate::handler::trust_device))
            .route("/devices/{session_id}/untrust", web::post().to(crate::handler::untrust_device))
            .route("/devices/{session_id}/revoke", web::post().to(crate::handler::revoke_device))
            .route("/devices/{user_id}/revoke-all", web::post().to(crate::handler::revoke_all_devices))
            
            // Users
            .route("/users/{user_id}", web::get().to(crate::handler::get_user))
            .route("/users/{user_id}", web::put().to(crate::handler::update_user))
            .route("/users/{user_id}", web::delete().to(crate::handler::delete_user))
            .route("/users", web::get().to(crate::handler::list_users))
            .route("/users/{user_id}/activate", web::post().to(crate::handler::activate_user))
            .route("/users/{user_id}/deactivate", web::post().to(crate::handler::deactivate_user))
    );
}
