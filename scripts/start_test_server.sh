#!/bin/bash

# Start the test server for integration tests
# This script is used in CI/CD to start the server before running integration tests

set -e

echo "Starting test server..."

# Set environment variables for test server
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/rustok_test"
export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/rustok_test"
export TEST_SERVER_URL="http://localhost:3000"
export TEST_AUTH_TOKEN="test_token"
export TEST_TENANT_ID="test-tenant"
export TEST_USER_ID="00000000-0000-0000-0000-000000000000"

# Navigate to server directory
cd /home/engine/project/apps/server

# Build the server
echo "Building server..."
cargo build --release

# Start the server in the background
echo "Starting server in background..."
cargo run --release &

# Store the PID
SERVER_PID=$!

# Wait for server to be ready
echo "Waiting for server to be ready..."
for i in {1..30}; do
  if curl -s http://localhost:3000/health > /dev/null; then
    echo "Server is ready!"
    break
  fi
  if [ $i -eq 30 ]; then
    echo "Server failed to start"
    exit 1
  fi
  sleep 1
done

# Export the PID so it can be killed later
echo $SERVER_PID > /tmp/test_server.pid

echo "Test server started successfully on port 3000"
