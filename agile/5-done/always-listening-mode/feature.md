---
discovery_phase: complete
---

# Feature: Always Listening Mode

**Created:** 2025-12-14
**Owner:** Michael Hindley
**Discovery Phase:** complete

## Description

Enable hands-free voice activation for heycat using a wake word ("Hey Cat"). When enabled, the app continuously listens for the wake word and automatically starts recording when detected. Supports both transcription mode (wake word → record → transcribe) and command mode (wake word → command → execute). This feature reduces friction for power users and improves accessibility for users who cannot easily use keyboard shortcuts or mouse clicks.

## BDD Scenarios

### User Persona
A power user or professional who uses heycat frequently for work tasks (meetings, content creation, documentation). They may also be a hands-busy user who needs voice control because their hands are occupied with other activities like cooking, working on a project, or multitasking.

### Problem Statement
Users face friction when activating recording - having to click or use a keyboard shortcut interrupts their flow and breaks concentration. Additionally, users with accessibility needs find manual activation difficult or impossible. This creates barriers to seamless voice capture when it's needed most.

```gherkin
Feature: Always Listening Mode

  Scenario: Happy path - Wake word triggers recording
    Given always-listening mode is enabled
    And the microphone is available
    When the user says the wake word "Hey Cat"
    Then the CatOverlay visual indicator shows recording has started
    And the app begins capturing audio
    When the user speaks their content
    And silence is detected for a few seconds
    Then recording automatically stops
    And the captured audio is transcribed

  Scenario: Happy path - Wake word triggers command
    Given always-listening mode is enabled
    And the microphone is available
    When the user says "Hey Cat" followed by a command phrase
    Then the CatOverlay visual indicator shows recording has started
    And the app captures the command
    And the command is interpreted and executed

  Scenario: Error case - False activation cancelled by user
    Given always-listening mode is enabled
    And the app incorrectly detected the wake word
    And recording has started
    When the user says "cancel" or "nevermind"
    Then recording is stopped
    And no transcription is saved

  Scenario: Error case - False activation auto-cancelled
    Given always-listening mode is enabled
    And the app incorrectly detected the wake word
    And recording has started
    When no speech is detected within the timeout period
    Then recording is automatically cancelled
    And no transcription is saved

  Scenario: Error case - Microphone unavailable
    Given always-listening mode is enabled
    When the microphone becomes unavailable or is in use by another application
    Then a status indicator shows always-listening is unavailable
    And wake word detection is paused
```

### Out of Scope
- Custom wake words (ability to change from default)
- Multi-language wake word detection
- Cloud-based wake word detection (all processing is local)
- Advanced command grammar (complex multi-step voice commands)
- Audio playback for feedback (visual-only in MVP, using existing CatOverlay)
- CPU/latency optimization (deferred post-MVP)
- Advanced VAD algorithms (using simple energy-based silence detection)

### Assumptions
- Wake word detection happens entirely on-device, no network required
- User has already granted microphone permissions to the app
- The existing Parakeet transcription model is used for wake word detection (small-window batching)
- Only one fixed wake word ("Hey Cat") will be supported in this initial implementation
- Visual feedback uses existing CatOverlay component (same as hotkey-triggered recording)

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [x] Wake word "Hey Cat" triggers recording when listening mode is enabled
- [x] Visual indicator (CatOverlay) shows when recording starts
- [x] Recording auto-stops after configurable silence period
- [x] User can cancel false activations with "cancel" or "nevermind"
- [x] Settings persist across app restarts
- [x] Listening mode integrates cleanly with existing hotkey recording

## Definition of Done

- [x] All specs completed
- [x] Technical guidance finalized
- [x] Code reviewed and approved
- [x] Tests written and passing
- [x] Documentation updated

---

## Feature Review

**Date:** 2025-12-16
**Reviewer:** Claude (Independent Review Agent)

### Smoke Test Results

- **Backend tests:** All passing (447+ tests)
- **Frontend tests:** All passing (226+ tests)
- **TCR check:** PASS - Committed at ffdcd6b

### BDD Scenario Verification

| Scenario | Status | Evidence |
|----------|--------|----------|
| Wake word triggers recording | VERIFIED | `WakeWordDetector` in `detector.rs` detects "Hey Cat", callback triggers recording via `HotkeyIntegration` |
| Wake word triggers command | VERIFIED | Recording captured and transcribed, integrates with existing voice commands system |
| False activation cancelled by user | VERIFIED | `CancelPhraseDetector` in `cancel.rs` detects "cancel"/"nevermind" within 3-second window |
| False activation auto-cancelled | VERIFIED | `SilenceDetector` in `silence.rs` implements `NoSpeechTimeout` (5 second default) |
| Microphone unavailable | VERIFIED | Pipeline emits `listening_unavailable` event on mic errors, frontend displays status |

### Spec Summary

All 8 specs and 1 integration bug independently reviewed and approved:

| Spec | Status | Reviewer Verdict |
|------|--------|------------------|
| wake-word-detector | completed | APPROVED (Round 2) |
| listening-state-machine | completed | APPROVED (Round 2) |
| listening-audio-pipeline | completed | APPROVED |
| activation-feedback | completed | APPROVED |
| auto-stop-detection | completed | APPROVED |
| cancel-commands | completed | APPROVED |
| frontend-listening-hook | completed | APPROVED |
| settings-persistence | completed | APPROVED |
| integration-wiring.bug | completed | APPROVED |

### High-Level Acceptance Criteria Verification

1. **Wake word "Hey Cat" triggers recording when listening mode is enabled**
   - VERIFIED: `WakeWordDetector::analyze_and_emit()` detects phrase and emits `wake_word_detected` event. Pipeline callback triggers recording via `HotkeyIntegration::handle_toggle()`.

2. **Visual indicator (CatOverlay) shows when recording starts**
   - VERIFIED: `CatOverlay` component responds to `recording_started` event. `useListening` hook exposes `isWakeWordDetected` transient state for visual feedback.

3. **Recording auto-stops after configurable silence period**
   - VERIFIED: `SilenceDetector` with configurable thresholds (default 2 seconds silence after speech). `RecordingDetectors` coordinator monitors during recording.

4. **User can cancel false activations with "cancel" or "nevermind"**
   - VERIFIED: `CancelPhraseDetector` detects cancel phrases within 3-second window. Emits `recording_cancelled` event and aborts without transcription.

5. **Settings persist across app restarts**
   - VERIFIED: `tauri-plugin-store` v2 integrated. `listeningEnabled` and `autoStartListening` preferences persisted and restored on launch.

6. **Listening mode integrates cleanly with existing hotkey recording**
   - VERIFIED: State machine handles concurrent activation correctly. Hotkey during Listening suspends (not disables) listening. Recording completion returns to Listening if `listening_enabled` flag is true.

### Architecture Summary

The implementation establishes a clean layered architecture:

```
Frontend (React)
├── useListening hook (events: listening_started, listening_stopped, wake_word_detected, listening_unavailable)
├── CatOverlay (visual feedback)
└── Settings panel (persistence toggle)

Tauri Commands
├── enable_listening → starts ListeningPipeline
├── disable_listening → stops ListeningPipeline
└── get_listening_status → returns current state

Backend (Rust)
├── ListeningPipeline (continuous audio capture, wake word detection)
├── RecordingDetectors coordinator (silence + cancel detection during recording)
├── WakeWordDetector (Parakeet-based phrase detection)
├── SilenceDetector (RMS-based energy detection)
├── CancelPhraseDetector (phrase matching)
└── CircularBuffer (bounded memory ~192KB)
```

### Test Coverage

- Backend: 447+ tests including new listening module tests
- Frontend: 226+ tests including `useListening.test.ts` with 12 test cases
- Integration: `integration-wiring.bug.md` verified all components are properly instantiated and connected

### Known Limitations (Out of Scope for MVP)

- No custom wake words (fixed "Hey Cat" only)
- No audio feedback (visual only via CatOverlay)
- CPU/latency optimization deferred to post-MVP
- No multi-language support

### Definition of Done Verification

- [x] All 8 specs completed and independently reviewed
- [x] Technical guidance finalized (`technical-guidance.md`)
- [x] All spec reviews: APPROVED
- [x] 673+ tests written and passing
- [x] Documentation in spec files and technical guidance

### Verdict

**APPROVED_FOR_DONE**

The Always Listening Mode feature is complete and ready for production. All BDD scenarios are implemented, all specs have been independently reviewed and approved, and comprehensive test coverage exists. The architecture is clean with proper separation of concerns between the listening pipeline, detection coordinators, and frontend hooks.
