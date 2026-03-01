use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use std::sync::Arc;

use app::config::AppConfig;
use app::routes::init_routes;
use app::state::AppState;

// Utils - Logger
use middleware::logger::init_with_level;

// Utils - Email
use database::init::{DatabaseConfig, init_database};
use utils::email::{EmailService, SmtpConfig};
use utils::sms::{SmsService, TwilioConfig};
use utils::websocket::{WsServerConfig, WsService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger - logs to both file and terminal
    // In debug mode: shows terminal output for testing
    // In release mode: logs to file only (terminal disabled)
    let _guard = if cfg!(debug_assertions) {
        // Debug/development: log to both file and terminal
        init_with_level("app", "debug")
    } else {
        // Release/production: log to file only (reuse app logger)
        init_with_level("app", "info")
    };
    middleware::tracing::info!("Starting application...");

    // Load configs
    let config = AppConfig::from_env();
    middleware::tracing::info!(
        "Configuration loaded from {:?}",
        if cfg!(debug_assertions) {
            "app/.env.local"
        } else {
            "app/.env.prod"
        }
    );

    let db_config = DatabaseConfig::new(&config.db_uri, &config.db_name);
    let db = Arc::new(init_database(db_config).expect("Failed to initialize database"));
    middleware::tracing::info!("MongoDB connected successfully, database");

    // Initialize email service
    middleware::tracing::info!("Initializing email service: {}", config.email_provider);
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
        "twilio" => Arc::new(SmsService::twilio(TwilioConfig::new(
            &config.sms_account_sid,
            &config.sms_auth_token,
            &config.sms_from_number,
        ))),
        _ => Arc::new(SmsService::twilio(TwilioConfig::new(
            &config.sms_account_sid,
            &config.sms_auth_token,
            &config.sms_from_number,
        ))),
    };

    // Initialize WebSocket service
    let ws_config = WsServerConfig::new(
        &config.ws_url.replace("wss://", "").replace("wss://", ""),
        9944,
    );
    let ws = Arc::new(WsService::new(ws_config));

    // Get MongoDB database instance for stores
    let mongo_db = db.database();

    // Create app state (general services only)
    let state = AppState::new(
        email,
        config.email_from.clone(),
        config.app_name.clone(),
        sms,
        ws,
        db.clone(),
        config.jwt_secret.clone(),
        config.jwt_expiry_minutes,
        config.refresh_token_expiry_days,
        config.frontend_url.clone(),
    );

    // Run server
    HttpServer::new(move || {
        let state_for_routes = state.clone();
        let routes_mongo_db = mongo_db.clone();
        App::new()
            .wrap(Cors::default())
            .app_data(web::Data::new(state.clone()))
            .configure(move |cfg| init_routes(cfg, &state_for_routes, &routes_mongo_db))
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
