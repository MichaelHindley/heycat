// App launcher action - opens applications on macOS

use crate::voice_commands::executor::{Action, ActionError, ActionResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;

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
            code: "INVALID_PARAMETER".to_string(),
            message: "Missing 'app' parameter".to_string(),
        })?;

        if app_name.trim().is_empty() {
            return Err(ActionError {
                code: "INVALID_PARAMETER".to_string(),
                message: "App name cannot be empty".to_string(),
            });
        }

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
            code: "EXECUTION_ERROR".to_string(),
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
                code: "NOT_FOUND".to_string(),
                message: format!("Application not found: {}", app_name),
            })
        } else {
            Err(ActionError {
                code: "OPEN_FAILED".to_string(),
                message: format!("Failed to open {}: {}", app_name, stderr.trim()),
            })
        }
    }
}

/// Close an application using the macOS `osascript` command
fn close_app(app_name: &str) -> Result<ActionResult, ActionError> {
    let script = format!("tell application \"{}\" to quit", app_name);

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| ActionError {
            code: "EXECUTION_ERROR".to_string(),
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
            code: "CLOSE_FAILED".to_string(),
            message: format!("Failed to close {}: {}", app_name, stderr.trim()),
        })
    }
}

#[cfg(test)]
#[path = "app_launcher_test.rs"]
mod tests;
