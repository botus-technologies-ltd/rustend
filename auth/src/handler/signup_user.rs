//! Signup handler

use actix_web::{Error, HttpResponse, web};
use crate::models::user::{CreateUserInput};
use crate::routes::AppState;
use crate::utils::errors::AuthError;
use crate::utils::types::{
    SendVerificationRequest, 
    VerifyCodeRequest,
    SignUpResponseData,
    SignUpRequest
};

use utils::response::ApiResponse;
use database::utils::parse_id;
use utils::hash::Hash;

pub async fn signup_user(
    state:      web::Data<AppState>,
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
        email:    signup_req.email.clone(),
        password: hashed_password,
        phone:    signup_req.phone.clone(),
    };

    // Create the user
    let user = state
        .users
        .create(create_input)
        .map_err(|e| AuthError::internal_error(&e.to_string()))?;

    // Return success response
    let response_data = SignUpResponseData {
        user_id: user.id.to_string(),
        email:   user.email,
    };

    let response = ApiResponse::success_data("User registered successfully", response_data);
    Ok(HttpResponse::Ok().json(response))

}

/// Verify email with code
pub async fn verify_email(
    state:       web::Data<AppState>,
    user_id:     web::Path<String>,
    _verify_req: web::Json<VerifyCodeRequest>,
) -> Result<HttpResponse, Error> {
    // Find the user
    let db_id = parse_id(&user_id).map_err(|_| AuthError::invalid_request("Invalid user ID"))?;

    let mut user = state
        .users
        .find_by_id(&db_id)?
        .ok_or_else(|| AuthError::not_found("User not found"))?;

    // For now, just mark as verified
    user.verify_user();
    let response: ApiResponse<()> = ApiResponse::ok("Email verified successfully");
    Ok(HttpResponse::Ok().json(response))
}

/// Send verification code
pub async fn send_verification_code(
    _state:    web::Data<AppState>,
    _send_req: web::Json<SendVerificationRequest>,
) -> Result<HttpResponse, Error> {
    let response: ApiResponse<()> = ApiResponse::ok("Verification code sent");
    Ok(HttpResponse::Ok().json(response))
}