---
status: done
severity: major
origin: manual
created: 2025-12-25
completed: 2025-12-25
parent_feature: null
parent_spec: null
---

# Bug: Paste Not Working

**Created:** 2025-12-25
**Owner:** Claude
**Severity:** Major

## Problem Description

Contents are in the clipboard but pasting is not happening

## Steps to Reproduce

1. Record audio with heycat
2. Stop recording (transcription completes)
3. Expected: text pastes into focused app. Actual: clipboard has text but paste doesn't happen

## Root Cause

Three issues in `src-tauri/src/keyboard/synth.rs`:

1. **Wrong event tap location:** `CGEventTapLocation::HID` posts events at hardware level, but `CGEventTapLocation::Session` is required for reliable cross-app event delivery.

2. **Missing pre-paste delay:** Clipboard write and paste happened back-to-back with no settling time. macOS needs ~20ms for the pasteboard to sync before Cmd+V can read from it.

3. **Insufficient internal delay:** The delay between key-down and key-up was reduced from 10ms to 1ms in commit 5b3ef3e, which was too fast for macOS to register the keystroke.

## Fix Approach

1. Added 20ms pre-delay before sending any events (allows clipboard sync)
2. Added 20ms delay between key-down and key-up (rdev library recommendation)
3. Changed `CGEventTapLocation::HID` to `CGEventTapLocation::Session` for reliable cross-app event delivery

Commits:
- 9214201 - initial timing fix (insufficient)
- 569823b - added pre-paste delay
- 8b2c9ab - switched to Session tap location (fixed)
- c52aa49 - added integration tests

## Acceptance Criteria

- [x] Bug no longer reproducible
- [x] Root cause addressed (not just symptoms)
- [x] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Record and transcribe text | Text pastes into focused app | [x] Verified |
| Integration test (ignored) | Function returns Ok | [x] Added in synth.rs |
