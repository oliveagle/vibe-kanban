#!/bin/bash
# Initialize and run VK backend server inside container

set -e

# Configure subuid/subgid for rootless podman
echo "Configuring subuid/subgid..."
echo 'root:100000:65536' > /etc/subuid
echo 'root:100000:65536' > /etc/subgid

# Create opencode config
echo "Creating opencode config..."
mkdir -p /root/.config/opencode
cat > /root/.config/opencode/opencode.json << 'EOF'
{
  "$schema": "https://opencode.ai/config.json",
  "model": "kimi-for-coding",
  "mcp": {
    "vibe_kanban": {
      "type": "local",
      "command": [
        "/usr/local/bin/mcp_task_server"
      ],
      "environment": {
        "VIBE_BACKEND_URL": "http://localhost:3000"
      }
    }
  }
}
EOF

# Navigate to project
cd /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban

# Run server
echo "Starting VK backend server..."
exec cargo run --release --bin server
