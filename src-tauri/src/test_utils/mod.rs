//! Test utilities for the heycat backend.
//!
//! This module provides shared mock implementations and test helpers
//! used across the test suite.

pub mod fixtures;
pub mod mock_backends;
pub mod mock_emitters;

pub use fixtures::ensure_test_model_files;
pub use mock_backends::{FailingShortcutBackend, MockShortcutBackend};
pub use mock_emitters::MockEmitter;
