# Bug: Audio Processing Architecture Technical Debt

**Created:** 2025-12-15
**Owner:** Claude
**Severity:** Major

## Description

Three senior Rust engineers reviewed the wake word, VAD (Voice Activity Detection), and audio transcription subsystems. The review identified significant architectural issues including:

- **Memory waste**: Two separate 3GB Parakeet model instances loaded (~6GB total)
- **Deadlock risk**: Callbacks run on analysis thread while holding locks
- **Tight coupling**: WakeWordDetector bypasses TranscriptionService trait abstraction
- **Inconsistent configuration**: VAD thresholds differ between components (0.3 vs 0.5)
- **No timeout protection**: Transcription can hang indefinitely with no recovery

The architecture is functional but has grown organically with poor isolation between components.

## Critical Issues Identified

### 1. Duplicate Parakeet Model Instances (CRITICAL)
- **Files**: `parakeet/manager.rs:14`, `listening/detector.rs:165`
- **Problem**: Two separate ParakeetTDT instances loaded (~6GB total memory)
- **Impact**: Excessive memory usage, slower startup, resource waste

### 2. Unsafe Callback Invocation (CRITICAL)
- **Files**: `listening/pipeline.rs:474-477`, `hotkey/integration.rs`
- **Problem**: Wake word callback runs on analysis thread while holding locks
- **Impact**: Can deadlock if callback attempts to acquire additional locks

### 3. WakeWordDetector Bypasses TranscriptionService (MAJOR)
- **Files**: `listening/detector.rs:7,165,349`
- **Problem**: Creates own ParakeetTDT instead of using trait abstraction
- **Impact**: Code duplication, untestable, inconsistent error handling

### 4. Inconsistent VAD Thresholds (MAJOR)
- **Files**: `listening/detector.rs` (0.3) vs `listening/silence.rs` (0.5)
- **Problem**: Different sensitivity in listening vs recording phases
- **Impact**: Confusing behavior, no documented rationale for difference

### 5. No Transcription Timeouts (MAJOR)
- **Files**: `hotkey/integration.rs:417`, `listening/detector.rs:349`
- **Problem**: Operations can hang indefinitely
- **Impact**: UI frozen showing "Transcribing..." with no recovery path

### 6. Duplicate Token-Joining Workaround (MAJOR)
- **Files**: `parakeet/manager.rs:136-143`, `listening/detector.rs:353-355`
- **Problem**: Same parakeet-rs bug workaround copy-pasted in two places
- **Impact**: Maintenance burden, risk of divergence

### 7. State Transition Race Condition (MAJOR)
- **Files**: `parakeet/manager.rs:112-122`
- **Problem**: State set to "Transcribing" BEFORE operation actually starts
- **Impact**: Brief window where state is inconsistent; if transcription fails, state stuck

### 8. Circular Dependency (MAJOR)
- **Files**: `listening/coordinator.rs:72`, `listening/pipeline.rs`
- **Problem**: Pipeline ↔ Coordinator have bidirectional calls
- **Impact**: Hard to reason about, potential for recursive lock acquisition

### 9. VAD Initialization Duplicated (MINOR)
- **Files**: `listening/detector.rs:229-239`, `listening/silence.rs:80-84`
- **Problem**: Identical VAD initialization code in two places
- **Impact**: DRY violation, maintenance burden

### 10. No VAD Abstraction (MINOR)
- **Files**: `listening/detector.rs`, `listening/silence.rs`
- **Problem**: Tight coupling to concrete VoiceActivityDetector
- **Impact**: Cannot mock for unit testing

## Expected Architecture

```
┌─────────────────────────────────────────────────────────┐
│              SharedTranscriptionModel                    │
│  ┌────────────────────────────────────────────────────┐ │
│  │           ParakeetTDT (3 GB) - SINGLE              │ │
│  │       Arc<Mutex<Option<ParakeetTDT>>>              │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────────────┬──────────────────────────────────┘
                       │
          ┌────────────┴────────────┐
          ↓                         ↓
  ┌───────────────┐         ┌───────────────┐
  │ Transcription │         │ WakeWord      │
  │ Manager       │         │ Detector      │
  └───────────────┘         └───────────────┘
          │                         │
          ↓                         ↓
  ┌────────────────────────────────────────────┐
  │         EventChannel (async)               │
  │   - No callbacks on analysis thread        │
  │   - Safe cross-component communication     │
  └────────────────────────────────────────────┘

  ┌────────────────────────────────────────────┐
  │           VadConfig (unified)              │
  │   - Single source of truth for thresholds  │
  │   - Documented rationale                   │
  └────────────────────────────────────────────┘
```

## Actual Architecture

```
┌─────────────────┐         ┌─────────────────┐
│ TranscriptionMgr│         │ WakeWordDetector│
│ ┌─────────────┐ │         │ ┌─────────────┐ │
│ │ ParakeetTDT │ │         │ │ ParakeetTDT │ │  <-- DUPLICATE
│ │   (3 GB)    │ │         │ │   (3 GB)    │ │      MODELS!
│ └─────────────┘ │         │ └─────────────┘ │
└────────┬────────┘         └────────┬────────┘
         │                           │
         │                           │ <-- BYPASSES TRAIT
         ↓                           ↓
┌─────────────────┐         ┌─────────────────┐
│ HotkeyIntegration│ ←────→ │ ListeningPipeline│
│   (async orch)  │ callback│  (analysis thd) │
└────────┬────────┘   ⚠️    └────────┬────────┘
         │   deadlock risk           │
         ↓                           ↓
┌─────────────────┐         ┌─────────────────┐
│ SilenceDetector │         │   Coordinator   │
│  VAD: 0.5 thres │         │                 │
└─────────────────┘         └─────────────────┘
         ↑                           ↑
         │                           │
    mismatch! ──────────────────────┘
         │
┌────────┴────────┐
│ WakeWord VAD    │
│ VAD: 0.3 thres  │
└─────────────────┘
```

## Files to Modify

### Critical (Memory & Safety)
- `src-tauri/src/parakeet/mod.rs` - Add shared model exports
- `src-tauri/src/parakeet/manager.rs` - Use shared model
- `src-tauri/src/listening/detector.rs` - Accept shared model, use TranscriptionService
- `src-tauri/src/listening/pipeline.rs` - Fix callback safety
- `src-tauri/src/hotkey/integration.rs` - Event channel subscription
- `src-tauri/src/lib.rs` - Initialize shared model

### Code Consolidation
- `src-tauri/src/listening/silence.rs` - Unified VAD config
- `src-tauri/src/listening/mod.rs` - VadConfig struct

### Robustness
- `src-tauri/src/parakeet/manager.rs` - State transition guards
- `src-tauri/src/hotkey/integration.rs` - Transcription timeout

## Definition of Done

- [ ] Single shared Parakeet model instance (memory reduced from ~6GB to ~3GB)
- [ ] Callbacks moved off analysis thread (no deadlock risk)
- [ ] WakeWordDetector uses TranscriptionService trait
- [ ] Unified VadConfig with documented threshold rationale
- [ ] Transcription timeout (60s) with graceful recovery
- [ ] Duplicate code extracted to shared utilities
- [ ] State transition race condition fixed
- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
