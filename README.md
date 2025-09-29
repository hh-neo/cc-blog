# Message Board API

A RESTful API built with Rust and PostgreSQL for a message board application with user authentication and article management.

## Features

- User registration and authentication
- JWT-based authentication
- Create, read, update, and delete articles
- Password hashing with Argon2
- PostgreSQL database integration

## Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- cargo

## Setup

1. **Install PostgreSQL** if not already installed

2. **Create the database**
   ```bash
   psql -U postgres
   CREATE DATABASE message_board;
   \q
   ```

3. **Run the database schema**
   ```bash
   psql -U postgres -d message_board -f schema.sql
   ```

4. **Configure environment variables**

   Edit the `.env` file and update with your database credentials:
   ```
   DATABASE_URL=postgres://postgres:yourpassword@localhost/message_board
   JWT_SECRET=your-secret-key-change-this-in-production
   SERVER_HOST=127.0.0.1
   SERVER_PORT=8080
   ```

5. **Build and run the application**
   ```bash
   cargo build
   cargo run
   ```

   The server will start at `http://127.0.0.1:8080`

## API Endpoints

### Authentication

- **Register**: `POST /api/auth/register`
  ```json
  {
    "username": "john_doe",
    "email": "john@example.com",
    "password": "secure_password"
  }
  ```

- **Login**: `POST /api/auth/login`
  ```json
  {
    "username": "john_doe",
    "password": "secure_password"
  }
  ```

- **Get Current User**: `GET /api/auth/me`
  - Requires: Authorization header with Bearer token

### Articles

- **Get All Articles**: `GET /api/articles`

- **Get Single Article**: `GET /api/articles/{id}`

- **Create Article**: `POST /api/articles`
  - Requires: Authorization header with Bearer token
  ```json
  {
    "title": "Article Title",
    "content": "Article content here..."
  }
  ```

- **Update Article**: `PUT /api/articles/{id}`
  - Requires: Authorization header with Bearer token
  - Only the author can update their own articles
  ```json
  {
    "title": "Updated Title (optional)",
    "content": "Updated content (optional)"
  }
  ```

- **Delete Article**: `DELETE /api/articles/{id}`
  - Requires: Authorization header with Bearer token
  - Only the author can delete their own articles

- **Get My Articles**: `GET /api/articles/my`
  - Requires: Authorization header with Bearer token
  - Returns all articles created by the authenticated user

### Health Check

- **Health**: `GET /api/health`

## Testing

Run tests with:
```bash
cargo test
```

## Example Usage

1. **Register a new user**
   ```bash
   curl -X POST http://localhost:8080/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"username":"testuser","email":"test@example.com","password":"testpass123"}'
   ```

2. **Login** (save the token from the response)
   ```bash
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"testuser","password":"testpass123"}'
   ```

3. **Create an article** (replace TOKEN with your actual token)
   ```bash
   curl -X POST http://localhost:8080/api/articles \
     -H "Authorization: Bearer TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"title":"My First Article","content":"This is the content"}'
   ```

## Project Structure

```
message_board/
├── src/
│   ├── main.rs           # Application entry point
│   ├── db/               # Database connection
│   ├── handlers/         # Request handlers
│   ├── middleware/       # Authentication middleware
│   ├── models/           # Data models
│   └── utils/            # Utility functions
├── tests/                # Test files
├── schema.sql            # Database schema
├── Cargo.toml            # Dependencies
├── .env                  # Environment variables
└── README.md             # This file
```

## Security Notes

- Change the JWT_SECRET in production
- Use strong passwords
- Keep your .env file secure and never commit it to version control
- Consider using HTTPS in production