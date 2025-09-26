use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

use crate::utils::jwt::verify_token;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|h| h.to_string());

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(auth_header) = auth_header {
                if let Some(token) = auth_header.strip_prefix("Bearer ") {
                    if let Ok(claims) = verify_token(token, &jwt_secret) {
                        fut.await.map(|res| {
                            res.request()
                                .extensions_mut()
                                .insert(claims);
                            res
                        })
                    } else {
                        fut.await
                    }
                } else {
                    fut.await
                }
            } else {
                fut.await
            }
        })
    }
}