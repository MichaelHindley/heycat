---
status: in-progress
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: Add missing Function to fn conversion in backendToDisplay

## Description

The `backendToDisplay()` function in `GeneralSettings.tsx` converts backend shortcut format (e.g., "CmdOrControl+Shift+R") to display symbols (e.g., "⌘⇧R"). Currently it's missing the conversion for "Function" → "fn", causing saved Fn hotkeys to display as literal "FunctionR" text after app restart instead of the proper "fn" symbol.

## Acceptance Criteria

- [ ] `backendToDisplay()` converts "Function" to "fn" in the shortcut string
- [ ] After app restart, a saved Fn+key hotkey displays correctly as "fn" prefix
- [ ] Existing conversions (CmdOrControl, Ctrl, Alt, Shift) still work correctly

## Test Cases

- [ ] Input: "Function+R" → Output: "fnR"
- [ ] Input: "Function+CmdOrControl+R" → Output: "fn⌘R"
- [ ] Input: "CmdOrControl+Shift+R" → Output: "⌘⇧R" (no regression)

## Dependencies

None

## Preconditions

- App has hotkey persistence working (saves to settings.json)
- The bug can be reproduced by setting Fn hotkey and restarting

## Implementation Notes

**File:** `src/pages/components/GeneralSettings.tsx:12-19`

Add `.replace(/Function/gi, "fn")` to the `backendToDisplay()` function chain.

```typescript
function backendToDisplay(shortcut: string): string {
  return shortcut
    .replace(/Function/gi, "fn")  // Add this line
    .replace(/CmdOrControl/gi, "⌘")
    .replace(/Ctrl/gi, "⌃")
    .replace(/Alt/gi, "⌥")
    .replace(/Shift/gi, "⇧")
    .replace(/\+/g, "");
}
```

## Related Specs

- `fix-backend-hotkey-loading.spec.md` - Fixes the backend hotkey registration issue

## Integration Points

- Production call site: `src/pages/components/GeneralSettings.tsx:12-19`
- Connects to: Settings loading flow, `get_recording_shortcut()` command

## Integration Test

- Test location: Manual test - set Fn hotkey, restart app, verify display
- Verification: [ ] Integration test passes
