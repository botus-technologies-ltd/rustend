use actix_web::web;

use auth::routes::configure as auth_configure;

/// Initialize all routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    // Auth routes
    auth_configure(cfg);
}
