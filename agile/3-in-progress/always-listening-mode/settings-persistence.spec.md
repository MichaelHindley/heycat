---
status: in-progress
created: 2025-12-14
completed: null
dependencies:
  - frontend-listening-hook
---

# Spec: Persist preferences and settings UI

## Description

Persist always-listening preferences across app sessions and provide a settings UI for users to configure the feature. Include options for enabling/disabling and auto-start behavior. Uses `tauri-plugin-store` for persistence.

## Acceptance Criteria

- [ ] `tauri-plugin-store` v2 added to project dependencies
- [ ] Store plugin initialized in `src-tauri/src/lib.rs`
- [ ] Store permissions added to `src-tauri/capabilities/default.json`
- [ ] Settings stored using Tauri store plugin
- [ ] `listeningEnabled` preference persisted and loaded on startup
- [ ] `autoStartListening` option to begin listening on app launch
- [ ] Settings panel UI component with toggle switches
- [ ] Settings accessible from main window
- [ ] Migration handles fresh installs (sensible defaults)

## Test Cases

- [ ] Settings persist across app restart
- [ ] Auto-start listening activates on launch when enabled
- [ ] Settings UI reflects current persisted values
- [ ] Changing settings updates persisted values immediately
- [ ] Default values applied for new installations

## Dependencies

- frontend-listening-hook (settings control hook behavior)

## Preconditions

- Frontend listening hook functional
- Existing settings infrastructure (if any)

## Implementation Notes

### Add dependency:
```toml
# src-tauri/Cargo.toml
tauri-plugin-store = "2"
```

### Add permissions:
```json
// src-tauri/capabilities/default.json - add to permissions array
"store:default"
```

### Initialize plugin:
```rust
// src-tauri/src/lib.rs
.plugin(tauri_plugin_store::Builder::new().build())
```

### Store schema:
```json
{
  "listening": {
    "enabled": false,
    "autoStartOnLaunch": false
  }
}
```

- Settings should sync with backend state on app startup
- Consider future extensibility (sensitivity threshold, custom wake word)

## Related Specs

- frontend-listening-hook.spec.md (controlled by these settings)
- activation-feedback.spec.md (visual feedback setting - deferred)

## Integration Points

- Production call site: Settings component, app initialization
- Connects to: useListening hook, Tauri store

## Integration Test

- Test location: `src/components/Settings.test.tsx`
- Verification: [ ] Integration test passes
