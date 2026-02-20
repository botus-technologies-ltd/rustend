use std::sync::Arc;

// Utils - Email
use utils::email::EmailService;

// Utils - SMS
use utils::sms::SmsService;

// Utils - WebSocket
use utils::websocket::WsService;

/// Application state
#[derive(Clone)]
pub struct AppState {
    // Email service
    pub email: Arc<EmailService>,
    
    // SMS service
    pub sms: Arc<SmsService>,
    
    // WebSocket service
    pub ws: Arc<WsService>,
    
    // JWT configuration
    pub jwt_secret: String,
    pub jwt_expiry_minutes: i64,
}

impl AppState {
    pub fn new(
        email: Arc<EmailService>,
        sms: Arc<SmsService>,
        ws: Arc<WsService>,
        jwt_secret: String,
    ) -> Self {
        Self {
            email,
            sms,
            ws,
            jwt_secret,
            jwt_expiry_minutes: 60, // default 1 hour
        }
    }
}
