//! Database ID types module
//! 
//! Provides flexible ID types that work across different databases:
//! - MongoDB: Uses ObjectId (12-byte binary)
//! - PostgreSQL/MySQL: Uses UUID
//! - SQLite: Uses i64
//!
//! Users can choose their preferred database and use the appropriate ID type.

use serde::{Deserialize, Serialize};

/// Enum representing different database ID types
/// Use this when you want your models to work with any database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DatabaseId {
    /// MongoDB ObjectId (12 bytes / 24 hex chars)
    ObjectId(ObjectId),
    /// PostgreSQL UUID / MySQL UUID
    Uuid(uuid::Uuid),
    /// SQLite auto-increment ID
    SqlId(i64),
    /// Custom string ID (for databases using string keys)
    String(String),
}

impl DatabaseId {
    /// Create from MongoDB ObjectId hex string
    pub fn from_object_id(hex: &str) -> Result<Self, IdError> {
        ObjectId::from_str(hex).map(DatabaseId::ObjectId)
    }

    /// Create from UUID string
    pub fn from_uuid(uuid: &str) -> Result<Self, IdError> {
        uuid::Uuid::parse_str(uuid)
            .map(DatabaseId::Uuid)
            .map_err(|_| IdError::InvalidFormat)
    }

    /// Create from i64 (SQL)
    pub fn from_sql_id(id: i64) -> Self {
        DatabaseId::SqlId(id)
    }

    /// Create from string
    pub fn from_string(id: &str) -> Self {
        DatabaseId::String(id.to_string())
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            DatabaseId::ObjectId(id) => id.to_string(),
            DatabaseId::Uuid(id) => id.to_string(),
            DatabaseId::SqlId(id) => id.to_string(),
            DatabaseId::String(id) => id.clone(),
        }
    }

    /// Get as i64 if SQL ID
    pub fn as_sql_id(&self) -> Option<i64> {
        match self {
            DatabaseId::SqlId(id) => Some(*id),
            _ => None,
        }
    }

    /// Get as UUID if UUID
    pub fn as_uuid(&self) -> Option<uuid::Uuid> {
        match self {
            DatabaseId::Uuid(id) => Some(*id),
            _ => None,
        }
    }

    /// Get as ObjectId if MongoDB
    pub fn as_object_id(&self) -> Option<&ObjectId> {
        match self {
            DatabaseId::ObjectId(id) => Some(id),
            _ => None,
        }
    }
}

impl std::fmt::Display for DatabaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// MongoDB ObjectId wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId {
    bytes: [u8; 12],
}

impl ObjectId {
    /// Create new ObjectId from 12 bytes
    pub fn from_bytes(bytes: [u8; 12]) -> Self {
        Self { bytes }
    }

    /// Parse from 24-character hex string
    pub fn from_str(hex: &str) -> Result<Self, IdError> {
        if hex.len() != 24 {
            return Err(IdError::InvalidFormat);
        }
        
        let mut bytes = [0u8; 12];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let s = std::str::from_utf8(chunk).map_err(|_| IdError::InvalidFormat)?;
            bytes[i] = u8::from_str_radix(s, 16).map_err(|_| IdError::InvalidFormat)?;
        }
        
        Ok(Self { bytes })
    }

    /// Convert to hex string
    pub fn to_string(&self) -> String {
        self.bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 12] {
        &self.bytes
    }

    /// Generate new ObjectId (for insert operations)
    pub fn new() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut bytes);
        // Set timestamp bytes (first 4 bytes)
        let timestamp = chrono::Utc::now().timestamp() as u32;
        bytes[0] = (timestamp >> 24) as u8;
        bytes[1] = (timestamp >> 16) as u8;
        bytes[2] = (timestamp >> 8) as u8;
        bytes[3] = timestamp as u8;
        Self { bytes }
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// ID generation errors
#[derive(Debug, Clone)]
pub enum IdError {
    InvalidFormat,
    ConversionFailed,
}

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdError::InvalidFormat => write!(f, "Invalid ID format"),
            IdError::ConversionFailed => write!(f, "ID conversion failed"),
        }
    }
}

impl std::error::Error for IdError {}

/// Type alias for common database ID (flexible)
/// Change this based on your database choice
pub type DbId = DatabaseId;

/// Result type for ID operations
pub type IdResult<T> = Result<T, IdError>;

// ============================================
// Convenience functions
// ============================================

/// Generate new ID based on database type
pub fn generate_id(db_type: DatabaseType) -> DbId {
    match db_type {
        DatabaseType::MongoDB => DatabaseId::ObjectId(ObjectId::new()),
        DatabaseType::PostgreSQL | DatabaseType::MySQL => {
            DatabaseId::Uuid(uuid::Uuid::new_v4())
        }
        DatabaseType::SQLite => DatabaseId::String(uuid::Uuid::new_v4().to_string()),
    }
}

/// Parse ID from string based on database type
pub fn parse_id(db_type: DatabaseType, value: &str) -> Result<DbId, IdError> {
    match db_type {
        DatabaseType::MongoDB => DatabaseId::from_object_id(value),
        DatabaseType::PostgreSQL | DatabaseType::MySQL => DatabaseId::from_uuid(value),
        DatabaseType::SQLite => {
            // Try as number first
            if let Ok(id) = value.parse::<i64>() {
                Ok(DatabaseId::SqlId(id))
            } else {
                Ok(DatabaseId::String(value.to_string()))
            }
        }
    }
}

/// Database type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    MongoDB,
    PostgreSQL,
    MySQL,
    SQLite,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::MongoDB => write!(f, "MongoDB"),
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::SQLite => write!(f, "SQLite"),
        }
    }
}
