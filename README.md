# 安全留言板系统

基于 Rust + MySQL 构建的高性能、安全的留言板系统，支持用户认证、文章管理和评论功能。

## 功能特性

- 用户注册和登录（JWT 认证）
- 文章 CRUD 操作
- 评论系统
- 全文搜索
- 登录日志审计
- 速率限制
- CORS 保护
- SQL 注入防护
- XSS 防护
- HTTPS 支持

## 技术栈

- **后端**: Rust (Actix-web)
- **数据库**: MySQL 8.0
- **认证**: JWT
- **部署**: Docker, Nginx
- **安全**: bcrypt, TLS 1.2/1.3

## 快速开始

### 前置要求

- Docker 和 Docker Compose
- 或 Rust 1.76+ 和 MySQL 8.0+

### 使用 Docker 部署（推荐）

1. 克隆项目并配置环境变量：
```bash
cp .env.example .env
# 编辑 .env 文件，修改密码和密钥
```

2. 生成 SSL 证书（生产环境）：
```bash
mkdir ssl
# 将你的 SSL 证书放入 ssl 目录
# cert.pem 和 key.pem
```

3. 启动服务：
```bash
docker-compose up -d
```

4. 初始化数据库（首次运行）：
```bash
docker exec -i message-board-db mysql -uroot -p < schema.sql
```

### 本地开发

1. 安装依赖：
```bash
cargo build
```

2. 设置环境变量：
```bash
cp .env.example .env
source .env
```

3. 初始化数据库：
```bash
mysql -u root -p < schema.sql
```

4. 运行应用：
```bash
cargo run
```

## API 文档

### 认证

#### 注册
```
POST /api/auth/register
Content-Type: application/json

{
  "username": "testuser",
  "email": "test@example.com",
  "password": "Password123!"
}
```

#### 登录
```
POST /api/auth/login
Content-Type: application/json

{
  "username_or_email": "testuser",
  "password": "Password123!"
}
```

### 文章管理

#### 创建文章
```
POST /api/articles
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "文章标题",
  "content": "文章内容",
  "is_published": true
}
```

#### 获取文章列表
```
GET /api/articles?page=1&per_page=10
```

#### 获取单篇文章
```
GET /api/articles/{article_id}
```

#### 更新文章
```
PUT /api/articles/{article_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "新标题",
  "content": "新内容"
}
```

#### 删除文章
```
DELETE /api/articles/{article_id}
Authorization: Bearer <token>
```

#### 搜索文章
```
GET /api/articles/search?q=关键词
```

### 评论

#### 添加评论
```
POST /api/articles/{article_id}/comments
Authorization: Bearer <token>
Content-Type: application/json

{
  "content": "评论内容"
}
```

#### 获取评论
```
GET /api/articles/{article_id}/comments
```

## 测试

运行 API 测试脚本：
```bash
chmod +x test_api.sh
./test_api.sh
```

## 安全特性

1. **密码安全**: 使用 bcrypt 哈希存储
2. **SQL 注入防护**: 使用参数化查询
3. **XSS 防护**: 输入验证和转义
4. **CSRF 防护**: JWT token 验证
5. **速率限制**: 防止暴力破解
6. **HTTPS**: 强制 TLS 加密
7. **审计日志**: 记录所有登录尝试
8. **最小权限**: Docker 容器以非 root 用户运行

## 生产部署建议

1. **环境变量**:
   - 修改 `JWT_SECRET` 为强随机密钥
   - 使用强数据库密码
   - 启用 HTTPS

2. **数据库**:
   - 定期备份
   - 启用 binlog
   - 配置主从复制

3. **监控**:
   - 配置日志收集（ELK/Prometheus）
   - 设置告警
   - 监控 API 响应时间

4. **安全加固**:
   - 使用 WAF
   - 配置 fail2ban
   - 定期安全审计
   - 及时更新依赖

## 性能优化

- 数据库连接池
- 异步处理
- 索引优化
- 缓存策略（可添加 Redis）
- CDN 加速（静态资源）

## 故障排查

查看日志：
```bash
# 应用日志
docker logs message-board-app

# 数据库日志
docker logs message-board-db

# Nginx 日志
docker logs message-board-nginx
```

## 许可

MIT License