// Voice command types and definitions
//
// Commands are stored in Turso. Use TursoClient for all CRUD and queries.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type of action to execute when a command matches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Open an application
    OpenApp,
    /// Type text via keyboard simulation
    TypeText,
    /// System control (volume, brightness, etc.)
    SystemControl,
    /// Custom user-defined action
    Custom,
}

impl std::str::FromStr for ActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open_app" => Ok(ActionType::OpenApp),
            "type_text" => Ok(ActionType::TypeText),
            "system_control" => Ok(ActionType::SystemControl),
            "custom" => Ok(ActionType::Custom),
            _ => Err(format!("Unknown action type: {}", s)),
        }
    }
}

/// A voice command definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandDefinition {
    /// Unique identifier for the command
    pub id: Uuid,
    /// Trigger phrase (e.g., "open slack")
    pub trigger: String,
    /// Type of action to execute
    pub action_type: ActionType,
    /// Action-specific parameters
    pub parameters: HashMap<String, String>,
    /// Whether the command is enabled
    pub enabled: bool,
}

/// Error types for voice command operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RegistryError {
    /// Trigger phrase is empty
    #[error("Trigger phrase cannot be empty")]
    EmptyTrigger,
    /// Command not found
    #[error("Command with ID {0} not found")]
    NotFound(Uuid),
    /// Failed to persist commands
    #[error("Failed to persist commands: {0}")]
    PersistenceError(String),
    /// Failed to load commands
    #[error("Failed to load commands: {0}")]
    LoadError(String),
}
