use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

use crate::models::Claims;
use crate::utils::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError)
        .map(|hash| hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AppError::InternalServerError)?;

    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub fn create_jwt(user_id: &str, username: &str) -> Result<String, AppError> {
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());

    let expiration = Utc::now() + Duration::hours(24);

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiration.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalServerError)
}

pub fn decode_jwt(token: &str) -> Result<Claims, AppError> {
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}