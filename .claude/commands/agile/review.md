---
description: Review the latest spec in in-review status using a fresh subagent for independence
---

Run an independent code review of a spec in `in-review` status:

1. Run `bun .claude/skills/agile/agile.ts review` to get the review prompt and spec details
2. Launch a **fresh subagent** using the Task tool to perform the review against the correct spec, using our project's guidelines
   - The subagent has NO context from the implementation - this ensures independence
   - Pass the review prompt output as the task description
3. The subagent will:
   - Read the implementation files referenced in the spec
   - Verify each acceptance criterion against the code
   - Check test coverage matches test cases
   - Append a `## Review` section to the spec file with verdict (APPROVED/NEEDS_WORK)

4. After the review, handle the status transition based on the verdict:
   - If **APPROVED**: Run `bun .claude/skills/agile/agile.ts spec status <issue> <spec> completed`
   - If **NEEDS_WORK**: Tell the user to run `/agile:fix` to address the feedback

**IMPORTANT:** Reviews must be done by a fresh subagent, not by the agent that implemented the spec.

**Review Lifecycle:**
- Specs start in `pending`, move to `in-progress` during implementation
- After implementation, specs transition to `in-review` (via `/agile:next`)
- After review: APPROVED → `completed`, NEEDS_WORK → `/agile:fix` → `in-progress` → re-review
