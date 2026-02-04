#!/bin/bash
# Switch Cargo registry mirror between official and Tsinghua

set -e

SOURCE="${1:-tuna}"
CARGO_CONFIG="$HOME/.cargo/config.toml"

mkdir -p "$HOME/.cargo"

if [ "$SOURCE" = "tuna" ]; then
    echo "Switching to Tsinghua mirror..."
    cat > "$CARGO_CONFIG" << 'EOF'
[registries]
tuna = { index = "sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/" }

[source.crates-io]
replace-with = "tuna"
EOF
    echo "✅ Switched to Tsinghua mirror"
elif [ "$SOURCE" = "official" ]; then
    echo "Switching to official crates.io..."
    rm -f "$CARGO_CONFIG"
    echo "✅ Switched to official crates.io"
else
    echo "Usage: $0 [tuna|official]"
    echo "  tuna     - Use Tsinghua University mirror"
    echo "  official - Use official crates.io"
    exit 1
fi
