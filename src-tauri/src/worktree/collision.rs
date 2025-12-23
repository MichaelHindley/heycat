// Collision detection for worktree data isolation
//
// This module detects if another instance is using the same data directory
// by checking for lock files. It provides user-friendly error messages with
// resolution steps when collisions are detected.
//
// Note: Most items are marked #[allow(dead_code)] because they're used:
// - In tests via cfg(test) re-exports in mod.rs
// - In production by lib.rs setup() (integration happens in this spec)

use crate::paths;
use crate::worktree::WorktreeContext;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Lock file name used to detect running instances
#[allow(dead_code)]
const LOCK_FILE_NAME: &str = "heycat.lock";

/// Lock file contents for identifying the instance
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LockInfo {
    /// Process ID of the lock holder
    pub pid: u32,
    /// Timestamp when the lock was created (Unix epoch seconds)
    pub timestamp: u64,
}

impl LockInfo {
    /// Create a new LockInfo for the current process
    pub fn current() -> Self {
        Self {
            pid: std::process::id(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Parse lock info from file content
    pub fn parse(content: &str) -> Option<Self> {
        let mut pid = None;
        let mut timestamp = None;

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("pid: ") {
                pid = value.trim().parse().ok();
            } else if let Some(value) = line.strip_prefix("timestamp: ") {
                timestamp = value.trim().parse().ok();
            }
        }

        Some(Self {
            pid: pid?,
            timestamp: timestamp?,
        })
    }

    /// Serialize lock info to file content
    pub fn serialize(&self) -> String {
        format!("pid: {}\ntimestamp: {}\n", self.pid, self.timestamp)
    }
}

/// Collision detection result
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionResult {
    /// No collision detected, safe to proceed
    NoCollision,
    /// Another instance is running with the same data directory
    InstanceRunning {
        /// PID of the running instance
        pid: u32,
        /// Path to the data directory
        data_dir: PathBuf,
        /// Path to the lock file
        lock_file: PathBuf,
    },
    /// Stale lock file found (process not running)
    StaleLock {
        /// Path to the stale lock file
        lock_file: PathBuf,
    },
}

/// Error types for collision detection
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionError {
    /// Data directory could not be determined
    DataDirNotFound,
    /// Failed to check or create lock file
    LockFileError(String),
}

impl std::fmt::Display for CollisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollisionError::DataDirNotFound => write!(f, "Could not determine data directory"),
            CollisionError::LockFileError(msg) => write!(f, "Lock file error: {}", msg),
        }
    }
}

impl std::error::Error for CollisionError {}

/// Check for collision with another running instance.
///
/// This function checks if another heycat instance is using the same data directory
/// by looking for a lock file. If a lock file exists and the process is still running,
/// a collision is detected.
///
/// # Arguments
/// * `worktree_context` - Optional worktree context for path resolution
///
/// # Returns
/// * `Ok(CollisionResult::NoCollision)` - Safe to proceed
/// * `Ok(CollisionResult::InstanceRunning { .. })` - Another instance is running
/// * `Ok(CollisionResult::StaleLock { .. })` - Stale lock file found (can be cleaned)
/// * `Err(CollisionError)` - Error during collision check
#[allow(dead_code)]
pub fn check_collision(
    worktree_context: Option<&WorktreeContext>,
) -> Result<CollisionResult, CollisionError> {
    let data_dir =
        paths::get_data_dir(worktree_context).map_err(|_| CollisionError::DataDirNotFound)?;
    let lock_file = data_dir.join(LOCK_FILE_NAME);

    check_collision_at(&lock_file, &data_dir)
}

/// Check for collision at a specific lock file path (for testing)
#[allow(dead_code)]
pub fn check_collision_at(lock_file: &PathBuf, data_dir: &PathBuf) -> Result<CollisionResult, CollisionError> {
    // If no lock file exists, no collision
    if !lock_file.exists() {
        return Ok(CollisionResult::NoCollision);
    }

    // Read and parse the lock file
    let content = fs::read_to_string(lock_file)
        .map_err(|e| CollisionError::LockFileError(e.to_string()))?;

    let lock_info = match LockInfo::parse(&content) {
        Some(info) => info,
        None => {
            // Malformed lock file - treat as stale
            return Ok(CollisionResult::StaleLock {
                lock_file: lock_file.clone(),
            });
        }
    };

    // Check if the process is still running
    if is_process_running(lock_info.pid) {
        // Process is running - collision detected
        Ok(CollisionResult::InstanceRunning {
            pid: lock_info.pid,
            data_dir: data_dir.clone(),
            lock_file: lock_file.clone(),
        })
    } else {
        // Process is not running - stale lock
        Ok(CollisionResult::StaleLock {
            lock_file: lock_file.clone(),
        })
    }
}

/// Create a lock file for the current instance.
///
/// # Arguments
/// * `worktree_context` - Optional worktree context for path resolution
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the created lock file
/// * `Err(CollisionError)` - Error creating lock file
#[allow(dead_code)]
pub fn create_lock(
    worktree_context: Option<&WorktreeContext>,
) -> Result<PathBuf, CollisionError> {
    let data_dir =
        paths::get_data_dir(worktree_context).map_err(|_| CollisionError::DataDirNotFound)?;

    // Ensure data directory exists
    fs::create_dir_all(&data_dir)
        .map_err(|e| CollisionError::LockFileError(format!("Failed to create data dir: {}", e)))?;

    let lock_file = data_dir.join(LOCK_FILE_NAME);
    create_lock_at(&lock_file)
}

/// Create a lock file at a specific path (for testing)
#[allow(dead_code)]
pub fn create_lock_at(lock_file: &PathBuf) -> Result<PathBuf, CollisionError> {
    let lock_info = LockInfo::current();

    let mut file = fs::File::create(lock_file)
        .map_err(|e| CollisionError::LockFileError(format!("Failed to create lock file: {}", e)))?;

    file.write_all(lock_info.serialize().as_bytes())
        .map_err(|e| CollisionError::LockFileError(format!("Failed to write lock file: {}", e)))?;

    Ok(lock_file.clone())
}

/// Remove the lock file for the current instance.
///
/// Should be called on graceful shutdown.
///
/// # Arguments
/// * `worktree_context` - Optional worktree context for path resolution
#[allow(dead_code)]
pub fn remove_lock(worktree_context: Option<&WorktreeContext>) -> Result<(), CollisionError> {
    let data_dir =
        paths::get_data_dir(worktree_context).map_err(|_| CollisionError::DataDirNotFound)?;
    let lock_file = data_dir.join(LOCK_FILE_NAME);

    remove_lock_at(&lock_file)
}

/// Remove a lock file at a specific path (for testing)
#[allow(dead_code)]
pub fn remove_lock_at(lock_file: &PathBuf) -> Result<(), CollisionError> {
    if lock_file.exists() {
        fs::remove_file(lock_file)
            .map_err(|e| CollisionError::LockFileError(format!("Failed to remove lock file: {}", e)))?;
    }
    Ok(())
}

/// Clean up a stale lock file.
///
/// # Arguments
/// * `lock_file` - Path to the stale lock file to remove
#[allow(dead_code)]
pub fn cleanup_stale_lock(lock_file: &PathBuf) -> Result<(), CollisionError> {
    remove_lock_at(lock_file)
}

/// Check if a process with the given PID is still running.
///
/// Uses platform-specific mechanisms:
/// - Unix: kill(pid, 0) - sends null signal to check existence
/// - Windows: Would use process enumeration (not yet implemented)
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // On Unix, we can use kill with signal 0 to check if process exists
        // This doesn't send any signal, just checks if the process exists
        // kill(pid, 0) returns 0 if process exists and we have permission
        // Returns -1 with ESRCH if process doesn't exist
        // Returns -1 with EPERM if process exists but we don't have permission (still running)
        let result = unsafe { libc::kill(pid as libc::pid_t, 0) };

        if result == 0 {
            return true; // Process exists and we have permission
        }

        // Check errno to distinguish between "not found" and "permission denied"
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        errno == libc::EPERM // EPERM means process exists but we don't have permission
    }

    #[cfg(windows)]
    {
        // On Windows, check if we can open the process with minimal access
        use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
        use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

        let handle: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle != 0 {
            unsafe { CloseHandle(handle) };
            true
        } else {
            false
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Unknown platform - assume process is not running to avoid blocking
        false
    }
}

/// Get a user-friendly error message for a collision.
///
/// # Arguments
/// * `collision` - The collision result to describe
///
/// # Returns
/// A tuple of (title, message, resolution_steps)
#[allow(dead_code)]
pub fn format_collision_error(collision: &CollisionResult) -> Option<(String, String, Vec<String>)> {
    match collision {
        CollisionResult::NoCollision => None,
        CollisionResult::InstanceRunning { pid, data_dir, .. } => {
            Some((
                "Another instance is running".to_string(),
                format!(
                    "Another heycat instance (PID: {}) is using the data directory:\n{}",
                    pid,
                    data_dir.display()
                ),
                vec![
                    "Close the other heycat instance".to_string(),
                    format!("Or kill the process: kill {}", pid),
                    "Then restart this instance".to_string(),
                ],
            ))
        }
        CollisionResult::StaleLock { lock_file } => {
            Some((
                "Stale lock file detected".to_string(),
                format!(
                    "A lock file from a previous crashed instance was found:\n{}",
                    lock_file.display()
                ),
                vec![
                    "This will be automatically cleaned up".to_string(),
                    "No action required".to_string(),
                ],
            ))
        }
    }
}
