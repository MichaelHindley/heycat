---
status: completed
created: 2025-12-01
completed: 2025-12-11
dependencies: ["list-recordings-backend", "recordings-list-ui"]
review_round: 1
---

# Spec: Error Handling

## Description

Handle error states gracefully when recordings have corrupted/missing files or incomplete metadata. Show inline error indicators and log details to appropriate consoles.

## Acceptance Criteria

- [ ] Corrupted/missing recordings show error indicator in list
- [ ] Recording still appears in list (not hidden)
- [ ] Incomplete metadata fields show inline error/placeholder
- [ ] Errors logged to frontend console with recording details
- [ ] OS-level file errors logged to Tauri backend console

## Test Cases

- [ ] Recording with missing file shows error indicator
- [ ] Recording with corrupt metadata shows partial data + error for missing
- [ ] Error details appear in browser console
- [ ] Backend logs file system errors
- [ ] User can still interact with other recordings

## Dependencies

- list-recordings-backend (provides error info)
- recordings-list-ui (displays errors)

## Preconditions

Backend can detect and report file/metadata errors

## Implementation Notes

- Backend should return error status per-recording, not fail entire request
- Consider a Recording type with optional error field
- Frontend console.error for visibility during development
- Tauri uses `log` crate or println! for backend logging

## Related Specs

- list-recordings-backend.spec.md (error detection)
- recordings-list-ui.spec.md (error display)

## Integration Points

- Production call site: Backend response, Frontend RecordingsList
- Connects to: Console logging (frontend and backend)

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

## Review

**Reviewed:** 2025-12-11
**Reviewer:** Claude (independent review agent)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Corrupted/missing recordings show error indicator in list | PASS | RecordingsList.tsx:147-160 - SVG error indicator rendered when `hasError` is true |
| Recording still appears in list (not hidden) | PASS | logic.rs:332-339 - Recordings with errors pushed to results vector |
| Incomplete metadata fields show inline error/placeholder | PASS | RecordingsList.tsx:164 shows "--:--", line 167 shows "--", line 182 shows "--" |
| Errors logged to frontend console with recording details | PASS | RecordingsList.tsx:76-84 - console.error() called with filename and file_path |
| OS-level file errors logged to Tauri backend console | PASS | logic.rs:264, 302, 310, 320-323 - error!() macro used |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Recording with missing file shows error indicator | PASS | RecordingsList.test.tsx:498-512 |
| Recording with corrupt metadata shows partial data + error for missing | PASS | RecordingsList.test.tsx:524-537, 578-602 |
| Error details appear in browser console | PASS | RecordingsList.test.tsx:604-618 |
| Backend logs file system errors | PASS | Backend tests verify struct serialization; logging uses tauri_plugin_log |
| User can still interact with other recordings | PASS | RecordingsList.test.tsx:636-662 |

### Code Quality

**Strengths:**
- Error handling follows fail-safe pattern (include recording with error rather than skip)
- Backend uses proper logging infrastructure (tauri_plugin_log)
- Frontend provides clear visual indicators (SVG icon, CSS class, placeholder text)
- Type safety enforced with optional error field in both TypeScript and Rust
- Console logging includes relevant context (filename, file_path)
- Error messages are user-friendly
- Proper use of `skip_serializing_if` to omit error field when None
- Recording error detail shown in expanded view with proper ARIA role="alert"

**Concerns:**
- None identified

### Integration Verification

| Check | Status | Evidence |
|-------|--------|----------|
| Mocked components instantiated in production? | N/A | No mocks used for this spec |
| Any "handled separately" without spec reference? | PASS | No deferrals found |
| Integration test exists and passes? | N/A | Unit-only spec |

### Deferral Audit

| Deferral Statement | Location | Tracking Reference |
|--------------------|----------|-------------------|
| None found | N/A | N/A |

### Verdict

APPROVED - All acceptance criteria are met with strong implementation quality. The error handling code is well-structured, follows best practices, and provides excellent user experience with clear visual indicators and informative error messages.
