use dotenvy::from_filename;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub db_uri:   String,
    pub db_name:    String,
    pub server_ip: String,
    pub server_port: u16,
    pub jwt_secret:  String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        
        // Load env variables from backend/.env
        from_filename("backend/.env").ok();

        let jwt_secret  = env::var("JWT_SECRET").expect("JWT SECRET Must be set in .env");
        let db_uri      = env::var("DB_URI").expect("db_uri must be set in .env");
        let db_name     = env::var("DB_NAME").expect("db_name must be set in .env");
        let server_ip   = env::var("SERVER_IP").expect("server_ip must be set in .env");
        let server_port    = env::var("SERVER_PORT")
            .expect("SERVER_PORT must be set in .env")
            .parse()
            .expect("SERVER_PORT must be a valid number");

        Self {
            db_uri,
            db_name,
            server_ip,
            server_port,
            jwt_secret
        }
    }
}
