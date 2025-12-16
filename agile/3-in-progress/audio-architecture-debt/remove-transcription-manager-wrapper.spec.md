---
status: pending
created: 2025-12-16
completed: null
dependencies: []
priority: P1
---

# Spec: Remove or repurpose TranscriptionManager wrapper

## Description

`TranscriptionManager` in `parakeet/manager.rs` is a thin wrapper around `SharedTranscriptionModel` that adds no value. It simply passes through all calls without adding any functionality. This violates DRY and creates unnecessary indirection.

Either remove the wrapper entirely (consumers use SharedTranscriptionModel directly) or give it a meaningful purpose (e.g., managing multiple models, caching, metrics).

## Acceptance Criteria

- [ ] Evaluate: Remove wrapper OR give it real responsibility
- [ ] If removing: Update all callers to use SharedTranscriptionModel directly
- [ ] If keeping: Add meaningful functionality (document what)
- [ ] Remove unused code paths
- [ ] Update tests accordingly

## Test Cases

- [ ] Test that transcription still works after change
- [ ] Test all callers updated correctly
- [ ] Test no regression in functionality

## Dependencies

- transcription-race-condition.spec.md (modifying same module)

## Preconditions

- Current TranscriptionManager wrapper exists

## Implementation Notes

**File:** `src-tauri/src/parakeet/manager.rs`

**Current state:**
TranscriptionManager wraps SharedTranscriptionModel but just forwards calls:
```rust
pub struct TranscriptionManager {
    shared_model: Arc<SharedTranscriptionModel>,
}

impl TranscriptionManager {
    pub fn new() -> Self { ... }

    pub fn transcribe_file(&self, path: &str) -> Result<...> {
        self.shared_model.transcribe_file(path)  // Just forwards!
    }

    pub fn get_shared_model(&self) -> Arc<SharedTranscriptionModel> {
        self.shared_model.clone()  // Exposes underlying model anyway
    }
}
```

**Option A: Remove wrapper (recommended)**
- Delete manager.rs
- Update exports in mod.rs
- Callers use SharedTranscriptionModel directly
- SharedTranscriptionModel is already Arc-wrapped and thread-safe

**Callers to update:**
- `src-tauri/src/commands/logic.rs` - AppState uses TranscriptionManager
- `src-tauri/src/hotkey/integration.rs` - Uses TranscriptionManager
- `src-tauri/src/listening/manager.rs` - Uses shared model

**Option B: Give it purpose**
If we keep it, possible responsibilities:
- Manage model lifecycle (load on demand, unload on timeout)
- Track transcription metrics
- Implement caching for repeated transcriptions
- Support multiple models (different languages/sizes)

**Recommendation:** Option A - Remove the wrapper. The wrapper adds cognitive overhead without benefit. SharedTranscriptionModel is already the right abstraction.

## Related Specs

- transcription-race-condition.spec.md (modifies shared.rs)
- shared-transcription-model.spec.md (completed)

## Integration Points

- Production call site: `src-tauri/src/commands/logic.rs`
- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: AppState, HotkeyIntegration

## Integration Test

- Test location: Existing transcription tests
- Verification: [ ] Integration test passes
