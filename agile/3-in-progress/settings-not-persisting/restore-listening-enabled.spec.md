---
status: in-progress
created: 2025-12-15
completed: null
dependencies: []
---

# Spec: Restore listening.enabled from store on startup

## Description

Backend reads the persisted `listening.enabled` setting from the Tauri store on app startup and initializes `ListeningManager` with that value instead of hardcoding `false`.

## Acceptance Criteria

- [ ] `ListeningManager` has a `with_enabled(enabled: bool)` constructor
- [ ] `lib.rs` setup reads `listening.enabled` from `settings.json` store
- [ ] `ListeningManager` is initialized with the stored value (defaults to `false` if not found)
- [ ] Unit tests exist for the new constructor

## Test Cases

- [ ] `with_enabled(true)` creates manager with `is_enabled() == true`
- [ ] `with_enabled(false)` creates manager with `is_enabled() == false`
- [ ] Manual: Enable listening, close app, reopen - listening should still be enabled

## Dependencies

None

## Preconditions

- Tauri store plugin is registered (`tauri_plugin_store`)
- Frontend already persists `listening.enabled` to store

## Implementation Notes

**Files Modified:**
- `src-tauri/src/listening/manager.rs:80-88` - Added `with_enabled` constructor
- `src-tauri/src/lib.rs:64-75` - Read from store and use new constructor

**Key Code:**
```rust
// lib.rs - reads store and initializes with stored value
let listening_enabled = app
    .store("settings.json")
    .ok()
    .and_then(|store| store.get("listening.enabled"))
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
let listening_state = Arc::new(Mutex::new(
    listening::ListeningManager::with_enabled(listening_enabled),
));
```

## Related Specs

None - single spec bug fix

## Integration Points

- Production call site: `src-tauri/src/lib.rs:72-74`
- Connects to: Tauri store plugin, listening module

## Integration Test

- Test location: N/A (manual verification required - store interaction)
- Verification: [x] N/A
