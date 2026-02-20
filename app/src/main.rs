use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use actix_cors::Cors;

use app::config::AppConfig;
use app::state::AppState;
use app::routes::init_routes;

// Utils - Email
use utils::email::{EmailService, SmtpConfig};

// Utils - SMS
use utils::sms::{SmsService, TwilioConfig};

// Utils - WebSocket
use utils::websocket::{WsService, WsServerConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configs
    let config = AppConfig::from_env();

    // Initialize email service
    let email = match config.email_provider.as_str() {
        "sendgrid" => Arc::new(EmailService::sendgrid(
            &config.email_api_key,
            &config.email_from,
        )),
        "smtp" => {
            // Parse SMTP settings from environment if needed
            let smtp_config = SmtpConfig::new("smtp.example.com", 587, "user", "pass");
            Arc::new(EmailService::smtp(smtp_config))
        }
        _ => Arc::new(EmailService::sendgrid(
            &config.email_api_key,
            &config.email_from,
        )),
    };

    // Initialize SMS service
    let sms = match config.sms_provider.as_str() {
        "twilio" => Arc::new(SmsService::twilio(
            TwilioConfig::new(
                &config.sms_account_sid,
                &config.sms_auth_token,
                &config.sms_from_number,
            )
        )),
        _ => Arc::new(SmsService::twilio(
            TwilioConfig::new(
                &config.sms_account_sid,
                &config.sms_auth_token,
                &config.sms_from_number,
            )
        )),
    };

    // Initialize WebSocket service
    let ws_config = WsServerConfig::new(&config.ws_url.replace("wss://", "").replace("wss://", ""), 9944);
    let ws = Arc::new(WsService::new(ws_config));

    // Create app state
    let state = AppState::new(
        email,
        sms,
        ws,
        config.jwt_secret.clone(),
    );

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
