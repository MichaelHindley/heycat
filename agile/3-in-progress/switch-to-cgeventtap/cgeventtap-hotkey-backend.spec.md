---
status: pending
created: 2025-12-19
completed: null
dependencies:
  - replace-iokit-hid
  - frontend-shortcut-display
---

# Spec: CGEventTap-based hotkey backend for fn key support

## Description

Replace Tauri's global-shortcut plugin with a CGEventTap-based hotkey backend on macOS. This enables fn key and media keys to be used as global hotkeys, similar to how Wispr Flow implements it. The existing `ShortcutBackend` trait abstraction makes this a drop-in replacement.

## Acceptance Criteria

- [ ] New `CGEventTapHotkeyBackend` struct implements `ShortcutBackend` trait
- [ ] fn key can be registered as part of a hotkey (e.g., fn+R)
- [ ] Media keys can be registered as hotkeys (e.g., Play/Pause to toggle recording)
- [ ] Modifier-only hotkeys work (e.g., just double-tap fn)
- [ ] Left/right modifier distinction available for hotkeys
- [ ] CGEventTap runs continuously when any hotkey is registered
- [ ] **Multi-OS support via factory function**: `create_shortcut_backend()` selects backend at compile time
  - macOS: CGEventTapHotkeyBackend (required for fn key, media keys)
  - Windows/Linux: TauriShortcutBackend (standard Tauri plugin)
- [ ] Existing hotkey functionality (Cmd+Shift+R) continues to work

## Test Cases

- [ ] Register fn+Command+R as hotkey → callback fires when pressed
- [ ] Register just "fn" as hotkey → callback fires on fn release
- [ ] Register Play/Pause media key → callback fires when pressed
- [ ] Multiple hotkeys registered → each fires independently
- [ ] Unregister hotkey → callback no longer fires
- [ ] Permission denied on macOS → returns error (user must grant permission)
- [ ] Rapid key presses → properly debounced
- [ ] `create_shortcut_backend()` returns correct backend type per OS

## Dependencies

- replace-iokit-hid - CGEventTap capture must work first
- frontend-shortcut-display - UI must handle fn/media key display
- integration-test - manual testing validates capture works

## Preconditions

- CGEventTap capture implementation complete and working
- Accessibility permission infrastructure in place

## Implementation Notes

### Multi-OS Architecture

```
create_shortcut_backend(app_handle) → Arc<dyn ShortcutBackend>
    ├─ macOS    → CGEventTapHotkeyBackend
    └─ Windows  → TauriShortcutBackend
```

Both the recording hotkey AND Escape key registration use this unified entrypoint.

### Files to Create/Modify

1. **`src-tauri/src/hotkey/cgeventtap_backend.rs`** (NEW - macOS only)
   - `CGEventTapHotkeyBackend` struct implementing `ShortcutBackend`
   - `ShortcutSpec` for matching key events to registered shortcuts
   - `parse_shortcut()` function to parse "fn+Command+R" format
   - `matches_shortcut()` function to compare events to specs

2. **`src-tauri/src/hotkey/mod.rs`**
   - Add `#[cfg(target_os = "macos")] mod cgeventtap_backend`
   - Add `create_shortcut_backend()` factory function
   - Add `HotkeyServiceDyn` for dynamic backend dispatch

3. **`src-tauri/src/lib.rs`**
   - Use `create_shortcut_backend()` for both recording hotkey and Escape key
   - Use `HotkeyServiceDyn` instead of generic `HotkeyService<B>`

### Key Implementation Details

```rust
pub struct CGEventTapHotkeyBackend {
    capture: Arc<Mutex<CGEventTapCapture>>,
    registered_shortcuts: Arc<Mutex<HashMap<String, ShortcutSpec>>>,
    callbacks: Arc<Mutex<HashMap<String, Box<dyn Fn() + Send + Sync>>>>,
}

struct ShortcutSpec {
    fn_key: bool,
    command: bool,
    control: bool,
    alt: bool,
    shift: bool,
    key_name: Option<String>,  // None for modifier-only
    is_media_key: bool,
}
```

### Shortcut String Format

Support both Tauri format and extended format:
- `"Command+Shift+R"` - standard Tauri format
- `"fn+Command+R"` or `"Function+Command+R"` - with fn key
- `"fn"` - modifier-only
- `"PlayPause"` - media key

## Related Specs

- cgeventtap-core.spec.md - underlying capture implementation
- replace-iokit-hid.spec.md - integration into keyboard_capture module
- frontend-shortcut-display.spec.md - UI display support

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (app initialization)
- Connects to: HotkeyIntegration, HotkeyService, keyboard_capture module

## Integration Test

- Test location: Manual + unit tests in cgeventtap_backend.rs
- Verification: [ ] fn key works as global hotkey
