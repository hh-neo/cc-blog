use actix_web::{
    dev::Payload,
    error::ErrorUnauthorized,
    http::header::AUTHORIZATION,
    Error, FromRequest, HttpRequest,
};
use futures::future::{ready, Ready};
use uuid::Uuid;

use crate::utils::decode_jwt;

pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        if let Some(auth_str) = auth_header {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match decode_jwt(token) {
                    Ok(claims) => {
                        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                            return ready(Ok(AuthUser {
                                user_id,
                                username: claims.username,
                            }));
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        ready(Err(ErrorUnauthorized("Invalid or missing authorization")))
    }
}