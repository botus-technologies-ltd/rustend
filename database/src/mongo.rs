//! MongoDB database module

use mongodb::{Client, options::ClientOptions};

use crate::init::{Database, DatabaseConfig};
use crate::utils::{DbResult, DatabaseType};

/// MongoDB connection
pub struct MongoConnection {
    client: Client,
    db_name: String,
}

impl MongoConnection {
    pub async fn new(config: DatabaseConfig) -> DbResult<Box<dyn Database>> {
        let client_options = ClientOptions::parse(config.connection_string)
            .await
            .map_err(|e| crate::utils::DbError::connection_failed(&e.to_string()))?;
        
        let client = Client::with_options(client_options)
            .map_err(|e| crate::utils::DbError::connection_failed(&e.to_string()))?;
        
        Ok(Box::new(MongoConnection {
            client,
            db_name: config.db_name,
        }))
    }

    /// Get the MongoDB client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get database instance
    pub fn database(&self) -> mongodb::Database {
        self.client.database(&self.db_name)
    }
}

impl Database for MongoConnection {
    fn db_type(&self) -> DatabaseType {
        DatabaseType::MongoDB
    }

    fn db_name(&self) -> &str {
        &self.db_name
    }

    fn ping(&self) -> DbResult<()> {
        // TODO: Implement ping
        Ok(())
    }

    fn close(&self) -> DbResult<()> {
        // TODO: Implement close
        Ok(())
    }
}
