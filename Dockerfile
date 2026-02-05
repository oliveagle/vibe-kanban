# ============================================
# Vibe-Kanban Production & Development Base Image
# ============================================
# This base image is shared between production and development
# Contains all common tools: Rust, Node.js, podman, neovim, opencode
#
# Usage:
#   Local build: podman build -f Dockerfile --target base -t vibe-kanban:base .
#   GitHub Actions: Uses pre-built ghcr.io/oliveagle/vibe-kanban/base:v{version}
#
# Version History:
#   v0.0.147 - Unified base image for prod and dev

# ============================================
# Arguments
# ============================================
ARG BASE_IMAGE=rust:1.80-slim-bookworm

# ============================================
# Base Image (Shared)
# ============================================
FROM ${BASE_IMAGE} AS base

ARG USE_MIRROR=false

# Configure apt source (mirror for dev, official for prod)
RUN if [ "$USE_MIRROR" = "true" ]; then \
        rm -f /etc/apt/sources.list.d/*.sources && \
        echo 'deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm main contrib non-free non-free-firmware' > /etc/apt/sources.list && \
        echo 'deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-updates main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
        echo 'deb http://mirrors.tuna.tsinghua.edu.cn/debian-security/ bookworm-security main contrib non-free non-free-firmware' >> /etc/apt/sources.list; \
    fi

# Install system dependencies (cached layer)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libclang-dev \
    clang \
    git \
    curl \
    ca-certificates \
    xz-utils \
    podman \
    slirp4netns \
    uidmap \
    neovim \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 20.x
RUN if [ "$USE_MIRROR" = "true" ]; then \
        curl -fsSL https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/v20.18.2/node-v20.18.2-linux-x64.tar.xz | tar -xJf - -C /usr/local --strip-components=1; \
    else \
        curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
        && apt-get install -y nodejs; \
    fi \
    && npm install -g npm@latest pnpm \
    && rm -rf /var/lib/apt/lists/*

# Configure npm registry (mirror for dev)
RUN if [ "$USE_MIRROR" = "true" ]; then \
        npm config set registry https://registry.npmmirror.com; \
    fi

# Install Rust nightly toolchain
RUN rustup toolchain install nightly-2025-12-04 --component rustfmt,rustc,rust-analyzer,rust-src,rust-std,cargo

# Install opencode globally
RUN npm install -g opencode-ai && \
    ln -s $(which opencode) /usr/local/bin/opencode || true

# Build MCP task server
COPY . /tmp/build
WORKDIR /tmp/build
RUN cargo fetch && \
    cargo build --release --bin mcp_task_server && \
    cp target/release/mcp_task_server /usr/local/bin/mcp_task_server && \
    chmod +x /usr/local/bin/mcp_task_server && \
    rm -rf /tmp/build

WORKDIR /app

# ============================================
# Production Runtime
# ============================================
FROM base AS prod-runtime

ARG POSTHOG_API_KEY
ARG POSTHOG_API_ENDPOINT

ENV VITE_PUBLIC_POSTHOG_KEY=$POSTHOG_API_KEY
ENV VITE_PUBLIC_POSTHOG_HOST=$POSTHOG_API_ENDPOINT
ENV HOST=0.0.0.0
ENV PORT=3000

# Copy source code
WORKDIR /app
COPY . .

# Build frontend
RUN npm run generate-types && \
    cd frontend && pnpm install && pnpm run build

# Build backend
RUN cargo build --release --bin server

# Create repos directory
RUN mkdir -p /repos

EXPOSE 3000

CMD ["cargo", "run", "--release", "--bin", "server"]

# ============================================
# Development Runtime
# ============================================
FROM base AS dev-runtime

ENV HOST=0.0.0.0
ENV PORT=3001
ENV RUST_LOG=debug
ENV DISABLE_WORKTREE_ORPHAN_CLEANUP=1
ENV VIBE_BACKEND_URL=http://localhost:3000

# Configure cargo to use proxy for crate downloads
RUN mkdir -p /usr/local/cargo && \
    echo '[http]' > /usr/local/cargo/config.toml && \
    echo 'proxy = "http://host.containers.internal:1080"' >> /usr/local/cargo/config.toml && \
    echo '' >> /usr/local/cargo/config.toml && \
    echo '[https]' >> /usr/local/cargo/config.toml && \
    echo 'proxy = "http://host.containers.internal:1080"' >> /usr/local/cargo/config.toml

EXPOSE 3001

VOLUME ["/app"]

WORKDIR /app

CMD ["sh", "-c", "cargo install cargo-watch && cargo watch -w crates -x 'run --bin server'"]
