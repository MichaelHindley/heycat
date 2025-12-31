---
description: Create a Docker development container for feature development
---

# Create Docker Container for Feature Development

You are creating a new Docker development container for developing a feature. This is part of the "cattle" container model - containers are created per-feature and deleted after the PR is merged.

## Prerequisites Check

1. Verify Docker is running:
   ```bash
   docker info >/dev/null 2>&1 && echo "Docker is running" || echo "Docker is not running"
   ```
   - If not running, ask user to start Docker Desktop or Docker daemon

2. Verify you are in the project root (docker-compose.yml exists):
   ```bash
   test -f docker-compose.yml && echo "Found docker-compose.yml" || echo "Not in project root"
   ```

## Execution Flow

### Step 1: Determine branch name

1. **Check for Linear issue**: Ask the user if they have a Linear issue ID (e.g., `HEY-123`)
   - If yes, ask for a short description (2-3 words, kebab-case)
   - Generate branch name: `HEY-123-short-description`

2. **Alternative**: If no Linear issue, suggest these formats:
   - For features: `feature/<name>` (e.g., `feature/dark-mode`)
   - For bugfixes: `fix/<name>` (e.g., `fix/memory-leak`)

**Preferred format for Linear issues**: `HEY-<id>-<description>`
- Example: `HEY-42-audio-improvements`
- This enables automatic PR linking in Linear when using `/submit-pr`

### Step 2: Verify SSH agent

For git operations inside the container, SSH agent forwarding should work:
```bash
ssh-add -l || echo "No SSH keys loaded"
```

Warn user if no SSH keys are loaded - they may have issues pushing/pulling inside the container.

### Step 3: Create the container

```bash
bun scripts/docker/create-container.ts <branch-name>
```

This script will:
1. Build the Docker image (if not already built)
2. Start a container with the branch name as ID
3. Create a new branch inside the container
4. Run `bun install` for dependencies
5. Print access instructions

### Step 4: Access the container

After creation, access the container:

```bash
docker exec -it heycat-dev-<container-id> bash
```

Or start Claude Code inside:
```bash
docker exec -it heycat-dev-<container-id> claude
```

### Step 5: Report success

Print the container details:
- Container name
- Branch name
- How to access the container
- How to run tests
- How to trigger macOS builds

## Cattle Workflow Reminder

Remind the user of the full workflow:
1. `/create-container` - You are here
2. Develop feature, commit changes (inside container)
3. `/mac-build` - Build Tauri app on macOS host when needed
4. `/submit-pr` - Push and create PR when ready for review
5. Make fixes if needed during review
6. `/close-container` - Delete container after PR is merged

## Docker vs Worktree

| Aspect | Docker Container | Git Worktree |
|--------|------------------|--------------|
| Use case | Cloud/remote development | Local macOS development |
| macOS builds | Via `/mac-build` (rsync + SSH) | Direct via `bun run tauri dev` |
| Isolation | Full OS-level isolation | Git-level isolation |
| Prerequisites | Docker | macOS with Rust/Swift toolchain |

## Linear Integration

When the branch name starts with a Linear issue ID (e.g., `HEY-123-fix-audio`):
- `/submit-pr` will automatically include `Closes HEY-123` in the PR body
- The PR will appear linked in the Linear issue
- When the PR is merged, the Linear issue will auto-close

## Troubleshooting

**"Docker is not running"**
- Start Docker Desktop or run `sudo systemctl start docker`

**"Failed to build Docker image"**
- Check Dockerfile.dev for errors
- Ensure you have enough disk space

**"Container already exists"**
- Remove the existing container: `docker rm -f heycat-dev-<id>`
- Or use a different branch name

**"SSH agent not forwarded"**
- Ensure `SSH_AUTH_SOCK` is set
- On macOS, the docker-compose.yml should handle this automatically
