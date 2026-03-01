use crate::routes::AppState;
use crate::utils::errors::AuthError;
use actix_web::{HttpRequest, web};
use middleware::token_validation::{ExtractedTokenInfo, validate_token_extraction};

pub async fn validate_access_token(
    req: &HttpRequest,
    state: &web::Data<AppState>,
) -> Result<ExtractedTokenInfo, AuthError> {
    // Extract token from request
    let token_info =
        validate_token_extraction(req).map_err(|e| AuthError::unauthorized(&e.message))?;

    // Validate token exists in active session
    match state.sessions.find_by_token(&token_info.token_hash) {
        Ok(Some(session)) => {
            if session.is_revoked {
                return Err(AuthError::unauthorized("Access token has been revoked"));
            }

            if session.is_expired() {
                return Err(AuthError::unauthorized("Access token has expired"));
            }
        }
        Ok(None) => {
            return Err(AuthError::unauthorized(
                "Access token not found in session store. Please re-authenticate.",
            ));
        }
        Err(_) => {
            return Err(AuthError::internal_error("Failed to validate session"));
        }
    }

    Ok(token_info)
}
