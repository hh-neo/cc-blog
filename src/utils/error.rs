use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Authentication failed")]
    AuthenticationError,

    #[error("User not found")]
    UserNotFound,

    #[error("Article not found")]
    ArticleNotFound,

    #[error("Username or email already exists")]
    DuplicateUser,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal server error")]
    InternalServerError,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let message = self.to_string();

        HttpResponse::build(status).json(json!({
            "error": message,
            "status": status.as_u16()
        }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthenticationError => StatusCode::UNAUTHORIZED,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::ArticleNotFound => StatusCode::NOT_FOUND,
            AppError::DuplicateUser => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}