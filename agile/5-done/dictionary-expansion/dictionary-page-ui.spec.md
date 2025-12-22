---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: ["dictionary-hook", "event-bridge-integration"]
review_round: 1
---

# Spec: Dictionary Page UI (Frontend)

## Description

Create the Dictionary page component with UI for managing dictionary entries. Includes entry list, add form, edit/delete actions, and validation. Add route and navigation item to AppShell.

See: `## Data Flow Diagram` in technical-guidance.md - "Dictionary Page" in React Components.

## Acceptance Criteria

- [ ] `/dictionary` route accessible
- [ ] Dictionary nav item in AppShell sidebar
- [ ] List of dictionary entries displayed
- [ ] Add new entry form (trigger + expansion fields)
- [ ] Edit existing entry (inline or modal)
- [ ] Delete entry with confirmation
- [ ] Validation: empty trigger shows error
- [ ] Validation: duplicate trigger shows error
- [ ] Loading and error states handled
- [ ] Empty state when no entries

## Test Cases

- [ ] Entry list displays correctly
- [ ] Add form: submits and clears on success
- [ ] Add form: shows error for empty trigger
- [ ] Edit: opens edit mode and saves changes
- [ ] Delete: shows confirmation before deleting
- [ ] Empty state shown when no entries
- [ ] Loading state shown while fetching

## Dependencies

- dictionary-hook.spec.md (provides useDictionary hook)
- event-bridge-integration.spec.md (keeps UI in sync)

## Preconditions

- useDictionary hook implemented
- Event Bridge listens for dictionary_updated

## Implementation Notes

**Files to create/modify:**
- `src/pages/Dictionary.tsx` - New page component (create)
- `src/pages/Dictionary.css` - Styles (create)
- `src/pages/index.ts` - Export Dictionary
- `src/routes.tsx` - Add /dictionary route
- `src/components/layout/AppShell.tsx` - Add nav item

**Route addition:**
```typescript
// src/routes.tsx
import { Dashboard, Commands, Recordings, Settings, Dictionary } from "./pages";

// In children array:
{ path: "dictionary", element: <Dictionary /> },
```

**Nav item pattern (from AppShell):**
```typescript
const navItems = [
  { id: "dashboard", label: "Dashboard", icon: HomeIcon },
  { id: "dictionary", label: "Dictionary", icon: BookIcon }, // Add this
  // ...
];
```

**Component structure:**
```tsx
export function Dictionary() {
  const { entries, addEntry, updateEntry, deleteEntry } = useDictionary();

  return (
    <div className="dictionary-page">
      <h1>Dictionary</h1>
      <AddEntryForm onSubmit={addEntry.mutate} />
      <EntryList
        entries={entries.data ?? []}
        onEdit={updateEntry.mutate}
        onDelete={deleteEntry.mutate}
      />
    </div>
  );
}
```

## Related Specs

- dictionary-hook.spec.md (data layer)
- event-bridge-integration.spec.md (sync)

## Integration Points

- Production call site: `src/routes.tsx` (route registration)
- Connects to: useDictionary hook, AppShell navigation

## Integration Test

- Test location: `src/pages/Dictionary.test.tsx`
- Verification: [ ] Page renders and allows CRUD operations

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Pre-Review Gates (Automated)

#### 1. Build Warning Check
```
warning: method `get` is never used
    = note: `#[warn(dead_code)]` on by default
warning: `heycat` (lib) generated 1 warning
```
**PASS**: The warning is pre-existing (unrelated to this spec).

#### 2. Command Registration Check
N/A - This spec is frontend-only (UI components).

#### 3. Event Subscription Check
**PASS**: Dictionary events are properly configured:
- Event defined: `DICTIONARY_UPDATED: "dictionary_updated"` in `src/lib/eventBridge.ts:41`
- Event listener in Event Bridge: Lines 152-158 invalidate `queryKeys.dictionary.all`

### Manual Review

#### 1. Is the code wired up end-to-end?
- [x] Dictionary page component created: `src/pages/Dictionary.tsx`
- [x] Route registered in `src/routes.tsx:127`: `{ path: "dictionary", element: <Dictionary /> }`
- [x] Navigation item added in `src/components/layout/AppShell.tsx:50`: `{ id: "dictionary", label: "Dictionary", icon: "Book" }`
- [x] Page exported from `src/pages/index.ts:16-17`
- [x] useDictionary hook called from production code (Dictionary.tsx:258)
- [x] Event Bridge listens for `dictionary_updated` and invalidates queries

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| Dictionary.tsx | component | routes.tsx:127 | YES |
| DictionaryProps | type | Dictionary.tsx:8 | YES |
| AddEntryForm | component | Dictionary.tsx:440 | YES |
| EntryItem | component | Dictionary.tsx:476 | YES |
| DictionaryEmptyState | component | Dictionary.tsx:464 | YES |

#### 3. Where does the data flow?

```
[UI Action] User navigates to /dictionary or adds/edits/deletes entry
     |
     v
[Component] src/pages/Dictionary.tsx
     | Uses useDictionary() hook
     v
[Hook] src/hooks/useDictionary.ts:15
     | entries query: invoke("list_dictionary_entries")
     | addEntry mutation: invoke("add_dictionary_entry")
     | updateEntry mutation: invoke("update_dictionary_entry")
     | deleteEntry mutation: invoke("delete_dictionary_entry")
     v
[Backend Commands] (from dependency specs)
     |
     v
[Event] emit("dictionary_updated")
     |
     v
[Event Bridge] src/lib/eventBridge.ts:152-158
     | listen('dictionary_updated')
     v
[Cache Invalidation] queryClient.invalidateQueries({ queryKey: queryKeys.dictionary.all })
     |
     v
[UI Re-render] Dictionary page refetches and displays updated entries
```

**PASS**: Complete data flow from UI action to cache invalidation.

#### 4. Are there any deferrals?
No new deferrals introduced by this spec. Pre-existing deferrals in codebase are unrelated.

#### 5. Automated check results
All 336 frontend tests pass including 14 new tests in `Dictionary.test.tsx`.

#### 6. Frontend-Only Integration Check

**App Entry Point Verification:**
- Dictionary route is registered in `src/routes.tsx:127`
- RootLayout wraps all pages with AppShell which provides navigation
- Navigation item added to AppShell's defaultNavItems array

**Hardcoded Value Check:**
No hardcoded values found. Dictionary page uses:
- `useDictionary()` hook for dynamic data
- `entries.data`, `entries.isLoading`, `entries.isError` for state
- Mutations for CRUD operations

**Hook Usage Check:**
| Hook | Created In | Called In | Passes Data To |
|------|------------|-----------|----------------|
| useDictionary | hooks/useDictionary.ts | Dictionary.tsx:258 | AddEntryForm, EntryList components |

**PASS**: Hook is called in page component (not just tests) and data flows to child components.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `/dictionary` route accessible | PASS | routes.tsx:127 - route registered |
| Dictionary nav item in AppShell sidebar | PASS | AppShell.tsx:50 - nav item added with Book icon |
| List of dictionary entries displayed | PASS | Dictionary.tsx:474-492 - EntryItem components rendered |
| Add new entry form (trigger + expansion fields) | PASS | Dictionary.tsx:439-445 - AddEntryForm with both fields |
| Edit existing entry (inline or modal) | PASS | Dictionary.tsx:120-165 - inline edit mode in EntryItem |
| Delete entry with confirmation | PASS | Dictionary.tsx:167-195 - confirmation dialog in EntryItem |
| Validation: empty trigger shows error | PASS | Dictionary.tsx:29-32 - "Trigger is required" validation |
| Validation: duplicate trigger shows error | PASS | Dictionary.tsx:34-37 - "This trigger already exists" validation |
| Loading and error states handled | PASS | Dictionary.tsx:400-427 - loading div and error Card with retry |
| Empty state when no entries | PASS | Dictionary.tsx:232-254 - DictionaryEmptyState component |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Entry list displays correctly | PASS | Dictionary.test.tsx:88-102 |
| Add form: submits and clears on success | PASS | Dictionary.test.tsx:104-151 |
| Add form: shows error for empty trigger | PASS | Dictionary.test.tsx:153-180 |
| Edit: opens edit mode and saves changes | PASS | Dictionary.test.tsx:210-252 |
| Delete: shows confirmation before deleting | PASS | Dictionary.test.tsx:254-290 |
| Empty state shown when no entries | PASS | Dictionary.test.tsx:292-305 |
| Loading state shown while fetching | PASS | Dictionary.test.tsx:307-317 |
| Search filtering works | PASS | Dictionary.test.tsx:319-338 |
| Error state displays with retry | PASS | Dictionary.test.tsx:356-367 |

### Code Quality

**Strengths:**
- Clean separation of concerns with AddEntryForm, EntryItem, and DictionaryEmptyState as subcomponents
- Proper use of useCallback for stable function references
- Comprehensive validation with user-friendly error messages
- Toast notifications for all CRUD operations
- Search/filter functionality for better UX
- Loading, error, and empty states all properly handled
- Good accessibility with aria-labels and role attributes
- Follows existing patterns (Tanstack Query, Event Bridge, toast notifications)

**Concerns:**
- None identified

### Verdict

**APPROVED** - Implementation fully satisfies all acceptance criteria. The Dictionary page UI is properly wired up with route registration, navigation item, and full CRUD functionality. Event Bridge integration ensures UI stays in sync with backend state changes. All 14 tests pass covering the required test cases plus additional edge cases.
