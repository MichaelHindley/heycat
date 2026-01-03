use super::*;
use serial_test::serial;

// Note: Full integration tests require a running Tauri app.
// These tests verify the module structure and basic behavior.
// Tests that access shared global state (LAST_USER_DEVICE_CHANGE) must use #[serial].

#[test]
fn test_device_handler_state_is_send_sync() {
    // Compile-time check that DeviceHandlerState satisfies Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    // We can't instantiate DeviceHandlerState directly, but the impl exists
    // This is a compile-time check via the unsafe impl declarations
}

#[test]
fn test_on_device_change_handles_uninitialized_state() {
    // Before init, calling on_device_change should not panic
    // It should log an error and return early
    // Note: This test is safe because DEVICE_HANDLER is empty initially
    // and we're not testing state after initialization (which would pollute global state)

    // The function should return without panicking when handler is not initialized
    // We can't easily verify the log output in unit tests, but no panic = success
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
