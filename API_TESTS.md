# API 测试脚本

这里提供了一些测试 API 的示例脚本，你可以直接复制粘贴到终端运行。

## 1. 用户注册

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123"
  }' | jq .
```

## 2. 用户登录

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "password123"
  }' | jq .
```

**注意**: 从登录响应中复制 `token` 值，用于后续需要认证的请求。

## 3. 获取留言列表

```bash
# 获取所有留言
curl http://localhost:3000/api/messages | jq .

# 分页查询
curl "http://localhost:3000/api/messages?page=1&per_page=5" | jq .
```

## 4. 创建留言（需要认证）

```bash
# 请将 YOUR_JWT_TOKEN 替换为实际的 JWT token
curl -X POST http://localhost:3000/api/messages \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "content": "这是我的第一条留言！"
  }' | jq .
```

## 5. 获取特定留言

```bash
# 请将 MESSAGE_ID 替换为实际的留言 UUID
curl http://localhost:3000/api/messages/MESSAGE_ID | jq .
```

## 6. 更新留言（需要认证，只能更新自己的留言）

```bash
curl -X PUT http://localhost:3000/api/messages/MESSAGE_ID \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "content": "这是更新后的留言内容"
  }' | jq .
```

## 7. 删除留言（需要认证，只能删除自己的留言）

```bash
curl -X DELETE http://localhost:3000/api/messages/MESSAGE_ID \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 完整测试流程脚本

```bash
#!/bin/bash

echo "=== 测试留言板 API ==="

# 1. 注册用户
echo "1. 注册用户..."
REGISTER_RESPONSE=$(curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123"
  }')

echo $REGISTER_RESPONSE | jq .

# 提取 token
TOKEN=$(echo $REGISTER_RESPONSE | jq -r '.token')
echo "Token: $TOKEN"

# 2. 创建留言
echo "2. 创建留言..."
MESSAGE_RESPONSE=$(curl -s -X POST http://localhost:3000/api/messages \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "content": "这是测试留言！"
  }')

echo $MESSAGE_RESPONSE | jq .

# 提取留言 ID
MESSAGE_ID=$(echo $MESSAGE_RESPONSE | jq -r '.id')
echo "Message ID: $MESSAGE_ID"

# 3. 获取所有留言
echo "3. 获取留言列表..."
curl -s http://localhost:3000/api/messages | jq .

# 4. 获取特定留言
echo "4. 获取特定留言..."
curl -s http://localhost:3000/api/messages/$MESSAGE_ID | jq .

# 5. 更新留言
echo "5. 更新留言..."
curl -s -X PUT http://localhost:3000/api/messages/$MESSAGE_ID \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "content": "这是更新后的留言内容！"
  }' | jq .

echo "=== 测试完成 ==="
```

将上述脚本保存为 `test_api.sh` 并运行：

```bash
chmod +x test_api.sh
./test_api.sh
```