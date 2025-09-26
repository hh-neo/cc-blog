#!/bin/bash

# API测试脚本
# 使用前请确保服务器已启动在 http://localhost:8080

API_URL="http://localhost:8080/api"
TOKEN=""

echo "=== 留言板 API 测试 ==="
echo ""

# 健康检查
echo "1. 健康检查..."
curl -s "$API_URL/health"
echo -e "\n"

# 注册用户
echo "2. 注册新用户..."
REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser'$(date +%s)'",
    "email": "test'$(date +%s)'@example.com",
    "password": "password123"
  }')

echo "$REGISTER_RESPONSE" | jq .
TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r .token)
echo "Token: $TOKEN"
echo ""

# 登录
echo "3. 用户登录..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser'$(date +%s)'",
    "password": "password123"
  }')

echo "$LOGIN_RESPONSE" | jq .
echo ""

# 获取个人信息
echo "4. 获取个人信息..."
curl -s -X GET "$API_URL/user/profile" \
  -H "Authorization: Bearer $TOKEN" | jq .
echo ""

# 创建文章
echo "5. 创建文章..."
CREATE_RESPONSE=$(curl -s -X POST "$API_URL/articles" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "测试文章",
    "content": "这是一篇测试文章的内容。"
  }')

echo "$CREATE_RESPONSE" | jq .
ARTICLE_ID=$(echo "$CREATE_RESPONSE" | jq -r .id)
echo "Article ID: $ARTICLE_ID"
echo ""

# 获取文章列表
echo "6. 获取文章列表..."
curl -s -X GET "$API_URL/articles?page=1&per_page=10" | jq .
echo ""

# 获取单个文章
echo "7. 获取单个文章..."
curl -s -X GET "$API_URL/articles/$ARTICLE_ID" | jq .
echo ""

# 更新文章
echo "8. 更新文章..."
curl -s -X PUT "$API_URL/articles/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "更新后的标题",
    "content": "更新后的内容"
  }' | jq .
echo ""

# 删除文章
echo "9. 删除文章..."
curl -s -X DELETE "$API_URL/articles/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN"
echo "文章已删除"
echo ""

echo "=== 测试完成 ==="