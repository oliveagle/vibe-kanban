# vibe-kanban justfile

# 默认显示帮助
[private]
default:
    @just --list

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

# 准备数据库
prepare-db:
    npm run prepare-db

# 生成 TypeScript 类型
generate-types:
    npm run generate-types
