use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Article {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateArticleInput {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArticleInput {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1))]
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub user_id: Option<String>,
}

impl Default for ArticleQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(10),
            user_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}