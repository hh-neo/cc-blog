# Message Board API

一个使用 Rust + MySQL 构建的留言板应用，提供用户注册登录和文章管理功能。

## 功能特性

- 用户注册与登录
- JWT 认证
- 文章的增删改查
- 密码加密存储
- RESTful API 设计
- 完整的单元测试和集成测试

## 技术栈

- **Rust** - 系统编程语言
- **Actix-Web** - Web 框架
- **SQLx** - 异步 SQL 工具包
- **MySQL** - 数据库
- **JWT** - 身份认证
- **Bcrypt** - 密码加密

## 项目结构

```
.
├── src/
│   ├── main.rs           # 应用入口
│   ├── lib.rs            # 库入口
│   ├── models.rs         # 数据模型
│   ├── db.rs             # 数据库连接
│   ├── auth.rs           # JWT 认证
│   └── handlers/         # API 处理器
│       ├── mod.rs
│       ├── user_handler.rs
│       └── post_handler.rs
├── tests/                # 集成测试
├── init.sql              # 数据库初始化脚本
└── .env.example          # 环境变量示例
```

## 快速开始

### 1. 前置要求

- Rust 1.70+
- MySQL 8.0+
- Cargo

### 2. 安装数据库

确保 MySQL 已安装并运行，然后执行初始化脚本：

```bash
mysql -u root -p < init.sql
```

### 3. 配置环境变量

复制 `.env.example` 到 `.env` 并修改配置：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```env
DATABASE_URL=mysql://root:your_password@localhost:3306/message_board
JWT_SECRET=your_secret_key_here_change_in_production
RUST_LOG=info
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

### 4. 运行项目

```bash
# 开发模式
cargo run

# 发布模式
cargo run --release
```

服务器将在 `http://127.0.0.1:8080` 启动。

### 5. 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test api_tests
```

## API 文档

### 认证相关

#### 注册用户

```http
POST /api/auth/register
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}
```

响应：

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": 1,
    "username": "testuser",
    "email": "test@example.com",
    "created_at": "2025-01-01T00:00:00Z"
  }
}
```

#### 用户登录

```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "testuser",
  "password": "password123"
}
```

响应格式同注册。

### 文章相关

#### 获取所有文章（无需认证）

```http
GET /api/posts
```

响应：

```json
[
  {
    "id": 1,
    "user_id": 1,
    "username": "testuser",
    "title": "文章标题",
    "content": "文章内容",
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T00:00:00Z"
  }
]
```

#### 获取单篇文章（无需认证）

```http
GET /api/posts/{id}
```

#### 创建文章（需要认证）

```http
POST /api/posts
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "文章标题",
  "content": "文章内容"
}
```

#### 更新文章（需要认证，仅作者）

```http
PUT /api/posts/{id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "新标题",
  "content": "新内容"
}
```

#### 删除文章（需要认证，仅作者）

```http
DELETE /api/posts/{id}
Authorization: Bearer {token}
```

## 数据库架构

### users 表

| 字段          | 类型         | 说明           |
| ------------- | ------------ | -------------- |
| id            | INT          | 主键，自增     |
| username      | VARCHAR(50)  | 用户名，唯一   |
| email         | VARCHAR(100) | 邮箱，唯一     |
| password_hash | VARCHAR(255) | 密码哈希       |
| created_at    | TIMESTAMP    | 创建时间       |
| updated_at    | TIMESTAMP    | 更新时间       |

### posts 表

| 字段       | 类型         | 说明         |
| ---------- | ------------ | ------------ |
| id         | INT          | 主键，自增   |
| user_id    | INT          | 外键，用户ID |
| title      | VARCHAR(200) | 文章标题     |
| content    | TEXT         | 文章内容     |
| created_at | TIMESTAMP    | 创建时间     |
| updated_at | TIMESTAMP    | 更新时间     |

## 安全性

- 密码使用 bcrypt 加密，默认 cost 为 12
- JWT token 有效期为 24 小时
- 文章的更新和删除仅限作者本人
- 使用参数化查询防止 SQL 注入
- 请在生产环境中修改 `JWT_SECRET`

## 测试

项目包含完整的测试覆盖：

- **单元测试**: 测试模型序列化、JWT 创建和验证
- **集成测试**: 测试完整的 API 流程

运行测试前请确保数据库已正确配置。

## 开发建议

1. 在生产环境中使用强密码和安全的 JWT_SECRET
2. 考虑添加请求频率限制
3. 可以添加日志记录和监控
4. 考虑使用 HTTPS
5. 定期备份数据库

## 许可证

MIT License