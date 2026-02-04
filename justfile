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
    
    # Layer 2: Runtime
    echo "📦 Building Layer 2: Runtime..."
    podman build -f Dockerfile.dev --target dev-runtime --build-arg USE_MIRROR=true -t vibe-kanban:dev-runtime-v1.0.0 .
    echo "✅ Layer 2 built: vibe-kanban:dev-runtime-v1.0.0"
    echo ""
    
    echo "🎉 All development images built successfully!"
    echo ""
    echo "Available images:"
    echo "  - vibe-kanban:dev-base-v1.0.0    (Rust 1.80 + system deps)"
    echo "  - vibe-kanban:dev-runtime-v1.0.0 (Full dev environment)"

# 启动后端开发容器 (port 3001) - 交互模式
# Usage: just dev-srv        # 前台运行 (推荐)
#        just dev-srv -d     # 后台运行
#        just dev-srv --no-build  # 直接运行已编译的二进制（跳过 cargo build）
dev-srv *args:
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
    
    # Create shared network if not exists
    podman network create vibe-kanban-dev 2>/dev/null || true
    
    # Determine run mode
    if [[ "{{args}}" == *"--no-build"* ]]; then
        RUN_MODE="direct"
    else
        RUN_MODE="watch"
    fi
    
    # Check if -d (daemon) mode
    if [[ "{{args}}" == *"-d"* ]]; then
        echo "Starting backend development container in background..."
        if [[ "$RUN_MODE" == "direct" ]]; then
            podman run -d \
                --name vibe-kanban-backend-dev \
                --replace \
                --network vibe-kanban-dev \
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
                -v /mnt/volume3/data/repos:/mnt/volume3/data/repos:rw \
                -v /var/tmp:/var/tmp:rw \
                -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
                --workdir /app \
                vibe-kanban:dev-runtime-v1.0.0 \
                sh -c "cargo run --bin server"
        else
            podman run -d \
                --name vibe-kanban-backend-dev \
                --replace \
                --network vibe-kanban-dev \
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
                -v /mnt/volume3/data/repos:/mnt/volume3/data/repos:rw \
                -v /var/tmp:/var/tmp:rw \
                -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
                --workdir /app \
                vibe-kanban:dev-runtime-v1.0.0
        fi
        echo "✅ Backend dev container started on http://localhost:3001"
        echo "View logs: podman logs -f vibe-kanban-backend-dev"
    else
        echo "Starting backend development container in interactive mode..."
        echo "Press Ctrl+C to stop"
        if [[ "$RUN_MODE" == "direct" ]]; then
            podman run -ti \
                --name vibe-kanban-backend-dev \
                --replace \
                --network vibe-kanban-dev \
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
                -v /mnt/volume3/data/repos:/mnt/volume3/data/repos:rw \
                -v /var/tmp:/var/tmp:rw \
                -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
                --workdir /app \
                vibe-kanban:dev-runtime-v1.0.0 \
                sh -c "cargo run --bin server"
        else
            podman run -ti \
                --name vibe-kanban-backend-dev \
                --replace \
                --network vibe-kanban-dev \
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
                -v /mnt/volume3/data/repos:/mnt/volume3/data/repos:rw \
                -v /var/tmp:/var/tmp:rw \
                -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
                --workdir /app \
                vibe-kanban:dev-runtime-v1.0.0
        fi
    fi

# 启动前端开发服务器 - 在容器内运行 (port 3000)
# Usage: just dev-ui         # 开发模式 (npm run dev)
#        just dev-ui --build  # 先 build 再启动 (生产模式)
dev-ui *args:
    #!/usr/bin/env bash
    if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "Error: Port 3000 is already in use"
        exit 1
    fi

    # Check if base image exists
    if ! podman images | grep -q "vibe-kanban.*dev-base-v1.0.0"; then
        echo "Dev image not found, building..."
        just dev-build-all
    fi

    # Create shared network if not exists
    podman network create vibe-kanban-dev 2>/dev/null || true

    # Check if build mode
    if [[ "{{args}}" == *"--build"* ]]; then
        # Check if dist exists and is not empty
        if [ ! -d "/mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/frontend/dist" ] || [ -z "$(ls -A /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/frontend/dist 2>/dev/null)" ]; then
            echo "Building frontend production bundle (dist not found or empty)..."
            podman run --rm \
                --network vibe-kanban-dev \
                -e VITE_API_BASE_URL=http://vibe-kanban-backend-dev:3001 \
                -e BACKEND_HOST=vibe-kanban-backend-dev \
                -e https_proxy=http://host.containers.internal:1080 \
                -e http_proxy=http://host.containers.internal:1080 \
                -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/frontend:/app/frontend:rw \
                -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/shared:/app/shared:rw \
                --workdir /app/frontend \
                node:20-alpine \
                sh -c "npm config set proxy http://host.containers.internal:1080 && npm config set https-proxy http://host.containers.internal:1080 && npm install && VITE_VK_SHARED_API_BASE=http://vibe-kanban-backend-dev:3001 npm run build"
        else
            echo "Using existing frontend build in dist/"
        fi

        echo "Starting frontend production server..."
        echo "Press Ctrl+C to stop"
        podman run -ti \
            --name vibe-kanban-frontend-dev \
            --replace \
            --network vibe-kanban-dev \
            -p 3000:3000 \
            -e VITE_API_BASE_URL=http://vibe-kanban-backend-dev:3001 \
            -e BACKEND_HOST=vibe-kanban-backend-dev \
            -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/frontend:/app/frontend:ro \
            -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/shared:/app/shared:ro \
            --workdir /app/frontend \
            node:20-alpine \
            sh -c "npm install -g serve && serve -s -L dist -l tcp://0.0.0.0:3000"
    else
        echo "Starting frontend development container..."
        echo "Press Ctrl+C to stop"
        podman run -ti \
            --name vibe-kanban-frontend-dev \
            --replace \
            --network vibe-kanban-dev \
            -p 3000:3000 \
            -e VITE_API_BASE_URL=http://vibe-kanban-backend-dev:3001 \
            -e BACKEND_HOST=vibe-kanban-backend-dev \
            -e https_proxy=http://host.containers.internal:1080 \
            -e http_proxy=http://host.containers.internal:1080 \
            -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/frontend:/app/frontend:rw \
            -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/shared:/app/shared:rw \
            --workdir /app/frontend \
            node:20-alpine \
            sh -c "npm config set proxy http://host.containers.internal:1080 && npm config set https-proxy http://host.containers.internal:1080 && npm install && npm run dev -- --port 3000 --host 0.0.0.0"
    fi

# 停止后端开发容器
dev-srv-stop:
    #!/usr/bin/env bash
    echo "Stopping backend dev container..."
    podman stop vibe-kanban-backend-dev 2>/dev/null || true
    podman rm vibe-kanban-backend-dev 2>/dev/null || true
    echo "✅ Backend dev container stopped"

# 停止前端开发容器
dev-ui-stop:
    #!/usr/bin/env bash
    echo "Stopping frontend dev container..."
    podman stop vibe-kanban-frontend-dev 2>/dev/null || true
    podman rm vibe-kanban-frontend-dev 2>/dev/null || true
    echo "✅ Frontend dev container stopped"

# 停止所有开发容器
dev-stop-all:
    just dev-srv-stop
    just dev-ui-stop

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
