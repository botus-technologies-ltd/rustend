//! Users handler

use actix_web::{web, HttpResponse, Error};

pub async fn get_user(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Get user" })))
}

pub async fn update_user(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>, _update_req: web::Json<serde_json::Value>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Update user" })))
}

pub async fn delete_user(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Delete user" })))
}

pub async fn list_users(_state: web::Data<crate::routes::AppState>, _query: web::Query<serde_json::Value>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "List users" })))
}

pub async fn deactivate_user(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Deactivate user" })))
}

pub async fn activate_user(_state: web::Data<crate::routes::AppState>, _user_id: web::Path<String>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Activate user" })))
}
