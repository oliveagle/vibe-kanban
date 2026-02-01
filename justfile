# ===========================================
# vibe-kanban justfile
# ===========================================

# 默认显示帮助
default:
    @just --list

# ===========================================
# DEVELOPMENT WORKFLOW
# ===========================================

# 同时启动前端和后端（监听所有网络接口）
dev host="0.0.0.0":
    #!/usr/bin/env bash
    export HOST={{host}}
    export FRONTEND_PORT=$(node scripts/setup-dev-environment.js frontend)
    export BACKEND_PORT=$(node scripts/setup-dev-environment.js backend)
    echo "Starting vibe-kanban..."
    echo "  Frontend: http://{{host}}:${FRONTEND_PORT}"
    echo "  Backend:  http://{{host}}:${BACKEND_PORT}"
    echo ""
    concurrently \
        "npm run backend:dev:watch" \
        "npm run frontend:dev"

# 启动前端开发服务器
frontend-dev:
    #!/usr/bin/env bash
    export FRONTEND_PORT=$(node scripts/setup-dev-environment.js frontend)
    npm run frontend:dev

# 启动后端开发服务器（监听所有网络接口）
backend-dev host="0.0.0.0":
    HOST={{host}} npm run backend:dev

# 重启开发服务（先停止再启动）
restart-dev host="0.0.0.0":
    just stop && sleep 2 && just dev {{host}}

# ===========================================
# SERVICE MANAGEMENT
# ===========================================

# 停止所有 vibe-kanban 相关进程
stop:
    #!/usr/bin/env bash
    echo "Stopping vibe-kanban services..."
    # Stop cargo watch processes
    pkill -f "cargo watch" 2>/dev/null || true
    # Stop backend server
    pkill -f "server" 2>/dev/null || true
    # Stop frontend dev server (vite)
    pkill -f "vite" 2>/dev/null || true
    # Stop any node processes in frontend
    pkill -f "node.*frontend" 2>/dev/null || true
    echo "Services stopped"

# 查看正在运行的进程
ps:
    #!/usr/bin/env bash
    echo "=== Vibe Kanban Processes ==="
    ps aux | grep -E "(cargo|vite|node.*frontend|server)" | grep -v grep || echo "No processes found"

# ===========================================
# HEALTH & LOGS
# ===========================================

# 健康检查 - 检查端口和服务状态
health:
    #!/usr/bin/env bash
    echo "=== Vibe Kanban Health Check ==="
    echo ""
    # Check frontend port (typically auto-assigned from scripts)
    for port in 3000 3001 3002; do
        if lsof -i :$port > /dev/null 2>&1; then
            echo "✓ Frontend (port $port): LISTENING"
        fi
    done
    # Check backend port (typically auto-assigned, check common ones)
    for port in 3001 8080 8081; do
        if lsof -i :$port > /dev/null 2>&1; then
            echo "✓ Backend (port $port): LISTENING"
        fi
    done
    echo ""
    echo "Run 'just ps' to see process details"

# 查看日志（需要配合日志文件使用）
logs:
    #!/usr/bin/env bash
    echo "Logs would be available here if using container mode"
    echo "For dev mode, logs are in the terminal running 'just dev'"

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

# 准备数据库（仅检查）
prepare-db-check:
    npm run prepare-db:check

# 生成 TypeScript 类型
generate-types:
    npm run generate-types

# 生成 TypeScript 类型（仅检查）
generate-types-check:
    npm run generate-types:check

# ===========================================
# REMOTE DEPLOYMENT
# ===========================================

# 启动远程开发环境
remote-dev:
    npm run remote:dev

# 准备远程数据库
remote-prepare-db:
    npm run remote:prepare-db

# ===========================================
# LOCAL PRODUCTION DEPLOYMENT (Docker/Podman)
# ===========================================

# 构建生产容器镜像（本地预编译版本）
container-build:
    #!/usr/bin/env bash
    echo "Building frontend..."
    (cd frontend && pnpm run build)
    echo "Building Rust binary..."
    cargo build --release --bin server
    echo "Building Podman image..."
    podman build -f Dockerfile.local -t vibe-kanban:local .
    echo "✅ Build complete: vibe-kanban:local"

# 启动生产容器（使用 podman）
up:
    #!/usr/bin/env bash
    echo "Starting vibe-kanban production container..."
    podman compose -f docker-compose.local.yml up -d
    echo "✅ Container started on http://localhost:37825"
    echo "Run 'just logs-container' to view logs"

# 停止生产容器（使用 podman）
down:
    podman compose -f docker-compose.local.yml down

# 重启生产容器
restart: down up

# 查看容器日志（使用 podman）
logs-container:
    podman compose -f docker-compose.local.yml logs -f

# 生产容器健康检查（使用 podman）
health-container:
    #!/usr/bin/env bash
    echo "=== Vibe Kanban Container Health Check ==="
    echo ""
    if podman ps --format '{{ '{{' }}.Names{{ '}}' }}' | grep -q vibe-kanban; then
        STATUS=$(podman ps --format '{{ '{{' }}.Status{{ '}}' }}' --filter name=vibe-kanban)
        echo "✓ Container: RUNNING ($STATUS)"
    else
        echo "✗ Container: NOT RUNNING"
    fi
    if lsof -i :37825 > /dev/null 2>&1; then
        echo "✓ Port 37825: LISTENING"
    else
        echo "✗ Port 37825: NOT LISTENING"
    fi
    echo ""
    echo "Run 'just logs-container' to view logs"

# ===========================================
# INSTALL & SETUP
# ===========================================

# 安装依赖
install:
    pnpm i
    cd frontend && pnpm i

# 准备开发环境（安装 cargo-watch, sqlx-cli）
setup:
    #!/usr/bin/env bash
    echo "Installing cargo-watch..."
    cargo install cargo-watch
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli
    echo "Setup complete!"
