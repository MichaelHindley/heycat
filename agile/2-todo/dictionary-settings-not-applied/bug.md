---
status: pending
severity: minor
origin: manual
created: 2025-12-23
completed: null
parent_feature: null
parent_spec: null
---

# Bug: Dictionary Settings Not Applied

**Created:** 2025-12-23
**Owner:** Claude
**Severity:** Minor

## Problem Description

Dictionary suffix and auto-enter settings are not being applied

## Steps to Reproduce

1. Open the Dictionary page in the app
2. Edit an existing dictionary entry
3. Enable "No punctuation" and/or "Auto-enter" settings
4. Save the entry - a success toast appears
5. Refresh the page or reopen the entry
6. Expected: Settings are persisted
7. Actual: Settings are reset to false/disabled

## Root Cause

**Tauri v2 command parameter naming mismatch.**

In Tauri v2, Rust command parameters are automatically converted from snake_case to camelCase for JavaScript invocation. The frontend must use camelCase parameter names when invoking commands.

**The bug is in `src/hooks/useDictionary.ts` lines 33-34 and 63-64:**

```typescript
// WRONG: Using snake_case
invoke("add_dictionary_entry", {
  auto_enter: data.autoEnter,      // Tauri expects "autoEnter"
  disable_suffix: data.disableSuffix,  // Tauri expects "disableSuffix"
});
```

When the frontend sends `{ auto_enter: true }`, Tauri is looking for `autoEnter` and doesn't find it. The Rust function receives `None` for both parameters, which defaults to `false` via `unwrap_or(false)`.

**Reference:** [Calling Rust from the Frontend | Tauri v2](https://v2.tauri.app/develop/calling-rust/)

## Fix Approach

Change the invoke parameter names from snake_case to camelCase in `src/hooks/useDictionary.ts`:

```typescript
// CORRECT: Using camelCase (Tauri's expected format)
invoke("add_dictionary_entry", {
  autoEnter: data.autoEnter,
  disableSuffix: data.disableSuffix,
});
```

This fix needs to be applied to both `addEntry` and `updateEntry` mutations.

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| [Test case description] | [Expected outcome] | [ ] |
