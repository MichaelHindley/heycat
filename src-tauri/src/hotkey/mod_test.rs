// Tests for hotkey module
// Test code is excluded from coverage since we measure production code coverage
#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

struct MockBackend {
    should_fail: bool,
    error_msg: String,
}

impl ShortcutBackend for MockBackend {
    fn register(&self, _: &str, callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
        if self.should_fail {
            Err(self.error_msg.clone())
        } else {
            callback(); // Actually invoke the callback
            Ok(())
        }
    }

    fn unregister(&self, _: &str) -> Result<(), String> {
        if self.should_fail {
            Err(self.error_msg.clone())
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_shortcut_constant() {
    assert_eq!(RECORDING_SHORTCUT, "CmdOrControl+Shift+R");
}

#[test]
fn test_map_already_registered() {
    assert_eq!(
        map_backend_error("already registered"),
        HotkeyError::AlreadyRegistered
    );
}

#[test]
fn test_map_conflict() {
    assert!(matches!(
        map_backend_error("conflict"),
        HotkeyError::Conflict(_)
    ));
}

#[test]
fn test_map_in_use() {
    assert!(matches!(
        map_backend_error("shortcut in use"),
        HotkeyError::Conflict(_)
    ));
}

#[test]
fn test_map_unknown_error() {
    assert!(matches!(
        map_backend_error("unknown"),
        HotkeyError::RegistrationFailed(_)
    ));
}

#[test]
fn test_service_register_success() {
    let svc = HotkeyService::new(MockBackend {
        should_fail: false,
        error_msg: String::new(),
    });
    assert!(svc.register_recording_shortcut(Box::new(|| {})).is_ok());
}

#[test]
fn test_service_register_conflict() {
    let svc = HotkeyService::new(MockBackend {
        should_fail: true,
        error_msg: "conflict".into(),
    });
    assert!(matches!(
        svc.register_recording_shortcut(Box::new(|| {})),
        Err(HotkeyError::Conflict(_))
    ));
}

#[test]
fn test_service_unregister_success() {
    let svc = HotkeyService::new(MockBackend {
        should_fail: false,
        error_msg: String::new(),
    });
    assert!(svc.unregister_recording_shortcut().is_ok());
}

#[test]
fn test_service_unregister_failure() {
    let svc = HotkeyService::new(MockBackend {
        should_fail: true,
        error_msg: "failed".into(),
    });
    assert!(matches!(
        svc.unregister_recording_shortcut(),
        Err(HotkeyError::RegistrationFailed(_))
    ));
}
