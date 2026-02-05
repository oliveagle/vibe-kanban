#!/bin/bash
# Install coding agents in the running container
# This is a background task that can be triggered manually

set -e

CONTAINER_NAME="vibe-kanban-backend-dev"

if ! podman ps | grep -q "$CONTAINER_NAME"; then
    echo "❌ Container $CONTAINER_NAME is not running"
    echo "Start it first with: just dev-srv"
    exit 1
fi

echo "Installing coding agents in container..."
echo "This may take a few minutes..."

podman exec "$CONTAINER_NAME" sh -c '
    mkdir -p /opt/agents
    cd /opt/agents
    
    # Check if already installed
    if [ -d "node_modules/@anthropic-ai/claude-code" ] && \
       [ -d "node_modules/@openai/codex" ] && \
       [ -d "node_modules/@google/gemini-cli" ]; then
        echo "✅ Agents already installed"
        exit 0
    fi
    
    npm init -y
    npm install @anthropic-ai/claude-code @openai/codex @google/gemini-cli
    echo "✅ Agents installed successfully"
'

echo "✅ Installation complete"
