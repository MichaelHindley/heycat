---
description: Delete worktree and clean up after PR is merged
---

# Close Worktree After PR Merged

You are deleting a worktree and cleaning up all associated data. This should be done after the PR has been merged.

## Prerequisites Check

1. Verify you are in a worktree (not main repo):
   - `.git` should be a file, not a directory
   - If in main repo, inform the user this command only works from worktrees

2. Check for clean working directory:
   ```bash
   git status --porcelain
   ```
   - If dirty, warn user that uncommitted changes will be lost
   - Ask for confirmation before proceeding

## Execution Flow

### Step 1: Check PR status (recommended)

Check if there's a PR for this branch and its status:

```bash
gh pr view --json state,mergedAt,url 2>/dev/null
```

**If PR exists and is merged:**
- Proceed with deletion

**If PR exists but NOT merged:**
- Warn user: "PR is not yet merged. Deleting the worktree will make it harder to make changes."
- Ask for explicit confirmation before proceeding
- Suggest they wait until PR is merged

**If no PR exists:**
- Warn user: "No PR found for this branch."
- Ask if they want to proceed anyway (maybe changes were abandoned)

### Step 2: Store paths before deletion

Get the main repo path (will be needed for navigation after deletion):
- The script will detect this automatically

### Step 3: Delete worktree and clean up

```bash
bun scripts/close-worktree.ts
```

This script will:
1. Detect worktree context (identifier, main repo path)
2. Run `git worktree remove <path>` from the main repo
3. Delete data directories:
   - `~/.local/share/heycat-<identifier>/`
   - `~/.config/heycat-<identifier>/`
   - `~/Library/Application Support/com.heycat.app/settings-<identifier>.json`
4. Print navigation instructions

Use `--force` flag to skip confirmation:
```bash
bun scripts/close-worktree.ts --force
```

### Step 4: Navigate to main repository

After successful deletion, you must navigate to the main repository since the current directory no longer exists:

```bash
cd <main-repo-path>
```

The script will print the exact path to use.

### Step 5: Verify cleanup

From the main repository:
```bash
git worktree list
```

The deleted worktree should no longer appear in the list.

## What Gets Deleted

| Location | Content |
|----------|---------|
| `worktrees/heycat-<id>/` | Git worktree directory |
| `~/.local/share/heycat-<id>/` | Application data (models, recordings, database) |
| `~/.config/heycat-<id>/` | Configuration files |
| `~/Library/.../settings-<id>.json` | Worktree-specific settings |

## Notes

- This is part of the "cattle" worktree model - worktrees are ephemeral
- The branch is NOT deleted from the remote - it stays as part of the merged PR
- If deletion fails, the script will provide manual cleanup instructions
- For orphaned data from old worktrees, use: `bun scripts/cleanup-worktree.ts --orphaned`

## Troubleshooting

**"This script must be run from a worktree"**
- You're in the main repo. Navigate to the worktree you want to close first.

**"Failed to remove git worktree"**
- The worktree may have uncommitted changes
- Use force removal: `git worktree remove --force <path>`

**"Working directory has uncommitted changes"**
- Commit or discard changes before closing
- Or use `--force` to delete anyway (changes will be lost)

**After deletion, shell shows error**
- Your current directory was deleted
- Run the `cd` command printed by the script

**Some data directories weren't deleted**
- Run `bun scripts/cleanup-worktree.ts --orphaned` to clean up leftovers
