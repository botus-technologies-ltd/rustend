//! Encryption module using AES-256-GCM
//! 
//! This module provides secure symmetric encryption for communication between
//! backend and frontend. Uses AES-256-GCM (Galois/Counter Mode) which provides
//! both confidentiality and authenticity.
//!
//! # Security Features
//! - AES-256-GCM: Industry-standard authenticated encryption
//! - Random 96-bit nonce for each encryption (prevents replay attacks)
//! - Authentication tag to verify data integrity
//! - No padding oracle vulnerabilities
//!
//! # Usage
//! ```ignore
//! use utils::encryption::AesGcmEncryption;
//!
//! // Initialize with a 32-byte key (256 bits)
//! let key = "your-32-byte-secret-key-here!!".as_bytes();
//! let encryptor = AesGcmEncryption::new(key).unwrap();
//!
//! // Encrypt data
//! let plaintext = "Sensitive data";
//! let encrypted = encryptor.encrypt(plaintext).unwrap();
//!
//! // Decrypt data
//! let decrypted = encryptor.decrypt(&encrypted).unwrap();
//! assert_eq!(plaintext, decrypted);
//! ```

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

/// Errors that can occur during encryption/decryption
#[derive(Debug, Clone)]
pub enum EncryptionError {
    InvalidKeyLength,
    InvalidCiphertext,
    DecryptionFailed,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::InvalidKeyLength => write!(f, "Key must be exactly 32 bytes"),
            EncryptionError::InvalidCiphertext => write!(f, "Invalid ciphertext format"),
            EncryptionError::DecryptionFailed => write!(f, "Decryption failed - data may be tampered"),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// AES-256-GCM Encryptor
/// 
/// Provides symmetric encryption using AES-256-GCM.
/// Each encryption generates a new random nonce.
pub struct AesGcmEncryption {
    cipher: Aes256Gcm,
}

impl AesGcmEncryption {
    /// Create a new encryptor with a 32-byte key
    /// 
    /// # Arguments
    /// * `key` - 32-byte (256-bit) secret key
    /// 
    /// # Returns
    /// Ok(Self) if key is exactly 32 bytes, Err otherwise
    pub fn new(key: &[u8]) -> Result<Self, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength);
        }
        
        let cipher = Aes256Gcm::new_from_slice(key)
            .expect("Key length is guaranteed to be 32 bytes");
        
        Ok(Self { cipher })
    }

    /// Encrypt plaintext and return base64-encoded ciphertext
    /// 
    /// Output format: [nonce (12 bytes)][ciphertext][auth tag (16 bytes)]
    /// All base64-encoded into a single string
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        // Generate random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| EncryptionError::DecryptionFailed)?;

        // Combine nonce + ciphertext
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        // Base64 encode
        Ok(BASE64.encode(&combined))
    }

    /// Encrypt plaintext from bytes
    pub fn encrypt_bytes(&self, plaintext: &[u8]) -> Result<String, EncryptionError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| EncryptionError::DecryptionFailed)?;

        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// Decrypt base64-encoded ciphertext
    pub fn decrypt(&self, ciphertext: &str) -> Result<String, EncryptionError> {
        // Base64 decode
        let combined = BASE64.decode(ciphertext)
            .map_err(|_| EncryptionError::InvalidCiphertext)?;

        // Must have at least nonce (12) + tag (16) = 28 bytes
        if combined.len() < 28 {
            return Err(EncryptionError::InvalidCiphertext);
        }

        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&combined[..12]);
        let encrypted_data = &combined[12..];

        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|_| EncryptionError::DecryptionFailed)?;

        String::from_utf8(plaintext)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }

    /// Decrypt to bytes
    pub fn decrypt_bytes(&self, ciphertext: &str) -> Result<Vec<u8>, EncryptionError> {
        let combined = BASE64.decode(ciphertext)
            .map_err(|_| EncryptionError::InvalidCiphertext)?;

        if combined.len() < 28 {
            return Err(EncryptionError::InvalidCiphertext);
        }

        let nonce = Nonce::from_slice(&combined[..12]);
        let encrypted_data = &combined[12..];

        self.cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }
}

/// Generate a random 32-byte key
/// 
/// # Example
/// ```ignore
/// let key = utils::encryption::generate_key();
/// let encryptor = AesGcmEncryption::new(&key).unwrap();
/// ```
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Generate a random key and return as hex string
pub fn generate_key_hex() -> String {
    let key = generate_key();
    key.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Generate a random key and return as base64 string
pub fn generate_key_base64() -> String {
    let key = generate_key();
    BASE64.encode(key)
}
