#!/bin/bash

set -e

VK_DATA_DIR="/root/.local/share/vibe-kanban"

# Ensure data directory exists with correct permissions
mkdir -p "$VK_DATA_DIR"
chmod 755 "$VK_DATA_DIR"

# Check if credentials file exists, if not create empty one
touch "$VK_DATA_DIR/credentials.json"
chmod 644 "$VK_DATA_DIR/credentials.json"

echo "Starting VK production server..."
exec cargo run --release --bin server
