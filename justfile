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
    podman build -f Dockerfile.dev --target base --build-arg USE_MIRROR=true -t vibe-kanban:dev-base-v1.0.0 .
    echo "✅ Layer 1 built: vibe-kanban:dev-base-v1.0.0"
    echo ""
    
    # Layer 2: Dev tools
    echo "📦 Building Layer 2: Dev tools (cargo-watch, Lightpanda, Node.js)..."
    podman build -f Dockerfile.dev --target dev-tools --build-arg USE_MIRROR=true -t vibe-kanban:dev-tools-v1.0.0 .
    echo "✅ Layer 2 built: vibe-kanban:dev-tools-v1.0.0"
    echo ""
    
    # Layer 3: Runtime
    echo "📦 Building Layer 3: Runtime..."
    podman build -f Dockerfile.dev --target dev-runtime --build-arg USE_MIRROR=true -t vibe-kanban:dev-runtime-v1.0.0 .
    echo "✅ Layer 3 built: vibe-kanban:dev-runtime-v1.0.0"
    echo ""
    
    echo "🎉 All development images built successfully!"
    echo ""
    echo "Available images:"
    echo "  - vibe-kanban:dev-base-v1.0.0    (Rust 1.80 + system deps)"
    echo "  - vibe-kanban:dev-tools-v1.0.0   (+ cargo-watch, Lightpanda, Node.js)"
    echo "  - vibe-kanban:dev-runtime-v1.0.0 (Full dev environment)"

# 启动后端开发容器 (port 3001)
dev-srv:
    #!/usr/bin/env bash
    if lsof -Pi :3001 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "Error: Port 3001 is already in use"
        exit 1
    fi
    
    # Check if dev image exists (use versioned tag)
    if ! podman images | grep -q "vibe-kanban.*dev-runtime-v1.0.0"; then
        echo "Dev image not found, building all layers..."
        just dev-build-all
    fi
    
    echo "Starting backend development container..."
    podman run -d \
        --name vibe-kanban-backend-dev \
        --replace \
        -p 3001:3001 \
        -e HOST=0.0.0.0 \
        -e PORT=3001 \
        -e VK_SHARED_API_BASE=http://localhost:3001 \
        -e RUST_LOG=debug \
        -e DISABLE_WORKTREE_ORPHAN_CLEANUP=1 \
        -e https_proxy=http://host.containers.internal:1080 \
        -e http_proxy=http://host.containers.internal:1080 \
        -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban:/app:rw \
        -v ${HOME}/repos:/repos:ro \
        -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
        --workdir /app \
        vibe-kanban:dev-runtime-v1.0.0
    
    echo "✅ Backend dev container started on http://localhost:3001"
    echo "View logs: podman logs -f vibe-kanban-backend-dev"

# 启动前端开发服务器 (port 3000)
dev-ui:
    #!/usr/bin/env bash
    if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "Error: Port 3000 is already in use"
        exit 1
    fi
    export FRONTEND_PORT=3000
    npm run frontend:dev

# 停止所有开发服务
stop-dev:
    #!/usr/bin/env bash
    echo "Stopping vibe-kanban services..."
    pkill -f "cargo watch" 2>/dev/null || true
    pkill -f "server" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    pkill -f "node.*frontend" 2>/dev/null || true
    echo "Services stopped"

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

# 构建生产容器镜像
container-build:
    #!/usr/bin/env bash
    echo "Building frontend..."
    (cd frontend && pnpm run build)
    echo "Building Rust binary..."
    cargo build --release --bin server
    echo "Building Podman image..."
    podman build -f Dockerfile.local -t vibe-kanban:local .
    echo "✅ Build complete: vibe-kanban:local"

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
