//! Database initialization module
//! 
//! Provides initialization functions for different databases.

use crate::utils::{DbError, DbResult, DatabaseType};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub connection_string: String,
    pub db_name: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_secs: u64,
}

impl DatabaseConfig {
    pub fn new(db_type: DatabaseType, connection_string: impl Into<String>, db_name: impl Into<String>) -> Self {
        Self {
            db_type,
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
    /// Get the database type
    fn db_type(&self) -> DatabaseType;
    
    /// Get database name
    fn db_name(&self) -> &str;
    
    /// Test the connection
    fn ping(&self) -> DbResult<()>;
    
    /// Close the connection
    fn close(&self) -> DbResult<()>;
}

/// Initialize database asynchronously
pub async fn init_database(config: DatabaseConfig) -> DbResult<Box<dyn Database>> {
    match config.db_type {
        DatabaseType::MongoDB => {
            #[cfg(feature = "mongodb")]
            {
                crate::mongo::MongoConnection::new(config).await
            }
            #[cfg(not(feature = "mongodb"))]
            {
                Err(DbError::not_supported("MongoDB not enabled. Add 'mongodb' feature to Cargo.toml"))
            }
        }
        DatabaseType::PostgreSQL => {
            #[cfg(feature = "postgres")]
            {
                crate::postgres::PostgresConnection::new(config).await
            }
            #[cfg(not(feature = "postgres"))]
            {
                Err(DbError::not_supported("PostgreSQL not enabled. Add 'postgres' feature to Cargo.toml"))
            }
        }
        DatabaseType::MySQL => {
            #[cfg(feature = "mysql")]
            {
                crate::mysql::MysqlConnection::new(config).await
            }
            #[cfg(not(feature = "mysql"))]
            {
                Err(DbError::not_supported("MySQL not enabled. Add 'mysql' feature to Cargo.toml"))
            }
        }
        DatabaseType::SQLite => {
            #[cfg(feature = "sqlite")]
            {
                crate::sqlite::SqliteConnection::new(config).await
            }
            #[cfg(not(feature = "sqlite"))]
            {
                Err(DbError::not_supported("SQLite not enabled. Add 'sqlite' feature to Cargo.toml"))
            }
        }
    }
}

/// Get default connection string for a database type
pub fn default_connection_string(db_type: DatabaseType) -> String {
    match db_type {
        DatabaseType::MongoDB => "mongodb://localhost:27017".to_string(),
        DatabaseType::PostgreSQL => "postgres://postgres:password@localhost:5432/dbname".to_string(),
        DatabaseType::MySQL => "mysql://root:password@localhost:3306/dbname".to_string(),
        DatabaseType::SQLite => "database.db".to_string(),
    }
}
