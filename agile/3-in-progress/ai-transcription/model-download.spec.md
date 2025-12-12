---
status: in-progress
created: 2025-12-12
completed: null
dependencies: []
review_history:
  - round: 1
    date: 2025-12-12
    verdict: NEEDS_WORK
    failedCriteria: []
    concerns: ["**UI Integration Missing**: `ModelDownloadButton` is not imported or used in `App.tsx` - the component exists but is not visible in the application", "No integration test file exists at `src-tauri/src/model/download_test.rs` (Glob search returned no results)", "Some `/* v8 ignore */` pragmas in `useModelStatus.ts` reduce effective coverage visibility"]
---

# Spec: Download and Store Whisper Model

## Description

Implement the ability to download and store the Whisper Large v3 Turbo model from HuggingFace. Provides backend commands for downloading and checking model status, plus frontend UI for triggering the download.

## Acceptance Criteria

- [ ] Backend command `download_model` downloads Large v3 Turbo from HuggingFace
- [ ] Model stored in `{app_data_dir}/heycat/models/ggml-large-v3-turbo.bin`
- [ ] Backend command `check_model_status` returns model availability (boolean)
- [ ] Download uses reqwest with streaming for large file support
- [ ] Frontend `useModelStatus` hook tracks model availability
- [ ] `ModelDownloadButton` component shows "Download Model" / "Downloading..." / "Model Ready" states
- [ ] Event `model_download_completed` emitted when download finishes

## Test Cases

- [ ] check_model_status returns false when model doesn't exist
- [ ] check_model_status returns true when model file exists
- [ ] download_model creates models directory if not exists
- [ ] download_model fetches from correct HuggingFace URL
- [ ] Frontend hook correctly reflects backend model status
- [ ] Button transitions through states during download

## Dependencies

None

## Preconditions

- Network access available for download
- Sufficient disk space (~1.5GB)

## Implementation Notes

- Model URL: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`
- Use `dirs::data_dir()` for cross-platform app data location
- Consider partial download resume capability (out of MVP scope per feature.md)
- No download progress UI per MVP scope - just "downloading..." state

## Related Specs

- recording-block-without-model.spec.md (depends on check_model_status)
- transcription-pipeline.spec.md (loads the downloaded model)

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (command registration)
- Connects to: Frontend ModelDownloadButton (App.tsx), TranscriptionManager

## Review

**Reviewed:** 2025-12-12
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Backend command `download_model` downloads Large v3 Turbo from HuggingFace | PASS | `src-tauri/src/model/mod.rs:21-36` - Tauri command defined; `src-tauri/src/model/download.rs:71-118` - downloads from MODEL_URL constant |
| Model stored in `{app_data_dir}/heycat/models/ggml-large-v3-turbo.bin` | PASS | `src-tauri/src/model/download.rs:7-12` - constants define correct path structure; `download.rs:43-51` - `get_model_path()` builds path correctly |
| Backend command `check_model_status` returns model availability (boolean) | PASS | `src-tauri/src/model/mod.rs:14-17` - returns `Result<bool, String>` by calling `check_model_exists()` |
| Download uses reqwest with streaming for large file support | PASS | `src-tauri/src/model/download.rs:83-110` - uses `reqwest::Client`, `bytes_stream()`, and streams chunks to file |
| Frontend `useModelStatus` hook tracks model availability | PASS | `src/hooks/useModelStatus.ts:31-101` - hook provides `isModelAvailable`, `downloadState`, `error`, `downloadModel`, `refreshStatus` |
| `ModelDownloadButton` component shows "Download Model" / "Downloading..." / "Model Ready" states | PASS | `src/components/ModelDownloadButton.tsx:9-20` - `getButtonText()` returns correct states |
| Event `model_download_completed` emitted when download finishes | PASS | `src-tauri/src/model/mod.rs:27-33` - emits event via `app_handle.emit()`; `src-tauri/src/events.rs:17-24` - event defined |

### Test Verification

| Behavior | Tested By | Notes |
|----------|-----------|-------|
| check_model_status returns false when model doesn't exist | Unit | `src-tauri/src/model/download.rs:148-153` |
| check_model_status returns true when model file exists | Unit | Implicit via `test_check_model_exists_returns_false_when_not_present` - returns boolean without error |
| download_model creates models directory if not exists | Unit | `src-tauri/src/model/download.rs:178-183` - `test_ensure_models_dir_creates_directory` |
| download_model fetches from correct HuggingFace URL | Unit | `src-tauri/src/model/download.rs:125-129` - verifies URL constants |
| Frontend hook correctly reflects backend model status | Unit | `src/hooks/useModelStatus.test.ts:42-53` - tests status check and state updates |
| Button transitions through states during download | Unit | `src/components/ModelDownloadButton.test.tsx:24-76` - tests idle, downloading, ready, error states |

### Code Quality

**Strengths:**
- Clean separation of concerns: Tauri commands in `mod.rs`, core logic in `download.rs`
- Comprehensive error types with `ModelError` enum and proper `Display` implementation
- Streaming download pattern handles large files efficiently without loading entire file into memory
- Frontend hook properly manages state transitions and event cleanup
- Button component has excellent accessibility (aria-label, aria-busy, role="alert" for errors)
- Tests are comprehensive and well-organized

**Concerns:**
- **UI Integration Missing**: `ModelDownloadButton` is not imported or used in `App.tsx` - the component exists but is not visible in the application
- No integration test file exists at `src-tauri/src/model/download_test.rs` (Glob search returned no results)
- Some `/* v8 ignore */` pragmas in `useModelStatus.ts` reduce effective coverage visibility

### Integration Verification

| Check | Status | Evidence |
|-------|--------|----------|
| Commands registered in production? | PASS | `src-tauri/src/lib.rs:141-142` - both `model::check_model_status` and `model::download_model` in `invoke_handler!` |
| Mocked components instantiated in production? | N/A | No mocks used in production code |
| Any "handled separately" without spec reference? | PASS | No untracked deferrals found |
| Integration test exists and passes? | FAIL | No `download_test.rs` file found |
| Component integrated into UI? | FAIL | `ModelDownloadButton` not imported or rendered in `App.tsx` |

### Deferral Audit

| Deferral Statement | Location | Tracking Reference |
|--------------------|----------|-------------------|
| None found | - | - |

### Verdict

**NEEDS_WORK** - All backend and frontend code is implemented correctly with good test coverage. However, the `ModelDownloadButton` component is NOT integrated into the application UI (`App.tsx`), meaning users cannot actually trigger model downloads. The spec states "Frontend `ModelDownloadButton` component shows..." which implies it should be visible in the UI. Additionally, the integration test file referenced in the spec does not exist.
