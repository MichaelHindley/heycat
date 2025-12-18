---
status: in-progress
created: 2025-12-17
completed: null
dependencies:
  - layout-shell
  - base-ui-components
  - toast-notifications
---

# Spec: Recordings Page

## Description

Build the Recordings page with list view, search, filter, and detail expansion for managing voice recordings and transcriptions.

**Source of Truth:** `ui.md` - Part 4.2 (Recordings), Part 3.5 (Lists)

## Acceptance Criteria

### Page Header
- [ ] Title: "Recordings"
- [ ] Subtitle: "Manage your voice recordings and transcriptions."

### Search & Filter Bar
- [ ] Search input with placeholder "Search recordings..."
- [ ] Filter dropdown: All, Transcribed, Pending
- [ ] Sort dropdown: Newest, Oldest, Longest, Shortest

### Recording List (ui.md 3.5, 4.2)
- [ ] Virtualized list for performance (100+ recordings)
- [ ] Collapsed item shows: play button, filename, date, duration, size, status badge

### Recording Item - Collapsed State
- [ ] Play/pause button on left
- [ ] Filename (truncated if long)
- [ ] Metadata: date, duration, file size
- [ ] Status badge: "Transcribed" (green) or "Transcribe" (button)
- [ ] More menu (kebab icon) for additional actions

### Recording Item - Expanded State (ui.md 4.2)
- [ ] Click to expand/collapse (accordion style)
- [ ] Shows transcription text (or "No transcription" message)
- [ ] Action buttons: Copy Text, Open File, Delete
- [ ] Transcription text is scrollable if long

### Actions
- [ ] Play: Plays audio (inline player or system)
- [ ] Transcribe: Triggers transcription, shows progress
- [ ] Copy Text: Copies transcription to clipboard, shows toast
- [ ] Open File: Opens in system file manager
- [ ] Delete: Confirmation dialog, then removes

### Empty State (ui.md 4.2)
- [ ] Friendly illustration or icon
- [ ] "No recordings yet"
- [ ] "Press ⌘⇧R or say 'Hey Cat' to start"
- [ ] Primary button: "Start Recording"

### Loading States
- [ ] Skeleton loaders while fetching
- [ ] Transcription progress indicator

## Test Cases

- [ ] List renders recordings correctly
- [ ] Search filters recordings by name/content
- [ ] Filter dropdown works
- [ ] Sort changes order
- [ ] Click expands/collapses item
- [ ] Play button plays audio
- [ ] Copy button copies and shows toast
- [ ] Delete shows confirmation and removes
- [ ] Empty state shows when no recordings
- [ ] Virtualization works for large lists

## Dependencies

- layout-shell (renders inside AppShell)
- base-ui-components (Card, Button, Input)
- toast-notifications (for copy/delete feedback)

## Preconditions

- Layout shell and toast system completed
- useRecordings hook available
- Audio playback capability

## Implementation Notes

**Files to create:**
```
src/pages/
├── Recordings.tsx
├── Recordings.test.tsx
└── components/
    ├── RecordingItem.tsx
    ├── RecordingItemExpanded.tsx
    └── RecordingsEmptyState.tsx
```

**Collapsed item from ui.md 3.5:**
```
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav                          |
|         Sep 25, 2022 • 00:00:28 • 3.6 MB                         |
+------------------------------------------------------------------+
```

**Expanded item from ui.md 4.2:**
```
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav                          |
|         Sep 25, 2022 • 00:00:28 • 3.6 MB                         |
|------------------------------------------------------------------|
|  TRANSCRIPTION                                                    |
|  Hello, this is a test recording for the HeyCat application.     |
|  I'm testing the voice transcription feature.                    |
|                                                                   |
|  [Copy Text]  [Open File]  [Delete]                              |
+------------------------------------------------------------------+
```

**Virtualization:**
- Use react-window or @tanstack/react-virtual
- Row height: collapsed ~60px, expanded ~200px (variable)

## Related Specs

- layout-shell, base-ui-components, toast-notifications (dependencies)
- page-dashboard (links here from recent activity)

## Integration Points

- Production call site: `src/App.tsx` routes to Recordings
- Connects to: useRecordings, useTranscription hooks, file system APIs

## Integration Test

- Test location: `src/pages/__tests__/Recordings.test.tsx`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-18
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Title: "Recordings" | PASS | src/pages/Recordings.tsx:239-241 |
| Subtitle: "Manage your voice recordings and transcriptions." | PASS | src/pages/Recordings.tsx:242-244 |
| Search input with placeholder "Search recordings..." | PASS | src/pages/Recordings.tsx:252-259 |
| Filter dropdown: All, Transcribed, Pending | PASS | src/pages/Recordings.tsx:263-273 |
| Sort dropdown: Newest, Oldest, Longest, Shortest | PASS | src/pages/Recordings.tsx:276-287 |
| Virtualized list for performance (100+ recordings) | FAIL | No virtualization library imported or used; spec notes recommend react-window or @tanstack/react-virtual |
| Collapsed item shows: play button, filename, date, duration, size, status badge | PASS | src/pages/components/RecordingItem.tsx:79-148 |
| Play/pause button on left | PASS | RecordingItem.tsx:88-103 |
| Filename (truncated if long) | PASS | RecordingItem.tsx:106-109 with truncate class |
| Metadata: date, duration, file size | PASS | RecordingItem.tsx:110-112 |
| Status badge: "Transcribed" (green) or "Transcribe" (button) | PASS | RecordingItem.tsx:116-138 |
| More menu (kebab icon) for additional actions | FAIL | No kebab menu implemented; actions in expanded view only |
| Click to expand/collapse (accordion style) | PASS | RecordingItem.tsx:80-148, Recordings.tsx:108-110 |
| Shows transcription text (or "No transcription" message) | PASS | RecordingItem.tsx:154-168 |
| Action buttons: Copy Text, Open File, Delete | PASS | RecordingItem.tsx:194-222 |
| Transcription text is scrollable if long | PASS | RecordingItem.tsx:159 max-h-32 overflow-y-auto |
| Play: Plays audio (inline player or system) | DEFERRED | handlePlay toggles state but no actual audio playback (comment says "For now") |
| Transcribe: Triggers transcription, shows progress | PASS | Recordings.tsx:118-142, RecordingItem.tsx:126-138 |
| Copy Text: Copies transcription to clipboard, shows toast | PASS | Recordings.tsx:144-161 |
| Open File: Opens in system file manager | PASS | Recordings.tsx:163-173, uses @tauri-apps/plugin-opener |
| Delete: Confirmation dialog, then removes | FAIL | Backend command `delete_recording` not registered in invoke_handler (lib.rs:294) |
| Empty state: Friendly illustration or icon | PASS | RecordingsEmptyState.tsx:13-15 (Mic icon) |
| Empty state: "No recordings yet" | PASS | RecordingsEmptyState.tsx:19 |
| Empty state: "Press Cmd+Shift+R or say 'Hey Cat' to start" | PASS | RecordingsEmptyState.tsx:22-24 |
| Empty state: Primary button "Start Recording" | PASS | RecordingsEmptyState.tsx:28-30 |
| Skeleton loaders while fetching | FAIL | Only shows "Loading recordings..." text, no skeleton loaders |
| Transcription progress indicator | PASS | Button loading state via isTranscribing prop |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| List renders recordings correctly | PASS | Recordings.test.tsx:116-132 |
| Search filters recordings by name/content | PASS | Recordings.test.tsx:151-193 |
| Filter dropdown works | PASS | Implicitly tested via search/filter tests |
| Sort changes order | MISSING | No explicit sort order test |
| Click expands/collapses item | PASS | Recordings.test.tsx:214-245 |
| Play button plays audio | MISSING | No explicit play button test |
| Copy button copies and shows toast | PASS | Recordings.test.tsx:266-311 |
| Delete shows confirmation and removes | PASS | Recordings.test.tsx:337-377 |
| Empty state shows when no recordings | PASS | Recordings.test.tsx:99-114 |
| Virtualization works for large lists | MISSING | Not implemented |

### Code Quality

**Strengths:**
- Well-organized component structure (Recordings, RecordingItem, RecordingsEmptyState)
- Good separation of concerns with individual handlers for each action
- Proper error handling with toast notifications
- Comprehensive test coverage for core functionality (17 passing tests)
- Proper TypeScript types exported for RecordingInfo

**Concerns:**
- DOM nesting warning: button inside button (RecordingItem.tsx - play button nested in expand button)
- Recordings page not exported from src/pages/index.ts
- Recordings page not wired up in App.tsx (line 74 shows "Page coming soon" placeholder)
- `delete_recording` command not registered in backend invoke_handler
- No virtualization for large lists (spec requirement)
- No skeleton loader UI for loading state

### Verdict

**NEEDS_WORK** - The Recordings page component is well-implemented but not wired up end-to-end. Critical issues:
1. Page not routed in App.tsx (line 72-78) - needs to render `<Recordings />` when `navItem === "recordings"`
2. Page not exported from src/pages/index.ts
3. `delete_recording` backend command not registered (would fail at runtime)
4. Virtualization not implemented (spec requirement for 100+ recordings)
5. DOM nesting violation (button inside button) causing React warning
