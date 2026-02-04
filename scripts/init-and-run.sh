#!/bin/bash

set -e

echo 'root:100000:65536' > /etc/subuid
echo 'root:100000:65536' > /etc/subgid

VK_DATA_DIR="/root/.local/share/vibe-kanban"
mkdir -p "$VK_DATA_DIR"

# Create MCP config for opencode
if [ ! -f "$VK_DATA_DIR/opencode.json" ]; then
  echo "Creating default opencode MCP config..."
  cat > "$VK_DATA_DIR/opencode.json" << 'EOF'
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

# Fix opencode CLI config - the desktop app creates incompatible config
# We need to ensure CLI gets an empty config, not the desktop config
if [ -L /root/.config/opencode ]; then
  # It's already a symlink, check if target has desktop config
  if grep -q '"config_version"' /root/.config/opencode/config.json 2>/dev/null; then
    echo "Removing desktop opencode config..."
    rm -f /root/.config/opencode/config.json
    echo '{}' > /root/.config/opencode/config.json
    chmod 444 /root/.config/opencode/config.json
  fi
elif [ -d /root/.config/opencode ]; then
  # It's a directory (desktop config), replace with fixed version
  echo "Fixing opencode CLI config..."
  rm -rf /root/.config/opencode
  mkdir -p /root/.config/opencode
  echo '{}' > /root/.config/opencode/config.json
  chmod 444 /root/.config/opencode/config.json
fi

cd /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban

echo "Starting VK backend server..."
exec cargo run --release --bin server
