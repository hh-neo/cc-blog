use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authentication failed")]
    AuthenticationError,

    #[error("Unauthorized")]
    UnauthorizedError,

    #[error("Not found")]
    NotFoundError,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    InternalError,

    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Password hash error")]
    HashError(#[from] bcrypt::BcryptError),

    #[error("Rate limit exceeded")]
    RateLimitError,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_response = ErrorResponse {
            error: self.to_string(),
            message: self.user_facing_message(),
        };
        HttpResponse::build(status).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::AuthenticationError => StatusCode::UNAUTHORIZED,
            AppError::UnauthorizedError => StatusCode::FORBIDDEN,
            AppError::NotFoundError => StatusCode::NOT_FOUND,
            AppError::RateLimitError => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl AppError {
    fn user_facing_message(&self) -> String {
        match self {
            AppError::DatabaseError(_) => "数据库操作失败".to_string(),
            AppError::ValidationError(msg) => format!("验证失败: {}", msg),
            AppError::AuthenticationError => "认证失败，请检查用户名和密码".to_string(),
            AppError::UnauthorizedError => "无权限访问此资源".to_string(),
            AppError::NotFoundError => "资源未找到".to_string(),
            AppError::BadRequest(msg) => format!("请求错误: {}", msg),
            AppError::InternalError => "服务器内部错误".to_string(),
            AppError::JwtError(_) => "令牌无效或已过期".to_string(),
            AppError::HashError(_) => "密码处理失败".to_string(),
            AppError::RateLimitError => "请求过于频繁，请稍后再试".to_string(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;