//! JWT Authentication types

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub algorithm: Algorithm,
    pub access_token_expire_minutes: i64,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        Self { secret: secret.into(), algorithm: Algorithm::HS256, access_token_expire_minutes: 60 }
    }
}

/// Claims in JWT token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType { Access, Refresh }

/// JWT Service
#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
        Self { config, encoding_key, decoding_key }
    }

    pub fn generate_access_token(&self, user_id: impl Into<String>, email: Option<String>) -> Result<String, jsonwebtoken::errors::Error> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims { sub: user_id.into(), email, exp: now + (self.config.access_token_expire_minutes * 60), iat: now };
        encode(&Header::new(self.config.algorithm), &claims, &self.encoding_key)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::new(self.config.algorithm);
        validation.validate_exp = true;
        let token_data: TokenData<Claims> = decode(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}
