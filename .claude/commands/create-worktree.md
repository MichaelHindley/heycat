---
description: Create a new ephemeral worktree for feature development
---

# Create Worktree for Feature Development

You are creating a new ephemeral worktree for developing a feature. This is part of the "cattle" worktree model - worktrees are created per-feature and deleted after the PR is merged.

**IMPORTANT**: All development must go through Linear. Freeform branch names are not allowed.

## Prerequisites Check

1. Verify you are in the main repository (not a worktree):
   - `.git` should be a directory, not a file
   - If already in a worktree, inform the user this command only works from the main repository

2. Check for clean working directory:
   ```bash
   git status --porcelain
   ```
   - If dirty, ask user to commit or stash changes first

## Execution Flow

### Step 1: Identify the Linear issue

**Ask the user for the Linear issue slug or identifier.** Examples:
- Issue slug: `docker-development-workflow`
- Issue identifier: `HEY-156`

If the user doesn't have a Linear issue yet, direct them to create one first using `/devloop:agile:issue` or `/devloop:agile:quick`.

### Step 2: Fetch latest main

```bash
git fetch origin main
```

### Step 3: Create the worktree

```bash
bun scripts/create-worktree.ts --issue <issue-slug-or-id>
```

This script will:
1. Validate the issue exists in Linear
2. Get the HEY-### identifier from Linear
3. Create branch with format: `HEY-###-<issue-title-slug>`
4. Create a git worktree at `worktrees/<branch-name>/`
5. Generate a unique hotkey and dev port for the worktree
6. Create a settings file with the unique hotkey

Example:
```bash
bun scripts/create-worktree.ts --issue docker-development-workflow
# Creates branch: HEY-156-docker-development-workflow
```

### Step 4: Navigate to the worktree

After creation, navigate to the worktree:

```bash
cd worktrees/<branch-name>
```

### Step 5: Install dependencies

```bash
bun install
```

### Step 6: Report success

Print the worktree details:
- Worktree path
- Branch name (with HEY-### prefix)
- Assigned hotkey
- Dev port
- Next steps for development

## Cattle Workflow Reminder

Remind the user of the full workflow:
1. `/create-worktree` - You are here
2. Develop feature, commit changes
3. `/submit-pr` - Push and create PR when ready for review
4. Make fixes if needed during review
5. `/close-worktree` - Delete worktree after PR is merged

## Notes

- Each worktree gets a unique dev port (1421-1429) so multiple instances can run simultaneously
- Each worktree gets a unique recording hotkey to avoid conflicts
- Data is stored in isolated directories (`~/.local/share/heycat-<id>/`)
- The worktree should be deleted after the PR is merged using `/close-worktree`

## Linear Integration

When the branch name starts with a Linear issue ID (e.g., `HEY-156-docker-development-workflow`):
- `/submit-pr` will automatically include `Closes HEY-156` in the PR body
- The PR will appear linked in the Linear issue
- When the PR is merged, the Linear issue will auto-close

This is why all branches MUST be created through a Linear issue - it ensures proper cross-linking.

## Troubleshooting

**"This command only works from the main repository"**
- You're in a worktree. Navigate to the main repository first.

**"Issue not found in Linear"**
- Verify the issue slug or identifier is correct
- Check that LINEAR_API_KEY is set in your environment

**"Branch already exists"**
- The branch was already created for this issue. Either:
  - Use the existing worktree
  - Delete the old branch: `git branch -D <branch-name>`

**"Path already exists"**
- A worktree directory already exists. Either remove it first or choose a different name.

**"Working directory is not clean"**
- Commit or stash your changes before creating a worktree.
