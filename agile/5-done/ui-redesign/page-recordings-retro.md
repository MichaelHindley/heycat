# Retrospective: Recordings Page

**Date:** 2025-12-18
**Spec:** agile/3-in-progress/ui-redesign/page-recordings.spec.md
**Review Rounds:** 3

## Summary

The Recordings page implementation required 3 review rounds to achieve APPROVED status. The first review passed implementation but the independent review caught critical issues: DOM nesting violations (button inside button), missing skeleton loaders, page not wired up in App.tsx, and most critically, a backend command (`delete_recording`) that was being invoked but never registered. These issues would have caused runtime failures.

## What Went Well

- Clear spec with wireframes helped visualize the expected UI
- Following the Commands page pattern accelerated development
- Test-first approach (writing tests before fixes) helped verify each issue was resolved
- TCR workflow caught issues incrementally and created checkpoint commits
- The independent review (fresh subagent) caught issues I missed after implementation

## Issues Encountered

### Prompt Improvement: Spec Loop Should Verify Backend Command Registration Before Implementation

**What happened:** I implemented the frontend to call `invoke("delete_recording", { filePath })` without first checking if the command existed in the backend. The spec mentioned "Delete: Confirmation dialog, then removes" but didn't specify whether the backend command existed.

**Impact:** First review round passed, but the independent review correctly identified this as a blocking issue - the feature would fail at runtime.

**Suggestion:** The `/agile:loop` implementation phase prompt should explicitly instruct: "Before implementing any invoke() calls to the backend, verify the command exists in src-tauri/src/lib.rs invoke_handler. If the command doesn't exist, implement it as part of this spec."

**Implementation hint:** Update `/devloop:agile:loop` prompt in the plugin to add a pre-implementation checklist item for backend command verification. Could also add this to the spec template under "Implementation Notes" as a required check.

---

### Template Update: Spec Should Distinguish Frontend-Only vs Full-Stack Features

**What happened:** The spec's acceptance criteria said "Delete: Confirmation dialog, then removes" but didn't clarify that this required backend work. I assumed the backend command already existed.

**Suggestion:** Add a section to the spec template called "Backend Requirements" that explicitly lists:
- Existing commands to use (with signatures)
- New commands to implement (with required parameters)
- Database/state changes needed

**Implementation hint:** Update the spec template at `skills/agile/templates/spec.template.md` to include a "## Backend Requirements" section with subsections for "Existing Commands" and "New Commands Needed".

---

### Workflow Enhancement: Review Should Check Backend Command Registration Automatically

**What happened:** The first review phase passed, but the second independent review caught that `delete_recording` wasn't registered. This back-and-forth wasted time.

**Impact:** Extra review round, more context tokens consumed, delayed completion.

**Suggestion:** Add an automated pre-review gate in the review prompt that checks:
1. Parse all `invoke("command_name"` calls from the frontend code
2. Verify each command exists in `lib.rs` invoke_handler
3. Fail fast with a clear message if any are missing

**Implementation hint:** Add to `/devloop:agile:review` prompt: "Before running the full review, grep for `invoke\(` in the changed frontend files and verify each command name exists in `src-tauri/src/lib.rs` invoke_handler. If any are missing, immediately return NEEDS_WORK with the list of unregistered commands."

---

### Prompt Improvement: DOM Nesting Validation Should Be Explicit

**What happened:** I nested a `<button>` inside another `<button>` in RecordingItem.tsx. This is invalid HTML and causes console warnings, but I didn't notice until the review.

**Impact:** Required refactoring the component structure in a second round.

**Suggestion:** Add to the implementation phase prompt: "When creating interactive list items, never nest interactive elements (button inside button, button inside anchor). Use proper semantic structure: a container div, separate buttons for different actions, and only one element should control expand/collapse."

**Implementation hint:** Add this as a "Common Mistakes to Avoid" section in the loop implementation prompt, or add a linting step that checks for nested interactive elements.

---

### Missing Feature: Automated Check for Page Wiring

**What happened:** I created the Recordings page component and exported it, but forgot to:
1. Import it in App.tsx
2. Add the route condition (`navItem === "recordings" && <Recordings />`)

**Impact:** The page would have been unreachable in the app.

**Suggestion:** The review should automatically check if new pages are wired up. For this project's routing pattern:
1. Check if the page is exported from `src/pages/index.ts`
2. Check if the page is imported in `src/App.tsx`
3. Check if there's a render condition for the page in the AppShell children

**Implementation hint:** Add to review prompt: "For page specs, verify the page is (1) exported from src/pages/index.ts, (2) imported in src/App.tsx, and (3) rendered conditionally in AppShell. Missing any of these = NEEDS_WORK."

---

### Documentation Gap: Spec Acceptance Criteria Ambiguity on Virtualization

**What happened:** The spec said "Virtualized list for performance (100+ recordings)" as an acceptance criterion, but the review ultimately marked this as "DEFERRED - acceptable for MVP". This ambiguity about what's required vs nice-to-have caused confusion.

**Impact:** Wasted time considering whether to implement virtualization, and uncertainty about whether it would block approval.

**Suggestion:** Specs should clearly distinguish between:
- **MUST** - Blocking criteria that will fail review if missing
- **SHOULD** - Expected but can be deferred with justification
- **COULD** - Nice-to-have that won't affect approval

**Implementation hint:** Update spec template to use RFC 2119 keywords (MUST/SHOULD/COULD) in acceptance criteria. Update review prompt to only block on MUST items.

## Priority Improvements

1. **Pre-review gate for backend command registration** - This would have caught the `delete_recording` issue immediately instead of requiring a full review round. High impact, low effort.

2. **Spec template with Backend Requirements section** - Forces spec authors to explicitly list what backend work is needed, reducing implementation surprises. Medium impact, low effort.

3. **RFC 2119 keywords in specs** - Removes ambiguity about what's blocking vs deferrable. Medium impact, medium effort (requires updating all existing specs).
