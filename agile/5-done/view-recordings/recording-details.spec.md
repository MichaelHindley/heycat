---
status: completed
created: 2025-12-01
completed: 2025-12-11
dependencies: ["recordings-list-ui"]
review_round: 1
---

# Spec: Recording Details

## Description

Make recording list entries expandable. When clicked, an entry expands to show additional metadata beyond the default filename/duration/date.

## Acceptance Criteria

- [ ] Clicking a recording entry expands it
- [ ] Expanded view shows additional metadata (file size, full path, etc.)
- [ ] Clicking again collapses the entry
- [ ] Only one entry can be expanded at a time (or multiple - decide)
- [ ] Smooth expand/collapse animation (optional, keep simple)

## Test Cases

- [ ] Clicking entry expands it
- [ ] Clicking expanded entry collapses it
- [ ] Expanded entry shows full metadata
- [ ] Other entries remain collapsed when one expands

## Dependencies

- recordings-list-ui (base component to extend)

## Preconditions

Recordings list UI displays entries

## Implementation Notes

- Could use CSS transitions for expand/collapse
- Consider accordion pattern (one open at a time)
- Metadata to show: file_size, full_path, any other available info

## Related Specs

- recordings-list-ui.spec.md (base component)
- open-recording.spec.md (action in expanded view)

## Integration Points

- Production call site: RecordingsList component
- Connects to: open-recording functionality

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

## Review

**Reviewed:** 2025-12-11
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Clicking a recording entry expands it | PASS | RecordingsList.tsx:113 - onClick handler toggles expansion; test at RecordingsList.test.tsx:162-182 |
| Expanded view shows additional metadata (file size, full path, etc.) | PASS | RecordingsList.tsx:128-137 - Shows file_size and file_path in details section |
| Clicking again collapses the entry | PASS | RecordingsList.tsx:51 - Toggle logic collapses if already expanded; test at RecordingsList.test.tsx:184-207 |
| Only one entry can be expanded at a time (or multiple - decide) | PASS | RecordingsList.tsx:48 - Single expandedPath state ensures accordion behavior; test at RecordingsList.test.tsx:238-266 |
| Smooth expand/collapse animation (optional, keep simple) | PASS | RecordingsList.css:65 - CSS transition on max-height with 0.2s ease-out |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Clicking entry expands it | PASS | RecordingsList.test.tsx:162-182 |
| Clicking expanded entry collapses it | PASS | RecordingsList.test.tsx:184-207 |
| Expanded entry shows full metadata | PASS | RecordingsList.test.tsx:209-236 |
| Other entries remain collapsed when one expands | PASS | RecordingsList.test.tsx:238-266 |

### Code Quality

**Strengths:**
- Clean, well-structured implementation with proper TypeScript types
- Excellent accessibility: uses semantic HTML (button, dl/dt/dd), aria-expanded, aria-hidden attributes
- Proper state management with React hooks
- CSS transitions are smooth and simple as specified
- Comprehensive test coverage including edge cases
- Dark mode support implemented in CSS
- Proper use of Tauri invoke pattern per architecture guidelines
- Helper functions (formatDuration, formatDate, formatFileSize) are properly tested
- Uses BEM-like CSS naming convention for clarity

**Concerns:**
- None identified

### Integration Verification

| Check | Status | Evidence |
|-------|--------|----------|
| Mocked components instantiated in production? | PASS | EmptyState is imported at RecordingsList.tsx:3 and used at line 100 |
| Any "handled separately" without spec reference? | PASS | No deferrals found in implementation |
| Integration test exists and passes? | N/A | Spec explicitly states "N/A (unit-only spec) - integration tested in integration.spec.md" |

### Deferral Audit

| Deferral Statement | Location | Tracking Reference |
|--------------------|----------|-------------------|
| None found | - | - |

### Verdict

APPROVED - Implementation fully meets all acceptance criteria with comprehensive test coverage. The expand/collapse functionality works correctly with smooth animations, proper accessibility support, and follows project conventions. The accordion pattern (single expansion) is correctly implemented and tested.
