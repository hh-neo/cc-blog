# Message Board API

A RESTful API built with Rust, Axum, and PostgreSQL for a message board application with user authentication and article management.

## Features

- User registration and authentication
- JWT token-based authentication
- CRUD operations for articles
- Input validation
- Password hashing with bcrypt
- PostgreSQL database integration

## Prerequisites

- Rust (latest stable version)
- PostgreSQL
- cargo

## Setup

1. Create a PostgreSQL database:
```bash
createdb message_board
```

2. Configure environment variables in `.env`:
```
DATABASE_URL=postgres://username:password@localhost:5432/message_board
JWT_SECRET=your-secret-key-change-this-in-production
PORT=3000
```

3. Install dependencies:
```bash
cargo build
```

4. Run database migrations:
```bash
# Option 1: Run all migrations at once
psql -U postgres -d message_board -f migrations/init.sql

# Option 3: If you have sqlx-cli installed:
# cargo install sqlx-cli --no-default-features --features postgres
# sqlx migrate run
```

5. Run the server:
```bash
cargo run
```

## API Endpoints

### Authentication

#### Register
```bash
POST /api/auth/register
Content-Type: application/json

{
  "username": "user123",
  "email": "user@example.com",
  "password": "password123"
}
```

#### Login
```bash
POST /api/auth/login
Content-Type: application/json

{
  "username": "user123",
  "password": "password123"
}
```

### Articles

#### Get All Articles (Public)
```bash
GET /api/articles
```

#### Get Article by ID (Public)
```bash
GET /api/articles/{id}
```

#### Create Article (Auth Required)
```bash
POST /api/articles
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "Article Title",
  "content": "Article content..."
}
```

#### Update Article (Auth Required)
```bash
PUT /api/articles/{id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "Updated Title",
  "content": "Updated content..."
}
```

#### Delete Article (Auth Required)
```bash
DELETE /api/articles/{id}
Authorization: Bearer {token}
```

## Testing with curl

1. Register a new user:
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"password123"}'
```

2. Login:
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password123"}'
```

3. Create an article (replace TOKEN with the JWT token from login):
```bash
curl -X POST http://localhost:3000/api/articles \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"My First Article","content":"This is the content of my article."}'
```

4. Get all articles:
```bash
curl http://localhost:3000/api/articles
```

## Project Structure

```
message_board/
├── src/
│   ├── main.rs           # Application entry point and route configuration
│   ├── db/
│   │   └── mod.rs        # Database connection and pool management
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── auth.rs       # Authentication handlers
│   │   └── article.rs    # Article CRUD handlers
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs       # JWT authentication middleware
│   └── models/
│       ├── mod.rs
│       ├── user.rs       # User models and DTOs
│       └── article.rs    # Article models and DTOs
├── migrations/           # SQL migration files
├── Cargo.toml           # Dependencies
└── .env                 # Environment variables
```