//! MySQL database module

use crate::init::{Database, DatabaseConfig};
use crate::utils::{DbResult, DatabaseType};

/// MySQL connection
pub struct MysqlConnection {
    // pool: mysql::Pool,
    db_name: String,
}

impl MysqlConnection {
    pub async fn new(config: DatabaseConfig) -> DbResult<Box<dyn Database>> {
        // TODO: Implement MySQL connection
        Ok(Box::new(MysqlConnection {
            db_name: config.db_name,
        }))
    }
}

impl Database for MysqlConnection {
    fn db_type(&self) -> DatabaseType {
        DatabaseType::MySQL
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
