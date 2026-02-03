# ===========================================
# vibe-kanban justfile
# ===========================================

# 默认显示帮助
default:
    @just --list

# ===========================================
# DEVELOPMENT WORKFLOW
# ===========================================

# 启动前端开发服务器 (port 3000)
dev-ui:
    #!/usr/bin/env bash
    export FRONTEND_PORT=3000
    npm run frontend:dev

# 启动后端开发服务器
dev-srv host="0.0.0.0":
    HOST={{host}} npm run backend:dev

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
