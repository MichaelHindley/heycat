//! Application initialization and setup.
//!
//! ## Module Organization
//!
//! - `setup`: Main application setup function
//! - `state`: Managed state type aliases
//! - `platform`: Platform-specific initialization (macOS/Windows/Linux)

mod platform;
pub mod setup;
pub mod state;

pub use setup::{on_window_destroyed, setup};
