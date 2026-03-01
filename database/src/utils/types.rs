//! Database ID types module
//!
//! Provides MongoDB ObjectId types for the database.

use mongodb::bson;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// MongoDB ObjectId wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectId {
    bytes: [u8; 12],
}

impl Serialize for ObjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as BSON ObjectId for MongoDB compatibility
        bson::oid::ObjectId::from_bytes(self.bytes).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ObjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize from BSON ObjectId
        let oid = bson::oid::ObjectId::deserialize(deserializer)?;
        Ok(ObjectId::from_bytes(oid.bytes()))
    }
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

/// Type alias for database ID (MongoDB ObjectId)
pub type DbId = ObjectId;

/// Result type for ID operations
pub type IdResult<T> = Result<T, IdError>;

/// Generate new MongoDB ObjectId
pub fn generate_id() -> DbId {
    ObjectId::new()
}

/// Parse ID from string
pub fn parse_id(value: &str) -> IdResult<DbId> {
    ObjectId::from_str(value)
}
