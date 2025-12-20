---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["deduplicate-transcription-callbacks"]
---

# Spec: Refactor HotkeyIntegration Config

## Description

Group related `HotkeyIntegration` fields into sub-structs for improved maintainability. The struct currently has 25+ fields, most optional, creating a complex initialization path. Grouping related fields (e.g., transcription-related, audio-related) into logical sub-structs improves readability and makes the builder pattern cleaner.

## Acceptance Criteria

- [ ] Related fields grouped into logical sub-structs (e.g., `TranscriptionConfig`, `AudioConfig`)
- [ ] Builder pattern updated to work with new structure
- [ ] All existing tests pass
- [ ] No functional behavior changes
- [ ] Documentation updated if needed

## Test Cases

- [ ] Existing HotkeyIntegration tests pass unchanged
- [ ] Builder pattern works correctly with new structure
- [ ] Default values preserved for all fields

## Dependencies

- deduplicate-transcription-callbacks (should be done first to avoid conflicts)

## Preconditions

The deduplicate-transcription-callbacks spec should be completed first to avoid merge conflicts during refactoring.

## Implementation Notes

Location: `src-tauri/src/hotkey/integration.rs:76-125`

Suggested grouping:
```rust
struct TranscriptionConfig {
    shared_model: Arc<SharedTranscriptionModel>,
    emitter: Arc<T>,
    semaphore: Arc<Semaphore>,
    timeout: Duration,
}

struct AudioConfig {
    audio_thread: Arc<Mutex<AudioThreadHandle>>,
    recording_state: Arc<Mutex<RecordingManager>>,
}

struct HotkeyIntegration<T, E, R, C> {
    transcription: Option<TranscriptionConfig>,
    audio: Option<AudioConfig>,
    // ... other fields
}
```

This is a larger refactoring that should be done carefully to avoid breaking changes.

## Related Specs

- deduplicate-transcription-callbacks (dependency)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: Multiple modules (TranscriptionEventEmitter, RecordingManager, etc.)

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

## Data Flow Documentation

### Current Structure (BEFORE)

The `HotkeyIntegration` struct has **24 flat fields** - all at the same level:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          HotkeyIntegration<R, T, C>                         │
├─────────────────────────────────────────────────────────────────────────────┤
│  DEBOUNCE/TIMING                                                            │
│  ├── last_toggle_time: Option<Instant>                                      │
│  └── debounce_duration: Duration                                            │
│                                                                             │
│  TRANSCRIPTION (scattered across struct)                                    │
│  ├── shared_transcription_model: Option<Arc<SharedTranscriptionModel>>      │
│  ├── transcription_emitter: Option<Arc<T>>                                  │
│  ├── transcription_semaphore: Arc<Semaphore>                                │
│  ├── transcription_timeout: Duration                                        │
│  └── transcription_callback: Option<Arc<dyn Fn(String)>>                    │
│                                                                             │
│  AUDIO (scattered)                                                          │
│  ├── audio_thread: Option<Arc<AudioThreadHandle>>                           │
│  ├── recording_state: Option<Arc<Mutex<RecordingManager>>>                  │
│  ├── recording_emitter: R                                                   │
│  └── recording_detectors: Option<Arc<Mutex<RecordingDetectors>>>            │
│                                                                             │
│  SILENCE DETECTION                                                          │
│  ├── silence_detection_enabled: bool                                        │
│  └── silence_config: Option<SilenceConfig>                                  │
│                                                                             │
│  VOICE COMMANDS                                                             │
│  ├── command_registry: Option<Arc<Mutex<CommandRegistry>>>                  │
│  ├── command_matcher: Option<Arc<CommandMatcher>>                           │
│  ├── action_dispatcher: Option<Arc<ActionDispatcher>>                       │
│  └── command_emitter: Option<Arc<C>>                                        │
│                                                                             │
│  LISTENING/WAKE WORD                                                        │
│  ├── listening_state: Option<Arc<Mutex<ListeningManager>>>                  │
│  └── listening_pipeline: Option<Arc<Mutex<ListeningPipeline>>>              │
│                                                                             │
│  ESCAPE KEY HANDLING                                                        │
│  ├── shortcut_backend: Option<Arc<dyn ShortcutBackend>>                     │
│  ├── escape_callback: Option<Arc<dyn Fn()>>                                 │
│  ├── escape_registered: Arc<AtomicBool>                                     │
│  ├── double_tap_window_ms: u64                                              │
│  └── double_tap_detector: Option<Arc<Mutex<DoubleTapDetector>>>             │
│                                                                             │
│  APP                                                                        │
│  └── app_handle: Option<AppHandle>                                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Current Builder Chain (Verbose - 16+ methods):**
```rust
HotkeyIntegration::new(emitter)
    .with_audio_thread(audio)
    .with_shared_transcription_model(model)
    .with_transcription_emitter(tx_emitter)
    .with_recording_state(rec_state)
    .with_listening_state(listen_state)
    .with_command_registry(registry)
    .with_command_matcher(matcher)
    .with_action_dispatcher(dispatcher)
    .with_command_emitter(cmd_emitter)
    .with_listening_pipeline(pipeline)
    .with_recording_detectors(detectors)
    .with_silence_config(config)
    .with_shortcut_backend(backend)
    .with_escape_callback(callback)
    .with_app_handle(handle)
```

### Proposed Structure (AFTER)

Group related fields into **logical sub-structs**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          HotkeyIntegration<R, T, C>                         │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ TranscriptionConfig                                                  │   │
│  │  ├── shared_model: Arc<SharedTranscriptionModel>                     │   │
│  │  ├── emitter: Arc<T>                                                 │   │
│  │  ├── semaphore: Arc<Semaphore>                                       │   │
│  │  ├── timeout: Duration                                               │   │
│  │  └── callback: Option<Arc<dyn Fn(String)>>                           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ AudioConfig                                                          │   │
│  │  ├── thread: Arc<AudioThreadHandle>                                  │   │
│  │  ├── recording_state: Arc<Mutex<RecordingManager>>                   │   │
│  │  ├── recording_emitter: R                                            │   │
│  │  └── detectors: Option<Arc<Mutex<RecordingDetectors>>>               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ SilenceDetectionConfig                                               │   │
│  │  ├── enabled: bool                                                   │   │
│  │  └── config: Option<SilenceConfig>                                   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ VoiceCommandConfig                                                   │   │
│  │  ├── registry: Arc<Mutex<CommandRegistry>>                           │   │
│  │  ├── matcher: Arc<CommandMatcher>                                    │   │
│  │  ├── dispatcher: Arc<ActionDispatcher>                               │   │
│  │  └── emitter: Arc<C>                                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ EscapeKeyConfig                                                      │   │
│  │  ├── backend: Arc<dyn ShortcutBackend>                               │   │
│  │  ├── callback: Arc<dyn Fn()>                                         │   │
│  │  ├── registered: Arc<AtomicBool>                                     │   │
│  │  ├── double_tap_window_ms: u64                                       │   │
│  │  └── detector: Option<Arc<Mutex<DoubleTapDetector>>>                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  // Top-level fields (few remaining)                                        │
│  ├── transcription: Option<TranscriptionConfig>                             │
│  ├── audio: Option<AudioConfig>                                             │
│  ├── silence: SilenceDetectionConfig                                        │
│  ├── voice_commands: Option<VoiceCommandConfig>                             │
│  ├── escape: Option<EscapeKeyConfig>                                        │
│  ├── listening_state: Option<Arc<Mutex<ListeningManager>>>                  │
│  ├── listening_pipeline: Option<Arc<Mutex<ListeningPipeline>>>              │
│  ├── app_handle: Option<AppHandle>                                          │
│  ├── last_toggle_time: Option<Instant>                                      │
│  └── debounce_duration: Duration                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Proposed Builder Chain (Clean - ~5 methods):**
```rust
HotkeyIntegration::new(emitter)
    .with_transcription(TranscriptionConfig {
        shared_model: model,
        emitter: tx_emitter,
        semaphore: Arc::new(Semaphore::new(2)),
        timeout: Duration::from_secs(60),
        callback: None,
    })
    .with_audio(AudioConfig {
        thread: audio,
        recording_state: rec_state,
        recording_emitter: emitter,
        detectors: Some(detectors),
    })
    .with_voice_commands(VoiceCommandConfig {
        registry, matcher, dispatcher, emitter: cmd_emitter
    })
    .with_escape(EscapeKeyConfig { ... })
    .with_app_handle(handle)
```

### Data Flow Example: Recording Stop → Transcription

**Current (scattered fields):**
```
toggle_recording()
    │
    ├── self.audio_thread.stop()           ← field 1
    ├── self.recording_state.get_buffer()  ← field 2
    ├── spawn_transcription()
    │     ├── self.shared_transcription_model  ← field 3
    │     ├── self.transcription_emitter       ← field 4
    │     ├── self.transcription_semaphore     ← field 5
    │     └── self.transcription_timeout       ← field 6
    └── 6 scattered fields accessed
```

**After (grouped configs):**
```
toggle_recording()
    │
    ├── self.audio.thread.stop()              ← AudioConfig
    ├── self.audio.recording_state.get_buffer()
    ├── spawn_transcription()
    │     └── self.transcription.*            ← TranscriptionConfig (all together)
    └── 2 config structs, clear ownership
```
