//! Platform-specific initialization for the application.
//!
//! Provides platform-specific hotkey registration and setup routines.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod other;

#[cfg(target_os = "macos")]
pub use macos::register_hotkey_with_release;

#[cfg(not(target_os = "macos"))]
pub use other::register_hotkey_with_release;
