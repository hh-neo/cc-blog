# 留言板系统 (Message Board)

一个使用 Rust + MySQL 构建的生产级留言板系统，包含用户认证和文章CRUD功能。

## 功能特性

- 用户注册和登录
- JWT 认证
- 文章的增删改查 (CRUD)
- 分页查询
- 错误处理和日志记录
- CORS 支持
- 生产级配置

## 技术栈

- **后端框架**: Actix-Web 4.9
- **数据库**: MySQL (通过 SQLx)
- **认证**: JWT
- **密码加密**: Bcrypt
- **运行时**: Tokio

## 前置要求

- Rust 1.70+
- MySQL 5.7+ 或 8.0+
- 安装了 cargo

## 快速开始

### 1. 克隆项目

```bash
git clone <repository-url>
cd messageboard
```

### 2. 设置数据库

创建 MySQL 数据库并执行 schema:

```bash
mysql -u root -p < schema.sql
```

或者手动执行：

```sql
CREATE DATABASE IF NOT EXISTS messageboard CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE messageboard;

-- 执行 schema.sql 中的表创建语句
```

### 3. 配置环境变量

复制环境变量示例文件并修改配置：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```env
# 数据库连接
DATABASE_URL=mysql://username:password@localhost:3306/messageboard

# JWT 密钥（生产环境请使用强密钥）
JWT_SECRET=your-secret-key-change-in-production

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# 日志级别
RUST_LOG=info
```

### 4. 编译和运行

开发模式：
```bash
cargo run
```

生产构建：
```bash
cargo build --release
./target/release/messageboard
```

服务器将在 `http://127.0.0.1:8080` 启动

## API 文档

### 认证相关

#### 注册
```http
POST /api/auth/register
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}
```

#### 登录
```http
POST /api/auth/login
Content-Type: application/json

{
  "username": "testuser",
  "password": "password123"
}
```

响应:
```json
{
  "user": {
    "id": "uuid",
    "username": "testuser",
    "email": "test@example.com",
    "created_at": "2024-01-01T00:00:00Z"
  },
  "token": "jwt-token"
}
```

### 用户相关

#### 获取个人信息（需要认证）
```http
GET /api/user/profile
Authorization: Bearer <jwt-token>
```

### 文章相关

#### 获取文章列表
```http
GET /api/articles?page=1&per_page=10&user_id=<optional>
```

#### 获取单个文章
```http
GET /api/articles/{id}
```

#### 创建文章（需要认证）
```http
POST /api/articles
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "title": "文章标题",
  "content": "文章内容"
}
```

#### 更新文章（需要认证，只能更新自己的文章）
```http
PUT /api/articles/{id}
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "title": "新标题（可选）",
  "content": "新内容（可选）"
}
```

#### 删除文章（需要认证，只能删除自己的文章）
```http
DELETE /api/articles/{id}
Authorization: Bearer <jwt-token>
```

### 健康检查
```http
GET /api/health
```

## 项目结构

```
messageboard/
├── src/
│   ├── config/        # 配置模块
│   ├── handlers/      # 请求处理器
│   ├── middleware/    # 中间件（认证等）
│   ├── models/        # 数据模型
│   ├── utils/         # 工具函数
│   └── main.rs        # 程序入口
├── schema.sql         # 数据库 Schema
├── Cargo.toml         # 依赖配置
├── .env.example       # 环境变量示例
└── README.md          # 项目文档
```

## 测试

运行测试（需要先配置测试数据库）：

```bash
cargo test
```

## 生产部署建议

1. **使用强 JWT 密钥**: 生产环境中使用至少 32 字符的随机字符串作为 JWT_SECRET
2. **数据库连接池**: 已配置，可根据负载调整 `max_connections`
3. **HTTPS**: 使用反向代理（如 Nginx）配置 HTTPS
4. **日志**: 配置适当的日志级别和日志收集
5. **监控**: 添加应用监控和性能指标
6. **备份**: 定期备份数据库

## 使用 systemd 部署（Linux）

创建 systemd 服务文件 `/etc/systemd/system/messageboard.service`:

```ini
[Unit]
Description=Message Board Service
After=network.target mysql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/path/to/messageboard
Environment="RUST_LOG=info"
ExecStart=/path/to/messageboard/target/release/messageboard
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable messageboard
sudo systemctl start messageboard
```

## 使用 Docker 部署

创建 `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/messageboard /usr/local/bin/messageboard
CMD ["messageboard"]
```

构建并运行：
```bash
docker build -t messageboard .
docker run -d -p 8080:8080 --env-file .env messageboard
```

## 安全建议

1. 定期更新依赖：`cargo update`
2. 使用安全审计工具：`cargo audit`
3. 限制 CORS 来源（生产环境不要使用 allow_any_origin）
4. 实施速率限制
5. 添加输入验证和清理
6. 使用 HTTPS
7. 定期轮换 JWT 密钥

## License

MIT