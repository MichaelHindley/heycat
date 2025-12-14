---
status: pending
created: 2025-12-14
completed: null
dependencies:
  - listening-state-machine
  - activation-feedback
---

# Spec: React hook for listening mode state

## Description

Create a React hook `useListening()` following existing patterns to manage listening mode state in the frontend. Subscribe to Tauri events and expose state/actions for UI components.

## Acceptance Criteria

- [ ] `useListening()` hook created in `src/hooks/useListening.ts`
- [ ] Exposes `isListening` boolean state
- [ ] Exposes `isWakeWordDetected` transient state
- [ ] Exposes `isMicAvailable` boolean for availability indicator
- [ ] Exposes `error` state for error handling
- [ ] `enableListening()` and `disableListening()` action functions
- [ ] Subscribes to: `listening_started`, `listening_stopped`, `wake_word_detected`, `listening_unavailable`
- [ ] Integrates cleanly with existing `useRecording` hook

## Test Cases

- [ ] Hook initializes with correct default state
- [ ] `enableListening()` calls Tauri command and updates state on event
- [ ] `disableListening()` calls Tauri command and updates state on event
- [ ] Wake word detection updates `isWakeWordDetected` temporarily
- [ ] Mic unavailable updates `isMicAvailable` to false
- [ ] Cleanup unsubscribes from all events on unmount

## Dependencies

- listening-state-machine (provides Tauri commands)
- activation-feedback (provides events to subscribe to)

## Preconditions

- Backend listening commands implemented
- Event system for listening state changes

## Implementation Notes

- Follow patterns from `useRecording.ts` and `useTranscription.ts`
- Event-driven state updates (not command response based)
- All listeners set up in async `setupListeners()` function within `useEffect`
- Cleanup happens in return function - calls all unlistens on unmount
- Consider coordination with `useRecording` hook for seamless state

```typescript
// Example interface
interface UseListeningReturn {
  isListening: boolean;
  isWakeWordDetected: boolean;
  isMicAvailable: boolean;
  error: string | null;
  enableListening: () => Promise<void>;
  disableListening: () => Promise<void>;
}
```

## Related Specs

- listening-state-machine.spec.md (backend commands)
- activation-feedback.spec.md (events subscribed to)
- settings-persistence.spec.md (settings integration)

## Integration Points

- Production call site: `src/App.tsx` or relevant component
- Connects to: useRecording, CatOverlay component

## Integration Test

- Test location: `src/hooks/useListening.test.ts`
- Verification: [ ] Integration test passes
