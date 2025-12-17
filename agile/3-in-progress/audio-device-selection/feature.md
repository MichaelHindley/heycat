---
discovery_phase: complete
---

# Feature: Audio Input Device Selection

**Created:** 2025-12-15
**Owner:** Claude
**Discovery Phase:** not_started

## Description

Add a device selection UI with persisted settings, allowing users to choose their preferred audio input device (microphone). This solves the Bluetooth audio quality degradation issue where using a Bluetooth headset's microphone forces a profile switch from A2DP (high-quality stereo) to HFP/HSP (low-quality mono).

By allowing users to select a different input device (e.g., MacBook's built-in microphone) while keeping their Bluetooth headset for audio output, they can maintain high audio quality during recording and listening modes.

## BDD Scenarios

### User Persona
A diverse range of heycat users—from casual users who want their preferred microphone to "just work," to power users with multiple audio devices who need fine control, to professionals (content creators, podcasters, remote workers) with specific audio requirements. All share a need for reliable, visible control over audio input selection.

### Problem Statement
Users experience two key pain points: (1) The system picks the wrong microphone by default—commonly selecting a webcam mic or Bluetooth headset mic instead of their preferred input device, which can also trigger Bluetooth profile switches that degrade audio quality. (2) There's no visibility into which audio device the app is currently using, leaving users uncertain whether their settings are applied correctly. This is critical to solve because recording is core functionality, users are reporting these issues, and improving this directly enhances UX.

```gherkin
Feature: Audio Input Device Selection

  Scenario: Happy path - User selects preferred audio device
    Given the user has multiple audio input devices available
    And the user opens the Settings page
    When the user navigates to the Listening settings tab
    And selects their preferred microphone from the device list
    Then the selected device is saved as the active input
    And the setting persists after closing and reopening the app

  Scenario: First-time setup - No device previously selected
    Given the user has never selected an audio input device
    And at least one audio input device is available
    When the user opens the Listening settings tab
    Then the system default device is shown as selected
    And the user can see all available devices to choose from

  Scenario: Device reconnection - Previously selected device becomes available
    Given the user previously selected "USB Microphone" as their input device
    And the "USB Microphone" was disconnected
    When the "USB Microphone" is reconnected
    Then the app automatically uses "USB Microphone" for audio input
    And the user sees their previously selected device is active

  Scenario: Error case - Selected device unavailable at recording time
    Given the user has selected "External Mic" as their input device
    And "External Mic" is disconnected
    When the user attempts to start recording
    Then a dialog prompts the user to select a different device or retry
    And recording does not start until a valid device is selected

  Scenario: Error case - No audio input devices found
    Given no audio input devices are detected on the system
    When the user opens the Listening settings tab
    Then a message indicates no devices are available
    And the user is prompted to connect an audio input device

  Scenario: Error case - Microphone permission denied
    Given the app does not have microphone access permission
    When the user opens the Listening settings tab
    Then a message explains microphone permission is required
    And the user is prompted to grant permission in system settings

  Scenario: Error case - Device disconnects mid-recording
    Given the user is actively recording with "USB Microphone"
    When "USB Microphone" is disconnected during recording
    Then recording stops
    And a dialog prompts the user to select a different device or retry

  Scenario: Audio level meter - Visual feedback for selected device
    Given the user has selected an audio input device
    When the user views the device selection UI
    Then an audio level meter displays real-time input levels
    And the user can verify the microphone is picking up sound
```

### Out of Scope
- Output device selection (speakers/headphones) - only input (microphone) selection
- Audio processing features (noise cancellation, gain control, audio effects)
- Multi-device recording (recording from multiple microphones simultaneously)

### Assumptions
- Tauri/Rust can enumerate and select audio input devices on all target platforms (macOS initially)

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Users can view a list of available audio input devices
- [ ] Users can select which device to use for recording/listening
- [ ] Selected device persists across app restarts
- [ ] System gracefully falls back to default device if selected device unavailable
- [ ] Settings UI integrates with existing Listening settings tab
- [ ] Audio level meter shows real-time input levels for the selected device

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated
