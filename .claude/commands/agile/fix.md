---
description: Fix issues from failed review feedback - parses NEEDS_WORK and guides fixes
---

Address feedback from a spec that received a NEEDS_WORK verdict:

1. Run `bun .claude/skills/agile/agile.ts fix` to:
   - Find the spec in `in-review` status with NEEDS_WORK verdict
   - Parse the review section to extract failed items
   - Display a structured fix guide showing exactly what needs to be fixed
   - Transition the spec back to `in-progress` for fixes

2. The output will show:
   - **Failed Acceptance Criteria**: Each FAIL item with evidence locations
   - **Missing Tests**: Each MISSING test case with expected locations
   - **Concerns**: Issues raised in Code Quality section

3. Address each item:
   - Fix the code issues at the referenced locations
   - Add missing tests
   - Address code quality concerns

4. Run TCR workflow after fixes: `tcr check`

5. When all fixes are complete, run `/agile:review` for re-review

**IMPORTANT:**
- Do NOT remove or modify the existing ## Review section
- Fix the actual code/tests, not the spec file
- The `review_round` field tracks how many review cycles have occurred
- Review history is preserved in the spec frontmatter

After completing fixes, run `/agile:review` for another independent review.
