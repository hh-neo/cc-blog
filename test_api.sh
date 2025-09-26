#!/bin/bash

BASE_URL="${BASE_URL:-http://localhost:8080/api}"
USERNAME="testuser_$(date +%s)"
EMAIL="${USERNAME}@example.com"
PASSWORD="TestPassword123!"

echo "=== Message Board API Test Script ==="
echo "Testing against: $BASE_URL"
echo ""

echo "1. Testing user registration..."
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")

if echo "$REGISTER_RESPONSE" | grep -q "token"; then
  echo "✓ Registration successful"
  TOKEN=$(echo "$REGISTER_RESPONSE" | grep -o '"token":"[^"]*' | sed 's/"token":"//')
  USER_ID=$(echo "$REGISTER_RESPONSE" | grep -o '"id":"[^"]*' | sed 's/"id":"//')
  echo "  Token: ${TOKEN:0:20}..."
else
  echo "✗ Registration failed: $REGISTER_RESPONSE"
  exit 1
fi

echo ""
echo "2. Testing user login..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username_or_email\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

if echo "$LOGIN_RESPONSE" | grep -q "token"; then
  echo "✓ Login successful"
else
  echo "✗ Login failed: $LOGIN_RESPONSE"
  exit 1
fi

echo ""
echo "3. Testing get user profile..."
PROFILE_RESPONSE=$(curl -s -X GET "$BASE_URL/users/profile" \
  -H "Authorization: Bearer $TOKEN")

if echo "$PROFILE_RESPONSE" | grep -q "$USERNAME"; then
  echo "✓ Profile retrieved successfully"
else
  echo "✗ Profile retrieval failed: $PROFILE_RESPONSE"
fi

echo ""
echo "4. Testing create article..."
ARTICLE_TITLE="Test Article $(date +%s)"
ARTICLE_RESPONSE=$(curl -s -X POST "$BASE_URL/articles" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"title\":\"$ARTICLE_TITLE\",\"content\":\"This is a test article content.\",\"is_published\":true}")

if echo "$ARTICLE_RESPONSE" | grep -q "id"; then
  echo "✓ Article created successfully"
  ARTICLE_ID=$(echo "$ARTICLE_RESPONSE" | grep -o '"id":"[^"]*' | sed 's/"id":"//')
  echo "  Article ID: $ARTICLE_ID"
else
  echo "✗ Article creation failed: $ARTICLE_RESPONSE"
  exit 1
fi

echo ""
echo "5. Testing get all articles..."
ARTICLES_RESPONSE=$(curl -s -X GET "$BASE_URL/articles")

if echo "$ARTICLES_RESPONSE" | grep -q "data"; then
  echo "✓ Articles retrieved successfully"
else
  echo "✗ Articles retrieval failed: $ARTICLES_RESPONSE"
fi

echo ""
echo "6. Testing get single article..."
SINGLE_ARTICLE_RESPONSE=$(curl -s -X GET "$BASE_URL/articles/$ARTICLE_ID")

if echo "$SINGLE_ARTICLE_RESPONSE" | grep -q "$ARTICLE_TITLE"; then
  echo "✓ Single article retrieved successfully"
else
  echo "✗ Single article retrieval failed: $SINGLE_ARTICLE_RESPONSE"
fi

echo ""
echo "7. Testing update article..."
UPDATED_TITLE="Updated $ARTICLE_TITLE"
UPDATE_RESPONSE=$(curl -s -X PUT "$BASE_URL/articles/$ARTICLE_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"title\":\"$UPDATED_TITLE\"}")

if echo "$UPDATE_RESPONSE" | grep -q "$UPDATED_TITLE"; then
  echo "✓ Article updated successfully"
else
  echo "✗ Article update failed: $UPDATE_RESPONSE"
fi

echo ""
echo "8. Testing add comment..."
COMMENT_RESPONSE=$(curl -s -X POST "$BASE_URL/articles/$ARTICLE_ID/comments" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"content\":\"This is a test comment.\"}")

if echo "$COMMENT_RESPONSE" | grep -q "id"; then
  echo "✓ Comment added successfully"
else
  echo "✗ Comment addition failed: $COMMENT_RESPONSE"
fi

echo ""
echo "9. Testing get comments..."
COMMENTS_RESPONSE=$(curl -s -X GET "$BASE_URL/articles/$ARTICLE_ID/comments")

if echo "$COMMENTS_RESPONSE" | grep -q "test comment"; then
  echo "✓ Comments retrieved successfully"
else
  echo "✗ Comments retrieval failed: $COMMENTS_RESPONSE"
fi

echo ""
echo "10. Testing search articles..."
SEARCH_RESPONSE=$(curl -s -X GET "$BASE_URL/articles/search?q=test")

if echo "$SEARCH_RESPONSE" | grep -q "\["; then
  echo "✓ Search completed successfully"
else
  echo "✗ Search failed: $SEARCH_RESPONSE"
fi

echo ""
echo "11. Testing delete article..."
DELETE_RESPONSE=$(curl -s -X DELETE "$BASE_URL/articles/$ARTICLE_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -o /dev/null -w "%{http_code}")

if [ "$DELETE_RESPONSE" = "204" ]; then
  echo "✓ Article deleted successfully"
else
  echo "✗ Article deletion failed: HTTP $DELETE_RESPONSE"
fi

echo ""
echo "12. Testing unauthorized access..."
UNAUTH_RESPONSE=$(curl -s -X GET "$BASE_URL/users/profile" \
  -o /dev/null -w "%{http_code}")

if [ "$UNAUTH_RESPONSE" = "403" ] || [ "$UNAUTH_RESPONSE" = "401" ]; then
  echo "✓ Unauthorized access properly blocked"
else
  echo "✗ Unauthorized access not properly blocked: HTTP $UNAUTH_RESPONSE"
fi

echo ""
echo "=== All tests completed ==="