use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("验证错误: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("JWT错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("加密错误: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("UUID解析错误: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("未授权: {0}")]
    Unauthorized(String),

    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("冲突: {0}")]
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match &self {
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误")
            }
            AppError::Validation(err) => {
                tracing::warn!("Validation error: {:?}", err);
                (StatusCode::BAD_REQUEST, "输入验证失败")
            }
            AppError::Jwt(err) => {
                tracing::warn!("JWT error: {:?}", err);
                (StatusCode::UNAUTHORIZED, "认证失败")
            }
            AppError::Bcrypt(err) => {
                tracing::error!("Bcrypt error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "密码处理错误")
            }
            AppError::Uuid(err) => {
                tracing::warn!("UUID error: {:?}", err);
                (StatusCode::BAD_REQUEST, "ID格式错误")
            }
            AppError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务错误")
            }
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "未授权访问"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "资源未找到"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "资源冲突"),
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string(),
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}