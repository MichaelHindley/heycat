---
status: completed
created: 2025-11-26
completed: 2025-11-27
dependencies: []
---

# Spec: Global Hotkey Registration

## Description

Integrate tauri-plugin-global-shortcut to register Cmd+Shift+R (macOS) / Ctrl+Shift+R (Windows/Linux) as a system-wide hotkey that works even when the app is not focused.

## Acceptance Criteria

- [x] Register platform-specific shortcut (CmdOrCtrl+Shift+R)
- [x] Callback invoked when shortcut pressed
- [x] Works when app window not focused (global system-wide)
- [x] Unregister shortcut on app cleanup
- [x] Handle conflicts with other apps gracefully (return error)

## Test Cases

- [x] Shortcut registration succeeds on supported platforms
- [x] Callback receives keypress events
- [x] Unregistration cleans up properly on app exit
- [x] Conflict detection returns descriptive error

## Dependencies

None

## Preconditions

- `tauri-plugin-global-shortcut` added to Cargo.toml
- `@tauri-apps/plugin-global-shortcut` added to package.json
- Permissions configured in `capabilities/default.json`

## Implementation Notes

- Add plugin init in `lib.rs`: `.plugin(tauri_plugin_global_shortcut::Builder::new().build())`
- Add permission: `"global-shortcut:allow-register"` to capabilities
- Use `CmdOrControl+Shift+R` for cross-platform shortcut
- Mark callback registration with `#[cfg_attr(coverage_nightly, coverage(off))]`

## Related Specs

- [hotkey-integration.spec.md](hotkey-integration.spec.md) - Connects hotkey to recording
