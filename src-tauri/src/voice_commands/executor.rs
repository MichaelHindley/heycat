// Action executor - dispatches commands to action implementations

use crate::voice_commands::actions::{AppLauncherAction, TextInputAction};
use crate::voice_commands::registry::{ActionType, CommandDefinition};
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

/// Result of an action execution
#[derive(Debug, Clone, Serialize)]
pub struct ActionResult {
    /// Description of what was done
    pub message: String,
    /// Optional additional data
    pub data: Option<serde_json::Value>,
}

/// Error during action execution
#[derive(Debug, Clone, Serialize)]
pub struct ActionError {
    /// Error code for categorization
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ActionError {}

/// Trait for action implementations
#[async_trait]
pub trait Action: Send + Sync {
    /// Execute the action with the given parameters
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError>;
}

/// Event names for command execution
pub mod event_names {
    pub const COMMAND_EXECUTED: &str = "command_executed";
    pub const COMMAND_FAILED: &str = "command_failed";
}

/// Payload for command_executed event
#[derive(Debug, Clone, Serialize)]
pub struct CommandExecutedPayload {
    pub command_id: String,
    pub trigger: String,
    pub result: ActionResult,
}

/// Payload for command_failed event
#[derive(Debug, Clone, Serialize)]
pub struct CommandFailedPayload {
    pub command_id: String,
    pub trigger: String,
    pub error: ActionError,
}


/// Stub implementation for SystemControl action
pub struct SystemControlAction;

#[async_trait]
impl Action for SystemControlAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        let control = parameters.get("control").ok_or_else(|| ActionError {
            code: "MISSING_PARAM".to_string(),
            message: "Missing 'control' parameter".to_string(),
        })?;

        // Stub implementation
        Ok(ActionResult {
            message: format!("Would execute system control: {}", control),
            data: None,
        })
    }
}

/// Stub implementation for Workflow action
pub struct WorkflowAction;

#[async_trait]
impl Action for WorkflowAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        let workflow = parameters.get("workflow").ok_or_else(|| ActionError {
            code: "MISSING_PARAM".to_string(),
            message: "Missing 'workflow' parameter".to_string(),
        })?;

        // Stub implementation - will be replaced by workflow-action spec
        Ok(ActionResult {
            message: format!("Would execute workflow: {}", workflow),
            data: None,
        })
    }
}

/// Stub implementation for Custom action
pub struct CustomAction;

#[async_trait]
impl Action for CustomAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        let script = parameters.get("script").ok_or_else(|| ActionError {
            code: "MISSING_PARAM".to_string(),
            message: "Missing 'script' parameter".to_string(),
        })?;

        // Stub implementation
        Ok(ActionResult {
            message: format!("Would execute custom script: {}", script),
            data: None,
        })
    }
}

/// Action dispatcher - routes commands to their implementations
pub struct ActionDispatcher {
    open_app: Arc<dyn Action>,
    type_text: Arc<dyn Action>,
    system_control: Arc<dyn Action>,
    workflow: Arc<dyn Action>,
    custom: Arc<dyn Action>,
}

impl Default for ActionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionDispatcher {
    /// Create a new dispatcher with default action implementations
    pub fn new() -> Self {
        Self {
            open_app: Arc::new(AppLauncherAction::new()),
            type_text: Arc::new(TextInputAction::new()),
            system_control: Arc::new(SystemControlAction),
            workflow: Arc::new(WorkflowAction),
            custom: Arc::new(CustomAction),
        }
    }

    /// Create a dispatcher with custom action implementations (for testing)
    pub fn with_actions(
        open_app: Arc<dyn Action>,
        type_text: Arc<dyn Action>,
        system_control: Arc<dyn Action>,
        workflow: Arc<dyn Action>,
        custom: Arc<dyn Action>,
    ) -> Self {
        Self {
            open_app,
            type_text,
            system_control,
            workflow,
            custom,
        }
    }

    /// Get the action implementation for a given action type
    pub fn get_action(&self, action_type: &ActionType) -> Arc<dyn Action> {
        match action_type {
            ActionType::OpenApp => self.open_app.clone(),
            ActionType::TypeText => self.type_text.clone(),
            ActionType::SystemControl => self.system_control.clone(),
            ActionType::Workflow => self.workflow.clone(),
            ActionType::Custom => self.custom.clone(),
        }
    }

    /// Execute a command asynchronously
    pub async fn execute(&self, command: &CommandDefinition) -> Result<ActionResult, ActionError> {
        let action = self.get_action(&command.action_type);
        action.execute(&command.parameters).await
    }
}

/// Execute a command in a spawned task with event emission
pub fn execute_command_async(
    app_handle: AppHandle,
    dispatcher: Arc<ActionDispatcher>,
    command: CommandDefinition,
) {
    let command_id = command.id;
    let trigger = command.trigger.clone();

    tokio::spawn(async move {
        let result = dispatcher.execute(&command).await;

        match result {
            Ok(action_result) => {
                let payload = CommandExecutedPayload {
                    command_id: command_id.to_string(),
                    trigger,
                    result: action_result,
                };
                let _ = app_handle.emit(event_names::COMMAND_EXECUTED, payload);
            }
            Err(action_error) => {
                let payload = CommandFailedPayload {
                    command_id: command_id.to_string(),
                    trigger,
                    error: action_error,
                };
                let _ = app_handle.emit(event_names::COMMAND_FAILED, payload);
            }
        }
    });
}

/// State for the executor
pub struct ExecutorState {
    pub dispatcher: Arc<ActionDispatcher>,
}

impl Default for ExecutorState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutorState {
    pub fn new() -> Self {
        Self {
            dispatcher: Arc::new(ActionDispatcher::new()),
        }
    }
}

/// Test a command by ID - executes immediately and returns result
#[tauri::command]
pub async fn test_command(
    app_handle: AppHandle,
    state: tauri::State<'_, crate::voice_commands::VoiceCommandsState>,
    executor_state: tauri::State<'_, ExecutorState>,
    id: String,
) -> Result<ActionResult, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?;

    let command = {
        let registry = state
            .registry
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        registry.get(uuid).cloned().ok_or_else(|| format!("Command not found: {}", id))?
    };

    let result = executor_state.dispatcher.execute(&command).await;

    match &result {
        Ok(action_result) => {
            let payload = CommandExecutedPayload {
                command_id: command.id.to_string(),
                trigger: command.trigger.clone(),
                result: action_result.clone(),
            };
            let _ = app_handle.emit(event_names::COMMAND_EXECUTED, payload);
        }
        Err(action_error) => {
            let payload = CommandFailedPayload {
                command_id: command.id.to_string(),
                trigger: command.trigger.clone(),
                error: action_error.clone(),
            };
            let _ = app_handle.emit(event_names::COMMAND_FAILED, payload);
        }
    }

    result.map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "executor_test.rs"]
mod tests;
