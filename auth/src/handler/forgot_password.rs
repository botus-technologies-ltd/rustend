//! Forgot password handler

use actix_web::{web, HttpResponse, Error};

pub async fn forgot_password(
    _state: web::Data<crate::routes::AppState>,
    _reset_req: web::Json<crate::utils::types::PasswordResetRequest>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Forgot password endpoint"
    })))
}
