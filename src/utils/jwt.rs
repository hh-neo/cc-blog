use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::utils::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: String, username: String) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp();

        Claims {
            sub: user_id,
            username,
            exp,
            iat: now.timestamp(),
        }
    }
}

pub fn create_token(user_id: &str, username: &str, secret: &str) -> Result<String, AppError> {
    let claims = Claims::new(user_id.to_string(), username.to_string());

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| AppError::InternalServerError)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::AuthenticationError)
}