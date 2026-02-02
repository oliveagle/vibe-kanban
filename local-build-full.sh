#!/bin/bash
set -e

echo "=== Building Vibe Kanban Local Images (Full Version) ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Build Rust backend
echo -e "${YELLOW}Step 1: Building Rust backend...${NC}"
cargo build --release --bin server
echo -e "${GREEN}✓ Backend binary built${NC}"
echo ""

# Step 2: Build frontend
echo -e "${YELLOW}Step 2: Building frontend...${NC}"
cd frontend
pnpm install
pnpm run build
cd ..
echo -e "${GREEN}✓ Frontend built${NC}"
echo ""

# Step 3: Build Docker images
echo -e "${YELLOW}Step 3: Building Docker images...${NC}"
docker-compose -f docker-compose.local.yml build --no-cache
echo -e "${GREEN}✓ Docker images built${NC}"
echo ""

# Step 4: Start services
echo -e "${YELLOW}Step 4: Starting services...${NC}"
docker-compose -f docker-compose.local.yml up -d
echo -e "${GREEN}✓ Services started${NC}"
echo ""

echo -e "${GREEN}=== All done! ===${NC}"
echo ""
echo "Services are now running:"
echo "  - Frontend: http://localhost:37826"
echo "  - Backend API: http://localhost:37825"
echo ""
echo "Commands:"
echo "  - View logs: docker-compose -f docker-compose.local.yml logs -f"
echo "  - Stop: docker-compose -f docker-compose.local.yml down"
echo "  - Restart: docker-compose -f docker-compose.local.yml restart"
