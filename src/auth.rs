use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::models::Claims;

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|val| val.to_str().ok());
        match auth_header {
            Some(token_str) => {
                if let Some(_token) = token_str.strip_prefix("Bearer ") {
                    let token = &token_str[7..];
                    if let Ok(token_data) = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(b"my_secret_key"),
                        &Validation::default(),
                    ) {
                        Ok(token_data.claims)
                    } else {
                        Err(StatusCode::UNAUTHORIZED)
                    }
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
            None => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
