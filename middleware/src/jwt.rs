//! JWT Authentication
//!
//! Provides JWT token generation, validation, and middleware for request authentication.

use actix_web::{
    Error, HttpMessage, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::task::{Context, Poll};

/// JWT configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub algorithm: Algorithm,
    pub access_token_expire_minutes: i64,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            algorithm: Algorithm::HS256,
            access_token_expire_minutes: 60,
        }
    }
}

/// Claims in JWT token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub exp: i64,
    pub iat: i64,
}

/// Token information including raw token for session validation
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub claims: Claims,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT Middleware for request authentication
pub struct JwtMiddleware {
    secret: String,
}

impl JwtMiddleware {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn from_secret(secret: impl Into<String>) -> Self {
        Self::new(secret.into())
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = JwtMiddlewareImpl<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareImpl {
            service: Rc::new(service),
            secret: self.secret.clone(),
        }))
    }
}

pub struct JwtMiddlewareImpl<S> {
    service: Rc<S>,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareImpl<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);
        let secret = self.secret.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let token_opt = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            match token_opt {
                Some(token) => {
                    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
                    let validation = Validation::default();

                    match decode::<Claims>(token, &decoding_key, &validation) {
                        Ok(token_data) => {
                            // Clone claims before moving into extensions
                            let claims = token_data.claims;

                            // Store the raw token for later validation against session store
                            req.extensions_mut().insert(TokenInfo {
                                claims: claims.clone(),
                                token: token.to_string(),
                            });
                            req.extensions_mut().insert(claims);
                        }
                        Err(e) => {
                            return Ok(req.into_response(
                                HttpResponse::Unauthorized()
                                    .json(serde_json::json!({
                                        "success": false,
                                        "message": format!("Invalid access token: {}", e)
                                    }))
                                    .map_into_right_body(),
                            ));
                        }
                    }
                }
                None => {
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "success": false,
                                "message": "Missing Authorization header"
                            }))
                            .map_into_right_body(),
                    ));
                }
            }

            // Continue to downstream service
            let res = svc.call(req).await;
            Ok(res?.map_into_left_body())
        })
    }
}

/// Extension trait to easily get Claims from request
pub trait JwtClaims {
    fn claims(&self) -> Option<Claims>;
    fn token_info(&self) -> Option<TokenInfo>;
}

impl JwtClaims for actix_web::HttpRequest {
    fn claims(&self) -> Option<Claims> {
        self.extensions().get::<Claims>().cloned()
    }

    fn token_info(&self) -> Option<TokenInfo> {
        self.extensions().get::<TokenInfo>().cloned()
    }
}

// Implement FromRequest for Claims to allow direct extraction in routes
impl actix_web::FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = futures_util::future::Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        if let Some(claims) = req.extensions().get::<Claims>() {
            futures_util::future::ready(Ok(claims.clone()))
        } else {
            futures_util::future::ready(Err(actix_web::error::ErrorUnauthorized(
                "Missing or invalid JWT token",
            )))
        }
    }
}
