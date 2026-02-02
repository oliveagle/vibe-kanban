#!/bin/bash
# Test script for local authentication

echo "Testing local authentication..."
echo ""

BASE_URL="http://localhost:37826"

# Test 1: Check local auth status
echo "1. Checking local auth status..."
curl -s "$BASE_URL/api/auth/local/status" | jq .
echo ""

# Test 2: Login with default credentials
echo "2. Testing login with default credentials (admin/admin)..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/auth/local/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin"}')
echo "$LOGIN_RESPONSE" | jq .
echo ""

# Extract token if login successful
TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.data.access_token // empty')

if [ -n "$TOKEN" ]; then
  echo "3. Testing /api/auth/user with token..."
  curl -s "$BASE_URL/api/auth/user" \
    -H "Authorization: Bearer $TOKEN" | jq .
  echo ""

  echo "4. Testing /api/auth/status with token..."
  curl -s "$BASE_URL/api/auth/status" \
    -H "Authorization: Bearer $TOKEN" | jq .
  echo ""
fi

echo "Done!"
