#!/bin/bash
set -e

MAIN_REPO="$(cd "$(dirname "$0")/.." && pwd)"
WORKTREES_DIR="$MAIN_REPO/worktrees"

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS] [branch-name]

Start a Claude session in a worktree (create new or resume existing).

Options:
  -i, --issue ID       Create worktree for Linear issue (e.g., HEY-123)
  -r, --resume NAME    Resume session in existing worktree heycat-NAME
  -l, --list           List available worktrees
  -h, --help           Show this help

Examples:
  $(basename "$0") --issue HEY-123      # Create worktree for Linear issue (PREFERRED)
  $(basename "$0") feature/audio        # Create new worktree, start Claude
  $(basename "$0") --resume audio       # Resume in heycat-audio worktree
  $(basename "$0") -l                   # List worktrees
  $(basename "$0")                      # Interactive: Claude helps choose
EOF
}

list_worktrees() {
  echo "Available worktrees:"
  if [ -d "$WORKTREES_DIR" ]; then
    local found=0
    for dir in "$WORKTREES_DIR"/heycat-*; do
      if [ -d "$dir" ]; then
        echo "  - $(basename "$dir")"
        found=1
      fi
    done
    if [ $found -eq 0 ]; then
      echo "  (none)"
    fi
  else
    echo "  (none)"
  fi
}

start_claude_in() {
  local path="$1"
  echo ""
  echo "Starting Claude in: $path"
  echo ""
  cd "$path"
  exec claude
}

# Parse arguments
RESUME=""
BRANCH_NAME=""
ISSUE_ID=""
while [[ $# -gt 0 ]]; do
  case $1 in
    -i|--issue) ISSUE_ID="$2"; shift 2 ;;
    -r|--resume) RESUME="$2"; shift 2 ;;
    -l|--list) list_worktrees; exit 0 ;;
    -h|--help) usage; exit 0 ;;
    -*) echo "Unknown option: $1"; usage; exit 1 ;;
    *) BRANCH_NAME="$1"; shift ;;
  esac
done

# Resume mode: go directly to existing worktree
if [ -n "$RESUME" ]; then
  WORKTREE_PATH="$WORKTREES_DIR/heycat-$RESUME"
  if [ ! -d "$WORKTREE_PATH" ]; then
    # Try without heycat- prefix (user might have typed full name)
    WORKTREE_PATH="$WORKTREES_DIR/$RESUME"
  fi
  if [ ! -d "$WORKTREE_PATH" ]; then
    echo "Error: Worktree not found: $RESUME"
    list_worktrees
    exit 1
  fi
  start_claude_in "$WORKTREE_PATH"
fi

# Create mode: use Claude CLI to create worktree
cd "$MAIN_REPO"

# Check for jq dependency
if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed."
  echo "Install with: brew install jq"
  exit 1
fi

SCHEMA='{"type":"object","properties":{"worktreePath":{"type":"string","description":"Full absolute path to the created worktree"},"success":{"type":"boolean"},"error":{"type":"string"}},"required":["success","worktreePath"]}'

# Build prompt with optional branch name or issue ID
if [ -n "$ISSUE_ID" ]; then
  BRANCH_CONTEXT="Linear issue: $ISSUE_ID
Ask me for a short description (2-3 words, kebab-case) to complete the branch name.
Branch format should be: $ISSUE_ID-<description> (e.g., $ISSUE_ID-fix-audio)"
elif [ -n "$BRANCH_NAME" ]; then
  BRANCH_CONTEXT="Branch name: $BRANCH_NAME"
else
  BRANCH_CONTEXT="Ask me for a Linear issue ID (e.g., HEY-123) or branch name.
PREFERRED: Use Linear issue format like HEY-123-description for automatic PR linking."
fi

PROMPT="Create a git worktree for feature development.

$BRANCH_CONTEXT

Steps:
1. Verify we're in the main repo (not a worktree) - check if .git is a directory
2. Check for clean working directory with git status --porcelain
3. If no branch/issue provided, ask what to create (prefer Linear issue format: HEY-xxx-description)
4. Fetch origin main
5. Run: bun scripts/create-worktree.ts <branch-name>
6. Run: cd <worktree-path> && bun install

IMPORTANT: Return the full absolute path to the worktree in your response.
NOTE: Branch names starting with HEY-xxx enable automatic PR linking in Linear."

echo "Creating worktree via Claude..."

RESULT=$(claude -p "$PROMPT" \
  --output-format json \
  --json-schema "$SCHEMA" \
  --allowedTools "Bash,Read" \
  2>&1)

# Extract worktree path from JSON response
WORKTREE_PATH=$(echo "$RESULT" | jq -r '.result.worktreePath // empty' 2>/dev/null)

if [ -z "$WORKTREE_PATH" ]; then
  echo "Failed to extract worktree path from Claude's response"
  echo ""
  echo "Response:"
  echo "$RESULT"
  exit 1
fi

if [ ! -d "$WORKTREE_PATH" ]; then
  echo "Error: Worktree path does not exist: $WORKTREE_PATH"
  exit 1
fi

start_claude_in "$WORKTREE_PATH"
