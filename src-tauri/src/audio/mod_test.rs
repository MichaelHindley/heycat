// Tests for audio capture module
// Test code is excluded from coverage since we measure production code coverage
#![cfg_attr(coverage_nightly, coverage(off))]

// Tests removed per docs/TESTING.md:
// - test_audio_buffer_new: Basic type usage, implicitly tested elsewhere
// - test_audio_buffer_default: Obvious default
// - test_audio_buffer_clone: Type system guarantee (Arc semantics)
// - test_capture_state_default_is_idle: Obvious default
// - test_capture_state_variants: Enum existence verified by type system
// - test_error_*: Pattern matching on error variants - type system handles this

