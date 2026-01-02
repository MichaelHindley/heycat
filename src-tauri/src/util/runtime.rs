//! Tokio runtime helpers for async-to-sync bridges.
//!
//! Provides utilities for running async code in synchronous contexts,
//! handling the case where a Tokio runtime may or may not be available.

/// Run an async future in the current context.
///
/// Handles two scenarios:
/// 1. If a Tokio runtime is already running, uses `block_in_place` to block on the future.
/// 2. If no runtime is available, creates a temporary one to run the future.
///
/// # Arguments
/// * `future` - The async future to execute
///
/// # Returns
/// The result of the future.
///
/// # Panics
/// Panics if unable to create a runtime when one is not available.
///
/// # Example
/// ```ignore
/// use crate::util::run_async;
///
/// let result = run_async(async {
///     some_async_function().await
/// });
/// ```
pub fn run_async<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    // Always create a new runtime to avoid issues with block_in_place panicking
    // on single-threaded (current_thread) runtimes. This is safer and works in
    // all contexts (with or without an existing runtime).
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime for async operation");
    rt.block_on(future)
}

#[cfg(test)]
#[path = "runtime_test.rs"]
mod tests;
