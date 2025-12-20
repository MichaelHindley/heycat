---
status: in-progress
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: Fix backend to read autoStartOnLaunch setting

## Description

Change the backend startup code to read `listening.autoStartOnLaunch` instead of `listening.enabled` when initializing the ListeningManager, so that the user's auto-start preference is correctly respected on app launch.

## Acceptance Criteria

- [ ] Backend reads `listening.autoStartOnLaunch` from settings store at startup
- [ ] When `autoStartOnLaunch` is false, app starts with listening disabled
- [ ] When `autoStartOnLaunch` is true, app starts with listening enabled
- [ ] Existing tests pass

## Test Cases

- [ ] App starts with listening OFF when `autoStartOnLaunch = false`
- [ ] App starts with listening ON when `autoStartOnLaunch = true`
- [ ] Setting change persists across app restarts

## Dependencies

None

## Preconditions

None

## Implementation Notes

Change in `src-tauri/src/lib.rs` lines 68-76:
- Replace `store.get("listening.enabled")` with `store.get("listening.autoStartOnLaunch")`
- Keep the default as `false` (don't auto-start by default)

## Related Specs

None - single spec fix

## Integration Points

- Production call site: `src-tauri/src/lib.rs:68-76`
- Connects to: ListeningManager initialization, Tauri store plugin

## Integration Test

- Test location: Manual verification required (app restart behavior)
- Verification: [ ] Integration test passes
