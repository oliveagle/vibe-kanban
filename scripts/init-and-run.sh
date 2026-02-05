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

# Fix opencode CLI config - remove config_version field that CLI doesn't recognize
# Keep other settings (theme, editor, etc.) that user may have configured
if [ -f /root/.config/opencode/config.json ]; then
  if grep -q '"config_version"' /root/.config/opencode/config.json 2>/dev/null; then
    echo "Fixing opencode config (removing config_version)..."
    # Remove config_version field and related desktop-only fields
    sed -i '/"config_version":/d' /root/.config/opencode/config.json
    sed -i '/"disclaimer_acknowledged":/d' /root/.config/opencode/config.json
    sed -i '/"onboarding_acknowledged":/d' /root/.config/opencode/config.json
    sed -i '/"last_app_version":/d' /root/.config/opencode/config.json
    sed -i '/"show_release_notes":/d' /root/.config/opencode/config.json
    sed -i '/"showcases":/d' /root/.config/opencode/config.json
  fi
fi

cd /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban

echo "Starting VK backend server..."
exec cargo run --release --bin server
