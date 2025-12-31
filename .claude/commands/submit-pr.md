---
description: Push branch and create pull request for review
---

# Submit Pull Request for Review

You are pushing a feature branch and creating a pull request for review. This is part of the "cattle" worktree workflow.

## Prerequisites Check

1. Verify you are in a worktree (not main repo):
   - `.git` should be a file, not a directory
   - If in main repo, inform the user this command only works from worktrees

2. Check for clean working directory:
   ```bash
   git status --porcelain
   ```
   - If dirty, ask user to commit changes first (uncommitted work won't be in the PR)

3. Verify there are commits to push:
   ```bash
   git fetch origin main
   git log origin/main..HEAD --oneline
   ```
   - If no commits, inform user there's nothing to submit

## Execution Flow

### Step 1: Get branch name

```bash
git rev-parse --abbrev-ref HEAD
```

### Step 2: Push branch to remote

```bash
git push -u origin <branch-name>
```

If the push fails due to divergence, ask user how to proceed:
- Force push (if they're okay losing remote changes)
- Pull and merge first
- Abort and resolve manually

### Step 3: Gather PR information

Collect information for the PR:

1. **Linear Issue ID**: Extract from branch name if present
   - Pattern: `HEY-\d+` at the start of branch name (e.g., `HEY-123-fix-login`)
   - Example: Branch `HEY-42-audio-improvements` → Issue ID `HEY-42`
   - If found, this will be added to PR body for auto-linking with Linear

2. **Title**: Derive from one of:
   - Linear issue title (if branch follows `HEY-123-*` pattern, fetch from Linear)
   - Branch name (convert `feature/dark-mode` → "Feature: Dark mode")
   - Ask user if unclear

3. **Summary**: Summarize the changes by looking at:
   ```bash
   git log origin/main..HEAD --format="%s"
   ```
   Create 1-3 bullet points describing what changed

4. **Test plan**: Ask user for test plan, or suggest based on changes

### Step 4: Create the PR

```bash
gh pr create --title "<title>" --body "$(cat <<'EOF'
## Summary
<bullet points>

Closes HEY-123

## Test plan
<test instructions>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

**Note**: Only include the `Closes HEY-xxx` line if a Linear issue ID was extracted in Step 3. This line auto-links the PR to the Linear issue and will close the issue when the PR is merged.

**IMPORTANT:** Before running `gh pr create`, verify with the user:
- Is the title correct?
- Is the summary accurate?
- Is the test plan complete?

### Step 5: Report success

After PR creation:
1. Print the PR URL
2. Remind user of next steps:
   - Worktree stays alive for review fixes
   - Push additional commits if changes requested
   - Run `/close-worktree` after PR is merged

## PR Body Format

Use this format for the PR body:

```markdown
## Summary
- <Main change 1>
- <Main change 2>
- <Main change 3 if applicable>

Closes HEY-123

## Test plan
- [ ] <Test step 1>
- [ ] <Test step 2>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

**Linear Integration**: The `Closes HEY-xxx` line links the PR to the Linear issue. Only include this if the branch name contains a Linear issue ID (e.g., `HEY-123-fix-audio`). You can also use `Fixes HEY-xxx` or just `HEY-xxx` (without auto-close).

## Handling Review Feedback

After creating the PR, if the user needs to make changes:
1. Make the changes in the worktree
2. Commit the changes
3. Push to the same branch:
   ```bash
   git push
   ```
4. The PR will automatically update

## Notes

- The PR targets the `main` branch by default
- GitHub will handle squash-and-merge when the PR is approved
- The worktree stays alive during review so you can make fixes
- After the PR is merged, run `/close-worktree` to clean up

## Troubleshooting

**"This command only works from worktrees"**
- You're in the main repo. Navigate to your worktree first.

**"No commits to push"**
- Your branch is up to date with main. Make some changes first.

**"gh: command not found"**
- Install GitHub CLI: `brew install gh`
- Authenticate: `gh auth login`

**"Push rejected (non-fast-forward)"**
- Remote has changes you don't have locally
- Pull first: `git pull --rebase origin <branch-name>`
- Then try pushing again

**"Pull request already exists"**
- A PR already exists for this branch
- View it: `gh pr view`
- Push additional commits to update it
