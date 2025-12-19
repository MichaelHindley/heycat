---
status: pending
created: 2025-12-19
completed: null
dependencies: []
---

# Spec: Accessibility permission handling

## Description

Implement Accessibility permission checking and prompting for macOS. CGEventTap requires Accessibility permission (not Input Monitoring). This spec adds functions to check if permission is granted and guide the user to enable it.

## Acceptance Criteria

- [ ] Check permission with AXIsProcessTrusted() FFI binding
- [ ] Prompt user with clear guidance to enable Accessibility in System Settings
- [ ] Open System Settings to correct pane (Privacy & Security > Accessibility)
- [ ] Return appropriate error when permission not granted

## Test Cases

- [ ] When Accessibility is enabled, permission check returns true
- [ ] When Accessibility is disabled, permission check returns false
- [ ] System Settings opens to correct pane when requested

## Dependencies

None - this is foundational

## Preconditions

- macOS 10.15+ (Catalina or later)
- ApplicationServices framework available

## Implementation Notes

FFI bindings needed:
```rust
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}
```

To open System Settings:
```rust
// Open Accessibility pane
open("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
```

Note: Unlike Input Monitoring, Accessibility permission doesn't have a programmatic "request" API - the user must manually enable it.

File location: `src-tauri/src/keyboard_capture/permissions.rs` (new file)

## Related Specs

- cgeventtap-core.spec.md - uses permission check
- replace-iokit-hid.spec.md - integration

## Integration Points

- Production call site: N/A (standalone module, will be integrated in replace-iokit-hid spec)
- Connects to: ApplicationServices framework

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A
