---
status: in-progress
created: 2025-12-14
completed: null
dependencies:
  - listening-state-machine
  - listening-audio-pipeline
---

# Spec: Visual feedback on activation

## Description

Provide clear visual feedback when the wake word is detected and recording begins. Use the existing CatOverlay component for visual indication, maintaining consistency with hotkey-triggered recording. Support accessibility by showing clear state transitions.

> **MVP Note**: Audio feedback (confirmation sounds) deferred to post-MVP. This spec focuses on visual feedback only using existing UI components.

## Acceptance Criteria

- [ ] CatOverlay shows when wake word detected (same as hotkey-triggered recording)
- [ ] Visual indicator distinguishes Listening vs Recording states (e.g., different animations or colors)
- [ ] Smooth transition between states
- [ ] Feedback appears immediately on detection event (<100ms UI response)

## Test Cases

- [ ] CatOverlay appears on wake word detection
- [ ] Visual indicator updates on state change
- [ ] Overlay correctly shows mic unavailable state
- [ ] State transitions are visually smooth
- [ ] Wake word activation looks identical to hotkey activation

## Dependencies

- listening-state-machine (provides state events)
- listening-audio-pipeline (provides mic availability events)

## Preconditions

- Listening state machine functional
- Cat overlay system functional
- Frontend event subscription working

## Implementation Notes

- Reuse existing CatOverlay component - no new UI components needed
- Subscribe to new events: `listening_started`, `listening_stopped`, `wake_word_detected`, `listening_unavailable`
- Consider adding a subtle "listening" indicator (e.g., different cat animation or icon badge)
- May need to extend `useCatOverlay` hook or create `useListening` hook to coordinate

## Related Specs

- listening-state-machine.spec.md (triggers feedback)
- frontend-listening-hook.spec.md (manages UI state)

## Integration Points

- Production call site: `src/components/CatOverlay.tsx`, `src/hooks/useCatOverlay.ts`
- Connects to: useListening hook, Tauri event listeners

## Integration Test

- Test location: `src/components/CatOverlay.test.tsx`
- Verification: [ ] Integration test passes
