use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            user_id: message.user_id,
            username: message.username,
            content: message.content,
            created_at: message.created_at,
            updated_at: message.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMessageRequest {
    #[validate(length(min = 1, max = 1000, message = "留言内容长度必须在1-1000字符之间"))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMessageRequest {
    #[validate(length(min = 1, max = 1000, message = "留言内容长度必须在1-1000字符之间"))]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub user_id: Option<Uuid>,
}

impl Default for MessageQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(10),
            user_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}