use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::models::{AppError, AppState, Claims};

impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|val| val.to_str().ok());
        match auth_header {
            Some(token_str) => {
                if let Some(value) = token_str.strip_prefix("Bearer ") {
                    let secret = &state.jwt_secret;

                    match decode::<Claims>(
                        value,
                        &DecodingKey::from_secret(secret.as_bytes()),
                        &Validation::default(),
                    ) {
                        Ok(token_data) => Ok(token_data.claims),
                        Err(err) => {
                            tracing::error!("JWT Decode Error: {:?}", err);
                            Err(AppError::Unauthorized(
                                "Invalid or Expired Token".to_string(),
                            ))
                        }
                    }
                } else {
                    Err(AppError::Unauthorized("Invalid Header Format".to_string()))
                }
            }
            None => Err(AppError::Unauthorized(
                "Missing Authorization Header".to_string(),
            )),
        }
    }
}
