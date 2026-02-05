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
    VERSION="0.0.148"
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
    VERSION="0.0.148"
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
    if ! podman images | grep -q "vibe-kanban.*dev-runtime-v0.0.148"; then
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
# Usage: just prod-pull [tag]
# 
# Available tags:
#   - dev: latest dev branch build (default)
#   - latest: latest release
#   - 0.0.147: specific version
#   - <commit-sha>: specific commit
#
# Example: just prod-pull latest
prod-pull tag="dev":
    #!/usr/bin/env bash
    TAG="{{tag}}"
    echo "Pulling production image from GHCR..."
    echo "Tag: $TAG"
    echo ""
    
    echo "📦 Pulling backend image..."
    if ! podman pull ghcr.io/oliveagle/vibe-kanban/backend:$TAG 2>&1; then
        echo "❌ Failed to pull production image"
        echo ""
        echo "Available tags:"
        echo "  - dev: latest dev branch build"
        echo "  - latest: latest release"
        echo "  - <version>: specific release (e.g., 0.0.147)"
        echo ""
        exit 1
    fi
    
    # Tag as local for docker-compose compatibility
    podman tag ghcr.io/oliveagle/vibe-kanban/backend:$TAG vibe-kanban:local
    
    echo ""
    echo "✅ Production image ready: vibe-kanban:local (from $TAG)"

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
# Usage: just prod-wait-pull [tag] [timeout_minutes]
# 
# Available tags:
#   - dev: latest dev branch build
#   - latest: latest release
#   - 0.0.147: specific version (only for release tags)
#   - <commit-sha>: specific commit
#
# Example: just prod-wait-pull dev 30
prod-wait-pull tag="dev" timeout="30":
    #!/usr/bin/env bash
    TAG="{{tag}}"
    TIMEOUT_MIN={{timeout}}
    IMAGE="ghcr.io/oliveagle/vibe-kanban/backend:$TAG"
    
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
            echo ""
            echo "Available tags on GHCR:"
            echo "  - dev: latest dev branch build"
            echo "  - latest: latest release"
            echo "  - <version>: specific release (e.g., 0.0.147)"
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
# VERSION MANAGEMENT
# ===========================================

# 升级版本号 (用于发布前)
# Usage: just bump-version [new_version]
# Example: just bump-version 0.0.148
bump-version new_version:
    #!/usr/bin/env bash
    NEW_VERSION="{{new_version}}"
    OLD_VERSION=$(cat package.json | grep '"version"' | head -1 | awk -F: '{ print $2 }' | sed 's/[",]//g' | tr -d '[[:space:]]')
    
    echo "Bumping version: $OLD_VERSION -> $NEW_VERSION"
    echo ""
    
    # Update package.json files
    echo "📦 Updating package.json files..."
    sed -i "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$NEW_VERSION\"/g" package.json
    sed -i "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$NEW_VERSION\"/g" frontend/package.json
    sed -i "s/\"version\": \"$OLD_VERSION\"/\"version\": \"$NEW_VERSION\"/g" npx-cli/package.json
    echo "✅ Updated package.json files"
    
    # Update justfile version references
    echo "🔧 Updating justfile..."
    sed -i "s/VERSION=\"$OLD_VERSION\"/VERSION=\"$NEW_VERSION\"/g" justfile
    sed -i "s/base:v$OLD_VERSION/base:v$NEW_VERSION/g" justfile
    sed -i "s/backend:$OLD_VERSION/backend:$NEW_VERSION/g" justfile
    sed -i "s/dev-runtime-v$OLD_VERSION/dev-runtime-v$NEW_VERSION/g" justfile
    echo "✅ Updated justfile"
    
    # Update docker-compose.local.yml
    echo "🐳 Updating docker-compose.local.yml..."
    sed -i "s/base:v$OLD_VERSION/base:v$NEW_VERSION/g" docker-compose.local.yml
    echo "✅ Updated docker-compose.local.yml"
    
    # Update AGENTS.md
    echo "📚 Updating AGENTS.md..."
    sed -i "s/$OLD_VERSION/$NEW_VERSION/g" AGENTS.md
    echo "✅ Updated AGENTS.md"
    
    echo ""
    echo "✅ Version bumped to $NEW_VERSION"
    echo ""
    echo "Next steps:"
    echo "  1. Review changes: git diff"
    echo "  2. Commit: git add -A && git commit -m \"chore: bump version to $NEW_VERSION\""
    echo "  3. Create PR to master"
    echo "  4. After merge, create tag: git tag v$NEW_VERSION && git push origin v$NEW_VERSION"
    echo "  5. Wait for GitHub Actions to build: just prod-wait-pull $NEW_VERSION 30"

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
