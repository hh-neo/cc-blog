use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: usize,
}

pub fn create_jwt(user_id: Uuid, username: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() + 24 * 3600; // 24 hours

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_string())
            } else {
                None
            }
        });

    let token = match token {
        Some(token) => token,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid authorization header" })),
            ));
        }
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

    match decode_jwt(&token, &secret) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid or expired token" })),
        )),
    }
}