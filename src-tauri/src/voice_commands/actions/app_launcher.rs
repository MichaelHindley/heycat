// App launcher action - opens applications on macOS

use crate::voice_commands::executor::{Action, ActionError, ActionErrorCode, ActionResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;

/// Validate app name to prevent path traversal or command injection
fn validate_app_name(name: &str) -> Result<(), ActionError> {
    // Reject empty names
    if name.trim().is_empty() {
        return Err(ActionError {
            code: ActionErrorCode::InvalidParameter,
            message: "App name cannot be empty".to_string(),
        });
    }

    // Reject path traversal attempts
    if name.contains('/') || name.contains("..") || name.contains('\0') {
        return Err(ActionError {
            code: ActionErrorCode::InvalidAppName,
            message: "App name contains invalid characters".to_string(),
        });
    }

    // Reject potential shell metacharacters
    if name.contains('`') || name.contains('$') || name.contains(';') || name.contains('|') || name.contains('&') {
        return Err(ActionError {
            code: ActionErrorCode::InvalidAppName,
            message: "App name contains invalid characters".to_string(),
        });
    }

    Ok(())
}

/// Action to open applications by name on macOS
pub struct AppLauncherAction;

impl AppLauncherAction {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppLauncherAction {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Action for AppLauncherAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        let app_name = parameters.get("app").ok_or_else(|| ActionError {
            code: ActionErrorCode::InvalidParameter,
            message: "Missing 'app' parameter".to_string(),
        })?;

        // Validate app name for security
        validate_app_name(app_name)?;

        // Check if we should close instead of open
        let should_close = parameters
            .get("close")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        if should_close {
            close_app(app_name)
        } else {
            open_app(app_name)
        }
    }
}

/// Open an application using the macOS `open` command
fn open_app(app_name: &str) -> Result<ActionResult, ActionError> {
    let output = Command::new("open")
        .arg("-a")
        .arg(app_name)
        .output()
        .map_err(|e| ActionError {
            code: ActionErrorCode::ExecutionError,
            message: format!("Failed to execute open command: {}", e),
        })?;

    if output.status.success() {
        Ok(ActionResult {
            message: format!("Opened application: {}", app_name),
            data: Some(serde_json::json!({
                "app": app_name,
                "action": "open"
            })),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for common error patterns
        if stderr.contains("Unable to find application") || stderr.contains("can't find application") {
            Err(ActionError {
                code: ActionErrorCode::NotFound,
                message: format!("Application not found: {}", app_name),
            })
        } else {
            Err(ActionError {
                code: ActionErrorCode::OpenFailed,
                message: format!("Failed to open {}: {}", app_name, stderr.trim()),
            })
        }
    }
}

/// Close an application using the macOS `osascript` command
fn close_app(app_name: &str) -> Result<ActionResult, ActionError> {
    // Sanitize app_name to prevent AppleScript injection
    // Escape backslashes first, then quotes
    let sanitized_name = app_name.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("tell application \"{}\" to quit", sanitized_name);

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| ActionError {
            code: ActionErrorCode::ExecutionError,
            message: format!("Failed to execute osascript command: {}", e),
        })?;

    if output.status.success() {
        Ok(ActionResult {
            message: format!("Closed application: {}", app_name),
            data: Some(serde_json::json!({
                "app": app_name,
                "action": "close"
            })),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ActionError {
            code: ActionErrorCode::CloseFailed,
            message: format!("Failed to close {}: {}", app_name, stderr.trim()),
        })
    }
}

#[cfg(test)]
#[path = "app_launcher_test.rs"]
mod tests;
