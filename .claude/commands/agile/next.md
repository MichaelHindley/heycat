---
description: Continue with the next spec in the current in-progress issue - auto-implements
---

Continue work on the current in-progress issue by finding and implementing the next spec:

1. Run `bun .claude/skills/agile/agile.ts list --stage 3-in-progress` to find the in-progress issue
2. Run `bun .claude/skills/agile/agile.ts work <issue-name>` to get spec status
3. Identify the next spec to work on (first in-progress, or first pending if none in-progress)
4. Read ONLY that spec file
5. Begin implementation following the acceptance criteria and test cases
6. Use the TCR workflow (`tcr check`) after completing implementation
7. Transition the spec to `in-review`: `bun .claude/skills/agile/agile.ts spec status <issue> <spec> in-review`

Do NOT read all spec files - only the specific next spec.

**IMPORTANT - DO NOT:**
- Add a "## Review" section yourself - use `/agile:review` after implementation
- Mark acceptance criteria as complete/checked in the spec
- Transition directly to `completed` - always go through `in-review` first

After implementation is complete, tests pass, and spec is transitioned to `in-review`, tell the user to run `/agile:review` for independent review.
