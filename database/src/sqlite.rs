//! SQLite database module

use crate::init::{Database, DatabaseConfig};
use crate::utils::{DbResult, DatabaseType};

/// SQLite connection
pub struct SqliteConnection {
    // conn: rusqlite::Connection,
    db_path: String,
}

impl SqliteConnection {
    pub async fn new(config: DatabaseConfig) -> DbResult<Box<dyn Database>> {
        // TODO: Implement SQLite connection
        Ok(Box::new(SqliteConnection {
            db_path: config.connection_string,
        }))
    }
}

impl Database for SqliteConnection {
    fn db_type(&self) -> DatabaseType {
        DatabaseType::SQLite
    }

    fn db_name(&self) -> &str {
        &self.db_path
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
