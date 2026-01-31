# Container Deployment Guide

## Overview

This project provides containerized deployment using **Podman** with separate frontend and backend services.

## Images

- **Backend**: `ghcr.io/oliveagle/vibe-kanban/backend:latest`
- **Frontend**: `ghcr.io/oliveagle/vibe-kanban/frontend:latest`

## Quick Start

### Production Deployment

```bash
# Pull and run pre-built images
podman-compose up -d

# Or build locally and run
podman-compose up -d --build
```

Access the application at:
- **Frontend**: http://localhost
- **Backend API**: http://localhost:3000

### Development Mode (Hot Reload)

```bash
# Run with source code mounting for live development
podman-compose -f podman-compose.dev.yml up
```

Access:
- **Frontend Dev**: http://localhost:3001 (Vite HMR)
- **Backend API**: http://localhost:3000

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
# Optional: Override default ports
FRONTEND_PORT=80
BACKEND_PORT=3000

# Optional: Analytics (PostHog)
VITE_PUBLIC_POSTHOG_KEY=your-key
VITE_PUBLIC_POSTHOG_HOST=your-host
```

### Volume Mounts

The compose files mount the following directories:

| Host Path | Container Path | Purpose |
|-----------|----------------|---------|
| `/mnt/volume3/data/repos` | `/repos` | Git repositories |
| `~/.ssh` | `/app/.ssh` | SSH keys for git |
| `~/.gitconfig` | `/app/.gitconfig` | Git configuration |
| `~/.config/gh` | `/app/.config/gh` | GitHub CLI config |

## Building Images Manually

### Backend

```bash
podman build -f Dockerfile.backend -t vibe-kanban/backend:latest .
```

### Frontend

```bash
podman build -f Dockerfile.frontend -t vibe-kanban/frontend:latest .
```

## GitHub Container Registry

Images are automatically built and pushed to GHCR on every push to `main` or `master` branch.

To pull images directly:

```bash
# Login to GHCR (requires GitHub token)
podman login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin

# Pull images
podman pull ghcr.io/oliveagle/vibe-kanban/backend:latest
podman pull ghcr.io/oliveagle/vibe-kanban/frontend:latest
```

## Troubleshooting

### Permission Issues

If you encounter permission errors with SSH keys:

```bash
# Ensure correct permissions on host
chmod 600 ~/.ssh/id_rsa
chmod 644 ~/.ssh/id_rsa.pub
```

### Network Issues

If the frontend cannot connect to the backend:

1. Check both containers are running: `podman ps`
2. Verify network connectivity: `podman network ls`
3. Check logs: `podman logs vibe-kanban-backend`

### Development Mode

For development mode, ensure your source code is readable by the container user:

```bash
# Fix permissions if needed
chmod -R u+rwx ./frontend ./crates
```
