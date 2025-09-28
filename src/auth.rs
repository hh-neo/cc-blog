use crate::models::Claims;
use actix_web::{dev::ServiceRequest, Error};
use actix_web::error::ErrorUnauthorized;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

pub fn create_jwt(user_id: i32, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub fn extract_user_from_request(req: &ServiceRequest) -> Result<Claims, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| ErrorUnauthorized("No authorization header"))?
        .to_str()
        .map_err(|_| ErrorUnauthorized("Invalid authorization header"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(ErrorUnauthorized("Invalid authorization format"));
    }

    let token = &auth_header[7..];
    let claims = verify_jwt(token)
        .map_err(|_| ErrorUnauthorized("Invalid token"))?;

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_creation_and_verification() {
        std::env::set_var("JWT_SECRET", "test_secret_key");

        let user_id = 1;
        let username = "testuser";

        let token = create_jwt(user_id, username).unwrap();
        let claims = verify_jwt(&token).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, username);
    }

    #[test]
    fn test_jwt_invalid_token() {
        std::env::set_var("JWT_SECRET", "test_secret_key");

        let result = verify_jwt("invalid_token");
        assert!(result.is_err());
    }

}