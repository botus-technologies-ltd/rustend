//! Database initialization module
//!
//! Provides initialization functions for MongoDB database.

use crate::mongo;
use crate::utils::DbResult;

/// Database configuration for MongoDB
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub db_name: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_secs: u64,
}

impl DatabaseConfig {
    pub fn new(connection_string: impl Into<String>, db_name: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            db_name: db_name.into(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout_secs: 30,
        }
    }

    pub fn with_pool(mut self, max: u32, min: u32) -> Self {
        self.max_connections = max;
        self.min_connections = min;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.connection_timeout_secs = secs;
        self
    }
}

/// Database connection trait
pub trait Database: Send + Sync {
    /// Get database name
    fn db_name(&self) -> &str;

    /// Test the connection
    fn ping(&self) -> DbResult<()>;

    /// Close the connection
    fn close(&self) -> DbResult<()>;
}

/// Initialize MongoDB database
pub fn init_database(config: DatabaseConfig) -> DbResult<mongo::MongoConnection> {
    mongo::MongoConnection::new(config)
}

/// Get default MongoDB connection string
pub fn default_connection_string() -> String {
    "mongodb://localhost:27017".to_string()
}
