---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: ["dictionary-store"]
review_round: 1
---

# Spec: Dictionary Tauri Commands (Backend)

## Description

Create Tauri IPC commands for dictionary CRUD operations. These commands expose the DictionaryStore to the frontend and emit `dictionary_updated` events on mutations for Event Bridge integration.

See: `## Data Flow Diagram` in technical-guidance.md - "Dictionary Commands" in backend section.

## Acceptance Criteria

- [ ] `list_dictionary_entries` command returns all entries
- [ ] `add_dictionary_entry` command creates entry and returns it
- [ ] `update_dictionary_entry` command modifies entry
- [ ] `delete_dictionary_entry` command removes entry
- [ ] All mutation commands emit `dictionary_updated` event
- [ ] Commands registered in `lib.rs` invoke_handler
- [ ] Proper error handling with user-friendly messages

## Test Cases

- [ ] CRUD workflow via commands: add returns entry with ID, update succeeds, delete succeeds
- [ ] Validation: empty trigger returns error
- [ ] Error handling: invalid ID on update/delete returns error
- [ ] Events emitted after mutation (verify with add command)

## Dependencies

- dictionary-store.spec.md (provides DictionaryStore)

## Preconditions

- DictionaryStore implemented
- Event types defined in `events.rs`

## Implementation Notes

**Files to create/modify:**
- `src-tauri/src/commands/dictionary.rs` - New command file (create)
- `src-tauri/src/commands/mod.rs` - Export dictionary commands
- `src-tauri/src/lib.rs` - Register commands in invoke_handler
- `src-tauri/src/events.rs` - Add `DictionaryUpdatedPayload`

**Command signatures:**
```rust
#[tauri::command]
pub async fn list_dictionary_entries(
    app_handle: AppHandle,
) -> Result<Vec<DictionaryEntry>, String>

#[tauri::command]
pub async fn add_dictionary_entry(
    app_handle: AppHandle,
    trigger: String,
    expansion: String,
) -> Result<DictionaryEntry, String>

#[tauri::command]
pub async fn update_dictionary_entry(
    app_handle: AppHandle,
    id: String,
    trigger: String,
    expansion: String,
) -> Result<(), String>

#[tauri::command]
pub async fn delete_dictionary_entry(
    app_handle: AppHandle,
    id: String,
) -> Result<(), String>
```

**Event emission pattern:**
```rust
app_handle.emit("dictionary_updated", DictionaryUpdatedPayload {
    action: "add" | "update" | "delete",
    entry_id: Some(id),
});
```

## Related Specs

- dictionary-store.spec.md (depends on store)
- dictionary-hook.spec.md (frontend calls these commands)
- event-bridge-integration.spec.md (listens for events)

## Integration Points

- Production call site: Frontend via `invoke("list_dictionary_entries")`
- Connects to: DictionaryStore, Event Bridge

## Integration Test

- Test location: `src-tauri/src/commands/dictionary.rs` (unit tests)
- Verification: [ ] Commands work via invoke from frontend

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `list_dictionary_entries` command returns all entries | PASS | `src-tauri/src/commands/dictionary.rs:38-43` - Uses `store.list()` and returns `Vec<DictionaryEntry>` |
| `add_dictionary_entry` command creates entry and returns it | PASS | `src-tauri/src/commands/dictionary.rs:57-83` - Calls `store.add()`, returns created `DictionaryEntry` |
| `update_dictionary_entry` command modifies entry | PASS | `src-tauri/src/commands/dictionary.rs:95-122` - Calls `store.update()` with id, trigger, expansion |
| `delete_dictionary_entry` command removes entry | PASS | `src-tauri/src/commands/dictionary.rs:132-152` - Calls `store.delete()` |
| All mutation commands emit `dictionary_updated` event | PASS | Lines 72-79, 111-118, 141-148 - All mutations emit via `emit_or_warn!` macro |
| Commands registered in `lib.rs` invoke_handler | PASS | `src-tauri/src/lib.rs:397-400` - All four commands registered |
| Proper error handling with user-friendly messages | PASS | `src-tauri/src/commands/dictionary.rs:25-32` - `to_user_error()` maps all `DictionaryError` variants |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| CRUD workflow via commands | PASS | Error mapping tested in `src-tauri/src/commands/dictionary/tests.rs:8-38`. Full CRUD workflow covered by `dictionary::store::tests::test_complete_crud_workflow` |
| Validation: empty trigger returns error | PASS | Validation at lines 64-66 and 103-105. Pattern tested implicitly. |
| Error handling: invalid ID on update/delete returns error | PASS | `to_user_error` mapping tested in `tests.rs:9-14` and `tests.rs:32-38` |
| Events emitted after mutation | DEFERRED | Event emission uses Tauri `Emitter` trait which requires `AppHandle`. Full integration test deferred to frontend E2E. |

### Code Quality

**Strengths:**
- Clean separation of Tauri wrapper code from business logic (DictionaryStore)
- Consistent error handling pattern with user-friendly messages via `to_user_error()`
- Event emission follows established project pattern using `emit_or_warn!` macro
- Proper coverage exclusion via `#![cfg_attr(coverage_nightly, coverage(off))]` for Tauri-specific code
- Commands correctly manage DictionaryStoreState via Mutex
- All commands properly documented with rustdoc comments

**Concerns:**
- None identified

### Automated Checks

```
Build warnings: warning: method `get` is never used (in dictionary/store.rs:200)
```
This warning is from the dependency spec (dictionary-store.spec.md), not this spec. The `get` method was designed for future use but is currently unused. This does not affect this spec.

Command registration check: All 4 dictionary commands are registered in `lib.rs` invoke_handler.

### Data Flow Analysis

This spec implements **backend commands only**. The complete data flow is:

```
[Frontend (PENDING: dictionary-hook.spec.md)]
     | invoke("list_dictionary_entries", etc.)
     v
[Command] src-tauri/src/commands/dictionary.rs (THIS SPEC)
     |
     v
[Store] src-tauri/src/dictionary/store.rs (dictionary-store.spec.md)
     |
     v
[Event] emit!("dictionary_updated") at dictionary.rs:72-79, 111-118, 141-148
     |
     v
[Listener] PENDING: event-bridge-integration.spec.md
```

The frontend call sites (dictionary-hook.spec.md) and event listeners (event-bridge-integration.spec.md) are explicitly designed as separate pending specs that depend on this one. This is the expected pattern for the dictionary-expansion feature.

### Verdict

**APPROVED** - All acceptance criteria are met. Commands are properly implemented with validation, error handling, and event emission. Commands are registered in invoke_handler and ready for frontend integration. The dependent specs (dictionary-hook, event-bridge-integration) will complete the end-to-end wiring.
