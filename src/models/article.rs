use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_published: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateArticleInput {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 65535))]
    pub content: String,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateArticleInput {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 65535))]
    pub content: Option<String>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ArticleWithAuthor {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub author_username: String,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_published: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Comment {
    pub id: String,
    pub article_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentInput {
    #[validate(length(min = 1, max = 10000))]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CommentWithUser {
    pub id: String,
    pub article_id: String,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl Article {
    pub fn new(title: String, content: String, author_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            content,
            author_id,
            view_count: 0,
            created_at: now,
            updated_at: now,
            is_published: true,
        }
    }
}

impl Comment {
    pub fn new(article_id: String, user_id: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            article_id,
            user_id,
            content,
            created_at: now,
            updated_at: now,
        }
    }
}