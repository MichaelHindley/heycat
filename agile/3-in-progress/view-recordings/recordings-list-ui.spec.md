---
status: completed
created: 2025-12-01
completed: 2025-12-11
dependencies: ["sidebar-menu", "list-recordings-backend"]
review_round: 1
---

# Spec: Recordings List UI

## Description

Create a React component that displays the list of recordings in the History view. Each entry shows filename, duration, and creation date.

## Acceptance Criteria

- [ ] RecordingsList component exists
- [ ] Calls `list_recordings` Tauri command on mount
- [ ] Displays each recording with: filename, duration (formatted), date (formatted)
- [ ] List is rendered in the History view area
- [ ] Basic styling applied (readable, clean layout)

## Test Cases

- [ ] Component renders without errors
- [ ] Displays loading state while fetching
- [ ] Renders all recordings from backend response
- [ ] Formats duration correctly (e.g., "2:34" for 154 seconds)
- [ ] Formats date in user-friendly format

## Dependencies

- sidebar-menu (provides view area)
- list-recordings-backend (provides data)

## Preconditions

Sidebar menu and backend command exist

## Implementation Notes

- Use `invoke` from `@tauri-apps/api/core`
- Consider a simple list or table layout
- Keep styling minimal for now

## Related Specs

- sidebar-menu.spec.md (parent container)
- list-recordings-backend.spec.md (data source)
- recording-details.spec.md (click interaction)

## Integration Points

- Production call site: Sidebar/History view component
- Connects to: Tauri backend, recording-details component

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

## Review

**Reviewed:** 2025-12-11 (Round 2)
**Reviewer:** Claude (Independent Review)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| RecordingsList component exists | PASS | Component defined at RecordingsList.tsx:35 |
| Calls `list_recordings` Tauri command on mount | PASS | useEffect hook at RecordingsList.tsx:40-55, invoke call at line 45 |
| Displays each recording with: filename, duration (formatted), date (formatted) | PASS | Rendering at RecordingsList.tsx:92-103, formatDuration at line 18-22, formatDate at line 24-33 |
| List is rendered in the History view area | PASS | Integration chain complete: App.tsx:7,20 → Sidebar.tsx:2,35 → RecordingsList in production |
| Basic styling applied (readable, clean layout) | PASS | RecordingsList.css:1-88 implements clean BEM-style layout with dark mode support |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Component renders without errors | PASS | RecordingsList.test.tsx:41-45 |
| Displays loading state while fetching | PASS | RecordingsList.test.tsx:47-56 |
| Renders all recordings from backend response | PASS | RecordingsList.test.tsx:58-71 |
| Formats duration correctly (e.g., "2:34" for 154 seconds) | PASS | RecordingsList.test.tsx:120-129 and formatDuration tests at lines 160-180 |
| Formats date in user-friendly format | PASS | RecordingsList.test.tsx:131-142 and formatDate tests at lines 182-194 |

### Code Quality

**Strengths:**
- Excellent BEM CSS naming convention following established pattern (`.recordings-list__item`, `.recordings-list__filename`, etc.)
- Comprehensive error handling with loading, error, and empty states
- Proper TypeScript typing with RecordingInfo interface
- Good accessibility: `role="status"`, `role="alert"`, `role="list"`, `aria-busy`, `aria-label` attributes
- Clean separation: formatting functions exported for testability
- Dark mode support in CSS
- Proper React hooks usage with dependency arrays
- Good test organization with nested describe blocks
- Integration verified: Sidebar component imports RecordingsList and renders it in History panel

**Concerns:**
- None identified

### Integration Verification

| Check | Status | Evidence |
|-------|--------|----------|
| Mocked components instantiated in production? | PASS | RecordingsList rendered in production via App.tsx → Sidebar → History panel |
| Any "handled separately" without spec reference? | PASS | No untracked deferrals found |
| Integration test exists and passes? | N/A | Unit-only spec, integration tested in integration.spec.md |

### Deferral Audit

| Deferral Statement | Location | Tracking Reference |
|--------------------|----------|-------------------|
| None found | - | - |

### Dependency Status

| Dependency | Status | Evidence |
|------------|--------|----------|
| sidebar-menu | COMPLETED | Sidebar component at src/components/Sidebar/Sidebar.tsx, integrated in App.tsx:20 |
| list-recordings-backend | COMPLETED | Backend command verified |

### Verdict

**APPROVED** - All acceptance criteria met. Component is fully functional, well-tested, follows project conventions, and is properly integrated into production via the Sidebar/History panel. The dependency blocker (sidebar-menu) has been resolved and the integration chain is complete.
