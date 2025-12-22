// Keyboard simulator tests
//
// Note: Actual keypress simulation requires system permissions (Accessibility on macOS)
// and an active display, so we mark integration tests with #[ignore].

use super::*;

#[test]
fn test_keyboard_simulator_creation() {
    // Verify we can create the simulator without panic
    // This tests the enigo initialization path
    let result = KeyboardSimulator::new();
    // On CI or systems without display, this may fail - that's expected
    // The important thing is it doesn't panic
    match result {
        Ok(_) => (), // Successfully created
        Err(e) => {
            // Expected on CI or headless systems
            assert!(
                e.contains("Failed to create keyboard simulator"),
                "Unexpected error: {}",
                e
            );
        }
    }
}

#[test]
#[ignore] // Requires display and keyboard permissions
fn test_enter_keypress_integration() {
    let mut simulator = KeyboardSimulator::new().expect("Failed to create simulator");
    let result = simulator.simulate_enter_keypress();
    assert!(result.is_ok(), "Enter keypress should succeed: {:?}", result);
}
