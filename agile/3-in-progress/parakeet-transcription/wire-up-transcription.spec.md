---
status: in-progress
created: 2025-12-13
completed: null
dependencies:
  - tdt-batch-transcription.spec.md
  - eou-streaming-transcription.spec.md
---

# Spec: Wire up TranscriptionManager

## Description

Connect the new TranscriptionManager (Parakeet) to the application initialization in `lib.rs` and update HotkeyIntegration to use TranscriptionManager instead of WhisperManager. This spec replaces all Whisper integration points with Parakeet equivalents while maintaining the same external behavior.

## Acceptance Criteria

- [ ] `lib.rs` creates `TranscriptionManager` instead of `WhisperManager`
- [ ] `TranscriptionManager` is registered with Tauri state via `app.manage()`
- [ ] `HotkeyIntegration` builder uses `.with_transcription_manager()` instead of `.with_whisper_manager()`
- [ ] Eager model loading at startup loads Parakeet model (if available) instead of Whisper
- [ ] Model existence check uses new model path structure (directory-based)
- [ ] Transcription mode (batch/streaming) is read from settings on startup
- [ ] Mode can be changed at runtime via Tauri command
- [ ] `transcription_partial` events are emitted during streaming mode
- [ ] `transcription_completed` events are emitted after batch/streaming completion
- [ ] All existing recording workflow continues to work

## Test Cases

- [ ] App starts successfully with TranscriptionManager
- [ ] Recording starts without errors
- [ ] Recording stops and triggers transcription
- [ ] Batch mode: transcription_completed event emitted with full text
- [ ] Streaming mode: transcription_partial events emitted during recording
- [ ] Mode switching via command updates TranscriptionManager behavior
- [ ] Model loading at startup succeeds when model files exist
- [ ] Model loading gracefully handles missing model files

## Dependencies

- `tdt-batch-transcription.spec.md` - TDT transcriber must be implemented
- `eou-streaming-transcription.spec.md` - EOU transcriber must be implemented

## Preconditions

- `TranscriptionManager` struct exists in `parakeet/manager.rs`
- `TranscriptionManager` implements the `TranscriptionService` trait
- Both `ParakeetTDT` and `ParakeetEOU` wrappers are implemented

## Implementation Notes

### lib.rs Changes

Replace:
```rust
// OLD
use crate::whisper;
let whisper_manager = Arc::new(whisper::WhisperManager::new());
app.manage(whisper_manager.clone());
// ...
.with_whisper_manager(whisper_manager)
```

With:
```rust
// NEW
use crate::parakeet;
let transcription_manager = Arc::new(parakeet::TranscriptionManager::new());
app.manage(transcription_manager.clone());
// ...
.with_transcription_manager(transcription_manager)
```

### Eager Model Loading Changes

Replace:
```rust
// OLD
if let Ok(true) = model::check_model_exists() {
    if let Ok(model_path) = model::download::get_model_path() {
        info!("Loading whisper model from {:?}...", model_path);
        match whisper::TranscriptionService::load_model(
            whisper_manager.as_ref(),
            &model_path,
        ) {
            Ok(()) => info!("Whisper model loaded successfully"),
            Err(e) => warn!("Failed to load whisper model: {}", e),
        }
    }
}
```

With:
```rust
// NEW
if let Ok(true) = model::check_model_exists_for_type(model::ModelType::ParakeetTDT) {
    if let Ok(model_dir) = model::download::get_model_dir(model::ModelType::ParakeetTDT) {
        info!("Loading Parakeet TDT model from {:?}...", model_dir);
        match transcription_manager.load_tdt_model(&model_dir) {
            Ok(()) => info!("Parakeet TDT model loaded successfully"),
            Err(e) => warn!("Failed to load Parakeet TDT model: {}", e),
        }
    }
}

if let Ok(true) = model::check_model_exists_for_type(model::ModelType::ParakeetEOU) {
    if let Ok(model_dir) = model::download::get_model_dir(model::ModelType::ParakeetEOU) {
        info!("Loading Parakeet EOU model from {:?}...", model_dir);
        match transcription_manager.load_eou_model(&model_dir) {
            Ok(()) => info!("Parakeet EOU model loaded successfully"),
            Err(e) => warn!("Failed to load Parakeet EOU model: {}", e),
        }
    }
}
```

### HotkeyIntegration Changes (src-tauri/src/hotkey/integration.rs)

Replace `WhisperManager` type with `TranscriptionManager`:

```rust
// OLD
use crate::whisper::{TranscriptionService, WhisperManager};

pub struct HotkeyIntegration<R, T, C> {
    whisper_manager: Option<Arc<WhisperManager>>,
    // ...
}

pub fn with_whisper_manager(mut self, manager: Arc<WhisperManager>) -> Self {
    self.whisper_manager = Some(manager);
    self
}
```

With:
```rust
// NEW
use crate::parakeet::{TranscriptionManager, TranscriptionService};

pub struct HotkeyIntegration<R, T, C> {
    transcription_manager: Option<Arc<TranscriptionManager>>,
    // ...
}

pub fn with_transcription_manager(mut self, manager: Arc<TranscriptionManager>) -> Self {
    self.transcription_manager = Some(manager);
    self
}
```

### spawn_transcription() Changes

The transcription spawning logic needs to handle both modes:

```rust
fn spawn_transcription(&self) {
    let transcription_manager = match &self.transcription_manager {
        Some(tm) => tm.clone(),
        None => return,
    };

    // Check current mode
    let mode = transcription_manager.current_mode();

    match mode {
        TranscriptionMode::Batch => {
            // Use TDT for batch transcription (existing flow)
            // ...existing transcription logic...
        }
        TranscriptionMode::Streaming => {
            // For streaming, transcription happens during recording
            // This is handled by streaming_audio_integration
            // Just emit final event here
        }
    }
}
```

### New Tauri Commands

Add to `lib.rs` invoke_handler:
```rust
tauri::generate_handler![
    // ...existing commands...
    parakeet::set_transcription_mode,
    parakeet::get_transcription_mode,
]
```

### TranscriptionManager Interface

```rust
impl TranscriptionManager {
    pub fn new() -> Self;
    pub fn load_tdt_model(&self, model_dir: &Path) -> Result<(), TranscriptionError>;
    pub fn load_eou_model(&self, model_dir: &Path) -> Result<(), TranscriptionError>;
    pub fn current_mode(&self) -> TranscriptionMode;
    pub fn set_mode(&self, mode: TranscriptionMode) -> Result<(), TranscriptionError>;
    pub fn is_tdt_loaded(&self) -> bool;
    pub fn is_eou_loaded(&self) -> bool;
}

impl TranscriptionService for TranscriptionManager {
    fn transcribe(&self, samples: &[f32]) -> TranscriptionResult<String>;
    fn is_loaded(&self) -> bool;
    fn state(&self) -> TranscriptionState;
    fn reset_to_idle(&self) -> TranscriptionResult<()>;
}
```

## Related Specs

- `parakeet-module-skeleton.spec.md` - Defines TranscriptionManager structure
- `tdt-batch-transcription.spec.md` - TDT implementation
- `eou-streaming-transcription.spec.md` - EOU implementation
- `streaming-audio-integration.spec.md` - Audio callback integration for streaming

## Integration Points

- Production call site: `src-tauri/src/lib.rs:run()` setup block
- Connects to:
  - `parakeet::TranscriptionManager`
  - `hotkey::HotkeyIntegration`
  - `model::download` (for path resolution)

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs` (extend existing)
- Verification:
  - [ ] Integration test with mock TranscriptionManager passes
  - [ ] Recording → stop → transcription flow works end-to-end

## Review

**Reviewed:** 2025-12-13
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `lib.rs` creates `TranscriptionManager` instead of `WhisperManager` | PASS | `src-tauri/src/lib.rs:72` - `let transcription_manager = Arc::new(parakeet::TranscriptionManager::new());` |
| `TranscriptionManager` is registered with Tauri state via `app.manage()` | PASS | `src-tauri/src/lib.rs:73` - `app.manage(transcription_manager.clone());` |
| `HotkeyIntegration` builder uses `.with_transcription_manager()` instead of `.with_whisper_manager()` | PASS | `src-tauri/src/lib.rs:137` - `.with_transcription_manager(transcription_manager)` and `src-tauri/src/hotkey/integration.rs:99-102` defines `with_transcription_manager()` method |
| Eager model loading at startup loads Parakeet model (if available) instead of Whisper | PASS | `src-tauri/src/lib.rs:102-125` - Loads both TDT and EOU models at startup with proper error handling |
| Model existence check uses new model path structure (directory-based) | PASS | `src-tauri/src/lib.rs:102,115` - Uses `check_model_exists_for_type(ModelType::ParakeetTDT/EOU)` and `get_model_dir()` |
| Transcription mode (batch/streaming) is read from settings on startup | FAIL | No settings integration found. Mode defaults to `Batch` in `TranscriptionManager::new()` (`src-tauri/src/parakeet/manager.rs:34`). No settings file is read at startup. |
| Mode can be changed at runtime via Tauri command | PASS | `src-tauri/src/parakeet/mod.rs:17-39` - `get_transcription_mode` and `set_transcription_mode` commands implemented; registered in `lib.rs:214-215` |
| `transcription_partial` events are emitted during streaming mode | PASS | `src-tauri/src/parakeet/streaming.rs:122` and `src-tauri/src/parakeet/streaming.rs:162` - `emit_transcription_partial()` called during streaming and finalization |
| `transcription_completed` events are emitted after batch/streaming completion | PASS | `src-tauri/src/hotkey/integration.rs:563` (batch mode) and `src-tauri/src/parakeet/streaming.rs:175` (streaming mode) |
| All existing recording workflow continues to work | PASS | Integration tests pass: `src-tauri/src/hotkey/integration_test.rs` includes tests for start/stop recording, debouncing, and full cycles |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| App starts successfully with TranscriptionManager | PASS | `src-tauri/src/hotkey/integration_test.rs:126-141` - `test_toggle_from_idle_starts_recording` |
| Recording starts without errors | PASS | `src-tauri/src/hotkey/integration_test.rs:126-141` |
| Recording stops and triggers transcription | PASS | `src-tauri/src/hotkey/integration_test.rs:143-164` - `test_toggle_from_recording_stops` |
| Batch mode: transcription_completed event emitted with full text | DEFERRED | Requires model to be loaded; no mock transcription manager. Test at `integration_test.rs:493-523` verifies flow but not event emission. |
| Streaming mode: transcription_partial events emitted during recording | DEFERRED | `src-tauri/src/parakeet/streaming.rs:326-337` tests mock emitter tracking but requires loaded EOU model for full integration test |
| Mode switching via command updates TranscriptionManager behavior | PASS | `src-tauri/src/parakeet/manager.rs:397-410` - `test_set_mode_to_streaming` and `test_set_mode_back_to_batch` |
| Model loading at startup succeeds when model files exist | DEFERRED | Requires actual model files; lib.rs startup code handles this but cannot be unit tested |
| Model loading gracefully handles missing model files | PASS | `src-tauri/src/lib.rs:110-112,123-125` - Logs info message when model not found instead of failing |

### Code Quality

**Strengths:**
- Clean builder pattern for `HotkeyIntegration` allows flexible composition
- Proper separation between batch and streaming transcription paths in `handle_toggle()`
- Comprehensive error handling with graceful fallbacks (model not found logs info, doesn't crash)
- Event system properly separates `transcription_partial` (streaming) and `transcription_completed` (both modes)
- Thread-safe design using `Arc<Mutex<>>` for shared state
- Good test coverage for integration scenarios including streaming wire-up tests

**Concerns:**
- **Settings integration missing**: The acceptance criterion "Transcription mode is read from settings on startup" is not implemented. The mode defaults to `Batch` and is only changeable via runtime Tauri command.
- Streaming finalization in `integration.rs:634-679` uses `std::thread::sleep(10ms)` which is a workaround for synchronization - could be fragile under load
- `handle_transcription_result()` in `integration.rs:685-702` has a TODO comment noting it doesn't do full command matching for streaming mode

### Verdict

**NEEDS_WORK** - The implementation is largely complete and well-structured, but the acceptance criterion "Transcription mode (batch/streaming) is read from settings on startup" has not been implemented. The mode currently defaults to `Batch` and can only be changed at runtime via Tauri command. Either implement settings persistence for the transcription mode, or update the spec to remove this requirement if it was deprioritized.
