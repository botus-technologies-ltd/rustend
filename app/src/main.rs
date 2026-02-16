use actix_web::{get, App, web, HttpServer, Responder};
use std::rc::Rc;

use crate::config::AppConfig;

#[get("/")]
async fn hello() -> impl Responder {
    "Hello from rust end"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configs
    let config = AppConfig::from_env();

    // Logging
    init_tracing();

    // Init DB
    let client = init_db(&config.mongo_uri, &config.mongo_db).await;
    let db = client.database(&config.mongo_db);
    let keys = &config.jwt_secret;
    let jwt_keys = JwtKeys::new(&keys);
    
    let state = AppState {};

    // Run server
    HttpServer::new(move || {
        App::new()
            .wrap(RequestLogger::new())
            .wrap(RateLimiter::new())
            .app_data(web::Data::new(state.clone()))
            .configure(|cfg| init_routes(cfg, &config.jwt_secret, Rc::new(state.sessions.clone())))
            .service(hello)
    })
    .bind((config.server_host, config.server_port))?
    .run()
    .await
}
