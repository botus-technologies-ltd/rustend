//! PostgreSQL database module

use crate::init::{Database, DatabaseConfig};
use crate::utils::{DbResult, DatabaseType};

/// PostgreSQL connection
pub struct PostgresConnection {
    // client: tokio_postgres::Client,
    db_name: String,
}

impl PostgresConnection {
    pub async fn new(config: DatabaseConfig) -> DbResult<Box<dyn Database>> {
        // TODO: Implement PostgreSQL connection
        Ok(Box::new(PostgresConnection {
            db_name: config.db_name,
        }))
    }
}

impl Database for PostgresConnection {
    fn db_type(&self) -> DatabaseType {
        DatabaseType::PostgreSQL
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
