//! Verification store module
//! 
//! Provides a generic verification store for email/phone verification codes.

use crate::models::verification::{VerificationCodeModel, CreateVerificationCode, VerificationMedium, VerificationPurpose};
use crate::utils::errors::{AuthResult};
use database::utils::DbId;

/// Verification store trait - implement this for each database
pub trait VerificationStore: Send + Sync {
    /// Create a new verification code
    fn create(&self, input: CreateVerificationCode) -> AuthResult<VerificationCodeModel>;
    
    /// Find verification code by ID
    fn find_by_id(&self, id: &DbId) -> AuthResult<Option<VerificationCodeModel>>;
    
    /// Find valid verification code by user_id, medium, and purpose
    fn find_valid_code(&self, user_id: &DbId, medium: VerificationMedium, purpose: VerificationPurpose) -> AuthResult<Option<VerificationCodeModel>>;
    
    /// Verify a code (mark as verified)
    fn verify(&self, id: &DbId) -> AuthResult<()>;
    
    /// Increment failed attempts
    fn increment_attempts(&self, id: &DbId) -> AuthResult<()>;
    
    /// Delete/expire a code
    fn delete(&self, id: &DbId) -> AuthResult<()>;
    
    /// Delete all codes for a user
    fn delete_all_for_user(&self, user_id: &DbId) -> AuthResult<u64>;
    
    /// Cleanup expired codes
    fn cleanup_expired(&self) -> AuthResult<u64>;
}
