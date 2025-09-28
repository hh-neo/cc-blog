# Blog API - Rust + Axum + MySQL

一个功能完整的博客/留言板 REST API，使用 Rust、Axum 框架和 MySQL 数据库构建，适合生产环境部署。

## 功能特性

- ✅ 用户注册和登录
- ✅ JWT 身份认证
- ✅ 文章的完整 CRUD 操作（增删改查）
- ✅ 密码加密存储（bcrypt）
- ✅ 输入验证
- ✅ CORS 支持
- ✅ 完整的单元测试和集成测试
- ✅ 生产环境就绪

## 技术栈

- **框架**: Axum 0.7
- **数据库**: MySQL (通过 sqlx)
- **认证**: JWT (jsonwebtoken)
- **密码加密**: bcrypt
- **异步运行时**: Tokio
- **日志**: tracing

## API 端点

### 公开端点（无需认证）

- `POST /register` - 用户注册
- `POST /login` - 用户登录
- `GET /posts` - 获取所有文章
- `GET /posts/:id` - 获取单个文章

### 受保护端点（需要 JWT token）

- `POST /posts` - 创建新文章
- `PUT /posts/:id` - 更新文章（仅作者）
- `DELETE /posts/:id` - 删除文章（仅作者）

## 快速开始

### 前置要求

- Rust 1.70+
- MySQL 5.7+ 或 8.0+
- Cargo

### 1. 克隆项目

```bash
git clone <repository-url>
cd cc-blog
```

### 2. 配置环境变量

复制 `.env.example` 到 `.env` 并修改配置：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```env
DATABASE_URL=mysql://your_username:your_password@localhost:3306/blog_db
JWT_SECRET=your_super_secret_jwt_key_min_32_chars
SERVER_ADDR=0.0.0.0:3000
```

### 3. 初始化数据库

```bash
mysql -u root -p < init.sql
```

或者手动执行：

```sql
CREATE DATABASE IF NOT EXISTS blog_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE blog_db;
-- 然后执行 init.sql 中的表创建语句
```

### 4. 运行项目

```bash
cargo run --release
```

服务器将在 `http://0.0.0.0:3000` 启动。

### 5. 运行测试

```bash
cargo test
```

## API 使用示例

### 注册用户

```bash
curl -X POST http://localhost:3000/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123"
  }'
```

响应：

```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "user": {
    "id": 1,
    "username": "testuser",
    "email": "test@example.com"
  }
}
```

### 登录

```bash
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "password123"
  }'
```

### 创建文章（需要认证）

```bash
curl -X POST http://localhost:3000/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "title": "我的第一篇文章",
    "content": "这是文章内容..."
  }'
```

### 获取所有文章

```bash
curl http://localhost:3000/posts
```

### 更新文章（需要认证，仅作者）

```bash
curl -X PUT http://localhost:3000/posts/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "title": "更新后的标题",
    "content": "更新后的内容"
  }'
```

### 删除文章（需要认证，仅作者）

```bash
curl -X DELETE http://localhost:3000/posts/1 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 生产环境部署

### Docker 部署（推荐）

1. 创建 Dockerfile：

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/blog-api /usr/local/bin/blog-api
EXPOSE 3000
CMD ["blog-api"]
```

2. 构建并运行：

```bash
docker build -t blog-api .
docker run -d -p 3000:3000 --env-file .env blog-api
```

### 传统部署

1. 编译发布版本：

```bash
cargo build --release
```

2. 复制可执行文件到服务器：

```bash
scp target/release/blog-api user@server:/opt/blog-api/
```

3. 在服务器上配置 systemd 服务：

创建 `/etc/systemd/system/blog-api.service`：

```ini
[Unit]
Description=Blog API Service
After=network.target mysql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/blog-api
EnvironmentFile=/opt/blog-api/.env
ExecStart=/opt/blog-api/blog-api
Restart=always

[Install]
WantedBy=multi-user.target
```

4. 启动服务：

```bash
sudo systemctl enable blog-api
sudo systemctl start blog-api
sudo systemctl status blog-api
```

### 使用 Nginx 反向代理

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 安全建议

1. **JWT Secret**: 使用至少 32 个字符的随机字符串
2. **HTTPS**: 生产环境务必使用 HTTPS
3. **密码策略**: 建议实施更强的密码策略
4. **Rate Limiting**: 添加速率限制防止滥用
5. **输入验证**: 已包含基本验证，可根据需要加强
6. **数据库**: 使用专用数据库用户，最小权限原则
7. **日志**: 监控日志，及时发现异常

## 项目结构

```
cc-blog/
├── src/
│   ├── main.rs              # 主程序入口
│   ├── lib.rs               # 库入口
│   ├── db.rs                # 数据库连接
│   ├── models.rs            # 数据模型
│   ├── auth.rs              # JWT 认证中间件
│   └── handlers/
│       ├── mod.rs           # handlers 模块
│       ├── user_handler.rs  # 用户相关接口
│       └── post_handler.rs  # 文章相关接口
├── tests/
│   └── api_tests.rs         # API 集成测试
├── init.sql                 # 数据库初始化脚本
├── Cargo.toml               # 项目配置
├── .env.example             # 环境变量示例
└── README.md                # 项目说明
```

## 测试

运行所有测试：

```bash
cargo test
```

运行特定测试：

```bash
cargo test test_register
cargo test test_create_post
```

## 性能优化

- 使用 `--release` 模式编译以获得最佳性能
- 配置数据库连接池大小（默认 10）
- 考虑添加 Redis 进行会话缓存
- 使用 CDN 加速静态资源

## 故障排查

### 数据库连接失败

检查：
- MySQL 服务是否运行
- DATABASE_URL 是否正确
- 用户权限是否足够

### JWT 验证失败

检查：
- JWT_SECRET 是否一致
- token 是否过期（默认 24 小时）

## 许可证

MIT

## 贡献

欢迎提交 Issue 和 Pull Request！

## 联系方式

如有问题，请提交 Issue。