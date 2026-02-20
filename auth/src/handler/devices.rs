//! Devices handler

use actix_web::{web, HttpResponse, Error};

pub async fn list_devices(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "List devices" })))
}

pub async fn get_device(_state: web::Data<crate::routes::AppState>, _session_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Get device" })))
}

pub async fn trust_device(_state: web::Data<crate::routes::AppState>, _session_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Trust device" })))
}

pub async fn untrust_device(_state: web::Data<crate::routes::AppState>, _session_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Untrust device" })))
}

pub async fn revoke_device(_state: web::Data<crate::routes::AppState>, _session_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Revoke device" })))
}

pub async fn revoke_all_devices(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Revoke all devices" })))
}
