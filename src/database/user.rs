use crate::database::DbPool;
use crate::models::{CreateUserRequest, User};
use anyhow::Result;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    pub async fn create_user(pool: &DbPool, request: &CreateUserRequest) -> Result<User> {
        let id = Uuid::new_v4();
        let password_hash = hash(&request.password, DEFAULT_COST)?;
        let now = Utc::now();

        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id.to_string())
        .bind(&request.username)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        let user = User {
            id,
            username: request.username.clone(),
            email: request.email.clone(),
            password_hash,
            created_at: now,
            updated_at: now,
        };

        Ok(user)
    }

    pub async fn find_by_username(pool: &DbPool, username: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    username: row.get("username"),
                    email: row.get("email"),
                    password_hash: row.get("password_hash"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    username: row.get("username"),
                    email: row.get("email"),
                    password_hash: row.get("password_hash"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
        Ok(bcrypt::verify(password, password_hash)?)
    }
}