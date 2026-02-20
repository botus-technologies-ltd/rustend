use dotenvy::from_filename;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    // Database
    pub db_uri: String,
    pub db_name: String,
    
    // Server
    pub server_ip: String,
    pub server_port: u16,
    
    // JWT
    pub jwt_secret: String,
    
    // WebSocket
    pub ws_url: String,
    
    // Email (SendGrid)
    pub email_provider: String,
    pub email_api_key: String,
    pub email_from: String,
    pub email_from_name: String,
    
    // SMS (Twilio)
    pub sms_provider: String,
    pub sms_account_sid: String,
    pub sms_auth_token: String,
    pub sms_from_number: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        // Load env variables from app/.env.local
        // In production, load from .env.prod
        let env_file = if cfg!(debug_assertions) {
            "app/.env.local"
        } else {
            "app/.env.prod"
        };
        from_filename(env_file).ok();

        Self {
            // Database
            db_uri: env::var("DB_URI").expect("DB_URI must be set in .env"),
            db_name: env::var("DB_NAME").expect("DB_NAME must be set in .env"),
            
            // Server
            server_ip: env::var("SERVER_IP").expect("SERVER_IP must be set in .env"),
            server_port: env::var("SERVER_PORT")
                .expect("SERVER_PORT must be set in .env")
                .parse()
                .expect("SERVER_PORT must be a valid number"),
            
            // JWT
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env"),
            
            // WebSocket
            ws_url: env::var("WS_URL").expect("WS_URL must be set in .env"),
            
            // Email
            email_provider: env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "sendgrid".to_string()),
            email_api_key: env::var("EMAIL_API_KEY").expect("EMAIL_API_KEY must be set in .env"),
            email_from: env::var("EMAIL_FROM").expect("EMAIL_FROM must be set in .env"),
            email_from_name: env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "App".to_string()),
            
            // SMS
            sms_provider: env::var("SMS_PROVIDER").unwrap_or_else(|_| "twilio".to_string()),
            sms_account_sid: env::var("SMS_ACCOUNT_SID").expect("SMS_ACCOUNT_SID must be set in .env"),
            sms_auth_token: env::var("SMS_AUTH_TOKEN").expect("SMS_AUTH_TOKEN must be set in .env"),
            sms_from_number: env::var("SMS_FROM_NUMBER").expect("SMS_FROM_NUMBER must be set in .env"),
        }
    }
}
