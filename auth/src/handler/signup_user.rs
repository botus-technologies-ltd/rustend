//! Signup handler

use actix_web::{web, HttpResponse, Error};

pub async fn signup_user(
    _state: web::Data<crate::routes::AppState>,
    _signup_req: web::Json<crate::utils::types::SignUpRequest>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Signup endpoint"
    })))
}

pub async fn verify_email(
    _state: web::Data<crate::routes::AppState>,
    _user_id: web::Path<String>,
    _verify_req: web::Json<crate::utils::types::VerifyCodeRequest>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Verify email endpoint"
    })))
}

pub async fn send_verification_code(
    _state: web::Data<crate::routes::AppState>,
    _send_req: web::Json<crate::utils::types::SendVerificationRequest>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Send verification code endpoint"
    })))
}
