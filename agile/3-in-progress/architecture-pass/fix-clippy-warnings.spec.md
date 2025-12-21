---
status: in-progress
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Fix Clippy Warnings

## Description

Fix all 15 clippy warnings that currently cause `cargo clippy -- -D warnings` to fail. These are code quality issues including redundant comparisons, derivable impls, complex types, and too-many-arguments violations.

**Severity:** Low (code quality improvement)

## Acceptance Criteria

- [ ] `cargo clippy -- -D warnings` passes with no errors
- [ ] `cargo test` passes (no behavioral changes)
- [ ] No new warnings introduced

## Test Cases

- [ ] `cargo clippy -- -D warnings` exits with code 0
- [ ] All existing tests continue to pass

## Dependencies

None

## Preconditions

None

## Implementation Notes

**Warnings to fix (15 total):**

### 1. Redundant/incorrect comparisons (2)
- `detector.rs:537` - Change `>= config.min_speech_frames + 1` to `> config.min_speech_frames`
- `detector.rs:553` - Remove redundant `remaining > 0` from `remaining > 0 && remaining >= 256`

### 2. Collapsible match pattern (2 warnings, 1 location)
- `commands/mod.rs:256` - Replace `if let Some(ref reason) = ... { match reason { StreamError => ... } }` with `if let Some(&StopReason::StreamError) = ...`

### 3. Too many arguments (6)
- `commands/mod.rs:383` - `enable_listening` (8 args)
- `commands/mod.rs:466` - `handle_wake_word_events` (8 args)
- `commands/mod.rs:521` - `handle_wake_word_detected` (8 args)
- `coordinator.rs:66` - `start_monitoring` (8 args)
- `coordinator.rs:155` - `detection_loop` (9 args)

**Strategy:** Add `#[allow(clippy::too_many_arguments)]` - these are internal functions where bundling into structs would hurt readability without real benefit.

### 4. Derivable Default impls (2)
- `cgeventtap_backend.rs:52` - Add `#[derive(Default)]` to `ShortcutSpec`
- `cgeventtap.rs:135` - Add `#[derive(Default)]` to `CapturedKeyEvent`

### 5. Complex types (3)
- `cgeventtap_backend.rs:228,290` - Create type alias for callback map
- `integration.rs:337` - Create type alias for double-tap detector

### 6. Unnecessary let binding (1)
- `pipeline.rs:552` - Return `std::mem::take(&mut *guard)` directly

## Related Specs

- `extract-magic-numbers.spec.md` (completed) - Both are code quality improvements

## Integration Points

- Production call site: N/A (pure refactor, no behavior change)
- Connects to: N/A

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A
