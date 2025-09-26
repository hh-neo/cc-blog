use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Authentication failed")]
    AuthenticationError,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found")]
    NotFound,

    #[error("Permission denied")]
    Forbidden,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            error: self.to_string(),
        };

        match self {
            AppError::AuthenticationError => {
                HttpResponse::Unauthorized().json(error_response)
            }
            AppError::BadRequest(_) | AppError::ValidationError(_) => {
                HttpResponse::BadRequest().json(error_response)
            }
            AppError::NotFound => {
                HttpResponse::NotFound().json(error_response)
            }
            AppError::Forbidden => {
                HttpResponse::Forbidden().json(error_response)
            }
            AppError::DatabaseError(_) | AppError::InternalServerError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Internal server error".to_string(),
                })
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::AuthenticationError => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) | AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::DatabaseError(_) | AppError::InternalServerError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}