// Wake word events for async communication from analysis thread
// Uses tokio::sync::mpsc channel to safely decouple detection from handling

/// Events emitted by the ListeningPipeline when wake word state changes
///
/// These events are sent through a tokio mpsc channel, allowing the analysis
/// thread to notify subscribers without blocking or risking deadlocks from
/// direct callback invocation.
#[derive(Debug, Clone)]
pub enum WakeWordEvent {
    /// Wake word was detected
    Detected {
        /// The transcribed text that matched the wake phrase
        text: String,
        /// Confidence score of the detection (0.0 to 1.0)
        confidence: f32,
    },
    /// Listening became unavailable (e.g., model not loaded, mic error)
    Unavailable {
        /// Reason why listening is unavailable
        reason: String,
    },
    /// An error occurred during detection
    Error {
        /// Error message
        message: String,
    },
}

impl WakeWordEvent {
    /// Create a new Detected event
    pub fn detected(text: impl Into<String>, confidence: f32) -> Self {
        Self::Detected {
            text: text.into(),
            confidence,
        }
    }

    /// Create a new Unavailable event
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable {
            reason: reason.into(),
        }
    }

    /// Create a new Error event
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    /// Check if this is a Detected event
    #[allow(dead_code)] // Used in tests
    pub fn is_detected(&self) -> bool {
        matches!(self, Self::Detected { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed per docs/TESTING.md:
    // - test_wake_word_event_debug: Debug trait test
    // - test_wake_word_event_clone: Type system guarantee

    // ==================== Event Factory Tests ====================
    // These test user-visible behavior: creating events via factory methods

    #[test]
    fn test_wake_word_event_detected() {
        let event = WakeWordEvent::detected("hey cat", 0.95);
        assert!(event.is_detected());

        if let WakeWordEvent::Detected { text, confidence } = event {
            assert_eq!(text, "hey cat");
            assert!((confidence - 0.95).abs() < f32::EPSILON);
        } else {
            panic!("Expected Detected variant");
        }
    }

    #[test]
    fn test_wake_word_event_unavailable() {
        let event = WakeWordEvent::unavailable("Model not loaded");
        assert!(!event.is_detected());

        if let WakeWordEvent::Unavailable { reason } = event {
            assert_eq!(reason, "Model not loaded");
        } else {
            panic!("Expected Unavailable variant");
        }
    }

    #[test]
    fn test_wake_word_event_error() {
        let event = WakeWordEvent::error("Detection failed");
        assert!(!event.is_detected());

        if let WakeWordEvent::Error { message } = event {
            assert_eq!(message, "Detection failed");
        } else {
            panic!("Expected Error variant");
        }
    }
}
