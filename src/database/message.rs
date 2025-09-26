use crate::database::DbPool;
use crate::models::{CreateMessageRequest, Message, MessageListResponse, MessageQuery, MessageResponse, UpdateMessageRequest};
use anyhow::Result;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

pub struct MessageRepository;

impl MessageRepository {
    pub async fn create_message(
        pool: &DbPool,
        user_id: &Uuid,
        username: &str,
        request: &CreateMessageRequest,
    ) -> Result<Message> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO messages (id, user_id, username, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(username)
        .bind(&request.content)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        let message = Message {
            id,
            user_id: *user_id,
            username: username.to_string(),
            content: request.content.clone(),
            created_at: now,
            updated_at: now,
        };

        Ok(message)
    }

    pub async fn find_by_id(pool: &DbPool, message_id: &Uuid) -> Result<Option<Message>> {
        let row = sqlx::query(
            "SELECT id, user_id, username, content, created_at, updated_at FROM messages WHERE id = ?"
        )
        .bind(message_id.to_string())
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let message = Message {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                    username: row.get("username"),
                    content: row.get("content"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(message))
            }
            None => Ok(None),
        }
    }

    pub async fn update_message(
        pool: &DbPool,
        message_id: &Uuid,
        user_id: &Uuid,
        request: &UpdateMessageRequest,
    ) -> Result<Option<Message>> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE messages SET content = ?, updated_at = ? WHERE id = ? AND user_id = ?"
        )
        .bind(&request.content)
        .bind(&now)
        .bind(message_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            let row = sqlx::query(
                "SELECT id, user_id, username, content, created_at, updated_at FROM messages WHERE id = ?"
            )
            .bind(message_id.to_string())
            .fetch_one(pool)
            .await?;

            let message = Message {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
                username: row.get("username"),
                content: row.get("content"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_message(pool: &DbPool, message_id: &Uuid, user_id: &Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM messages WHERE id = ? AND user_id = ?"
        )
        .bind(message_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn list_messages(pool: &DbPool, query: &MessageQuery) -> Result<MessageListResponse> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(10);
        let offset = ((page - 1) * per_page) as i64;

        let (rows, total) = match query.user_id {
            Some(user_id) => {
                let rows = sqlx::query(
                    "SELECT id, user_id, username, content, created_at, updated_at FROM messages WHERE user_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
                )
                .bind(user_id.to_string())
                .bind(per_page as i64)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let total_row = sqlx::query(
                    "SELECT COUNT(*) as count FROM messages WHERE user_id = ?"
                )
                .bind(user_id.to_string())
                .fetch_one(pool)
                .await?;

                let total: i64 = total_row.get("count");

                (rows, total)
            }
            None => {
                let rows = sqlx::query(
                    "SELECT id, user_id, username, content, created_at, updated_at FROM messages ORDER BY created_at DESC LIMIT ? OFFSET ?"
                )
                .bind(per_page as i64)
                .bind(offset)
                .fetch_all(pool)
                .await?;

                let total_row = sqlx::query("SELECT COUNT(*) as count FROM messages")
                    .fetch_one(pool)
                    .await?;

                let total: i64 = total_row.get("count");

                (rows, total)
            }
        };

        let messages = rows
            .into_iter()
            .map(|row| {
                Message {
                    id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                    user_id: Uuid::parse_str(&row.get::<String, _>("user_id")).unwrap(),
                    username: row.get("username"),
                    content: row.get("content"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                }
            })
            .map(MessageResponse::from)
            .collect();

        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

        Ok(MessageListResponse {
            messages,
            total,
            page,
            per_page,
            total_pages,
        })
    }
}