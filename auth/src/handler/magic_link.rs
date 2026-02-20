//! Magic link handler

use actix_web::{web, HttpRequest, HttpResponse, Error};

pub async fn request_magic_link(_req: HttpRequest, _state: web::Data<crate::routes::AppState>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Request magic link" })))
}

pub async fn verify_magic_link(_req: HttpRequest, _state: web::Data<crate::routes::AppState>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Verify magic link" })))
}
