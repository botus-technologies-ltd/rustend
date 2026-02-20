//! Reset password handler

use actix_web::{web, HttpResponse, Error};

pub async fn reset_password(
    _state: web::Data<crate::routes::AppState>,
    _reset_req: web::Json<crate::utils::types::PasswordResetConfirm>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Reset password endpoint"
    })))
}

pub async fn change_password(
    _state: web::Data<crate::routes::AppState>,
    _user_id: web::Path<String>,
    _change_req: web::Json<crate::utils::types::ChangePasswordRequest>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Change password endpoint"
    })))
}
