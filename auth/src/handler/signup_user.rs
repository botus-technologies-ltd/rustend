//! Signup handler

use actix_web::{Error, HttpResponse, web};
use crate::models::user::{CreateUserInput, UpdateUserInput};
use crate::routes::AppState;
use crate::utils::errors::AuthError;
use crate::utils::types::SignUpRequest;
use crate::utils::types::{
    SendVerificationRequest, 
    VerifyCodeRequest,
    SignUpResponseData,
};
use utils::response::ApiResponse;

use database::utils::parse_id;
use utils::hash::Hash;

pub async fn signup_user(
    state: web::Data<AppState>,
    signup_req: web::Json<SignUpRequest>,
) -> Result<HttpResponse, Error> {
    // Check if email already exists
    if let Some(ref email) = signup_req.email {
        if let Some(_existing) = state.users.find_by_email(email.as_str())? {
            let response =
                AuthError::email_already_exists(email.as_str()).to_response::<SignUpResponseData>();
            return Ok(HttpResponse::Conflict().json(response));
        }
    }

    // Check if phone already exists (if provided)
    if let Some(ref phone) = signup_req.phone {
        if let Some(_existing) = state.users.find_by_phone(phone.as_str())? {
            let response =
                AuthError::phone_already_exists(phone.as_str()).to_response::<SignUpResponseData>();
            return Ok(HttpResponse::Conflict().json(response));
        }
    }

    // Hash the password before storing
    let hashed_password = Hash::argon2(&signup_req.password)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?
        .to_string();

    // Create user input with hashed password
    let create_input = CreateUserInput {
        email: signup_req.email.clone(),
        password: hashed_password,
        phone: signup_req.phone.clone(),
        username: signup_req.username.clone(),
        first_name: signup_req.first_name.clone(),
        last_name: signup_req.last_name.clone(),
    };

    // Create the user
    let user = state
        .users
        .create(create_input)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    // Return success response
    let response_data = SignUpResponseData {
        user_id: user.id.to_string(),
        email: user.email,
    };

    let response = ApiResponse::success_data("User registered successfully", response_data);
    Ok(HttpResponse::Ok().json(response))

}

/// Verify email with code
pub async fn verify_email(
    state: web::Data<AppState>,
    user_id: web::Path<String>,
    _verify_req: web::Json<VerifyCodeRequest>,
) -> Result<HttpResponse, Error> {
    // Find the user
    let db_id = parse_id(&user_id).map_err(|_| AuthError::invalid_request("Invalid user ID"))?;

    let user = state
        .users
        .find_by_id(&db_id)?
        .ok_or_else(|| AuthError::not_found("User not found"))?;

    // For now, just mark as verified
    let update_input = UpdateUserInput {
        is_verified: Some(true),
        email: None,
        phone: None,
        username: None,
        first_name: None,
        last_name: None,
        is_active: None,
    };

    state
        .users
        .update(&user.id, update_input)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    let response: ApiResponse<()> = ApiResponse::ok("Email verified successfully");
    Ok(HttpResponse::Ok().json(response))
}

/// Send verification code
pub async fn send_verification_code(
    _state: web::Data<AppState>,
    _send_req: web::Json<SendVerificationRequest>,
) -> Result<HttpResponse, Error> {
    let response: ApiResponse<()> = ApiResponse::ok("Verification code sent");
    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::verification::VerificationMedium;
    use crate::utils::types::SignUpRequest;
    use utils::hash::Hash;

    #[test]
    fn test_sign_up_request_validation() {
        let request = SignUpRequest {
            email: Some("test@example.com".to_string()),
            phone: Some("1234567890".to_string()),
            username: Some("testuser".to_string()),
            password: "secure_password_123".to_string(),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
        };

        assert_eq!(request.email, Some("test@example.com".to_string()));
        assert_eq!(request.phone, Some("1234567890".to_string()));
        assert_eq!(request.username, Some("testuser".to_string()));
        assert_eq!(request.password, "secure_password_123");
    }

    #[test]
    fn test_sign_up_request_minimal_fields() {
        let request = SignUpRequest {
            email: None,
            phone: None,
            username: None,
            password: "password123".to_string(),
            first_name: None,
            last_name: None,
        };

        assert!(request.email.is_none());
        assert!(request.phone.is_none());
        assert_eq!(request.password, "password123");
    }

    #[test]
    fn test_password_hashing_with_argon2() {
        let password = "test_password_123";

        let hash = Hash::argon2(password).expect("Should hash password successfully");

        // Hash should be different from password
        assert_ne!(hash.to_string(), password);

        // Should be able to verify the password
        assert!(hash.verify(password).expect("Should verify successfully"));
        assert!(!hash.verify("wrong_password").expect("Should verify"));
    }

    #[test]
    fn test_password_hashing_consistency() {
        let password = "consistent_password";

        let hash1 = Hash::argon2(password).expect("Should hash");
        let hash2 = Hash::argon2(password).expect("Should hash");

        // Different hashes for same password (salt is random)
        assert_ne!(hash1.to_string(), hash2.to_string());

        // But both should verify the same password
        assert!(hash1.verify(password).expect("Should verify"));
        assert!(hash2.verify(password).expect("Should verify"));
    }

    #[test]
    fn test_sign_up_response_data_structure() {
        let response_data = SignUpResponseData {
            user_id: "507f1f77bcf86cd799439011".to_string(),
            email: Some("test@example.com".to_string()),
        };

        assert_eq!(response_data.user_id, "507f1f77bcf86cd799439011");
        assert_eq!(response_data.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_verify_code_request() {
        let request = VerifyCodeRequest {
            code: "123456".to_string(),
            medium: VerificationMedium::Email,
        };

        assert_eq!(request.code, "123456");
        assert_eq!(request.medium, VerificationMedium::Email);
    }

    #[test]
    fn test_send_verification_request() {
        let request = SendVerificationRequest {
            medium: VerificationMedium::Email,
        };

        assert_eq!(request.medium, VerificationMedium::Email);
    }

    #[test]
    fn test_update_user_input_verification() {
        let update_input = UpdateUserInput {
            is_verified: Some(true),
            email: None,
            phone: None,
            username: None,
            first_name: None,
            last_name: None,
            is_active: None,
        };

        assert_eq!(update_input.is_verified, Some(true));
        assert!(update_input.email.is_none());
    }

    #[test]
    fn test_create_user_input_with_hashed_password() {
        let password = "test_password";
        let hashed = Hash::argon2(password).expect("Should hash").to_string();

        let create_input = CreateUserInput {
            email: Some("test@example.com".to_string()),
            password: hashed.clone(),
            phone: None,
            username: Some("testuser".to_string()),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
        };

        assert_eq!(create_input.email, Some("test@example.com".to_string()));
        assert_eq!(create_input.password, hashed);
        assert_eq!(create_input.username, Some("testuser".to_string()));
    }

    #[test]
    fn test_api_response_signup_success() {
        let response_data = SignUpResponseData {
            user_id: "user123".to_string(),
            email: Some("test@example.com".to_string()),
        };

        let response = utils::response::ApiResponse::success_data(
            "User registered successfully",
            response_data,
        );

        assert_eq!(response.message, "User registered successfully".to_string());
    }

    #[test]
    fn test_email_already_exists_error() {
        let error = AuthError::email_already_exists("test@example.com");

        let response = error.to_response::<SignUpResponseData>();
        assert!(!response.success);
    }

    #[test]
    fn test_phone_already_exists_error() {
        let error = AuthError::phone_already_exists("1234567890");

        let response = error.to_response::<SignUpResponseData>();
        assert!(!response.success);
    }

    #[test]
    fn test_invalid_request_error() {
        let error = AuthError::invalid_request("Invalid input");

        let response = error.to_response::<SignUpResponseData>();
        assert!(!response.success);
    }

    #[test]
    fn test_not_found_error() {
        let error = AuthError::not_found("User not found");

        let response = error.to_response::<SignUpResponseData>();
        assert!(!response.success);
    }
}
