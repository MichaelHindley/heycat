---
last-updated: 2025-12-25
status: ready
---

# Technical Guidance: Paste Not Working

## Root Cause Analysis

The CoreGraphics Cmd+V simulation in `simulate_cmd_v_paste()` is not being received by the target application. The clipboard write succeeds but the simulated keystroke fails to trigger paste.

**Call flow:**
1. Text is written to clipboard via `app_handle.clipboard().write_text()` ✅
2. `simulate_paste()` calls `simulate_cmd_v_paste()`
3. CoreGraphics posts Cmd+V key events to HID

**Suspected cause:** CGEventTapLocation::HID events may not reach apps depending on focus state or accessibility permissions. Alternatively, there may be a timing race between clipboard write and the paste keystroke.

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/keyboard/synth.rs` | CoreGraphics paste simulation |
| `src-tauri/src/hotkey/integration.rs` | `copy_and_paste()` entry point |
| `src-tauri/src/transcription/service.rs` | Alternative paste call site |

## Fix Approach

1. Add additional trace logging to narrow down where paste fails
2. Check if timing/delay between clipboard write and paste is sufficient
3. Verify focus is on target app before simulating paste

## Regression Risk

- Changing timing could affect paste reliability in other scenarios
- Focus handling changes could affect hotkey behavior

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-25 | Clipboard write confirmed working | Narrows issue to paste simulation |

## Open Questions

- [ ] Does this happen consistently or intermittently?
- [ ] What application is being pasted into?
- [ ] Are there any PASTE-TRACE logs in console?
