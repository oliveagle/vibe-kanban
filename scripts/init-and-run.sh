#!/bin/bash

set -e

echo 'root:100000:65536' > /etc/subuid
echo 'root:100000:65536' > /etc/subgid

VK_DATA_DIR="/root/.local/share/vibe-kanban"
OPENCODE_CONFIG_FILE="$VK_DATA_DIR/opencode.json"

if [ ! -f "$OPENCODE_CONFIG_FILE" ]; then
  echo "Creating default opencode config..."
  mkdir -p "$VK_DATA_DIR"
  cat > "$OPENCODE_CONFIG_FILE" << 'EOF'
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
fi

if [ ! -L /root/.config/opencode ]; then
  mkdir -p /root/.config
  ln -sf "$VK_DATA_DIR" /root/.config/opencode
fi

cd /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban

echo "Starting VK backend server..."
exec cargo run --release --bin server
