use std::sync::Arc;

use database::init::Database;
use utils::email::EmailService;
use utils::sms::SmsService;
use utils::websocket::WsService;

/// Application state - general services for the entire application
#[derive(Clone)]
pub struct AppState {
    pub email: Arc<EmailService>,
    pub email_from: String,
    pub app_name: String,
    pub sms: Arc<SmsService>,
    pub ws: Arc<WsService>,
    pub db: Arc<dyn Database>,
    pub jwt_secret: String,
    pub jwt_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,

    // Frontend URL
    pub frontend_url: String,
}

impl AppState {
    pub fn new(
        email: Arc<EmailService>,
        email_from: String,
        app_name: String,
        sms: Arc<SmsService>,
        ws: Arc<WsService>,
        db: Arc<dyn Database>,
        jwt_secret: String,
        jwt_expiry_minutes: i64,
        refresh_token_expiry_days: i64,
        frontend_url: String,
    ) -> Self {
        Self {
            email,
            email_from,
            app_name,
            sms,
            ws,
            db,
            jwt_secret,
            jwt_expiry_minutes,
            refresh_token_expiry_days,
            frontend_url,
        }
    }
}
