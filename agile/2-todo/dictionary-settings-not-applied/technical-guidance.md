---
last-updated: 2025-12-23
status: validated
---

# Technical Guidance: Dictionary Settings Not Applied

## Root Cause Analysis

**Tauri v2 command parameter naming mismatch.**

In Tauri v2, Rust command parameters using snake_case (e.g., `auto_enter`) are automatically converted to camelCase (e.g., `autoEnter`) for JavaScript invocation. The frontend must use camelCase parameter names.

The bug is in `src/hooks/useDictionary.ts` where the invoke calls use snake_case:

```typescript
// WRONG - Tauri ignores these because it expects camelCase
invoke("update_dictionary_entry", {
  auto_enter: data.autoEnter,      // Should be: autoEnter
  disable_suffix: data.disableSuffix,  // Should be: disableSuffix
});
```

When Tauri receives `{ auto_enter: true }`, it looks for `autoEnter`, doesn't find it, and the Rust function receives `None`, defaulting to `false`.

## Key Files

| File | Purpose |
|------|---------|
| `src/hooks/useDictionary.ts` | Frontend hook - **contains the bug** (lines 33-34, 63-64) |
| `src-tauri/src/commands/dictionary.rs` | Backend commands - correctly structured |
| `src/pages/Dictionary.tsx` | UI component - correctly passes data |

## Fix Approach

Change parameter names from snake_case to camelCase in the invoke calls:

```typescript
// addEntry mutation (lines 29-35)
invoke<DictionaryEntry>("add_dictionary_entry", {
  trigger: data.trigger,
  expansion: data.expansion,
  suffix: data.suffix,
  autoEnter: data.autoEnter,       // Changed from auto_enter
  disableSuffix: data.disableSuffix,  // Changed from disable_suffix
});

// updateEntry mutation (lines 58-65)
invoke<void>("update_dictionary_entry", {
  id: data.id,
  trigger: data.trigger,
  expansion: data.expansion,
  suffix: data.suffix,
  autoEnter: data.autoEnter,       // Changed from auto_enter
  disableSuffix: data.disableSuffix,  // Changed from disable_suffix
});
```

## Regression Risk

**Low risk.** This is a straightforward naming fix. The change aligns the frontend with Tauri v2's expected parameter naming convention.

Testing should verify:
- Adding entries with settings works
- Updating entries with settings works
- Existing entries without settings are unaffected
- The text expansion (suffix/auto-enter) actually triggers during transcription

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-23 | Tauri v2 auto-converts snake_case params to camelCase | Identified root cause |
| 2025-12-23 | Frontend using wrong casing for `auto_enter`/`disable_suffix` | Parameters silently ignored |
| 2025-12-23 | Backend receives `None`, defaults to `false` | Settings never saved |

## Open Questions

None - root cause identified and fix approach validated.
