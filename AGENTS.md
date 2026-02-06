# Repository Guidelines

## Project Structure & Module Organization
- `crates/`: Rust workspace crates — `server` (API + bins), `db` (SQLx models/migrations), `executors`, `services`, `utils`, `deployment`, `local-deployment`, `remote`.
- `frontend/`: React + TypeScript app (Vite, Tailwind). Source in `frontend/src`.
- `frontend/src/components/dialogs`: Dialog components for the frontend.
- `remote-frontend/`: Remote deployment frontend.
- `shared/`: Generated TypeScript types (`shared/types.ts`). Do not edit directly.
- `assets/`, `dev_assets_seed/`, `dev_assets/`: Packaged and local dev assets.
- `npx-cli/`: Files published to the npm CLI package.
- `scripts/`: Dev helpers (ports, DB preparation).
- `docs/`: Documentation files.

## Managing Shared Types Between Rust and TypeScript

ts-rs allows you to derive TypeScript types from Rust structs/enums. By annotating your Rust types with #[derive(TS)] and related macros, ts-rs will generate .ts declaration files for those types.
When making changes to the types, you can regenerate them using `pnpm run generate-types`
Do not manually edit shared/types.ts, instead edit crates/server/src/bin/generate_types.rs

## Version Management (CRITICAL)

**ALL version numbers must be updated together to prevent pollution:**

When making any changes that affect:
- Dockerfile (base image changes)
- Cargo.toml dependencies
- API changes
- Container orchestration

**You MUST bump the version number in ALL of these files:**
1. All `crates/*/Cargo.toml` files (9 crates)
2. `package.json` (root)
3. `frontend/package.json`
4. `npx-cli/package.json`
5. `justfile` (image version tags)

**Current version:** `0.0.148`
**Image tags:** `vibe-kanban:dev-base-v0.0.148`, `vibe-kanban:dev-runtime-v0.0.148`

**Release tagging:**
- `release-version` tag always points to current stable release
- Version tags: `v0.0.148`, `v0.0.148`, etc.

## Build, Test, and Development Commands
- Install: `pnpm i`
- Run dev (frontend + backend with ports auto-assigned): `pnpm run dev`
- Backend (watch): `pnpm run backend:dev:watch`
- Frontend (dev): `pnpm run frontend:dev`
- Type checks: `pnpm run check` (frontend) and `pnpm run backend:check` (Rust cargo check)
- Rust tests: `cargo test --workspace`
- Generate TS types from Rust: `pnpm run generate-types` (or `generate-types:check` in CI)
- Prepare SQLx (offline): `pnpm run prepare-db`
- Prepare SQLx (remote package, postgres): `pnpm run remote:prepare-db`
- Local NPX build: `pnpm run build:npx` then `pnpm pack` in `npx-cli/`

## Container-Based Development Commands (Just)

**Build dev images:**

The development environment uses a two-layer architecture with **fail-fast** strategy:

1. **base** (Layer 1): Rust toolchain + system dependencies
   - Pre-built by GitHub Actions and published to GHCR
   - `just dev-build-all` pulls this image - **fails fast if unavailable**
   - No automatic fallback to local build (to catch issues early)

2. **dev-runtime** (Layer 2): Development runtime configuration
   - Built locally on top of base
   - Fast to build (~1-2 minutes)

```bash
just dev-build-all        # Pull base from GHCR, build dev-runtime locally
just dev-build-base       # Build base locally (slow, only if GHCR fails)
```

**Fail Fast Principle:**
- `just dev-build-all` fails immediately if GHCR pull fails
- No hidden fallbacks - explicit error with clear next steps
- Forces quick detection of network/auth issues

**Image Sources:**
- **GitHub Container Registry** (recommended): `ghcr.io/oliveagle/vibe-kanban/base:v{version}`
- **Local build** (manual fallback): `just dev-build-base` with official sources

**Start backend in container:**
```bash
just dev-srv          # Interactive mode
just dev-srv -d       # Background mode
just dev-srv --no-build  # Skip cargo-watch, direct run
```

**Start frontend in container:**
```bash
just dev-ui           # Development mode
just dev-ui --build   # Production build mode (recommended for external access)
```

**Stop containers:**
```bash
just dev-srv-stop
```

**Install coding agents (optional):**
```bash
just dev-install-agents   # Install Claude Code, Codex, Gemini CLI
```

Coding agents are not installed by default to avoid slowing down container startup. Run this command manually after starting the container. The installation is persistent across container restarts.

### Production Deployment Commands

**⚠️ Production images MUST be pulled from GHCR, never built locally**

```bash
# Pull production image from GHCR
just prod-pull              # Pull dev branch build (default)
just prod-pull latest       # Pull latest release
just prod-pull 0.0.148      # Pull specific version
just prod-wait-pull dev 30  # Wait for GitHub Actions build and pull

# Start/stop production container
just prod-up                # Pull (if needed) and start container
just prod-down              # Stop container
just prod-logs              # View container logs
```

**Available Image Tags:**
| Tag | Description |
|-----|-------------|
| `dev` | Latest dev branch build (default) |
| `latest` | Latest release |
| `0.0.148` | Specific version (only for releases) |
| `<commit-sha>` | Specific commit build |

**Production vs Development:**
| Aspect | Development | Production |
|--------|-------------|------------|
| **Image Source** | Local build on top of GHCR base | Pulled from GHCR only |
| **Build Command** | `just dev-build-all` | `just prod-pull` |
| **Start Command** | `just dev-srv` | `just prod-up` |
| **Hot Reload** | Yes (cargo-watch) | No |
| **Port** | 3000 | 37825 (via docker-compose) |

## Coding Style & Naming Conventions
- Rust: `rustfmt` enforced (`rustfmt.toml`); group imports by crate; snake_case modules, PascalCase types.
- TypeScript/React: ESLint + Prettier (2 spaces, single quotes, 80 cols). PascalCase components, camelCase vars/functions, kebab-case file names where practical.
- Keep functions small, add `Debug`/`Serialize`/`Deserialize` where useful.

## Justfile & Script Organization

**Keep justfile simple - complex logic goes in scripts:**

- justfile should only contain simple command invocations (1-2 lines)
- Complex bash logic (>5 lines or heredocs) must go in `scripts/` directory
- This prevents justfile parsing issues (e.g., `[source]` being interpreted as just syntax)
- Scripts in `scripts/` must be executable (`chmod +x`)

**Example:**
```just
# Good - simple invocation
dev-cargo-mirror source="tuna":
    ./scripts/cargo-mirror.sh {{source}}

# Bad - complex logic in justfile
dev-cargo-mirror source="tuna":
    #!/usr/bin/env bash
    if [ "{{source}}" = "tuna" ]; then
        cat > "$CARGO_CONFIG" << 'EOF'
[source.crates-io]
replace-with = "tuna"
EOF
    fi
```

## Testing Guidelines

### Rust Tests

**Test Organization:**
- **Integration tests** must be placed in `crates/{package}/tests/` directory as separate files
- Do NOT write tests inside source files (no `#[cfg(test)]` modules in `src/`)
- Each test file should focus on a specific module or functionality

**Test File Naming:**
- `crates/utils/tests/msg_store_tests.rs` - tests for `msg_store` module
- `crates/server/tests/task_server_tests.rs` - tests for `task_server` module
- `crates/executors/tests/session_manager_tests.rs` - tests for `session` module

**Running Tests:**
```bash
cargo test --workspace           # Run all tests
cargo test --package utils       # Run tests for specific package
cargo test test_name_pattern     # Run specific test
```

**Test Best Practices:**
- Use `tokio::test` for async tests
- Use temporary directories for file I/O tests ( cleaned up automatically)
- Mock external dependencies (HTTP clients, etc.)
- Test edge cases: empty inputs, errors, concurrency

### Frontend Tests
- Ensure `pnpm run check` and `pnpm run lint` pass
- If adding runtime logic, include lightweight tests (e.g., Vitest) in the same directory

## Container Architecture

### Image Consistency Rule

**Development and production images must maintain structural and functional consistency:**

| Aspect | Requirement |
|--------|-------------|
| **Base Image** | Same OS version (rust:1.80-slim-bookworm) |
| **System Dependencies** | Same apt packages installed |
| **Directory Structure** | Same file locations and paths |
| **Tool Installations** | Same tools: opencode, nvim, podman, mcp_task_server, etc. |
| **Functionality** | Same capabilities and behaviors |
| **Only Difference** | Development image uses domestic mirrors for faster builds in China |

**⚠️ CRITICAL:** All tools (opencode, nvim, podman, mcp_task_server, etc.) **MUST** be installed in the **Dockerfile** during image build, NOT at runtime. Containers are ephemeral - runtime changes are lost on restart.

### GitHub Registry Images (Production)

**⚠️ CRITICAL: Production images MUST be pulled from GHCR, never built locally**

This ensures environment consistency and security. All production images are built by GitHub Actions with official sources.

**Source Configuration: Use official sources only**
- apt: Official Debian repositories
- cargo/crates.io: Official registry
- npm: Official registry.npmjs.org

**Images:**
- `ghcr.io/oliveagle/vibe-kanban/base:v{version}` - Base image with all tools (shared between prod and dev)
- `ghcr.io/oliveagle/vibe-kanban/backend:latest` - Production backend with embedded frontend
- `ghcr.io/oliveagle/vibe-kanban/backend:v{version}` - Versioned production backend

**Production Deployment Commands:**
```bash
# Pull and run production image (recommended)
just prod-up

# Or manually:
just prod-pull [tag]        # Pull specific tag (default: dev)
just prod-pull latest       # Pull latest release
just prod-pull 0.0.148      # Pull specific version
just prod-wait-pull dev 30  # Wait for GitHub Actions and pull
just prod-up                # Start container
just prod-down              # Stop container
just prod-logs              # View logs
```

**Image Tags:**
- `dev` - Latest dev branch build (default for `just prod-pull`)
- `latest` - Latest release
- `0.0.148` - Specific version (only created for release tags)
- `<commit-sha>` - Specific commit build

**Why no local production builds?**
- Ensures all production deployments use identical, tested images
- Prevents "works on my machine" issues
- Security: Only CI/CD pipeline can create production images
- Traceability: Every production image has a Git commit SHA

### Development Images (Dockerfile.dev)

**Source Configuration: Use domestic mirrors when available**
- apt: Tsinghua University mirror (when `USE_MIRROR=true`)
- cargo/crates.io: Tsinghua sparse index (when `USE_MIRROR=true`)
- npm: Taobao mirror (when `USE_MIRROR=true`)

**Images:**
- `vibe-kanban:dev-base-v{version}` - Base layer with Rust + system deps + mirrors
- `vibe-kanban:dev-runtime-v{version}` - Runtime layer with cargo-watch + proxy config
- `vibe-kanban:local` - Production image pulled from GHCR (via `just prod-pull`)

### Agent Detection in Containers
- opencode requires `/root/.config/opencode/opencode.json` or `/root/.config/opencode/` directory to be detected as "installed".
- GitHub backend image has opencode binary at `/usr/local/bin/opencode`, but config must be created manually in container.

**Setup opencode in container:**
```bash
podman exec vibe-kanban-backend mkdir -p /root/.config/opencode
podman exec vibe-kanban-backend touch /root/.config/opencode/opencode.json
podman restart vibe-kanban-backend
```

### Data Persistence in Containers

Containers mount host directories for persistence. This applies to both development (`just dev-srv`) and production deployments:

**Mounted Volumes:**
- `$HOME/.local/share/vibe-kanban/` → `/root/.local/share/vibe-kanban/` (data directory)
- `/mnt/volume3/data` → `/mnt/volume3/data` (repositories)
- `/run/user/1000/podman/podman.sock` → `/var/run/docker.sock` (container socket)

**Data Directory Structure:**

All configuration is organized under `$HOME/.local/share/vibe-kanban/`:

```
$HOME/.local/share/vibe-kanban/
├── db.sqlite                    # VK database
├── config.json                  # VK configuration
├── credentials.json             # VK credentials
├── opencode.json                # MCP configuration for opencode
│
├── agents/                      # Agent-specific configurations
│   └── opencode/                # Opencode configuration
│       ├── config.json          # CLI configuration (auto-fixed from desktop)
│       └── mcp.json             # MCP configuration (symlink to ../../opencode.json)
│
└── cache/                       # Cache directories
    └── npm/                     # NPM cache
```

**Symlink Strategy:**

Standard agent config paths are symlinked to the VK data directory:
- `/root/.config/opencode` → `$VK_DATA_DIR/agents/opencode`

This ensures all persistent data is within the mounted volume for backup and migration.

**⚠️ SYMLINK RESTRICTION:**
Files or directories in `$HOME/.local/share/vibe-kanban/` that are symlinks pointing **outside** this directory will **NOT work** inside the container. The container can only access paths within the mounted volume.

**Check for problematic symlinks:**
```bash
./scripts/check-data-symlinks.sh
```

**Fix problematic symlinks:**
```bash
# Replace symlink with actual file
cp $(readlink symlink_name) symlink_name

# Or copy directory content
cp -rL $(readlink dir_name) dir_name
```

### Default Configuration

**Default Agent:** `opencode`
- Model: `kimi-for-coding`
- MCP: vibe_kanban server configured automatically
- Config location: `/root/.config/opencode/opencode.json`

**Default Editor:** `nvim` (Neovim)
- Installed in dev-runtime image
- Available at: `/usr/bin/nvim`
- Fallback: `vim` or `vi`

**Current tools installed in Dockerfile.dev:**
- `opencode` - via `npm install -g opencode-ai`
- `nvim` - via `apt-get install neovim`
- `podman` - via `apt-get install podman`
- `mcp_task_server` - built from source during image build

To use these defaults, ensure:
1. Container is started with proper volume mounts (see Container-Based Development below)
2. opencode.json is created with MCP configuration
3. Tools are installed in Dockerfile (see Dockerfile.dev)

**Current tools installed in Dockerfile.dev:**
- `opencode` - via `npm install -g opencode-ai`
- `nvim` - via `apt-get install neovim`
- `podman` - via `apt-get install podman`
- `mcp_task_server` - built from source during image build

To use these defaults, ensure:
1. Container is started with proper volume mounts (see Container-Based Development below)
2. opencode.json is created with MCP configuration
3. Tools are installed in Dockerfile (see Dockerfile.dev)

**Container Network Setup:**
- When using podman-compose, containers may not resolve each other by service name.
- Frontend nginx must proxy to `host.containers.internal:37825` (host port) instead of `backend:3000`.
- This is handled automatically in `nginx-frontend.conf`.

## Frontend Debugging

### Backend Debug Log API

When debugging frontend issues (especially WebSocket connections, real-time updates, or UI state problems), use the backend debug log API to send logs from the frontend to the backend logs.

**API Endpoint:** `POST /api/debug/log`

**Request Format:**
```json
{
  "level": "error|warn|info|debug",
  "message": "日志消息",
  "context": "可选的上下文信息"
}
```

**Example Usage:**
```javascript
// Send frontend error to backend logs
fetch('/api/debug/log', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    level: 'error',
    message: 'WebSocket connection failed',
    context: 'execution process: 7747af7a-c911-4e96-8587-910b6d7c8f1b'
  })
});
```

**When to use:**
- WebSocket connection errors
- Real-time update failures
- UI state synchronization issues
- Any frontend error that needs backend context

**Viewing logs:**
```bash
# View backend logs in real-time
just dev-srv-logs
# or
podman logs -f vibe-kanban-backend-dev
```

Look for `[FRONTEND]` prefix in the logs to identify frontend-generated log entries.

## Optional Features Configuration

### Discord Integration

Discord online count display is disabled by default. To enable it:

**Environment Variable:**
```bash
VITE_ENABLE_DISCORD=true
```

**Default:** `false` (disabled)

### PostHog Analytics

PostHog analytics is disabled by default. To enable it:

**Environment Variables:**
```bash
VITE_ENABLE_POSTHOG=true
VITE_POSTHOG_API_KEY=your_api_key
VITE_POSTHOG_API_ENDPOINT=https://your-instance.posthog.com
```

**Default:** `false` (disabled)

**Note:** All three variables must be set for analytics to work.

## Security & Config Tips
- Use `.env` for local overrides; never commit secrets. Key envs: `FRONTEND_PORT`, `BACKEND_PORT`, `HOST` 
- Dev ports and assets are managed by `scripts/setup-dev-environment.js`.

## Git Workflow & Branch Strategy

We follow **Git Flow** with three main branches:

### Branches

- **`feature/*`** - Feature development branches
  - Created from: `dev`
  - Merged to: `dev`
  - Naming: `feature/container-orchestration`, `feature/ui-improvements`

- **`dev`** - Development/Integration branch
  - Created from: `master`
  - Merged to: `master`
  - Auto-builds images with `:dev` tag
  - Used for testing and integration

- **`master`**/`**main**` - Production/Release branch
  - Protected branch
  - Requires version bump for PRs
  - Auto-builds images with `:latest` and version tags
  - Represents stable releases

### Workflow

```
feature/my-feature → dev → master/main
```

1. **Create feature branch** from `dev`:
   ```bash
   git checkout dev
   git pull origin dev
   git checkout -b feature/my-feature
   ```

2. **Develop and test** locally

3. **Create PR** to `dev` branch
   - No version bump required
   - Standard PR template

4. **After testing in dev**, create PR to `master`:
   - **Must bump version** in all package.json files
   - Follows semantic versioning (e.g., `0.0.145` → `0.0.146`)
   - Version check CI will validate

### Version Management

**Current Version:** `0.0.148`

All package.json files must maintain the same version number:
- `package.json` (root) - Source of truth
- `npx-cli/package.json` - Must match root
- `frontend/package.json` - Must match root

**Quick Version Bump:**
```bash
just bump-version 0.0.148   # Bump to new version
```

**Manual Version Update (if needed):**
1. Update root `package.json` version
2. Update `npx-cli/package.json` to match
3. Update `frontend/package.json` to match
4. Update version references in `justfile`, `docker-compose.local.yml`, `AGENTS.md`
5. Commit all changes together

**Release Workflow:**

```
feature/my-feature → dev → master → git tag v0.0.148
```

1. **Develop on feature branch**
   ```bash
   git checkout dev
   git pull origin dev
   git checkout -b feature/my-feature
   # ... develop ...
   git push origin feature/my-feature
   # Create PR to dev
   ```

2. **Merge to dev**
   - PR to `dev` branch
   - GitHub Actions auto-builds `:dev` tag
   - Test with: `just prod-pull dev`

3. **Prepare release (version bump)**
   ```bash
   git checkout dev
   git pull origin dev
   just bump-version 0.0.148
   git add -A
   git commit -m "chore: bump version to 0.0.148"
   git push origin dev
   # Create PR to master
   ```

4. **Create release tag**
   ```bash
   git checkout master
   git pull origin master
   git tag v0.0.148
   git push origin v0.0.148
   ```

5. **Wait for build and deploy**
   ```bash
   just prod-wait-pull 0.0.148 30
   just prod-up
   ```

**Version Number Rules:**
- Follow semantic versioning: `MAJOR.MINOR.PATCH`
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes
- Current project is pre-1.0, so we use `0.0.xxx`

### Container Image Versioning

**Production (master/main):**
- `ghcr.io/oliveagle/vibe-kanban/backend:latest` - Latest release
- `ghcr.io/oliveagle/vibe-kanban/backend:0.0.148` - Specific version

**Development (dev branch):**
- `ghcr.io/oliveagle/vibe-kanban/backend:dev` - Latest dev build
- `ghcr.io/oliveagle/vibe-kanban/backend:<commit-sha>` - Specific commit

**Base Image (shared):**
- `ghcr.io/oliveagle/vibe-kanban/base:v0.0.148` - Versioned base
- `ghcr.io/oliveagle/vibe-kanban/base:latest` - Latest base

**For rollbacks:**
```bash
# Use specific version instead of latest
just prod-pull 0.0.146
just prod-up
```

## Browser Automation in Backend Container

The backend container (`vibe-kanban-backend`) has browser automation tools installed for testing and debugging.

### Lightpanda Browser

Lightpanda is a lightweight, headless browser that supports CDP (Chrome DevTools Protocol).

**Installation:**
```bash
podman exec vibe-kanban-backend sh -c "
  export https_proxy=http://host.containers.internal:1080
  curl -L -o /usr/local/bin/lightpanda \
    https://github.com/lightpanda-io/browser/releases/download/nightly/lightpanda-x86_64-linux
  chmod a+x /usr/local/bin/lightpanda
"
```

**Start CDP server:**
```bash
podman exec vibe-kanban-backend /usr/local/bin/lightpanda serve --host 0.0.0.0 --port 9222
```

**Fetch page content:**
```bash
podman exec vibe-kanban-backend /usr/local/bin/lightpanda fetch --dump http://localhost:3000/projects
```

### Playwright with Lightpanda

Playwright is installed in `/opt` directory and can connect to Lightpanda via CDP.

**Installation:**
```bash
podman exec vibe-kanban-backend sh -c "
  cd /opt
  export https_proxy=http://host.containers.internal:1080
  npm install playwright
"
```

**Usage example:**
```javascript
const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.connectOverCDP('ws://localhost:9222/');
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto('http://localhost:3000/projects');
  // Note: screenshot is not yet supported by Lightpanda
  const content = await page.content();
  console.log(content);
  await browser.close();
})();
```

**Limitations:**
- Lightpanda is still in development, some features like screenshots are not yet supported
- WebSocket is not fully supported in Lightpanda
- For full Playwright features, consider using the standard Playwright with Chromium

## Container-Based Development Environment

### Architecture (v0.0.146+)

For container-based development, a single backend container serves both API and frontend (embedded via rust_embed):

```
┌─────────────────────────────────────────────────────────────────┐
│                         Host Machine                             │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              vibe-kanban-dev Network                    │   │
│  │  ┌──────────────────────────────────────────────────┐  │   │
│  │  │         Unified Backend Container                 │  │   │
│  │  │                                                   │  │   │
│  │  │  ┌─────────────┐      ┌──────────────────────┐  │  │   │
│  │  │  │   Frontend  │      │   Backend API        │  │  │   │
│  │  │  │   (embedded)│      │   Port: 3000         │  │  │   │
│  │  │  └─────────────┘      └──────────────────────┘  │  │   │
│  │  │                                                   │  │   │
│  │  │  Image: vibe-kanban:dev-runtime-v0.0.146         │  │   │
│  │  └──────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

**Note:** As of v0.0.146, the separate frontend container has been removed. The backend serves the frontend via rust_embed on the same port (3000).

### Container Network Configuration

The backend container joins the shared network `vibe-kanban-dev`:

```bash
# Create network (if not exists)
podman network create vibe-kanban-dev

# Access the unified service:
# http://localhost:3000 (both frontend and API)
```

### Required Volume Mounts

**Backend Container (`dev-srv`):**
- `/app` (codebase): Read-write for hot-reload
- `/repos` (optional): Read-only access to local repos
- `/var/tmp`: Read-write for workspace/worktree storage
- `/run/user/1000/podman/podman.sock` (optional): Docker/Podman socket for container orchestration

### Port Mappings

- **Unified Service**: Host port 3000 → Container port 3000 (serves both frontend and API)

The port should be accessible from external machines if needed.

### Development Commands

```bash
# Build development images (first time only)
just dev-build-all

# Start unified backend container (serves both API and frontend)
just dev-srv -d

# Stop dev container
just dev-srv-stop
```

### Frontend Build Optimization

The frontend supports code splitting via manual chunks:

- `vendor-react`: Core React libraries
- `vendor-ui`: UI animation and component libraries
- `vendor-data`: Data management (React Query, Zustand)
- `vendor-editor`: Code editor components
- `vendor-form`: Form handling libraries

Routes are lazy-loaded using `React.lazy()` for better initial load performance.

### WebSocket Configuration

WebSocket connections from browser → backend flow:

```
Browser → Backend Container (3000)
        (HTTP/WS on same origin)
```

**Note:** As of v0.0.146, WebSocket uses the same origin as the frontend (port 3000). No cross-container communication needed.

## Container Network Proxy Setup

When running containers that need internet access (e.g., for downloading dependencies), you can use the host's proxy via `host.containers.internal`.

### Using Proxy in Containers

**Proxy URL:** `http://host.containers.internal:1080`

**Correct way to start backend dev container (with privileged mode for podman):**
```bash
podman run -d \
  --name vibe-kanban-backend-dev \
  --privileged \
  -p 3000:3000 \
  -v /mnt/volume3/data:/mnt/volume3/data:rw \
  -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
  -e HOST=0.0.0.0 \
  -e PORT=3000 \
  -e VIBE_BACKEND_URL=http://localhost:3000 \
  localhost/vibe-kanban:dev-runtime-v0.0.148 \
  sh -c "echo 'root:100000:65536' > /etc/subuid && \
         echo 'root:100000:65536' > /etc/subgid && \
         mkdir -p /root/.config/opencode && \
         cat > /root/.config/opencode/opencode.json << 'EOF'
{\n  \"\$schema\": \"https://opencode.ai/config.json\",\n  \"model\": \"kimi-for-coding\",\n  \"mcp\": {\n    \"vibe_kanban\": {\n      \"type\": \"local\",\n      \"command\": [\n        \"/usr/local/bin/mcp_task_server\"\n      ],\n      \"environment\": {\n        \"VIBE_BACKEND_URL\": \"http://localhost:3000\"\n      }\n    }\n  }\n}\nEOF && \
         cd /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban && \
         cargo run --release --bin server"
```

**Notes:**
- `--privileged` is required for podman to work inside the container
- `/mnt/volume3/data` is mounted to access all repos and data
- subuid/subgid configuration is required for rootless podman
- opencode.json is automatically created with MCP server configuration
- This setup allows the VK backend to use `pull_image` MCP tool with proxy support

**Example - Running backend dev container with proxy and Tsinghua mirror (OLD - for reference only):**
```bash
podman run -d \
  --name vibe-kanban-backend-dev \
  --user root \
  -p 3000:3000 \
  -e https_proxy=http://host.containers.internal:1080 \
  -e http_proxy=http://host.containers.internal:1080 \
  -v /path/to/vibe-kanban:/app:rw \
  -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
  --workdir /app \
  localhost/vibe-kanban:dev-runtime-v0.0.146 \
  sh -c "sed -i 's/archive.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list && \
         sed -i 's/security.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list && \
         apt-get update && \
         apt-get install -y cargo pkg-config libssl-dev libsqlite3-dev libclang-dev clang && \
         cargo install cargo-watch && \
         cargo watch -w crates -x 'run --bin server'"
```

**Example - Installing packages with proxy:**
```bash
podman exec vibe-kanban-backend sh -c "
  export https_proxy=http://host.containers.internal:1080
  apt-get update && apt-get install -y some-package
"
```

**Note:** This assumes you have a proxy service (like trojan-go) running on the host at port 1080.
