//! Mock shortcut backends for testing.

use std::sync::{Arc, Mutex};

/// Mock shortcut backend that tracks registered/unregistered shortcuts.
#[derive(Default)]
pub struct MockShortcutBackend {
    pub registered: Arc<Mutex<Vec<String>>>,
    pub callbacks: Arc<Mutex<std::collections::HashMap<String, Box<dyn Fn() + Send + Sync>>>>,
}

impl MockShortcutBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a shortcut is currently registered.
    pub fn is_registered(&self, shortcut: &str) -> bool {
        self.registered
            .lock()
            .unwrap()
            .contains(&shortcut.to_string())
    }

    /// Simulate pressing a registered shortcut (triggers callback).
    pub fn simulate_press(&self, shortcut: &str) {
        if let Some(callback) = self.callbacks.lock().unwrap().get(shortcut) {
            callback();
        }
    }
}

impl crate::hotkey::ShortcutBackend for MockShortcutBackend {
    fn register(
        &self,
        shortcut: &str,
        callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        let mut registered = self.registered.lock().unwrap();
        if registered.contains(&shortcut.to_string()) {
            return Err("Shortcut already registered".to_string());
        }
        registered.push(shortcut.to_string());
        self.callbacks
            .lock()
            .unwrap()
            .insert(shortcut.to_string(), callback);
        Ok(())
    }

    fn unregister(&self, shortcut: &str) -> Result<(), String> {
        let mut registered = self.registered.lock().unwrap();
        if let Some(pos) = registered.iter().position(|s| s == shortcut) {
            registered.remove(pos);
            self.callbacks.lock().unwrap().remove(shortcut);
            Ok(())
        } else {
            Err("Shortcut not registered".to_string())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Mock shortcut backend that always fails registration.
pub struct FailingShortcutBackend {
    pub unregister_attempts: Arc<Mutex<Vec<String>>>,
}

impl FailingShortcutBackend {
    pub fn new() -> Self {
        Self {
            unregister_attempts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the number of unregister attempts.
    pub fn unregister_attempt_count(&self) -> usize {
        self.unregister_attempts.lock().unwrap().len()
    }
}

impl Default for FailingShortcutBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::hotkey::ShortcutBackend for FailingShortcutBackend {
    fn register(
        &self,
        _shortcut: &str,
        _callback: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        Err("Registration always fails".to_string())
    }

    fn unregister(&self, shortcut: &str) -> Result<(), String> {
        self.unregister_attempts
            .lock()
            .unwrap()
            .push(shortcut.to_string());
        Err("Nothing to unregister".to_string())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
