// Recording module for managing recording state

mod state;
pub use state::{AudioData, RecordingManager, RecordingState, RecordingStateError};

mod coordinator;
pub use coordinator::{CoordinatorError, RecordingCoordinator, RecordingMetadata};

#[cfg(test)]
mod state_test;

#[cfg(test)]
mod coordinator_test;
