---
description: Create a new ephemeral worktree for feature development
---

# Create Worktree for Feature Development

You are creating a new ephemeral worktree for developing a feature. This is part of the "cattle" worktree model - worktrees are created per-feature and deleted after the PR is merged.

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

### Step 2: Fetch latest main

```bash
git fetch origin main
```

### Step 3: Create the worktree

```bash
bun scripts/create-worktree.ts <branch-name>
```

This script will:
1. Create a git worktree at `worktrees/heycat-<branch-name>/`
2. Create a new branch from current HEAD
3. Generate a unique hotkey and dev port for the worktree
4. Create a settings file with the unique hotkey

### Step 4: Navigate to the worktree

After creation, navigate to the worktree:

```bash
cd worktrees/heycat-<branch-name>
```

### Step 5: Install dependencies

```bash
bun install
```

### Step 6: Report success

Print the worktree details:
- Worktree path
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

When the branch name starts with a Linear issue ID (e.g., `HEY-123-fix-audio`):
- `/submit-pr` will automatically include `Closes HEY-123` in the PR body
- The PR will appear linked in the Linear issue
- When the PR is merged, the Linear issue will auto-close

## Troubleshooting

**"This command only works from the main repository"**
- You're in a worktree. Navigate to the main repository first.

**"Branch already exists"**
- Choose a different branch name, or use the existing branch with:
  ```bash
  git worktree add worktrees/heycat-<branch-name> <branch-name>
  ```

**"Path already exists"**
- A worktree directory already exists. Either remove it first or choose a different name.

**"Working directory is not clean"**
- Commit or stash your changes before creating a worktree.
