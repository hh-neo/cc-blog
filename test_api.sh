#!/bin/bash

# Test script for Message Board API
# Make sure the server is running on localhost:3000

API_URL="http://localhost:3000"

echo "Testing Message Board API..."
echo ""

# Test health check
echo "1. Testing health check..."
curl -s "$API_URL/health"
echo -e "\n"

# Test user registration
echo "2. Testing user registration..."
REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser'$(date +%s)'",
    "email": "test'$(date +%s)'@example.com",
    "password": "password123"
  }')
echo "$REGISTER_RESPONSE" | jq '.'
TOKEN=$(echo "$REGISTER_RESPONSE" | jq -r '.token')
echo ""

# Test login
echo "3. Testing login..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser'$(date +%s)'",
    "password": "password123"
  }')
echo "$LOGIN_RESPONSE" | jq '.'
echo ""

# Create an article
echo "4. Creating an article..."
ARTICLE_RESPONSE=$(curl -s -X POST "$API_URL/api/articles" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test Article",
    "content": "This is a test article content."
  }')
echo "$ARTICLE_RESPONSE" | jq '.'
ARTICLE_ID=$(echo "$ARTICLE_RESPONSE" | jq -r '.id')
echo ""

# Get all articles
echo "5. Getting all articles..."
curl -s "$API_URL/api/articles" | jq '.'
echo ""

# Get specific article
echo "6. Getting specific article..."
curl -s "$API_URL/api/articles/$ARTICLE_ID" | jq '.'
echo ""

# Update article
echo "7. Updating article..."
curl -s -X PUT "$API_URL/api/articles/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Updated Test Article",
    "content": "This is the updated content."
  }' | jq '.'
echo ""

# Delete article
echo "8. Deleting article..."
curl -s -X DELETE "$API_URL/api/articles/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN"
echo "Article deleted"
echo ""

echo "API testing completed!"