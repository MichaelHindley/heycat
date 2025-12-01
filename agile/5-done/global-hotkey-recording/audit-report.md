# Post-Implementation Audit Report

**Feature:** Global Hotkey Recording
**Date:** 2025-11-28
**Status:** All specs marked "completed" - Feature non-functional

---

## Executive Summary

All 12 specs for this feature have been marked "completed" and received "APPROVED" reviews. However, **the feature does not actually record audio**. The core issue is that the audio capture infrastructure was implemented and tested in isolation, but never integrated into the running application.

When a user presses the hotkey:
- State transitions correctly (Idle → Recording → Processing → Idle)
- Events are emitted to the frontend
- The UI shows "Recording" status
- **No audio is captured** - the buffer remains empty
- No WAV file is created (or an empty file_path is returned)

---

## Root Cause Analysis

### What Happened

1. Specs were implemented in isolation with mock backends
2. Each spec's unit tests pass because they use mocks (`MockBackend`, `MockFileWriter`)
3. Integration between components was assumed but never implemented
4. The `CpalBackend` (real microphone capture) exists but is never instantiated

### Architecture Gap

```
EXPECTED FLOW:
  Hotkey → RecordingCoordinator → AudioCaptureService → CpalBackend → Buffer → WAV → File

ACTUAL FLOW:
  Hotkey → HotkeyIntegration → RecordingManager (state only) → Empty buffer → No file
```

The `hotkey-integration.spec.md` includes this note (line 25):
> "State transitions use `RecordingManager` directly rather than `RecordingCoordinator` because cpal audio streams are not `Send + Sync`, making them incompatible with Tauri's threaded callback model. Audio capture will be managed separately when that spec is implemented."

**That "separate spec" was never created or implemented.**

---

## Affected Components

| Component | Status | Issue |
|-----------|--------|-------|
| `AudioCaptureService` | Built, tested | Never instantiated in app |
| `CpalBackend` | Built, tested | Never instantiated in app |
| `RecordingCoordinator` | Built, tested | Never used - app uses RecordingManager directly |
| `RecordingManager` | Working | Only manages state, not audio capture |
| `HotkeyIntegration` | Working | Emits events, transitions state, but no audio |
| WAV Encoding | Working | Never receives samples (buffer always empty) |
| Event Emission | Working | Events fire but metadata has duration=0, sample_count=0, file_path="" |

---

## Evidence

### 1. No Audio Capture Instantiation

**File:** `src-tauri/src/lib.rs:40`
```rust
let recording_state = Arc::new(Mutex::new(recording::RecordingManager::new()));
```

Only `RecordingManager` is created. No `CpalBackend`, `AudioCaptureService`, or `RecordingCoordinator` is instantiated.

### 2. Comment Acknowledges Missing Integration

**File:** `src-tauri/src/hotkey/integration.rs:44-45`
```rust
// Note: Audio capture is managed separately from state transitions due to
// thread-safety constraints (cpal streams can't be shared across threads).
```

This "separate management" does not exist in the codebase.

### 3. Empty Buffer Handling

**File:** `src-tauri/src/commands/logic.rs:75-82`
```rust
let file_path = if !samples.is_empty() {
    let writer = SystemFileWriter;
    encode_wav(&samples, DEFAULT_SAMPLE_RATE, &writer)
        .map_err(|e| format!("Encoding error: {:?}", e))?
} else {
    // No samples recorded - return placeholder
    String::new()
};
```

When `samples.is_empty()` (which is always true), `file_path` is returned as an empty string.

### 4. Metadata Shows Zero Recording

**File:** `src-tauri/src/hotkey/integration.rs:87-93`
```rust
let sample_count = samples.len();  // Always 0
let duration_secs = sample_count as f64 / DEFAULT_SAMPLE_RATE as f64;  // Always 0.0
let metadata = RecordingMetadata {
    duration_secs,
    file_path: String::new(),  // WAV encoding handled by IPC commands
    sample_count,
};
```

---

## Why Reviews Passed

1. **Mock-based testing** - All specs use mock backends in tests
   - `MockBackend` in `audio/mod_test.rs`
   - `MockFileWriter` in `recording/coordinator_test.rs`
   - Tests verify behavior against mocks, not real hardware

2. **No integration tests** - No test captures actual microphone audio and verifies WAV output

3. **Review scope limitation** - Reviews verified code matched spec acceptance criteria, but specs didn't require proof of integration

4. **Incremental development** - Each spec was reviewed as its own unit without verifying end-to-end flow

---

## Additional Issues Found

### Build Warnings (23 Rust + 1 TypeScript)

**Unused Rust code (20+ warnings):**
- `AudioCaptureService` - never constructed
- `CpalBackend::new()` - never called
- `RecordingCoordinator` and all methods - never used
- `AudioConfig`, `CaptureState`, `AudioCaptureError` - never used
- `HotkeyService::unregister_recording_shortcut()` - never called
- `HotkeyIntegration::with_debounce()`, `is_debouncing()` - never called
- `RecordingManager::reset_to_idle()` - never called
- `RecordingErrorPayload`, `emit_recording_error()` - never used

**Unused TypeScript import:**
- `src/App.test.tsx:3` - `waitFor` imported but never used

### Logging

**Current state:** Almost non-existent
- Only one `eprintln!` in `src-tauri/src/audio/cpal_backend.rs:52` for audio stream errors
- No logging crate (log, tracing, env_logger)
- No frontend logging (no console.log)
- No way to debug what's happening during "recording"

### File Output Location

Files *would* be saved to:
- **macOS:** `~/Library/Application Support/heycat/recordings/`
- **Linux:** `~/.local/share/heycat/recordings/`
- **Windows:** `%APPDATA%/heycat/recordings/`

But since no samples are captured, no files exist.

---

## Recommended Fixes

### 1. Integrate Audio Capture (Critical)

Create a new spec that:
- Instantiates `CpalBackend` at app startup
- Connects the audio stream to `RecordingManager`'s buffer
- Handles the thread-safety constraint (cpal streams are not Send+Sync)
- Potential approach: Run audio capture in a dedicated thread with channel-based communication

### 2. Add Comprehensive Logging

- Add `tracing` crate with structured logging
- Log all state transitions with timestamps
- Log audio device enumeration and selection
- Log recording start/stop with metadata
- Log file save operations with paths

### 3. Add End-to-End Integration Test

Create a test that:
- Starts the app (or test harness)
- Triggers hotkey
- Captures actual audio (even if synthetic test signal)
- Verifies WAV file exists with non-zero samples
- Verifies file plays back correctly

### 4. Clean Up Unused Code

Either:
- **Remove** infrastructure code that won't be used (if pivoting to different approach)
- **Integrate** the existing code (if the architecture is sound)
- **Silence warnings** with `#[allow(dead_code)]` and explanatory comments (temporary)

### 5. Update Review Process

For multi-component features:
- Require integration test as acceptance criteria
- Require demo of working end-to-end flow
- Add "Integration Verification" section to review template

---

## Lessons Learned

1. **Mock-based testing is necessary but not sufficient** - Tests passing doesn't mean the feature works

2. **Comments like "handled separately" must have corresponding implementation** - If code says "X will be done elsewhere", there must be a spec/ticket tracking X

3. **Feature completion requires end-to-end verification** - Not just checklist of specs marked "done"

4. **The gap between "it compiles and tests pass" and "it works" can be enormous** - Integration is where most complexity lives

5. **Review process should include integration proof** - For features spanning multiple components

---

## Appendix: Spec Status Summary

| Spec | Status | Actually Working |
|------|--------|------------------|
| global-hotkey.spec.md | completed | Yes - hotkey triggers callback |
| audio-capture.spec.md | completed | **No** - code exists but not integrated |
| wav-encoding.spec.md | completed | Yes - but never receives samples |
| recording-state-manager.spec.md | completed | Yes - state transitions work |
| recording-coordinator.spec.md | completed | **No** - code exists but not used |
| tauri-ipc-commands.spec.md | completed | Partial - commands work but return empty data |
| event-emission.spec.md | completed | Yes - events fire with empty metadata |
| recording-state-hook.spec.md | completed | Yes - hook receives (empty) events |
| recording-indicator.spec.md | completed | Yes - UI shows (misleading) status |
| hotkey-integration.spec.md | completed | Partial - integrates state/events, not audio |
| app-integration.spec.md | completed | Yes - app renders |
| transcription-buffer.spec.md | in-progress | Irrelevant until audio capture works |

---

## Defensive Prompting Recommendations

This section analyzes gaps in the current TCR skill, Agile skill, and CLAUDE.md instructions that allowed a fully-reviewed feature to be completely broken, and proposes "defensive prompting" changes to prevent recurrence.

### Process Gap Analysis

| Gap | Current State | How It Failed |
|-----|---------------|---------------|
| **Integration tests** | Not required by any workflow | Components work alone, fail together |
| **Smoke tests** | Not part of review checklist | Code compiles but feature doesn't function |
| **Mock verification** | Reviews don't check production usage | Mock tests pass, but mocked code never runs |
| **Deferral tracking** | No enforcement of "handled separately" | Deferred work silently dropped |
| **End-to-end gate** | No requirement before completion | Feature "complete" but broken |

### Why Current Process Failed

1. **TCR enforces unit test discipline, not integration**
   - 100% coverage achieved on mocked implementations
   - Tests verify mock behavior, not production behavior
   - No check that production code instantiates what tests mock

2. **Agile reviews verify "code matches spec", not "feature works"**
   - Reviews check: acceptance criteria met, tests exist, code quality good
   - Reviews DON'T check: feature actually works end-to-end
   - "APPROVED" means "implementation matches spec", not "feature functions"

3. **Specs are islands**
   - Each spec reviewed independently
   - No verification components integrate in production code
   - Dependency system tracks build order, not runtime integration

4. **Comments become silent gaps**
   - `hotkey-integration.spec.md` says "Audio capture will be managed separately"
   - No mechanism ensures "separate" work is tracked
   - Deferrals become permanent gaps

---

### Recommended Changes

#### 1. CLAUDE.md: Add Integration Verification Rules

Add new section after "Review Independence":

```markdown
## Integration Verification

For multi-component features:

1. **Mock Usage Audit**: When reviewing specs with mocked dependencies, verify the mocked component is actually instantiated in production code (lib.rs, main.tsx, etc.)

2. **Deferral Tracking**: Any comment like "handled separately", "will be implemented later", or "managed elsewhere" MUST reference a specific spec or ticket. Flag as NEEDS_WORK if no reference exists.

3. **Final Integration Spec**: Multi-component features require a final "integration" spec that:
   - Verifies all components are wired together in production
   - Includes a smoke test (manual or automated)
   - Documents the end-to-end flow with file:line references

4. **Feature Completion Gate**: Before moving to 4-review:
   - All "handled separately" comments must have corresponding completed specs
   - Integration test OR documented smoke test must exist
```

#### 2. TCR Skill: Add Integration Awareness

**Enhance `tcr check` output:**
- Warn when changed files use traits that have mock implementations
- Suggest verifying production instantiation for heavily-mocked code

**Add `tcr smoke` command (future):**
```bash
bun .claude/skills/tcr/tcr.ts smoke [command]  # Run app and verify basic functionality
```

**Add to SKILL.md tips:**
```markdown
## Mock Awareness

When your tests use mocks (MockBackend, MockFileWriter, etc.), remember:
- Mocks prove the interface contract works
- They do NOT prove production code uses the interface
- After completing a mocked component, verify it's instantiated in lib.rs/main.tsx
```

#### 3. Agile Review Template: Add Integration Section

Modify `.claude/skills/agile/commands/review.ts` to include:

```markdown
### Integration Verification

| Check | Status | Evidence |
|-------|--------|----------|
| Mocked components instantiated in production? | PASS/FAIL/N/A | lib.rs:XX or "no mocks used" |
| Any "handled separately" without spec reference? | PASS/FAIL | List any untracked deferrals |
| Feature works end-to-end? | PASS/FAIL/UNTESTED | Smoke test result or "requires integration spec" |

### Deferral Audit

| Deferral Statement | Location | Tracking Reference |
|--------------------|----------|-------------------|
| [quote] | file:line | spec-name or MISSING |
```

#### 4. Spec Template: Add Integration Fields

Modify `agile/templates/spec.template.md`:

```markdown
## Integration Points

[Where does this connect to other components? List the production files where this code is called/instantiated]

- Production call site: `lib.rs:XX` or "standalone module"
- Connects to: [list other modules this integrates with]

## Smoke Test

[Manual verification steps to prove this works end-to-end]

1. Start app with `bun run tauri dev`
2. [action to trigger feature]
3. [expected observable result]
4. Verified: [ ] Yes / [ ] No / [ ] N/A (unit-only spec)
```

#### 5. Agile Workflow: Add Transition Gates

**Modify `agile.ts move` validation for `4-review`:**

Before allowing transition to 4-review, check:
1. All specs completed (existing)
2. No "handled separately" comments without spec references (new)
3. Integration spec exists OR smoke test documented in at least one spec (new)

---

### Defensive Prompting Principles

Like defensive programming, defensive prompting assumes failures will occur and builds in safeguards:

| Principle | Implementation |
|-----------|----------------|
| **Assume mocks lie** | Require production instantiation proof |
| **Assume deferrals are forgotten** | Require explicit tracking references |
| **Assume reviews miss integration** | Add integration checklist to reviews |
| **Assume specs are isolated** | Require end-to-end verification |
| **Assume "done" doesn't mean "works"** | Gate completion on smoke tests |

### Summary of Changes

| File | Change | Priority |
|------|--------|----------|
| `CLAUDE.md` | Add "Integration Verification" section | High |
| `agile/templates/spec.template.md` | Add "Integration Points" and "Smoke Test" sections | High |
| `.claude/skills/agile/commands/review.ts` | Add integration verification to review template | High |
| `.claude/skills/agile/agile.ts` (move command) | Add integration gate for 4-review transition | Medium |
| `.claude/skills/tcr/SKILL.md` | Add "Mock Awareness" tips | Medium |
| `.claude/skills/tcr/tcr.ts` | Future: add `smoke` command | Low |
