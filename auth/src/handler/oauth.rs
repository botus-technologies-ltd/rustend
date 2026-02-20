//! OAuth handler

use actix_web::{web, HttpResponse, Error};

pub async fn oauth_redirect(_state: web::Data<crate::routes::AppState>, _provider: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "OAuth redirect" })))
}

pub async fn oauth_callback(_state: web::Data<crate::routes::AppState>, _provider: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "OAuth callback" })))
}

pub async fn link_oauth(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Link OAuth" })))
}

pub async fn unlink_oauth(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Unlink OAuth" })))
}

pub async fn list_oauth_connections(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "List OAuth connections" })))
}
