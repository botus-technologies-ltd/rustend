use actix_web::web;

use crate::state::AppState;
use auth::routes::configure as auth_configure;
use auth::routes::init as auth_init;

/// Initialize all routes
/// Accepts MongoDB database instance (from MongoConnection.database())
pub fn init_routes(
    cfg: &mut web::ServiceConfig,
    state: &AppState,
    mongo_db: &mongodb::sync::Database,
) {
    // Initialize auth module with database and config
    let auth_state = auth_init(
        mongo_db,
        state.jwt_secret.clone(),
        state.jwt_expiry_minutes,
        state.refresh_token_expiry_days,
        state.email.clone(),
        state.email_from.clone(),
        state.app_name.clone(),
        state.frontend_url.clone(),
    );

    // Auth routes
    cfg.app_data(web::Data::new(auth_state.clone()));
    auth_configure(cfg, auth_state.jwt_secret.clone());
}
