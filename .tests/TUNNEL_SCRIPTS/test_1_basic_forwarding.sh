#!/bin/bash
# Test 1: Basic HTTP Forwarding (GET request)

echo "=== Test 1: Basic HTTP Forwarding (GET request) ==="

# Test GET request through tunnel
response=$(curl -s -w "\n%{http_code}" http://localhost:8080/)
body=$(echo "$response" | head -n -1)
status=$(echo "$response" | tail -n 1)

if [ "$status" = "200" ]; then
    echo "✅ PASS: GET request forwarded successfully (status: $status)"
    exit 0
else
    echo "❌ FAIL: Expected status 200, got $status"
    echo "Response body: $body"
    exit 1
fi
