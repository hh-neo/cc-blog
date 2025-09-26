use crate::models::user::Claims;
use crate::utils::errors::AppResult;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

pub struct JwtService {
    secret: String,
    expiration: i64,
}

impl JwtService {
    pub fn new(secret: String, expiration: i64) -> Self {
        Self { secret, expiration }
    }

    pub fn generate_token(&self, user_id: &str, username: &str) -> AppResult<String> {
        let now = Utc::now().timestamp() as usize;
        let exp = (Utc::now().timestamp() + self.expiration) as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            exp,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}