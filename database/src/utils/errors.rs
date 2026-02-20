//! Database errors module
//! 
//! Provides error types for database operations.

use serde::{Deserialize, Serialize};

/// Database error enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbError {
    pub code: DbErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl DbError {
    pub fn new(code: DbErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for DbError {}

/// Database error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbErrorCode {
    // Connection errors
    ConnectionFailed,
    ConnectionTimeout,
    PoolExhausted,
    
    // Query errors
    QueryFailed,
    NotFound,
    DuplicateKey,
    ConstraintViolation,
    InvalidQuery,
    
    // Transaction errors
    TransactionFailed,
    RollbackFailed,
    
    // Migration errors
    MigrationFailed,
    MigrationNotFound,
    
    // General errors
    InternalError,
    InvalidConfiguration,
    NotSupported,
}

impl std::fmt::Display for DbErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DbErrorCode::ConnectionFailed => "CONNECTION_FAILED",
            DbErrorCode::ConnectionTimeout => "CONNECTION_TIMEOUT",
            DbErrorCode::PoolExhausted => "POOL_EXHAUSTED",
            DbErrorCode::QueryFailed => "QUERY_FAILED",
            DbErrorCode::NotFound => "NOT_FOUND",
            DbErrorCode::DuplicateKey => "DUPLICATE_KEY",
            DbErrorCode::ConstraintViolation => "CONSTRAINT_VIOLATION",
            DbErrorCode::InvalidQuery => "INVALID_QUERY",
            DbErrorCode::TransactionFailed => "TRANSACTION_FAILED",
            DbErrorCode::RollbackFailed => "ROLLBACK_FAILED",
            DbErrorCode::MigrationFailed => "MIGRATION_FAILED",
            DbErrorCode::MigrationNotFound => "MIGRATION_NOT_FOUND",
            DbErrorCode::InternalError => "INTERNAL_ERROR",
            DbErrorCode::InvalidConfiguration => "INVALID_CONFIGURATION",
            DbErrorCode::NotSupported => "NOT_SUPPORTED",
        };
        write!(f, "{}", s)
    }
}

/// Helper functions for common errors
impl DbError {
    pub fn connection_failed(msg: &str) -> Self {
        Self::new(DbErrorCode::ConnectionFailed, msg)
    }

    pub fn connection_timeout(msg: &str) -> Self {
        Self::new(DbErrorCode::ConnectionTimeout, msg)
    }

    pub fn not_found(msg: &str) -> Self {
        Self::new(DbErrorCode::NotFound, msg)
    }

    pub fn duplicate_key(msg: &str) -> Self {
        Self::new(DbErrorCode::DuplicateKey, msg)
    }

    pub fn query_failed(msg: &str) -> Self {
        Self::new(DbErrorCode::QueryFailed, msg)
    }

    pub fn internal_error(msg: &str) -> Self {
        Self::new(DbErrorCode::InternalError, msg)
    }

    pub fn invalid_config(msg: &str) -> Self {
        Self::new(DbErrorCode::InvalidConfiguration, msg)
    }

    pub fn not_supported(msg: &str) -> Self {
        Self::new(DbErrorCode::NotSupported, msg)
    }
}

/// Result type for database operations
pub type DbResult<T> = Result<T, DbError>;
