//! Escape key listener registration and handling for HotkeyIntegration.
//!
//! Manages the double-tap Escape key detection for cancelling recordings.

use crate::events::{current_timestamp, hotkey_events, CommandEventEmitter, RecordingEventEmitter, TranscriptionEventEmitter};
use crate::hotkey::double_tap::DoubleTapDetector;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use super::HotkeyIntegration;

impl<R, T, C> HotkeyIntegration<R, T, C>
where
    R: RecordingEventEmitter,
    T: TranscriptionEventEmitter + 'static,
    C: CommandEventEmitter + 'static,
{
    /// Register the Escape key listener for cancel functionality
    ///
    /// Called when recording starts. Only registers if both shortcut_backend
    /// and escape_callback are configured. The listener is automatically
    /// unregistered when recording stops.
    ///
    /// Uses double-tap detection: single Escape presses are ignored, only
    /// double-taps within the configured time window trigger the cancel callback.
    ///
    /// IMPORTANT: The actual registration is deferred to a spawned thread to avoid
    /// re-entrancy deadlock. When this function is called from within a global shortcut
    /// callback (e.g., the recording hotkey), calling backend.register() synchronously
    /// would deadlock because the shortcut manager's lock is already held.
    pub(crate) fn register_escape_listener(&mut self) {
        // Skip if already registered or not configured
        if self.escape_registered.load(Ordering::SeqCst) {
            crate::debug!("Escape listener already registered, skipping");
            return;
        }

        // Get escape config - skip if not configured
        let escape_config = match &self.escape {
            Some(c) => c,
            None => {
                crate::debug!("No escape config, skipping Escape registration");
                return;
            }
        };

        let backend = escape_config.backend.clone();

        let callback = match &escape_config.callback {
            Some(c) => c.clone(),
            None => {
                crate::debug!("No escape callback configured, skipping Escape registration");
                return;
            }
        };

        // Create double-tap detector with the configured window
        // The detector wraps the callback and only invokes it on double-tap
        let boxed_callback: Box<dyn Fn() + Send + Sync> = Box::new(move || callback());
        let detector = Arc::new(Mutex::new(DoubleTapDetector::with_window(
            boxed_callback,
            escape_config.double_tap_window_ms,
        )));
        self.double_tap_detector = Some(detector.clone());

        // In tests, use synchronous registration (mock backends don't have deadlock issues)
        // In production, spawn registration on a separate thread to avoid re-entrancy deadlock
        #[cfg(test)]
        {
            match backend.register(
                super::super::ESCAPE_SHORTCUT,
                Box::new(move || {
                    // Use try_lock to avoid blocking the CGEventTap callback
                    // If lock is contended, skip this escape tap rather than freezing keyboard
                    if let Ok(mut det) = detector.try_lock() {
                        if det.on_tap() {
                            crate::debug!("Double-tap Escape detected, cancel triggered");
                        } else {
                            crate::trace!("Single Escape tap recorded, waiting for double-tap");
                        }
                    } else {
                        crate::trace!("Skipping escape tap - detector lock contended");
                    }
                }),
            ) {
                Ok(()) => {
                    self.escape_registered.store(true, Ordering::SeqCst);
                    crate::info!(
                        "Escape key listener registered for recording cancel (double-tap required)"
                    );
                }
                Err(e) => {
                    crate::warn!("Failed to register Escape key listener: {}", e);
                    self.double_tap_detector = None;
                    // Emit notification that key blocking is unavailable
                    if let Some(ref emitter) = self.hotkey_emitter {
                        emitter.emit_key_blocking_unavailable(
                            hotkey_events::KeyBlockingUnavailablePayload {
                                reason: format!("Failed to register Escape key listener: {}", e),
                                timestamp: current_timestamp(),
                            },
                        );
                    }
                }
            }
        }

        #[cfg(not(test))]
        {
            // Clone the Arc<AtomicBool> for the spawned thread to set after successful registration
            let escape_registered = self.escape_registered.clone();
            // Clone the hotkey emitter for the spawned thread to emit notifications on failure
            let hotkey_emitter = self.hotkey_emitter.clone();

            // Spawn registration on a separate thread to avoid re-entrancy deadlock
            // This is necessary because we're called from within a global shortcut callback,
            // and the shortcut manager holds a lock during callback execution.
            std::thread::spawn(move || {
                // Small delay to ensure the calling shortcut callback has completed
                std::thread::sleep(std::time::Duration::from_millis(10));

                match backend.register(
                    crate::hotkey::ESCAPE_SHORTCUT,
                    Box::new(move || {
                        // Use try_lock to avoid blocking the CGEventTap callback
                        // If lock is contended, skip this escape tap rather than freezing keyboard
                        if let Ok(mut det) = detector.try_lock() {
                            if det.on_tap() {
                                crate::debug!("Double-tap Escape detected, cancel triggered");
                            } else {
                                crate::trace!("Single Escape tap recorded, waiting for double-tap");
                            }
                        } else {
                            crate::trace!("Skipping escape tap - detector lock contended");
                        }
                    }),
                ) {
                    Ok(()) => {
                        // Only set escape_registered to true AFTER successful registration
                        escape_registered.store(true, Ordering::SeqCst);
                        crate::info!(
                            "Escape key listener registered for recording cancel (double-tap required)"
                        );
                    }
                    Err(e) => {
                        crate::warn!("Failed to register Escape key listener: {}", e);
                        // escape_registered remains false, so unregister won't attempt cleanup
                        // Emit notification that key blocking is unavailable
                        if let Some(ref emitter) = hotkey_emitter {
                            emitter.emit_key_blocking_unavailable(
                                hotkey_events::KeyBlockingUnavailablePayload {
                                    reason: format!(
                                        "Failed to register Escape key listener: {}",
                                        e
                                    ),
                                    timestamp: current_timestamp(),
                                },
                            );
                        }
                    }
                }
            });
        }
    }

    /// Unregister the Escape key listener
    ///
    /// Called when recording stops (either normally or via cancellation).
    /// Safe to call even if listener was never registered. Also resets the
    /// double-tap detector state.
    ///
    /// IMPORTANT: Like register_escape_listener, the actual unregistration is deferred
    /// to a spawned thread to avoid re-entrancy deadlock when called from within a
    /// global shortcut callback (e.g., the recording hotkey or Escape key itself).
    pub(crate) fn unregister_escape_listener(&mut self) {
        // Reset double-tap detector state - use try_lock to avoid deadlock when called
        // from within the escape callback (which already holds this lock)
        if let Some(ref detector) = self.double_tap_detector {
            if let Ok(mut det) = detector.try_lock() {
                det.reset();
            }
            // If try_lock fails, we're being called from within the escape callback.
            // The detector will be dropped anyway, so skipping reset is fine.
        }
        self.double_tap_detector = None;

        if !self.escape_registered.load(Ordering::SeqCst) {
            return;
        }

        let backend = match &self.escape {
            Some(c) => c.backend.clone(),
            None => return,
        };

        // Mark as unregistered immediately
        self.escape_registered.store(false, Ordering::SeqCst);

        // In tests, use synchronous unregistration (mock backends don't have deadlock issues)
        // In production, spawn unregistration on a separate thread to avoid re-entrancy deadlock
        #[cfg(test)]
        {
            match backend.unregister(super::super::ESCAPE_SHORTCUT) {
                Ok(()) => {
                    crate::debug!("Escape key listener unregistered");
                }
                Err(e) => {
                    crate::warn!("Failed to unregister Escape key listener: {}", e);
                }
            }
        }

        #[cfg(not(test))]
        {
            // Spawn unregistration on a separate thread to avoid re-entrancy deadlock
            // This is necessary because we may be called from within a global shortcut callback
            // (e.g., when stopping via recording hotkey or cancelling via Escape double-tap).
            std::thread::spawn(move || {
                // Small delay to ensure the calling shortcut callback has completed
                std::thread::sleep(std::time::Duration::from_millis(10));

                match backend.unregister(crate::hotkey::ESCAPE_SHORTCUT) {
                    Ok(()) => {
                        crate::debug!("Escape key listener unregistered");
                    }
                    Err(e) => {
                        // This can happen if registration failed or was never completed
                        crate::warn!("Failed to unregister Escape key listener: {}", e);
                    }
                }
            });
        }
    }
}
