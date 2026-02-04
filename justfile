# ===========================================
# vibe-kanban justfile
# ===========================================

# 默认显示帮助
default:
    @just --list

# ===========================================
# DEVELOPMENT WORKFLOW
# ===========================================

# 构建所有开发环境镜像层 (只需执行一次)
dev-build-all:
    #!/usr/bin/env bash
    echo "Building development environment images..."
    echo ""
    
    # Layer 1: Base (Rust + system deps)
    echo "📦 Building Layer 1: Base (Rust 1.80 + system dependencies)..."
    podman build -f Dockerfile.dev --target base --build-arg USE_MIRROR=true -t vibe-kanban:dev-base-v0.0.147 .
    echo "✅ Layer 1 built: vibe-kanban:dev-base-v0.0.147"
    echo ""
    
    # Layer 2: Runtime
    echo "📦 Building Layer 2: Runtime..."
    podman build -f Dockerfile.dev --target dev-runtime --build-arg USE_MIRROR=true -t vibe-kanban:dev-runtime-v0.0.147 .
    echo "✅ Layer 2 built: vibe-kanban:dev-runtime-v0.0.147"
    echo ""
    
    echo "🎉 All development images built successfully!"
    echo ""
    echo "Available images:"
    echo "  - vibe-kanban:dev-base-v0.0.147    (Rust 1.80 + system deps)"
    echo "  - vibe-kanban:dev-runtime-v0.0.147 (Full dev environment)"

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
# LOCAL PRODUCTION DEPLOYMENT (Docker/Podman)
# ===========================================

# 构建生产容器镜像 (单一后端镜像，包含嵌入的前端)
container-build:
    #!/usr/bin/env bash
    echo "Building frontend for embedding..."
    (cd frontend && pnpm run build)
    echo "Building unified container image (backend + embedded frontend)..."
    podman build -f Dockerfile.backend -t vibe-kanban:local .
    echo "✅ Build complete: vibe-kanban:local (backend with embedded frontend)"

# 启动生产容器
up:
    #!/usr/bin/env bash
    echo "Starting vibe-kanban production container..."
    podman compose -f docker-compose.local.yml up -d
    echo "✅ Container started on http://localhost:37825"

# 停止生产容器
down:
    podman compose -f docker-compose.local.yml down

# 查看容器日志
logs-container:
    podman compose -f docker-compose.local.yml logs -f

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
