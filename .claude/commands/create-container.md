---
description: Create a Docker development container for feature development
---

# Create Docker Container for Feature Development

You are creating a new Docker development container for developing a feature. This is part of the "cattle" container model - containers are created per-feature and deleted after the PR is merged.

**IMPORTANT**: All development must go through Linear. Freeform branch names are not allowed.

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

### Step 1: Identify the Linear issue

**Ask the user for the Linear issue slug or identifier.** Examples:
- Issue slug: `docker-development-workflow`
- Issue identifier: `HEY-156`

If the user doesn't have a Linear issue yet, direct them to create one first using `/devloop:agile:issue` or `/devloop:agile:quick`.

### Step 2: Verify SSH agent

For git operations inside the container, SSH agent forwarding should work:
```bash
ssh-add -l || echo "No SSH keys loaded"
```

Warn user if no SSH keys are loaded - they may have issues pushing/pulling inside the container.

### Step 3: Create the container

```bash
bun scripts/docker/create-container.ts --issue <issue-slug-or-id>
```

This script will:
1. Validate the issue exists in Linear
2. Get the HEY-### identifier from Linear
3. Build the Docker image (if not already built)
4. Start a container with branch format: `HEY-###-<issue-title-slug>`
5. Create a new branch inside the container
6. Run `bun install` for dependencies
7. Print access instructions

Example:
```bash
bun scripts/docker/create-container.ts --issue docker-development-workflow
# Creates branch: HEY-156-docker-development-workflow
```

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
- Branch name (with HEY-### prefix)
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

When the branch name starts with a Linear issue ID (e.g., `HEY-156-docker-development-workflow`):
- `/submit-pr` will automatically include `Closes HEY-156` in the PR body
- The PR will appear linked in the Linear issue
- When the PR is merged, the Linear issue will auto-close

This is why all branches MUST be created through a Linear issue - it ensures proper cross-linking.

## Troubleshooting

**"Docker is not running"**
- Start Docker Desktop or run `sudo systemctl start docker`

**"Issue not found in Linear"**
- Verify the issue slug or identifier is correct
- Check that LINEAR_API_KEY is set in your environment

**"Failed to build Docker image"**
- Check Dockerfile.dev for errors
- Ensure you have enough disk space

**"Container already exists"**
- Remove the existing container: `docker rm -f heycat-dev-<id>`
- Or the branch was already created for this issue

**"SSH agent not forwarded"**
- Ensure `SSH_AUTH_SOCK` is set
- On macOS, the docker-compose.yml should handle this automatically
