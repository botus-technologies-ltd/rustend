//! Hashing module for passwords, OTPs, and other sensitive data
//! 
//! Provides secure hashing using Argon2 (recommended) and bcrypt.
//! Use this for storing passwords, OTPs, API keys, and other sensitive data.
//!
//! # Argon2 (Recommended)
//! Argon2 won the Password Hashing Competition and is the recommended algorithm.
//! It's resistant to GPU/ASIC attacks and allows memory and parallelism tuning.
//!
//! # Bcrypt
//! Bcrypt is widely supported and battle-tested. Good for compatibility with
//! other systems, but Argon2 is preferred for new projects.
//!
//! # Usage
//! ```ignore
//! use utils::hash::{Hash, Hasher};
//!
//! // Hash a password
//! let password = "secure_password_123";
//! let hash = Hash::argon2(password).unwrap();
//! 
//! // Verify password
//! assert!(hash.verify(password).unwrap());
//! assert!(!hash.verify("wrong_password").unwrap());
//! ```

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};

/// Errors that can occur during hashing/verification
#[derive(Debug, Clone)]
pub enum HashError {
    HashingFailed(String),
    VerificationFailed,
    InvalidHash,
}

impl std::fmt::Display for HashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashError::HashingFailed(msg) => write!(f, "Hashing failed: {}", msg),
            HashError::VerificationFailed => write!(f, "Password verification failed"),
            HashError::InvalidHash => write!(f, "Invalid hash format"),
        }
    }
}

impl std::error::Error for HashError {}

/// Hash output containing the algorithm, salt, and hash
/// Serializes to a single string format for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash {
    algorithm: String,
    hash: String,
}

impl Hash {
    /// Create Argon2 hash from plaintext
    /// 
    /// Uses Argon2id variant with secure defaults:
    /// - Memory: 64MB
    /// - Iterations: 3
    /// - Parallelism: 4
    pub fn argon2(password: &str) -> Result<Self, HashError> {
        let salt = SaltString::generate(&mut OsRng);
        
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| HashError::HashingFailed(e.to_string()))?
            .to_string();

        Ok(Self {
            algorithm: "argon2".to_string(),
            hash: password_hash,
        })
    }

    /// Create Argon2 hash with custom parameters
    /// 
    /// # Arguments
    /// * `password` - Plaintext to hash
    /// * `memory` - Memory in KB (default 65536 = 64MB)
    /// * `iterations` - Number of iterations (default 3)
    /// * `parallelism` - Parallel threads (default 4)
    pub fn argon2_custom(password: &str, memory: u32, iterations: u32, parallelism: u32) -> Result<Self, HashError> {
        let salt = SaltString::generate(&mut OsRng);
        
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(memory, iterations, parallelism, None)
                .map_err(|e| HashError::HashingFailed(e.to_string()))?,
        );
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| HashError::HashingFailed(e.to_string()))?
            .to_string();

        Ok(Self {
            algorithm: "argon2".to_string(),
            hash: password_hash,
        })
    }

    /// Create bcrypt hash
    /// 
    /// Uses default cost (12). Higher = more secure but slower.
    /// Cost range: 4-31
    pub fn bcrypt(password: &str) -> Result<Self, HashError> {
        let hashed = hash(password, DEFAULT_COST)
            .map_err(|e| HashError::HashingFailed(e.to_string()))?;

        Ok(Self {
            algorithm: "bcrypt".to_string(),
            hash: hashed,
        })
    }

    /// Create bcrypt hash with custom cost
    pub fn bcrypt_cost(password: &str, cost: u32) -> Result<Self, HashError> {
        let hashed = hash(password, cost)
            .map_err(|e| HashError::HashingFailed(e.to_string()))?;

        Ok(Self {
            algorithm: "bcrypt".to_string(),
            hash: hashed,
        })
    }

    /// Verify plaintext against stored hash
    /// Automatically detects algorithm from stored hash
    pub fn verify(&self, plaintext: &str) -> Result<bool, HashError> {
        match self.algorithm.as_str() {
            "argon2" => self.verify_argon2(plaintext),
            "bcrypt" => self.verify_bcrypt(plaintext),
            _ => Err(HashError::InvalidHash),
        }
    }

    /// Verify using Argon2
    fn verify_argon2(&self, plaintext: &str) -> Result<bool, HashError> {
        let parsed_hash = PasswordHash::new(&self.hash)
            .map_err(|_| HashError::InvalidHash)?;

        Ok(Argon2::default()
            .verify_password(plaintext.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Verify using bcrypt
    fn verify_bcrypt(&self, plaintext: &str) -> Result<bool, HashError> {
        match verify(plaintext, &self.hash) {
            Ok(result) => Ok(result),
            Err(_) => Ok(false),
        }
    }

    /// Get the raw hash string (for storage)
    pub fn to_string(&self) -> String {
        self.hash.clone()
    }

    /// Parse hash from stored string
    pub fn from_string(s: &str) -> Result<Self, HashError> {
        // Detect algorithm from format
        // Argon2 starts with $argon2
        // Bcrypt starts with $2a$, $2b$, or $2y$
        let algorithm = if s.starts_with("$argon2") {
            "argon2"
        } else if s.starts_with("$2a$") || s.starts_with("$2b$") || s.starts_with("$2y$") {
            "bcrypt"
        } else {
            return Err(HashError::InvalidHash);
        };

        Ok(Self {
            algorithm: algorithm.to_string(),
            hash: s.to_string(),
        })
    }
}

/// Convenience trait for one-liner hashing
pub trait Hasher {
    fn hash(&self) -> Result<Hash, HashError>;
    fn verify(&self, hash: &Hash) -> Result<bool, HashError>;
}

impl Hasher for str {
    fn hash(&self) -> Result<Hash, HashError> {
        Hash::argon2(self)
    }

    fn verify(&self, hash: &Hash) -> Result<bool, HashError> {
        hash.verify(self)
    }
}

impl Hasher for String {
    fn hash(&self) -> Result<Hash, HashError> {
        Hash::argon2(self)
    }

    fn verify(&self, hash: &Hash) -> Result<bool, HashError> {
        hash.verify(self)
    }
}

/// Generate a secure random string (for OTPs, API keys, etc.)
/// Uses alphanumeric characters (A-Z, a-z, 0-9)
pub fn generate_random(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a numeric OTP (e.g., 6 digits)
pub fn generate_otp(digits: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..digits)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect()
}

/// Generate a hex string (for API keys, tokens)
pub fn generate_hex(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2_hash_and_verify() {
        let password = "secure_password_123";
        let hash = Hash::argon2(password).unwrap();
        
        assert!(hash.verify(password).unwrap());
        assert!(!hash.verify("wrong_password").unwrap());
    }

    #[test]
    fn test_bcrypt_hash_and_verify() {
        let password = "secure_password_123";
        let hash = Hash::bcrypt(password).unwrap();
        
        assert!(hash.verify(password).unwrap());
        assert!(!hash.verify("wrong_password").unwrap());
    }

    #[test]
    fn test_from_string() {
        let hash = Hash::argon2("password").unwrap();
        let hash_str = hash.to_string();
        
        let parsed = Hash::from_string(&hash_str).unwrap();
        assert!(parsed.verify("password").unwrap());
    }

    #[test]
    fn test_hasher_trait() {
        let hash = "password".hash().unwrap();
        assert!(hash.verify("password").unwrap());
    }

    #[test]
    fn test_generate_otp() {
        let otp = generate_otp(6);
        assert_eq!(otp.len(), 6);
        assert!(otp.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_random() {
        let s = generate_random(16);
        assert_eq!(s.len(), 16);
    }
}
