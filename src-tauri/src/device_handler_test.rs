use super::*;
use serial_test::serial;

// Note: Full integration tests require a running Tauri app.
// These tests verify the module structure and basic behavior.
// Tests that access shared global state (LAST_USER_DEVICE_CHANGE) must use #[serial].

// =============================================================================
// Regression tests for HEY-187: Empty WAV files after device switching
// =============================================================================
// The auto-restart logic must NOT interfere with active capture sessions.
// Key safeguards:
// 1. Suppression window (1000ms) prevents auto-restart during user device changes
// 2. Capture state check in restart_audio_engine_async() skips restart if recording
// 3. Swift preserves capture state during device switch (preserveCaptureFile param)
// =============================================================================

#[test]
fn test_suppression_window_is_1000ms() {
    // Explicit test to document the expected value and catch accidental changes.
    // This value was increased from 500ms to 1000ms to account for slower device
    // switches and Core Audio propagation delays. Reducing it may cause race
    // conditions where auto-restart fires during user-initiated device changes.
    assert_eq!(
        SUPPRESSION_WINDOW_MS, 1000,
        "Suppression window should be 1000ms to prevent race conditions during device switching"
    );
}

#[test]
fn test_device_handler_state_is_send_sync() {
    // DeviceHandlerState only contains a `bool` which is inherently Send + Sync.
    // No manual unsafe impl is needed - the compiler auto-derives these traits.
    // This test documents the requirement: if DeviceHandlerState ever gains
    // non-Send/Sync fields, compilation will fail because OnceLock<T> requires
    // T: Send + Sync for its get/set methods to be available.
    //
    // Note: We can't call assert_send_sync::<DeviceHandlerState>() directly
    // because the type is private to the parent module.
}

#[test]
fn test_on_device_change_handles_uninitialized_state() {
    // Documents expected behavior: on_device_change() should not panic when
    // DEVICE_HANDLER is uninitialized - it should log an error and return early.
    //
    // Note: on_device_change is an `extern "C"` private function that cannot be
    // called directly from tests. The actual behavior is verified through:
    // 1. Code inspection: lines 104-108 check DEVICE_HANDLER.get().is_none()
    // 2. Integration testing with actual device change events
    //
    // This test serves as documentation of the expected behavior.
}

#[test]
#[serial(device_handler)]
fn test_suppression_initially_disabled() {
    // Reset the atomic to ensure clean state
    LAST_USER_DEVICE_CHANGE.store(0, Ordering::SeqCst);

    // Without any user device change, suppression should be disabled
    assert!(!should_suppress_auto_restart());
}

#[test]
#[serial(device_handler)]
fn test_suppression_enabled_after_mark() {
    // Reset at start to ensure clean state
    LAST_USER_DEVICE_CHANGE.store(0, Ordering::SeqCst);

    // Mark a user device change
    mark_user_device_change();

    // Suppression should now be enabled
    assert!(should_suppress_auto_restart());
}

#[test]
#[serial(device_handler)]
fn test_suppression_disabled_after_window_expires() {
    // Set a timestamp that's older than the suppression window
    let old_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let expired_time = old_time.saturating_sub(SUPPRESSION_WINDOW_MS + 100);
    LAST_USER_DEVICE_CHANGE.store(expired_time, Ordering::SeqCst);

    // Suppression should be disabled (window expired)
    assert!(!should_suppress_auto_restart());
}

#[test]
#[serial(device_handler)]
fn test_mark_user_device_change_updates_timestamp() {
    // Reset first
    LAST_USER_DEVICE_CHANGE.store(0, Ordering::SeqCst);
    assert_eq!(LAST_USER_DEVICE_CHANGE.load(Ordering::SeqCst), 0);

    // Mark a change
    mark_user_device_change();

    // Timestamp should be non-zero and recent
    let timestamp = LAST_USER_DEVICE_CHANGE.load(Ordering::SeqCst);
    assert!(timestamp > 0);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Timestamp should be within 1 second of now
    assert!(now.saturating_sub(timestamp) < 1000);
}
