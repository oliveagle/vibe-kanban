#!/bin/bash
# VK Backend Container Startup Script
# Usage: ./start-vk-backend.sh [-d|--daemon]

set -e

# Parse arguments
DAEMON_MODE=false
if [[ "$1" == "-d" || "$1" == "--daemon" ]]; then
    DAEMON_MODE=true
fi

# Container configuration
CONTAINER_NAME="vibe-kanban-backend-dev"
IMAGE="localhost/vibe-kanban:dev-runtime-v0.0.147"
PORT="3000"

# Stop existing container if running
echo "Stopping existing container..."
podman stop "$CONTAINER_NAME" 2>/dev/null || true
podman rm "$CONTAINER_NAME" 2>/dev/null || true

# Start container
echo "Starting VK backend container..."

if [ "$DAEMON_MODE" = true ]; then
    echo "Running in daemon mode..."
    podman run -d \
        --name "$CONTAINER_NAME" \
        --privileged \
        -p "${PORT}:3000" \
        -e HOST=0.0.0.0 \
        -e PORT=3000 \
        -e VIBE_BACKEND_URL="http://localhost:${PORT}" \
        -e RUST_LOG=debug \
        -e DISABLE_WORKTREE_ORPHAN_CLEANUP=1 \
        -v /mnt/volume3/data:/mnt/volume3/data:rw \
        -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
        "$IMAGE" \
        /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/scripts/init-and-run.sh
    
    echo "✅ Container started in background"
    echo "View logs: podman logs -f $CONTAINER_NAME"
    echo "API: http://localhost:${PORT}"
else
    echo "Running in interactive mode (Ctrl+C to stop)..."
    podman run -ti \
        --name "$CONTAINER_NAME" \
        --privileged \
        -p "${PORT}:3000" \
        -e HOST=0.0.0.0 \
        -e PORT=3000 \
        -e VIBE_BACKEND_URL="http://localhost:${PORT}" \
        -e RUST_LOG=debug \
        -e DISABLE_WORKTREE_ORPHAN_CLEANUP=1 \
        -v /mnt/volume3/data:/mnt/volume3/data:rw \
        -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
        "$IMAGE" \
        /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/scripts/init-and-run.sh
fi
