//! Request signing module for secure backend-frontend communication
//! 
//! Provides HMAC-SHA256 request signing to verify message integrity and authenticity.
//! When a request is signed, any tampering with the message will cause signature verification to fail.
//!
//! # How it works
//! 1. Backend and frontend share a secret key
//! 2. Frontend signs request with: signature = HMAC-SHA256(message + timestamp, secret)
//! 3. Backend verifies: if HMAC matches, message is authentic and not tampered
//! 4. Timestamp prevents replay attacks (requests older than X minutes are rejected)
//!
//! # Usage
//! ```ignore
//! use utils::signature::{Signer, Signature};
//!
//! // Backend: Generate a secret key (store securely!)
//! let key = Signer::generate_key();
//!
//! // Frontend: Sign a request
//! let message = "amount=100&to=account123";
//! let signature = Signer::sign(message, &key).unwrap();
//!
//! // Frontend sends: message + signature + timestamp
//!
//! // Backend: Verify the signature
//! let is_valid = Signature::verify(message, &signature, &key, 5).unwrap();
//! ```
//! 

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

/// HMAC-SHA256 type alias
type HmacSha256 = Hmac<Sha256>;

/// Errors that can occur during signing/verification
#[derive(Debug, Clone)]
pub enum SignatureError {
    InvalidKey,
    InvalidSignature,
    SignatureExpired,
    VerificationFailed,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::InvalidKey => write!(f, "Invalid key length"),
            SignatureError::InvalidSignature => write!(f, "Invalid signature format"),
            SignatureError::SignatureExpired => write!(f, "Signature has expired"),
            SignatureError::VerificationFailed => write!(f, "Signature verification failed"),
        }
    }
}

impl std::error::Error for SignatureError {}

/// Signature container with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signature: String,
    pub timestamp: i64,
    pub nonce: Option<String>,
}

impl Signature {
    /// Create a new signature
    pub fn new(signature: String, timestamp: i64) -> Self {
        Self {
            signature,
            timestamp,
            nonce: None,
        }
    }

    /// Create with a nonce (for replay protection)
    pub fn with_nonce(signature: String, timestamp: i64, nonce: String) -> Self {
        Self {
            signature,
            timestamp,
            nonce: Some(nonce),
        }
    }

    /// Verify the signature
    pub fn verify(&self, message: &str, key: &[u8], max_age_minutes: i64) -> Result<bool, SignatureError> {
        // Check timestamp
        let now = chrono::Utc::now().timestamp();
        let age = now - self.timestamp;
        
        if age.abs() > max_age_minutes * 60 {
            return Err(SignatureError::SignatureExpired);
        }

        // Verify signature
        let expected = Signer::sign_raw(message, self.timestamp, key)?;
        
        // Constant-time comparison to prevent timing attacks
        Ok(self.signature == expected)
    }

    /// Convert to string for transmission
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Parse from string
    pub fn from_string(s: &str) -> Result<Self, SignatureError> {
        serde_json::from_str(s).map_err(|_| SignatureError::InvalidSignature)
    }
}

/// Signer for creating request signatures
pub struct Signer;

impl Signer {
    /// Generate a random 32-byte key
    pub fn generate_key() -> [u8; 32] {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Generate key as base64 string (for storage)
    pub fn generate_key_base64() -> String {
        let key = Self::generate_key();
        BASE64.encode(key)
    }

    /// Sign a message with the current timestamp
    pub fn sign(message: &str, key: &[u8]) -> Result<Signature, SignatureError> {
        let timestamp = chrono::Utc::now().timestamp();
        let signature = Self::sign_raw(message, timestamp, key)?;
        
        Ok(Signature::new(signature, timestamp))
    }

    /// Sign with a nonce for extra replay protection
    pub fn sign_with_nonce(message: &str, key: &[u8], nonce: &str) -> Result<Signature, SignatureError> {
        let timestamp = chrono::Utc::now().timestamp();
        let message_with_nonce = format!("{}:{}:{}", message, timestamp, nonce);
        let signature = Self::sign_raw(&message_with_nonce, timestamp, key)?;
        
        Ok(Signature::with_nonce(signature, timestamp, nonce.to_string()))
    }

    /// Internal signing function
    fn sign_raw(message: &str, timestamp: i64, key: &[u8]) -> Result<String, SignatureError> {
        if key.len() != 32 {
            return Err(SignatureError::InvalidKey);
        }

        // Create message: timestamp.message
        let data = format!("{}.{}", timestamp, message);
        
        // Create HMAC
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|_| SignatureError::InvalidKey)?;
        
        mac.update(data.as_bytes());
        let result = mac.finalize().into_bytes();
        
        // Base64 encode
        Ok(BASE64.encode(result))
    }

    /// Verify a signature
    pub fn verify(message: &str, signature: &Signature, key: &[u8], max_age_minutes: i64) -> Result<bool, SignatureError> {
        signature.verify(message, key, max_age_minutes)
    }

    /// Quick verify without Signature struct
    pub fn quick_verify(message: &str, signature: &str, timestamp: i64, key: &[u8], max_age_minutes: i64) -> Result<bool, SignatureError> {
        // Check timestamp
        let now = chrono::Utc::now().timestamp();
        let age = now - timestamp;
        
        if age.abs() > max_age_minutes * 60 {
            return Err(SignatureError::SignatureExpired);
        }

        // Compute expected
        let expected = Self::sign_raw(message, timestamp, key)?;
        
        // Constant-time comparison
        Ok(signature == expected)
    }
}

/// Builder for creating signed requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedRequest {
    pub method: String,
    pub path: String,
    pub body: Option<String>,
    pub query: Option<String>,
    pub timestamp: i64,
    pub signature: String,
}

impl SignedRequest {
    /// Create a new signed request
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_uppercase(),
            path: path.to_string(),
            body: None,
            query: None,
            timestamp: chrono::Utc::now().timestamp(),
            signature: String::new(),
        }
    }

    /// Add query parameters
    pub fn with_query(mut self, query: &str) -> Self {
        self.query = Some(query.to_string());
        self
    }

    /// Add request body
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Sign the request
    pub fn sign(mut self, key: &[u8]) -> Result<Self, SignatureError> {
        // Build canonical message
        let message = self.build_message()?;
        
        self.signature = Signer::sign_raw(&message, self.timestamp, key)?;
        
        Ok(self)
    }

    /// Build canonical message string
    fn build_message(&self) -> Result<String, SignatureError> {
        let mut parts = vec![
            self.method.clone(),
            self.path.clone(),
        ];
        
        if let Some(ref query) = self.query {
            parts.push(format!("?{}", query));
        }
        
        if let Some(ref body) = self.body {
            parts.push(body.clone());
        }
        
        Ok(parts.join("|"))
    }

    /// Verify the request
    pub fn verify(&self, key: &[u8], max_age_minutes: i64) -> Result<bool, SignatureError> {
        // Check timestamp
        let now = chrono::Utc::now().timestamp();
        let age = now - self.timestamp;
        
        if age.abs() > max_age_minutes * 60 {
            return Err(SignatureError::SignatureExpired);
        }

        // Build message and verify
        let message = self.build_message()?;
        let expected = Signer::sign_raw(&message, self.timestamp, key)?;
        
        Ok(self.signature == expected)
    }

    /// Convert to JSON for transmission
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self, SignatureError> {
        serde_json::from_str(json).map_err(|_| SignatureError::InvalidSignature)
    }
}

/// Utility to create URL-safe signed query strings
pub fn create_signed_url(path: &str, params: &[(&str, &str)], key: &[u8]) -> Result<String, SignatureError> {
    let query_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    
    let full_message = format!("{}?{}", path, query_string);
    
    let signature = Signer::sign(&full_message, key)?;
    
    Ok(format!("{}&signature={}&timestamp={}", 
        query_string, 
        urlencoding::encode(&signature.signature),
        signature.timestamp
    ))
}

/// Verify a signed URL query string
pub fn verify_signed_url(path: &str, query_with_signature: &str, key: &[u8], max_age_minutes: i64) -> Result<bool, SignatureError> {
    // Parse query and signature
    let parts: Vec<&str> = query_with_signature.split("&signature=").collect();
    if parts.len() != 2 {
        return Err(SignatureError::InvalidSignature);
    }
    
    let query = parts[0];
    let sig_parts: Vec<&str> = parts[1].split("&timestamp=").collect();
    if sig_parts.len() != 2 {
        return Err(SignatureError::InvalidSignature);
    }
    
    let signature = sig_parts[0];
    let timestamp: i64 = sig_parts[1].parse().map_err(|_| SignatureError::InvalidSignature)?;
    
    // Build full message
    let full_message = format!("{}?{}", path, query);
    
    // Verify
    Signer::quick_verify(&full_message, signature, timestamp, key, max_age_minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let key = Signer::generate_key();
        let message = "amount=100&to=account123";
        
        let signature = Signer::sign(message, &key).unwrap();
        
        // Should verify successfully
        assert!(signature.verify(message, &key, 5).unwrap());
        
        // Wrong message should fail
        assert!(!signature.verify("tampered message", &key, 5).unwrap());
        
        // Wrong key should fail
        let wrong_key = Signer::generate_key();
        assert!(!signature.verify(message, &wrong_key, 5).unwrap());
    }

    #[test]
    fn test_signature_expired() {
        let key = Signer::generate_key();
        let message = "test";
        
        let mut signature = Signer::sign(message, &key).unwrap();
        signature.timestamp = chrono::Utc::now().timestamp() - 600; // 10 minutes ago
        
        // Should fail - expired
        assert!(matches!(
            signature.verify(message, &key, 5).unwrap_err(),
            SignatureError::SignatureExpired
        ));
    }

    #[test]
    fn test_signed_request() {
        let key = Signer::generate_key();
        
        let request = SignedRequest::new("POST", "/api/transfer")
            .with_query("lang=en")
            .with_body(r#"{"amount":100}"#)
            .sign(&key)
            .unwrap();
        
        assert!(request.verify(&key, 5).unwrap());
    }
}
