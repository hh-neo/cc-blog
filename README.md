# Gold 留言板项目

基于 Rust Axum 框架构建的现代化留言板应用，支持 JWT 认证、MySQL 数据库和完整的 CRUD 操作。

## ✅ 项目状态

**当前状态**: 项目已完成，可以正常编译运行！

- ✅ 编译通过
- ✅ 所有功能实现完成
- ✅ 提供启动脚本
- ✅ 包含完整 API 测试

## 功能特性

- 🔐 用户注册和登录（JWT 认证）
- 📝 留言的增删改查操作
- 🔒 基于 JWT 的 API 保护
- 📊 分页和查询功能
- 🚀 高性能 Axum Web 框架
- 🛡️ 完善的错误处理和验证

## 技术栈

- **后端框架**: Axum 0.7
- **数据库**: MySQL 8.0
- **认证**: JWT (jsonwebtoken)
- **密码加密**: bcrypt
- **数据库层**: SQLx
- **序列化**: Serde
- **验证**: Validator
- **日志**: Tracing

## 快速启动

### 方式1: 使用启动脚本（推荐）

```bash
# 如果已安装 MySQL
./start.sh

# 使用 Docker（推荐新用户）
./start-docker.sh
```

### 方式2: 手动启动

1. **启动 MySQL**:
   ```bash
   # 使用 Homebrew
   brew services start mysql

   # 或使用 Docker
   docker run --name mysql-gold -e MYSQL_ROOT_PASSWORD=password -p 3306:3306 -d mysql:8.0
   ```

2. **创建数据库**:
   ```sql
   CREATE DATABASE gold_messageboard;
   ```

3. **运行数据库迁移**:
   ```bash
   mysql -u root -p gold_messageboard < migrations/001_create_users.sql
   mysql -u root -p gold_messageboard < migrations/002_create_messages.sql
   ```

4. **启动项目**:
   ```bash
   cargo run
   ```

服务器将在 `http://127.0.0.1:3000` 启动。

## API 接口

### 认证相关
- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录

### 留言相关
- `GET /api/messages` - 获取留言列表（支持分页和用户筛选）
- `GET /api/messages/:id` - 获取单条留言
- `POST /api/messages` - 创建留言（需要认证）
- `PUT /api/messages/:id` - 更新留言（需要认证）
- `DELETE /api/messages/:id` - 删除留言（需要认证）

## API 测试

查看 [`API_TESTS.md`](./API_TESTS.md) 文件获取详细的 API 测试示例和脚本。

### 快速测试

```bash
# 1. 用户注册
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "email": "test@example.com", "password": "password123"}'

# 2. 用户登录
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "password123"}'

# 3. 获取留言列表
curl http://localhost:3000/api/messages
```

## 项目结构

```
src/
├── auth/           # JWT 认证相关
├── config/         # 配置管理
├── database/       # 数据库连接和仓库层
├── errors/         # 错误处理
├── handlers/       # HTTP 请求处理器
├── middleware/     # 中间件
├── models/         # 数据模型
├── routes.rs       # 路由配置
└── main.rs         # 应用入口

migrations/         # 数据库迁移脚本
scripts/           # 启动脚本
.env               # 环境变量配置
```

## 环境变量配置

创建或修改 `.env` 文件：

```env
# 数据库配置
DATABASE_URL=mysql://root:password@localhost:3306/gold_messageboard

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# JWT密钥 (生产环境请更换为随机生成的强密钥)
JWT_SECRET=your-secret-key-change-this-in-production-env

# 日志级别
RUST_LOG=info
```

## 安全特性

- 密码使用 bcrypt 加密存储
- JWT Token 用于 API 认证
- CORS 跨域支持
- 输入验证和错误处理
- SQL 注入防护（SQLx 参数化查询）
- 用户只能修改/删除自己的留言

## 开发和调试

```bash
# 检查代码
cargo check

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 部署建议

1. **生产环境配置**：
   - 更改 JWT_SECRET 为强随机密钥
   - 使用 HTTPS
   - 配置数据库连接池
   - 启用日志记录

2. **性能优化**：
   - 配置数据库索引
   - 使用 Redis 缓存
   - 启用 gzip 压缩

3. **监控和日志**：
   - 配置结构化日志
   - 添加指标监控
   - 错误追踪

## 故障排除

### 编译错误
- 确保使用 Rust 2021 edition 或更高版本
- 运行 `cargo clean` 然后 `cargo build`

### 数据库连接错误
- 检查 MySQL 服务是否运行
- 验证 `.env` 文件中的数据库连接信息
- 确保数据库 `gold_messageboard` 已创建

### 端口占用
- 修改 `.env` 中的 `SERVER_PORT` 为其他端口
- 或使用 `lsof -ti:3000 | xargs kill` 杀死占用进程

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License