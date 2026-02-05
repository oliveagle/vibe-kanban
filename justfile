# ===========================================
# vibe-kanban justfile
# ===========================================

# 默认显示帮助
default:
    @just --list

# ===========================================
# DEVELOPMENT WORKFLOW
# ===========================================

# 更新 Rust 依赖 (手动触发，更新 Cargo.lock 和下载新依赖)
dev-update-deps:
    #!/usr/bin/env bash
    echo "Updating Rust dependencies..."
    cargo update
    echo "✅ Dependencies updated. Run 'just dev-build-all' to rebuild images."

# 切换 Cargo 镜像源 (official/tuna)
dev-cargo-mirror source="tuna":
    ./scripts/cargo-mirror.sh {{source}}

# 在运行中的容器里安装 coding agents (Claude Code, Codex, Gemini CLI)
# 需要容器已启动: just dev-srv
dev-install-agents:
    ./scripts/install-agents.sh

# 拉取开发环境镜像
# 从 GitHub Container Registry 拉取预构建的 base（推荐，更快）
# 如果拉取失败，请手动运行: just dev-build-base
dev-build-all:
    #!/usr/bin/env bash
    VERSION="0.0.147"
    echo "Setting up development environment (v$VERSION)..."
    echo ""

    # Try to pull pre-built base from GitHub (public repo, no auth needed)
    echo "📦 Pulling base from GitHub Container Registry..."
    if ! podman pull ghcr.io/oliveagle/vibe-kanban/base:v$VERSION 2>&1; then
        echo ""
        echo "❌ Failed to pull base image from GHCR"
        echo ""
        echo "Options:"
        echo "  1. Check your network connection"
        echo "  2. Build locally: just dev-build-base"
        echo ""
        exit 1
    fi
    
    echo "✅ Pulled base from GHCR"
    podman tag ghcr.io/oliveagle/vibe-kanban/base:v$VERSION vibe-kanban:base-v$VERSION
    echo ""

    # Build dev-runtime locally (fast, based on base)
    echo "📦 Building dev-runtime..."
    podman build -f Dockerfile.dev-runtime --build-arg BASE_IMAGE=ghcr.io/oliveagle/vibe-kanban/base:v$VERSION \
        -t vibe-kanban:dev-runtime-v$VERSION .
    echo "✅ Built dev-runtime"
    echo ""

    echo "🎉 Development environment ready!"
    echo ""
    echo "Available images:"
    echo "  - vibe-kanban:base-v$VERSION        (Shared base with tools)"
    echo "  - vibe-kanban:dev-runtime-v$VERSION (Dev environment with hot-reload)"

# 本地构建 base（需要良好的网络环境）
dev-build-base:
    #!/usr/bin/env bash
    VERSION="0.0.147"
    echo "Building base locally..."
    echo "⚠️  This requires access to crates.io and github.com"
    echo ""
    podman build -f Dockerfile --network=host --target base \
        --build-arg USE_MIRROR=true \
        --build-arg HTTPS_PROXY=http://localhost:1080 \
        --build-arg HTTP_PROXY=http://localhost:1080 \
        -t vibe-kanban:base-v$VERSION .

# 启动后端开发容器 (port 3000)
# Usage: just dev-srv        # 前台运行
#        just dev-srv -d     # 后台运行
dev-srv *args:
    #!/usr/bin/env bash
    if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "Error: Port 3000 is already in use"
        exit 1
    fi
    
    # Check if dev image exists
    if ! podman images | grep -q "vibe-kanban.*dev-runtime-v0.0.147"; then
        echo "Dev image not found, building all layers..."
        just dev-build-all
    fi
    
    # Run startup script
    if [[ "{{args}}" == *"-d"* ]]; then
        ./scripts/start-vk-backend.sh -d
    else
        ./scripts/start-vk-backend.sh
    fi

# [DEPRECATED] 前端开发服务器已合并到后端，使用 `just dev-srv` 启动统一服务
# 前端代码修改后需要重新构建: (cd frontend && pnpm run build)
dev-ui *args:
    #!/usr/bin/env bash
    echo "⚠️  前端开发服务器已弃用"
    echo ""
    echo "新的开发流程:"
    echo "  1. just dev-srv          # 启动后端 (端口 3000，包含嵌入的前端)"
    echo "  2. (cd frontend && pnpm run dev)  # 如需热重载前端，单独运行"
    echo ""
    echo "或者直接使用 docker-compose:"
    echo "  podman compose -f docker-compose.dev.yml up"
    exit 0

# 停止后端开发容器
dev-srv-stop:
    #!/usr/bin/env bash
    echo "Stopping backend dev container..."
    podman stop vibe-kanban-backend-dev 2>/dev/null || true
    podman rm vibe-kanban-backend-dev 2>/dev/null || true
    echo "✅ Backend dev container stopped"

# [DEPRECATED] 前端开发容器已移除，使用 `just dev-srv-stop` 停止后端
dev-ui-stop:
    #!/usr/bin/env bash
    echo "⚠️  前端开发容器已弃用，使用 `just dev-srv-stop` 停止服务"

# 停止所有开发容器
dev-stop-all:
    just dev-srv-stop

# ===========================================
# BUILD & CHECK
# ===========================================

# 运行所有检查
check:
    npm run check

# 运行测试
test:
    cargo test --workspace

# 格式化代码
format:
    cargo fmt --all
    cd frontend && npm run format

# 构建项目
build:
    pnpm run build:npx

# ===========================================
# DATABASE & TYPES
# ===========================================

# 准备数据库
prepare-db:
    npm run prepare-db

# 生成 TypeScript 类型
generate-types:
    npm run generate-types

# ===========================================
# PRODUCTION DEPLOYMENT (Pull from GHCR only)
# ===========================================
# 生产镜像必须从 GitHub Container Registry 拉取
# 不允许本地构建，确保环境一致性

# 拉取生产镜像 (从 GHCR)
# Usage: just prod-pull [version]  # 默认版本为 0.0.147
prod-pull version="0.0.147":
    #!/usr/bin/env bash
    VERSION="{{version}}"
    echo "Pulling production image from GHCR..."
    echo "Version: $VERSION"
    echo ""
    
    echo "📦 Pulling backend image..."
    if ! podman pull ghcr.io/oliveagle/vibe-kanban/backend:$VERSION 2>&1; then
        echo "❌ Failed to pull production image"
        echo ""
        echo "Options:"
        echo "  1. Check your network connection"
        echo "  2. Verify version exists: ghcr.io/oliveagle/vibe-kanban/backend:$VERSION"
        echo "  3. Use 'latest' tag: just prod-pull latest"
        echo ""
        exit 1
    fi
    
    # Tag as local for docker-compose compatibility
    podman tag ghcr.io/oliveagle/vibe-kanban/backend:$VERSION vibe-kanban:local
    
    echo ""
    echo "✅ Production image ready: vibe-kanban:local (from $VERSION)"

# 启动生产容器
prod-up:
    #!/usr/bin/env bash
    # Check if image exists, if not pull it
    if ! podman images | grep -q "vibe-kanban.*local"; then
        echo "Production image not found, pulling from GHCR..."
        just prod-pull
    fi
    
    echo "Starting vibe-kanban production container..."
    podman compose -f docker-compose.local.yml up -d
    echo "✅ Container started on http://localhost:37825"

# 停止生产容器
prod-down:
    podman compose -f docker-compose.local.yml down

# 查看生产容器日志
prod-logs:
    podman compose -f docker-compose.local.yml logs -f

# 等待 GitHub Actions 构建完成并拉取镜像
# Usage: just prod-wait-pull [version] [timeout_minutes]
# Example: just prod-wait-pull 0.0.148 30
prod-wait-pull version="0.0.147" timeout="30":
    #!/usr/bin/env bash
    VERSION="{{version}}"
    TIMEOUT_MIN={{timeout}}
    IMAGE="ghcr.io/oliveagle/vibe-kanban/backend:$VERSION"
    
    echo "⏳ Waiting for GitHub Actions to build image..."
    echo "Image: $IMAGE"
    echo "Timeout: ${TIMEOUT_MIN} minutes"
    echo ""
    
    START_TIME=$(date +%s)
    TIMEOUT_SEC=$((TIMEOUT_MIN * 60))
    CHECK_INTERVAL=30
    
    while true; do
        CURRENT_TIME=$(date +%s)
        ELAPSED=$((CURRENT_TIME - START_TIME))
        
        if [ $ELAPSED -ge $TIMEOUT_SEC ]; then
            echo ""
            echo "❌ Timeout after ${TIMEOUT_MIN} minutes"
            echo "Image not available: $IMAGE"
            exit 1
        fi
        
        # Try to pull the image
        echo -n "[$((ELAPSED / 60))m$((ELAPSED % 60))s] Checking... "
        
        if podman pull $IMAGE 2>/dev/null; then
            echo "✅"
            echo ""
            echo "✅ Image pulled successfully!"
            
            # Tag as local for docker-compose compatibility
            podman tag $IMAGE vibe-kanban:local
            echo "✅ Tagged as vibe-kanban:local"
            exit 0
        fi
        
        echo "not ready, retrying in ${CHECK_INTERVAL}s..."
        sleep $CHECK_INTERVAL
    done

# ===========================================
# INSTALL & SETUP
# ===========================================

# 安装依赖
install:
    pnpm i
    cd frontend && pnpm i

# 准备开发环境
setup:
    #!/usr/bin/env bash
    echo "Installing cargo-watch..."
    cargo install cargo-watch
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli
    echo "Setup complete!"
