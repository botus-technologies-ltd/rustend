//! MongoDB database module

use mongodb::sync::{Client, Database};

use crate::init::{Database as DbTrait, DatabaseConfig};
use crate::utils::DbResult;

/// MongoDB connection
pub struct MongoConnection {
    client: Client,
    db_name: String,
}

impl MongoConnection {
    pub fn new(config: DatabaseConfig) -> DbResult<MongoConnection> {
        let client = Client::with_uri_str(config.connection_string)
            .map_err(|e| crate::utils::DbError::connection_failed(&e.to_string()))?;

        Ok(MongoConnection {
            client,
            db_name: config.db_name,
        })
    }

    /// Get the MongoDB client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get database instance
    pub fn database(&self) -> Database {
        self.client.database(&self.db_name)
    }
}

impl DbTrait for MongoConnection {
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
