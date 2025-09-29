use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Article {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateArticleRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArticleRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1))]
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArticleResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Article> for ArticleResponse {
    fn from(article: Article) -> Self {
        ArticleResponse {
            id: article.id,
            user_id: article.user_id,
            title: article.title,
            content: article.content,
            created_at: article.created_at,
            updated_at: article.updated_at,
        }
    }
}