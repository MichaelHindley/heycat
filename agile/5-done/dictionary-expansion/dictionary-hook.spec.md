---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: ["tauri-commands"]
review_round: 1
---

# Spec: Dictionary Hook (Frontend)

## Description

Create the `useDictionary` React hook for dictionary CRUD operations. Uses Tanstack Query for data fetching and mutations, following the existing patterns in `useSettings.ts`. Provides optimistic updates and error handling.

See: `## Data Flow Diagram` in technical-guidance.md - "Hooks Layer" section.

## Acceptance Criteria

- [ ] `useDictionary()` hook exports query and mutation functions
- [ ] `entries` - Tanstack Query for listing entries
- [ ] `addEntry` - Mutation for adding entries
- [ ] `updateEntry` - Mutation for updating entries
- [ ] `deleteEntry` - Mutation for deleting entries
- [ ] Query keys defined in `queryKeys.ts`
- [ ] TypeScript types for `DictionaryEntry`

## Test Cases

- [ ] addEntry mutation calls backend with trigger/expansion
- [ ] updateEntry/deleteEntry mutations call backend correctly
- [ ] Error state exposed when mutation fails
- [ ] Loading state exposed during fetch

## Dependencies

- tauri-commands.spec.md (provides backend commands)

## Preconditions

- Backend Tauri commands implemented
- Tanstack Query configured in app

## Implementation Notes

**Files to create/modify:**
- `src/hooks/useDictionary.ts` - New hook file (create)
- `src/lib/queryKeys.ts` - Add dictionary query keys
- `src/types/dictionary.ts` - Type definitions (create)

**Type definition:**
```typescript
export interface DictionaryEntry {
  id: string;
  trigger: string;
  expansion: string;
}
```

**Query key pattern:**
```typescript
// src/lib/queryKeys.ts
export const queryKeys = {
  // ... existing keys
  dictionary: {
    all: ['dictionary'] as const,
    list: () => [...queryKeys.dictionary.all, 'list'] as const,
  },
};
```

**Hook structure:**
```typescript
export function useDictionary() {
  const entries = useQuery({
    queryKey: queryKeys.dictionary.list(),
    queryFn: () => invoke<DictionaryEntry[]>('list_dictionary_entries'),
  });

  const addEntry = useMutation({
    mutationFn: (data: { trigger: string; expansion: string }) =>
      invoke<DictionaryEntry>('add_dictionary_entry', data),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  // ... updateEntry, deleteEntry mutations

  return { entries, addEntry, updateEntry, deleteEntry };
}
```

## Related Specs

- tauri-commands.spec.md (backend commands)
- event-bridge-integration.spec.md (handles cache invalidation)
- dictionary-page-ui.spec.md (uses this hook)

## Integration Points

- Production call site: `src/pages/Dictionary.tsx`
- Connects to: Tauri commands, Tanstack Query, Event Bridge

## Integration Test

- Test location: `src/hooks/useDictionary.test.ts`
- Verification: [x] Hook tests pass with mocked invoke

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `useDictionary()` hook exports query and mutation functions | PASS | `src/hooks/useDictionary.ts:39` returns `{ entries, addEntry, updateEntry, deleteEntry }` |
| `entries` - Tanstack Query for listing entries | PASS | `src/hooks/useDictionary.ts:16-19` uses `useQuery` with `list_dictionary_entries` command |
| `addEntry` - Mutation for adding entries | PASS | `src/hooks/useDictionary.ts:21-25` uses `useMutation` calling `add_dictionary_entry` |
| `updateEntry` - Mutation for updating entries | PASS | `src/hooks/useDictionary.ts:27-30` uses `useMutation` calling `update_dictionary_entry` |
| `deleteEntry` - Mutation for deleting entries | PASS | `src/hooks/useDictionary.ts:32-36` uses `useMutation` calling `delete_dictionary_entry` |
| Query keys defined in `queryKeys.ts` | PASS | `src/lib/queryKeys.ts:30-35` defines `dictionary.all` and `dictionary.list()` |
| TypeScript types for `DictionaryEntry` | PASS | `src/types/dictionary.ts:5-12` defines interface with `id`, `trigger`, `expansion` |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| addEntry mutation calls backend with trigger/expansion | PASS | `src/hooks/useDictionary.test.ts:69-94` |
| updateEntry/deleteEntry mutations call backend correctly | PASS | `src/hooks/useDictionary.test.ts:126-176` |
| Error state exposed when mutation fails | PASS | `src/hooks/useDictionary.test.ts:96-123` |
| Loading state exposed during fetch | PASS | `src/hooks/useDictionary.test.ts:40-47` |

### Code Quality

**Strengths:**
- Follows existing patterns from `useSettings.ts` and other hooks
- Properly documents that mutations do NOT invalidate queries (Event Bridge handles it)
- Clean, focused hook implementation with clear separation of concerns
- Query keys follow the established convention
- Type definitions are clean and match backend struct
- Tests follow TESTING.md guidelines: behavior-focused, not implementation details

**Concerns:**
- None identified

### Frontend-Only Integration Check

This spec creates a hook but no UI component. Per the spec's Integration Points:
- Production call site: `src/pages/Dictionary.tsx` - handled by `dictionary-page-ui.spec.md` (pending, depends on this spec)
- The hook is correctly exported and ready for use
- No hardcoded values or missing wiring in this scope

| Hook | Created In | Called In | Passes Data To |
|------|------------|-----------|----------------|
| useDictionary | src/hooks/useDictionary.ts | TEST-ONLY (correct for this spec) | Will be Dictionary.tsx per dictionary-page-ui.spec.md |

The TEST-ONLY status is acceptable here because:
1. The spec explicitly states production call site is `src/pages/Dictionary.tsx`
2. There is a dependent spec `dictionary-page-ui.spec.md` that will wire up the hook
3. This follows the layered spec approach where hooks are implemented before pages

### Pre-Review Gate Results

```
Build Warning Check: 1 warning - `method get is never used` in src/dictionary/store.rs:200
  This is backend code from tauri-commands spec, NOT this frontend spec. Not a blocker.

Command Registration Check: N/A - this spec adds no Tauri commands
Event Subscription Check: N/A - this spec adds no event listeners
```

### Verdict

**APPROVED** - The hook implementation is complete, follows established patterns, has proper test coverage, and is correctly scoped. Production wiring will be handled by the dependent `dictionary-page-ui.spec.md`.
