use crate::models::user::Claims;
use crate::utils::{jwt::JwtService, AppError};
use actix_web::{
    dev::ServiceRequest,
    error::Error,
    http::header,
    web::{Data, ReqData},
    FromRequest, HttpMessage,
};
use actix_web_lab::middleware::from_fn;
use std::future::{ready, Ready};

pub struct AuthUser {
    pub user_id: String,
    pub username: String,
}

impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_user = req
            .extensions()
            .get::<AuthUser>()
            .cloned()
            .ok_or(AppError::UnauthorizedError);

        ready(auth_user)
    }
}

impl Clone for AuthUser {
    fn clone(&self) -> Self {
        Self {
            user_id: self.user_id.clone(),
            username: self.username.clone(),
        }
    }
}

pub async fn auth_middleware(
    req: ServiceRequest,
    next: impl actix_web::dev::Service<
        ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = Error,
    >,
) -> Result<actix_web::dev::ServiceResponse, Error> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_str) = auth_header {
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            if let Some(jwt_service) = req.app_data::<Data<JwtService>>() {
                if let Ok(claims) = jwt_service.verify_token(token) {
                    req.extensions_mut().insert(AuthUser {
                        user_id: claims.sub,
                        username: claims.username,
                    });
                }
            }
        }
    }

    next.call(req).await
}

pub async fn require_auth(
    req: ServiceRequest,
    next: impl actix_web::dev::Service<
        ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = Error,
    >,
) -> Result<actix_web::dev::ServiceResponse, Error> {
    let has_auth = req.extensions().get::<AuthUser>().is_some();

    if !has_auth {
        return Err(AppError::UnauthorizedError.into());
    }

    next.call(req).await
}