// Worktree-aware path resolution for data directories
//
// This module provides centralized path resolution that incorporates the worktree
// identifier when running from a git worktree. This ensures models, recordings,
// and other data files are stored in isolated locations per worktree.
//
// Path format:
// - Main repo: ~/.local/share/heycat/
// - Worktree:  ~/.local/share/heycat-{worktree_id}/
//
// Config format:
// - Main repo: ~/.config/heycat/
// - Worktree:  ~/.config/heycat-{worktree_id}/

use crate::worktree::WorktreeContext;
use std::path::PathBuf;

/// Base app directory name
const APP_DIR_NAME: &str = "heycat";

/// Error types for path resolution
#[derive(Debug, Clone, PartialEq)]
pub enum PathError {
    /// Data directory not found (platform issue)
    DataDirNotFound,
    /// Failed to create directory (used by ensure_dir_exists helper)
    #[allow(dead_code)]
    DirectoryCreationFailed(String),
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::DataDirNotFound => write!(f, "Data directory not found"),
            PathError::DirectoryCreationFailed(msg) => {
                write!(f, "Failed to create directory: {}", msg)
            }
        }
    }
}

impl std::error::Error for PathError {}

/// Get the app directory name based on worktree context.
///
/// Returns `heycat-{identifier}` when in a worktree, `heycat` otherwise.
fn get_app_dir_name(worktree_context: Option<&WorktreeContext>) -> String {
    match worktree_context {
        Some(ctx) => format!("{}-{}", APP_DIR_NAME, ctx.identifier),
        None => APP_DIR_NAME.to_string(),
    }
}

/// Get the base data directory path.
///
/// Returns:
/// - Main repo: `~/.local/share/heycat/`
/// - Worktree: `~/.local/share/heycat-{identifier}/`
pub fn get_data_dir(worktree_context: Option<&WorktreeContext>) -> Result<PathBuf, PathError> {
    let data_dir = dirs::data_dir().ok_or(PathError::DataDirNotFound)?;
    Ok(data_dir.join(get_app_dir_name(worktree_context)))
}

/// Get the models directory path.
///
/// Returns:
/// - Main repo: `~/.local/share/heycat/models/`
/// - Worktree: `~/.local/share/heycat-{identifier}/models/`
pub fn get_models_dir(worktree_context: Option<&WorktreeContext>) -> Result<PathBuf, PathError> {
    Ok(get_data_dir(worktree_context)?.join("models"))
}

/// Get the recordings directory path.
///
/// Returns:
/// - Main repo: `~/.local/share/heycat/recordings/`
/// - Worktree: `~/.local/share/heycat-{identifier}/recordings/`
pub fn get_recordings_dir(worktree_context: Option<&WorktreeContext>) -> Result<PathBuf, PathError> {
    Ok(get_data_dir(worktree_context)?.join("recordings"))
}

/// Ensure a directory exists, creating it if necessary.
///
/// Returns the path to the directory on success.
///
/// Note: Currently unused as each module handles directory creation inline
/// (e.g., ensure_models_dir, WAV encoding, config persistence). Kept as a
/// helper for future centralization if needed.
#[allow(dead_code)]
pub fn ensure_dir_exists(path: &PathBuf) -> Result<PathBuf, PathError> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| PathError::DirectoryCreationFailed(e.to_string()))?;
    }
    Ok(path.clone())
}

#[cfg(test)]
#[path = "paths_test.rs"]
mod tests;
