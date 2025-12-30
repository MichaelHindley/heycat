// Voice commands module - command matching and execution
//
// Tauri commands use TursoClient for persistence.
// This file contains Tauri-specific wrappers and is excluded from coverage.
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod actions;
pub mod executor;
pub mod matcher;
pub mod registry;

use crate::turso::{events as turso_events, TursoClient};
use registry::{ActionType, CommandDefinition, RegistryError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::AppHandle;
use uuid::Uuid;

/// Type alias for Turso client state
pub type TursoClientState = Arc<TursoClient>;

/// DTO for command definition (for Tauri serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDto {
    pub id: String,
    pub trigger: String,
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
}

impl From<&CommandDefinition> for CommandDto {
    fn from(cmd: &CommandDefinition) -> Self {
        let action_type = match cmd.action_type {
            ActionType::OpenApp => "open_app",
            ActionType::TypeText => "type_text",
            ActionType::SystemControl => "system_control",
            ActionType::Custom => "custom",
        };
        Self {
            id: cmd.id.to_string(),
            trigger: cmd.trigger.clone(),
            action_type: action_type.to_string(),
            parameters: cmd.parameters.clone(),
            enabled: cmd.enabled,
        }
    }
}

impl From<CommandDefinition> for CommandDto {
    fn from(cmd: CommandDefinition) -> Self {
        CommandDto::from(&cmd)
    }
}

/// Input for adding a new command
#[derive(Debug, Clone, Deserialize)]
pub struct AddCommandInput {
    pub trigger: String,
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
}

/// Input for updating an existing command
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCommandInput {
    pub id: String,
    pub trigger: String,
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
}

/// Map RegistryError to user-friendly error messages
fn to_user_error(error: RegistryError) -> String {
    match error {
        RegistryError::EmptyTrigger => "Trigger phrase cannot be empty".to_string(),
        RegistryError::NotFound(id) => format!("Command with ID '{}' not found", id),
        RegistryError::PersistenceError(msg) => format!("Failed to save command: {}", msg),
        RegistryError::LoadError(msg) => format!("Failed to load commands: {}", msg),
    }
}

/// Get all registered commands
#[tauri::command]
pub async fn get_commands(
    turso_client: tauri::State<'_, TursoClientState>,
) -> Result<Vec<CommandDto>, String> {
    turso_client
        .list_voice_commands()
        .await
        .map(|commands| commands.into_iter().map(CommandDto::from).collect())
        .map_err(to_user_error)
}

/// Add a new command
#[tauri::command]
pub async fn add_command(
    app_handle: AppHandle,
    turso_client: tauri::State<'_, TursoClientState>,
    input: AddCommandInput,
) -> Result<CommandDto, String> {
    let action_type: ActionType = input.action_type.parse()?;
    let cmd = CommandDefinition {
        id: Uuid::new_v4(),
        trigger: input.trigger,
        action_type,
        parameters: input.parameters,
        enabled: input.enabled,
    };

    turso_client
        .add_voice_command(&cmd)
        .await
        .map_err(to_user_error)?;

    // Emit voice_commands_updated event
    turso_events::emit_voice_commands_updated(&app_handle, "add", &cmd.id.to_string());

    crate::info!("Added voice command: {}", cmd.trigger);
    Ok(CommandDto::from(&cmd))
}

/// Remove a command by ID
#[tauri::command]
pub async fn remove_command(
    app_handle: AppHandle,
    turso_client: tauri::State<'_, TursoClientState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?;

    turso_client
        .delete_voice_command(uuid)
        .await
        .map_err(to_user_error)?;

    // Emit voice_commands_updated event
    turso_events::emit_voice_commands_updated(&app_handle, "delete", &id);

    crate::info!("Deleted voice command: {}", id);
    Ok(())
}

/// Update an existing command
#[tauri::command]
pub async fn update_command(
    app_handle: AppHandle,
    turso_client: tauri::State<'_, TursoClientState>,
    input: UpdateCommandInput,
) -> Result<CommandDto, String> {
    let uuid = Uuid::parse_str(&input.id).map_err(|e| format!("Invalid UUID: {}", e))?;
    let action_type: ActionType = input.action_type.parse()?;
    let cmd = CommandDefinition {
        id: uuid,
        trigger: input.trigger,
        action_type,
        parameters: input.parameters,
        enabled: input.enabled,
    };

    turso_client
        .update_voice_command(&cmd)
        .await
        .map_err(to_user_error)?;

    // Emit voice_commands_updated event
    turso_events::emit_voice_commands_updated(&app_handle, "update", &input.id);

    crate::info!("Updated voice command: {}", input.id);
    Ok(CommandDto::from(&cmd))
}
