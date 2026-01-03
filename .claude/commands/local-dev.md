---
description: Local Docker Desktop development workflow (no SSH required)
---

# Local Docker Development

You are helping the user develop with Docker Desktop on their local macOS machine. This workflow uses bind mounts - files are shared between the container and host automatically, so no SSH or rsync is needed.

> **Remote Docker host?** Use `/create-container` and `/mac-build` instead.

## When to Use

- Local macOS with Docker Desktop installed
- Want isolation for tests/linting but native macOS builds
- Simpler than the full remote Docker workflow

## Quick Start

```bash
bun scripts/docker/local-dev.ts --shell
```

This starts the container (if needed) and opens an interactive shell.

## Commands

| Command | Purpose |
|---------|---------|
| `--shell` or `-s` | Start container and enter bash shell |
| `--build` or `-b` | Run Tauri release build on host |
| `--dev` or `-d` | Start Tauri dev server on host |
| `--stop` | Stop the container |
| `--help` | Show help |

## Workflow

### 1. Start Development

```bash
bun scripts/docker/local-dev.ts --shell
```

### 2. In Container: Tests and Linting

```bash
# Run TypeScript tests
bun test

# Run Rust tests
cd src-tauri && cargo test

# Lint
bun run lint
```

### 3. On Host: Tauri Builds

Exit the container (or use another terminal) and run on your Mac:

```bash
# Development server with hot reload
bun scripts/docker/local-dev.ts --dev

# Release build
bun scripts/docker/local-dev.ts --build
```

Files are bind-mounted, so changes made in the container are immediately available on the host.

### 4. Stop When Done

```bash
bun scripts/docker/local-dev.ts --stop
```

## How It Works

The docker-compose.yml includes a bind mount:

```yaml
volumes:
  - .:/workspace:cached
```

This shares the source code between container and host:
- Edit in container → changes appear on host
- Edit on host → changes appear in container
- No rsync or SSH needed

## Local vs Remote Docker

| Aspect | Local (this command) | Remote (`/mac-build`) |
|--------|---------------------|----------------------|
| File sync | Bind mount (automatic) | rsync over SSH |
| macOS builds | Direct on host | Triggered via SSH |
| SSH required | No | Yes |
| Use case | Docker Desktop on Mac | Cloud/remote Docker |

## Troubleshooting

### "Docker is not running"

Start Docker Desktop or verify Docker daemon is running.

### "Container won't start"

Check if the image is built:

```bash
docker images | grep heycat
```

If not, build it:

```bash
docker compose build
```

### "Permission denied on files"

Ensure the container user matches your host user:

```bash
# In docker-compose.yml, these should match your host user
USER_ID: $(id -u)
GROUP_ID: $(id -g)
```

### "Tauri build fails"

Ensure you have the prerequisites on your Mac:

```bash
# Check Rust
rustc --version

# Check Xcode CLI tools
xcode-select --print-path
```

## Related Commands

- `/create-container` - For remote Docker development
- `/mac-build` - For remote Docker macOS builds
- `/close-container` - Close a remote container
