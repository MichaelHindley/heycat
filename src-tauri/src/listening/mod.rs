// Listening module for always-on wake word detection
// Provides WakeWordDetector using Parakeet for on-device wake phrase recognition
// and ListeningManager for coordinating listening state with recording

mod buffer;
mod detector;
mod manager;
mod pipeline;

pub use buffer::CircularBuffer;
pub use detector::{WakeWordDetector, WakeWordDetectorConfig, WakeWordError, WakeWordResult};
pub use manager::{ListeningError, ListeningManager, ListeningStatus};
pub use pipeline::{ListeningPipeline, PipelineConfig, PipelineError};
