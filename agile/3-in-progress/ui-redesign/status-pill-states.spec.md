---
status: in-progress
created: 2025-12-17
completed: null
dependencies:
  - design-system-foundation
  - base-ui-components
---

# Spec: Status Pill States

## Description

Create the status pill component that displays in the header, showing the current app state with appropriate colors and animations.

**Source of Truth:** `ui.md` - Part 2.2 (Header Bar), Part 3.4 (Status Indicators), Part 5.3 (State Transitions)

## Acceptance Criteria

### Status Pill Component
- [ ] Pill-shaped container with rounded ends
- [ ] Text label showing current state
- [ ] Icon appropriate to state (optional)
- [ ] Smooth transitions between states

### State Visualizations (ui.md 2.2, 5.3)
- [ ] **Idle**: Neutral gray background, "Ready" text
- [ ] **Listening**: Teal background with pulse/glow animation, "Listening..." text
- [ ] **Recording**: Red background with pulse animation, "Recording" text, duration timer
- [ ] **Processing**: Amber background with spinner, "Processing..." text

### Animations (ui.md 1.6, 3.4)
- [ ] Recording pulse: Red glow pulsing at 1.5s interval
- [ ] Listening glow: Soft teal ambient glow, breathing at 2s interval
- [ ] Processing: Rotating spinner or animated dots
- [ ] State transitions: Smooth color/size transitions (200ms)

### Integration
- [ ] Connects to app state (useRecording, useTranscription hooks)
- [ ] Updates in real-time as app state changes

## Test Cases

- [ ] Idle state renders gray pill with "Ready"
- [ ] Listening state shows teal with animation
- [ ] Recording state shows red with pulse and timer
- [ ] Processing state shows amber with spinner
- [ ] Transitions animate smoothly between states
- [ ] Component responds to hook state changes

## Dependencies

- design-system-foundation (uses state colors, animations)
- base-ui-components (may use StatusIndicator primitives)

## Preconditions

- Design system with state colors defined
- App state hooks available (useRecording, useTranscription, etc.)

## Implementation Notes

**Files to create:**
```
src/components/ui/
├── StatusPill.tsx
└── StatusPill.test.tsx
```

**State colors from ui.md:**
```css
--recording:    #EF4444    /* Red pulse */
--listening:    #5BB5B5    /* Teal glow */
--processing:   #F59E0B    /* Amber */
--idle:         #737373    /* Neutral gray */
```

**Animation keyframes:**
```css
@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.7; transform: scale(1.1); }
}

@keyframes breathe {
  0%, 100% { box-shadow: 0 0 10px var(--listening); }
  50% { box-shadow: 0 0 20px var(--listening); }
}
```

**State machine mapping:**
- Idle → status: 'idle'
- Listening (wake word active) → status: 'listening'
- Recording → status: 'recording'
- Transcribing → status: 'processing'

## Related Specs

- design-system-foundation, base-ui-components (dependencies)
- layout-shell (renders StatusPill in header)
- page-dashboard (uses status info)

## Integration Points

- Production call site: `src/components/layout/Header.tsx`
- Connects to: useRecording, useTranscription, useListening hooks

## Integration Test

- Test location: `src/components/ui/__tests__/StatusPill.test.tsx`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-18
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Pill-shaped container with rounded ends | PASS | src/components/ui/StatusPill.tsx:68 `rounded-full` class |
| Text label showing current state | PASS | src/components/ui/StatusPill.tsx:85-86 displays label |
| Icon appropriate to state (optional) | PASS | src/components/ui/StatusPill.tsx:79-84 Loader2 spinner for processing |
| Smooth transitions between states | PASS | src/components/ui/StatusPill.tsx:71 `transition-all duration-[var(--duration-normal)]` |
| Idle: Neutral gray background, "Ready" text | FAIL | Missing CSS class `bg-neutral-400` should use --idle color |
| Listening: Teal background with pulse/glow, "Listening..." text | FAIL | Missing CSS class definition for `bg-listening` |
| Recording: Red background with pulse, "Recording" text, duration timer | FAIL | Missing CSS class definition for `bg-recording` |
| Processing: Amber background with spinner, "Processing..." text | FAIL | Missing CSS class definition for `bg-processing` |
| Recording pulse animation | PASS | src/styles/globals.css:250-256 and StatusPill.tsx:35 |
| Listening glow animation | PASS | src/styles/globals.css:259-265 and StatusPill.tsx:29 |
| Processing spinner animation | PASS | src/components/ui/StatusPill.tsx:81 animate-spin |
| State transition animations | PASS | src/components/ui/StatusPill.tsx:71 200ms transitions |
| Connects to app state hooks | FAIL | No production usage - only type import in useAppStatus.ts |
| Updates in real-time as app state changes | FAIL | Component not rendered in production code |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Idle state renders gray pill with "Ready" | PASS | src/components/ui/StatusPill.test.tsx:7-14 |
| Listening state shows teal with animation | PASS | src/components/ui/StatusPill.test.tsx:16-24 |
| Recording state shows red with pulse and timer | PASS | src/components/ui/StatusPill.test.tsx:26-34 |
| Processing state shows amber with spinner | PASS | src/components/ui/StatusPill.test.tsx:36-48 |
| Transitions animate smoothly between states | PASS | src/components/ui/StatusPill.tsx:71 transition classes |
| Component responds to hook state changes | MISSING | No integration test with actual hooks |
| AutoTimer increments during recording | PASS | src/components/ui/StatusPill.test.tsx:106-123 |
| AutoTimer resets on status change | PASS | src/components/ui/StatusPill.test.tsx:125-142 |

### Code Quality

**Strengths:**
- Well-structured component with clear separation between StatusPill and AutoTimerStatusPill
- Comprehensive unit tests with good coverage
- Proper accessibility attributes (role="status", aria-live="polite", aria-label)
- Clean animation keyframes defined in globals.css
- Type-safe status configuration

**Concerns:**
- **CRITICAL**: Missing CSS utility classes - `bg-listening`, `bg-recording`, and `bg-processing` are referenced in StatusPill.tsx but not defined in any CSS file
- **CRITICAL**: Component not integrated into production code - StatusPill is only imported as a type in useAppStatus.ts, never rendered
- **CRITICAL**: Header.tsx uses StatusIndicator component, not StatusPill - wrong component is being used in production
- Missing integration test that verifies the component works with actual useRecording/useTranscription/useListening hooks
- useAppStatus hook exists and provides correct status derivation but is not used anywhere

### Verdict

**NEEDS_WORK** - Component not wired up end-to-end. Three critical issues:
1. Missing CSS classes (bg-listening, bg-recording, bg-processing) will cause broken styling
2. Header.tsx uses StatusIndicator instead of StatusPill - component exists but not connected
3. useAppStatus hook exists but has no production call site
