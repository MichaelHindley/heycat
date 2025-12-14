---
status: pending
created: 2025-12-14
completed: null
dependencies: []
---

# Spec: CTC English-Only Transcription Support

## Description

Add support for the Parakeet CTC (Connectionist Temporal Classification) model - an English-only variant that offers faster transcription with punctuation and capitalization. This extends the current dual-model system (TDT/EOU) to a three-model system, with a three-way mode toggle in the frontend.

**CTC Model Source**: https://huggingface.co/onnx-community/parakeet-ctc-0.6b-ONNX/tree/main/onnx

**Key Features**:
- English-only (no multilingual support)
- Fast & accurate transcription
- Includes punctuation and capitalization
- Uses `Parakeet::from_pretrained()` API from parakeet-rs

## Acceptance Criteria

**Backend - Model Definition:**
- [ ] Add `ParakeetCTC` variant to `ModelType` enum with serde name `"ctc"`
- [ ] Add `dir_name()` returning `"parakeet-ctc"` for CTC variant
- [ ] Add `ModelManifest::ctc()` with HuggingFace URLs and file sizes

**Backend - Transcription:**
- [ ] Add `BatchCTC` variant to `TranscriptionMode` enum
- [ ] Add `load_ctc_model()` to `TranscriptionManager`
- [ ] Add `is_ctc_loaded()` check method
- [ ] Implement CTC transcription using `Parakeet::from_pretrained()` and `transcribe_file()`
- [ ] Add CTC to eager loading at startup (`lib.rs`)
- [ ] Return text-only output (no timestamps)

**Frontend - Model Management:**
- [ ] Extend `ModelType` type: `"tdt" | "eou" | "ctc"`
- [ ] Add CTC status tracking in `useMultiModelStatus` hook
- [ ] Add CTC `ModelDownloadCard` to `TranscriptionSettings` with description "English-only, Fast & Accurate"

**Frontend - Mode Selection:**
- [ ] Convert `ModeToggle` from two-way to three-way toggle
- [ ] Options: "Batch (Multilingual)" → TDT, "Batch CTC (English)" → CTC, "Streaming" → EOU
- [ ] Update `set_transcription_mode` to handle `BatchCTC`

## Test Cases

- [ ] Unit: `ModelManifest::ctc()` returns 3 files (model.onnx, model.onnx_data, tokenizer.json)
- [ ] Unit: `get_model_dir(ModelType::ParakeetCTC)` returns correct path ending in `parakeet-ctc`
- [ ] Unit: `check_model_exists_for_type(ParakeetCTC)` validates all 3 files present
- [ ] Unit: `TranscriptionMode::BatchCTC` serializes to correct string
- [ ] Integration: CTC download completes with progress events
- [ ] Integration: CTC transcription produces text output with punctuation

## Dependencies

None (extends existing infrastructure)

## Preconditions

- TDT/EOU infrastructure already in place (specs completed)
- parakeet-rs supports CTC model loading via `Parakeet::from_pretrained()`

## Implementation Notes

### Model Files Required

Download from `https://huggingface.co/onnx-community/parakeet-ctc-0.6b-ONNX/resolve/main/onnx/`:
- `model.onnx`
- `model.onnx_data`
- `tokenizer.json`

### Model Manifest

```rust
impl ModelManifest {
    pub fn ctc() -> Self {
        Self {
            model_type: ModelType::ParakeetCTC,
            base_url: "https://huggingface.co/onnx-community/parakeet-ctc-0.6b-ONNX/resolve/main/onnx/".into(),
            files: vec![
                ModelFile { name: "model.onnx".into(), size_bytes: TBD },
                ModelFile { name: "model.onnx_data".into(), size_bytes: TBD },
                ModelFile { name: "tokenizer.json".into(), size_bytes: TBD },
            ],
        }
    }
}
```

### TranscriptionMode Extension

```rust
pub enum TranscriptionMode {
    Batch,      // TDT - multilingual
    BatchCTC,   // CTC - English-only, faster
    Streaming,  // EOU - real-time
}
```

### CTC Loading (different from TDT/EOU)

```rust
// CTC uses Parakeet::from_pretrained() instead of TDT/EOU specific loaders
use parakeet_rs::Parakeet;

let parakeet = Parakeet::from_pretrained(model_dir, None)?;
let result = parakeet.transcribe_file(audio_path)?;
// result.text contains the transcribed text with punctuation
```

### Files to Modify

**Backend:**
- `src-tauri/src/model/download.rs` - Add CTC to ModelType enum, create manifest
- `src-tauri/src/parakeet/types.rs` - Add BatchCTC to TranscriptionMode
- `src-tauri/src/parakeet/manager.rs` - Add CTC context, loading, and transcription
- `src-tauri/src/lib.rs` - Add CTC to eager loading at startup

**Frontend:**
- `src/hooks/useMultiModelStatus.ts` - Add CTC status tracking
- `src/components/TranscriptionSettings/TranscriptionSettings.tsx` - Add CTC ModelDownloadCard
- `src/components/TranscriptionSettings/ModeToggle.tsx` - Convert to three-way toggle

## Related Specs

- `multi-file-model-download.spec.md` - Pattern for model manifests and download logic
- `tdt-batch-transcription.spec.md` - Pattern for batch transcription implementation
- `frontend-model-settings.spec.md` - Pattern for model download UI

## Integration Points

- Production call site: `src-tauri/src/parakeet/mod.rs` - Tauri commands for transcription
- Connects to: `events.rs` (transcription events), `TranscriptionSettings` (UI)

## Integration Test

- Test location: Manual test via frontend UI
- Verification: [ ] Integration test passes
