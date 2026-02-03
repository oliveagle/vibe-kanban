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

## Coding Style & Naming Conventions
- Rust: `rustfmt` enforced (`rustfmt.toml`); group imports by crate; snake_case modules, PascalCase types.
- TypeScript/React: ESLint + Prettier (2 spaces, single quotes, 80 cols). PascalCase components, camelCase vars/functions, kebab-case file names where practical.
- Keep functions small, add `Debug`/`Serialize`/`Deserialize` where useful.

## Testing Guidelines
- Rust: prefer unit tests alongside code (`#[cfg(test)]`), run `cargo test --workspace`. Add tests for new logic and edge cases.
- Frontend: ensure `pnpm run check` and `pnpm run lint` pass. If adding runtime logic, include lightweight tests (e.g., Vitest) in the same directory.

## Container Architecture

### GitHub Registry Images (Production)
- `ghcr.io/oliveagle/vibe-kanban/backend:latest` - Backend API only. **Note**: Compiled with placeholder HTML if frontend/dist missing. Serves API on port 3000.
- `ghcr.io/oliveagle/vibe-kanban/frontend:latest` - Frontend static files only. Nginx serves on port 80, proxies `/api` to backend service.
- **Must deploy together**: Frontend container proxies API requests to backend container via docker-compose internal network.

### Local Image (Development)
- `vibe-kanban:local` - Combined image with both frontend and backend. Use `just container-build` to create.
- Single container serves both frontend (port 80) and API (port 3000).

### Agent Detection in Containers
- opencode requires `/root/.config/opencode/opencode.json` or `/root/.config/opencode/` directory to be detected as "installed".
- GitHub backend image has opencode binary at `/usr/local/bin/opencode`, but config must be created manually in container.

**Setup opencode in container:**
```bash
podman exec vibe-kanban-backend mkdir -p /root/.config/opencode
podman exec vibe-kanban-backend touch /root/.config/opencode/opencode.json
podman restart vibe-kanban-backend
```

**Container Network Setup:**
- When using podman-compose, containers may not resolve each other by service name.
- Frontend nginx must proxy to `host.containers.internal:37825` (host port) instead of `backend:3000`.
- This is handled automatically in `nginx-frontend.conf`.

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

All package.json files must maintain the same version number:
- `package.json` (root) - Source of truth
- `npx-cli/package.json` - Must match root
- `frontend/package.json` - Must match root

**When updating version:**
1. Update root `package.json` version
2. Update `npx-cli/package.json` to match
3. Update `frontend/package.json` to match
4. Commit all changes together

### Container Image Versioning

**Production (master/main):**
- `ghcr.io/oliveagle/vibe-kanban/backend:latest`
- `ghcr.io/oliveagle/vibe-kanban/backend:0.0.145`
- `ghcr.io/oliveagle/vibe-kanban/frontend:latest`
- `ghcr.io/oliveagle/vibe-kanban/frontend:0.0.145`

**Development (dev branch):**
- `ghcr.io/oliveagle/vibe-kanban/backend:dev`
- `ghcr.io/oliveagle/vibe-kanban/frontend:dev`

**For rollbacks:**
```bash
# Use specific version instead of latest
podman pull ghcr.io/oliveagle/vibe-kanban/backend:0.0.145
podman pull ghcr.io/oliveagle/vibe-kanban/frontend:0.0.145
```

**Local development images:**
- Tag local builds with version: `vibe-kanban:local-0.0.145`
- Keep `vibe-kanban:local` as latest local build

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

## Container Network Proxy Setup

When running containers that need internet access (e.g., for downloading dependencies), you can use the host's proxy (trojan-go) via `host.containers.internal`.

### Using Proxy in Containers

**Proxy URL:** `http://host.containers.internal:1080`

**Example - Running backend dev container with proxy and Tsinghua mirror:**
```bash
podman run -d \
  --name vibe-kanban-backend-dev \
  --user root \
  -p 3001:3001 \
  -e https_proxy=http://host.containers.internal:1080 \
  -e http_proxy=http://host.containers.internal:1080 \
  -v /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban:/app:rw \
  -v /run/user/1000/podman/podman.sock:/var/run/docker.sock:ro \
  --workdir /app \
  localhost/vibe-kanban:local-backend \
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
