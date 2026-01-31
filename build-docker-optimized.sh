#!/bin/bash
# 优化的 Docker 构建脚本

set -e

echo "=== 步骤 1: 构建基础镜像 (只需要运行一次) ==="
echo "如果已经构建过,这一步会很快(使用缓存)"
podman build -f Dockerfile.base -t vibe-kanban-builder:latest .

echo ""
echo "=== 步骤 2: 构建应用镜像 ==="
echo "现在不需要重新安装 Rust,直接使用基础镜像"
podman build -f Dockerfile.optimized -t vibe-kanban:latest --network=host .

echo ""
echo "构建完成!"
echo "查看镜像大小:"
podman images | grep vibe-kanban
