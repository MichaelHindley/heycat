// Listening module for always-on wake word detection
// Provides WakeWordDetector using Parakeet for on-device wake phrase recognition

mod buffer;
mod detector;

pub use buffer::CircularBuffer;
pub use detector::{WakeWordDetector, WakeWordDetectorConfig, WakeWordResult};
