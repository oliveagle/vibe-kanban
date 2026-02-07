#!/bin/bash

set -e

echo 'root:100000:65536' > /etc/subuid
echo 'root:100000:65536' > /etc/subgid

VK_DATA_DIR="/root/.local/share/vibe-kanban"
AGENTS_DIR="$VK_DATA_DIR/agents"
CACHE_DIR="$VK_DATA_DIR/cache"

mkdir -p "$VK_DATA_DIR"
mkdir -p "$AGENTS_DIR/opencode"
mkdir -p "$CACHE_DIR/npm"

# Create MCP config for opencode in VK data directory
if [ ! -f "$VK_DATA_DIR/opencode.json" ]; then
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

# Link MCP config to opencode agent directory
if [ ! -L "$AGENTS_DIR/opencode/mcp.json" ]; then
  ln -sf "$VK_DATA_DIR/opencode.json" "$AGENTS_DIR/opencode/mcp.json"
fi

# Create opencode CLI config in VK directory (remove incompatible fields from desktop config)
OPENCODE_CONFIG="$AGENTS_DIR/opencode/config.json"
if [ -f /root/.config/opencode/config.json ]; then
  if grep -q '"config_version"' /root/.config/opencode/config.json 2>/dev/null; then
    sed -e '/"config_version":/d' \
        -e '/"disclaimer_acknowledged":/d' \
        -e '/"onboarding_acknowledged":/d' \
        -e '/"last_app_version":/d' \
        -e '/"show_release_notes":/d' \
        -e '/"showcases":/d' \
        /root/.config/opencode/config.json > "$OPENCODE_CONFIG"
  else
    cp /root/.config/opencode/config.json "$OPENCODE_CONFIG"
  fi
elif [ ! -f "$OPENCODE_CONFIG" ]; then
  echo '{}' > "$OPENCODE_CONFIG"
fi

# Create symlinks from standard locations to VK directory
mkdir -p /root/.config
if [ ! -L /root/.config/opencode ]; then
  rm -rf /root/.config/opencode
  ln -sf "$AGENTS_DIR/opencode" /root/.config/opencode
fi

cd /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban

# Use SQLx offline mode to avoid database connection at compile time
export SQLX_OFFLINE=true

echo "Starting VK backend server..."
exec cargo run --release --bin server
