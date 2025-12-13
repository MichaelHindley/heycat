// Workflow action - executes multi-step command sequences

use crate::voice_commands::executor::{Action, ActionError, ActionResult, ActionDispatcher};
use crate::voice_commands::registry::ActionType;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Default delay between steps in milliseconds
pub const DEFAULT_STEP_DELAY_MS: u64 = 100;

/// A single step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Type of action for this step
    pub action_type: String,
    /// Parameters for the action
    pub parameters: HashMap<String, String>,
    /// Optional delay after this step in milliseconds
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

/// Result of a single step
#[derive(Debug, Clone, Serialize)]
pub struct StepResult {
    pub step_index: usize,
    pub action_type: String,
    pub result: ActionResult,
}

/// Action to execute a workflow of multiple steps
pub struct WorkflowAction {
    dispatcher: Arc<ActionDispatcher>,
}

impl WorkflowAction {
    pub fn new(dispatcher: Arc<ActionDispatcher>) -> Self {
        Self { dispatcher }
    }
}

fn parse_action_type(s: &str) -> Result<ActionType, ActionError> {
    match s {
        "open_app" => Ok(ActionType::OpenApp),
        "type_text" => Ok(ActionType::TypeText),
        "system_control" => Ok(ActionType::SystemControl),
        "workflow" => Ok(ActionType::Workflow),
        "custom" => Ok(ActionType::Custom),
        _ => Err(ActionError {
            code: "INVALID_ACTION_TYPE".to_string(),
            message: format!("Unknown action type: {}", s),
        }),
    }
}

#[async_trait]
impl Action for WorkflowAction {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ActionResult, ActionError> {
        // Get workflow steps from parameters
        let steps_json = parameters.get("steps").ok_or_else(|| ActionError {
            code: "INVALID_PARAMETER".to_string(),
            message: "Missing 'steps' parameter".to_string(),
        })?;

        // Parse steps from JSON
        let steps: Vec<WorkflowStep> = serde_json::from_str(steps_json).map_err(|e| ActionError {
            code: "PARSE_ERROR".to_string(),
            message: format!("Failed to parse workflow steps: {}", e),
        })?;

        // Empty workflow is a no-op
        if steps.is_empty() {
            return Ok(ActionResult {
                message: "Workflow completed (no steps)".to_string(),
                data: Some(serde_json::json!({
                    "steps_executed": 0,
                    "results": []
                })),
            });
        }

        // Get global delay between steps
        let default_delay = parameters
            .get("delay_ms")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_STEP_DELAY_MS);

        let mut step_results: Vec<StepResult> = Vec::new();

        // Execute each step sequentially
        for (index, step) in steps.iter().enumerate() {
            let action_type = parse_action_type(&step.action_type)?;
            let action = self.dispatcher.get_action(&action_type);

            // Execute the step
            let result = action.execute(&step.parameters).await;

            match result {
                Ok(action_result) => {
                    step_results.push(StepResult {
                        step_index: index,
                        action_type: step.action_type.clone(),
                        result: action_result,
                    });
                }
                Err(error) => {
                    // Stop on first error
                    return Err(ActionError {
                        code: "STEP_FAILED".to_string(),
                        message: format!(
                            "Workflow failed at step {} ({}): {}",
                            index + 1,
                            step.action_type,
                            error.message
                        ),
                    });
                }
            }

            // Apply delay after step (use step-specific delay if provided, else global)
            let delay = step.delay_ms.unwrap_or(default_delay);
            if delay > 0 && index < steps.len() - 1 {
                sleep(Duration::from_millis(delay)).await;
            }
        }

        // Aggregate results
        Ok(ActionResult {
            message: format!("Workflow completed: {} steps executed", step_results.len()),
            data: Some(serde_json::json!({
                "steps_executed": step_results.len(),
                "results": step_results
            })),
        })
    }
}

#[cfg(test)]
#[path = "workflow_test.rs"]
mod tests;
