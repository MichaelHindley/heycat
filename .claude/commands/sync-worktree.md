---
description: Update worktree with latest changes from origin/main via rebase
---

# Sync Worktree with Origin Main

You are updating a worktree branch with the latest changes from origin/main using rebase.

## Prerequisites Check

1. Verify you are in a worktree (not main repo):
   - `.git` should be a file, not a directory
   - If in main repo, inform the user this command only works from worktrees

2. Check for clean working directory:
   ```bash
   git status --porcelain
   ```
   - If dirty, ask user to commit or stash changes first

## Execution Flow

### Step 1: Fetch and rebase

```bash
git fetch origin main
git rebase origin/main
```

### Step 2: Handle conflicts (if any)

**CRITICAL: DO NOT resolve conflicts automatically. Human approval is REQUIRED.**

If the rebase reports conflicts, you MUST:

1. **STOP** - Do not proceed with resolution automatically

2. **SHOW the user the conflicts** - For each conflicting file:
   - Display the file path
   - Show the conflict markers with both versions
   - Explain what "ours" (your current branch) contains
   - Explain what "theirs" (origin/main) contains

3. **ASK the user for resolution** - Present options like:
   - "Keep our version (current branch)"
   - "Keep their version (origin/main)"
   - "Merge both changes (explain how)"
   - "Let me resolve manually"

4. **WAIT for explicit user approval** before making any changes

5. **Only after user approves**, implement their chosen resolution:
   - Make the approved changes
   - Stage resolved files:
     ```bash
     git add <files>
     ```
   - Continue the rebase:
     ```bash
     git rebase --continue
     ```
   - If more conflicts appear, repeat from step 1

**Why this matters:** Automatic conflict resolution can silently discard important changes. The user must understand and approve how conflicts are resolved.

### Step 3: Verify success

After successful rebase:

1. Show the commits that were rebased:
   ```bash
   git log --oneline origin/main..HEAD
   ```
2. Confirm the branch is now based on the latest main

## Notes

- This rebases your current branch onto the latest origin/main
- Your commits will be replayed on top of main's latest state
- No commits are modified on main - only your branch is updated

## Troubleshooting

**"This command only works from worktrees"**
- You're in the main repo. Navigate to your worktree directory first.

**"Working directory is not clean"**
- Commit or stash your changes before syncing.

**"Cannot rebase: You have unstaged changes"**
- Same as above - commit or stash first.

**Rebase aborted mid-way**
- Run `git rebase --abort` to return to the state before the rebase
- Then try again after addressing the issue
