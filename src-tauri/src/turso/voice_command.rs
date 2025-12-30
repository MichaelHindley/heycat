// VoiceCommand CRUD operations using Turso/libsql
//
// Provides database operations for voice commands using SQL queries.

use libsql::params;
use std::collections::HashMap;
use uuid::Uuid;

use super::client::TursoClient;
use crate::voice_commands::registry::{ActionType, CommandDefinition, RegistryError};

impl TursoClient {
    /// Add a new voice command.
    ///
    /// Validates trigger is not empty, generates timestamp,
    /// and inserts into the database.
    ///
    /// # Arguments
    /// * `cmd` - The command definition to add
    ///
    /// # Returns
    /// Ok(()) on success
    pub async fn add_voice_command(&self, cmd: &CommandDefinition) -> Result<(), RegistryError> {
        // Validate trigger
        if cmd.trigger.trim().is_empty() {
            return Err(RegistryError::EmptyTrigger);
        }

        let created_at = chrono::Utc::now().to_rfc3339();

        // Serialize parameters to JSON
        let parameters_json = serde_json::to_string(&cmd.parameters)
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        self.execute(
            r#"INSERT INTO voice_command
               (id, trigger, action_type, parameters_json, enabled, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![
                cmd.id.to_string(),
                cmd.trigger.clone(),
                action_type_to_string(&cmd.action_type),
                parameters_json,
                cmd.enabled as i32,
                created_at
            ],
        )
        .await
        .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Update an existing voice command.
    ///
    /// # Arguments
    /// * `cmd` - The command definition with updated values
    ///
    /// # Returns
    /// Ok(()) on success
    pub async fn update_voice_command(&self, cmd: &CommandDefinition) -> Result<(), RegistryError> {
        // Validate trigger
        if cmd.trigger.trim().is_empty() {
            return Err(RegistryError::EmptyTrigger);
        }

        // Check if command exists
        let exists = self.voice_command_exists(cmd.id).await?;
        if !exists {
            return Err(RegistryError::NotFound(cmd.id));
        }

        // Check for trigger conflict with other commands
        let mut rows = self
            .query(
                "SELECT id FROM voice_command WHERE trigger = ?1 AND id != ?2",
                params![cmd.trigger.clone(), cmd.id.to_string()],
            )
            .await
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        if rows
            .next()
            .await
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?
            .is_some()
        {
            return Err(RegistryError::PersistenceError(format!(
                "Trigger '{}' already exists",
                cmd.trigger
            )));
        }

        // Serialize parameters to JSON
        let parameters_json = serde_json::to_string(&cmd.parameters)
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        self.execute(
            r#"UPDATE voice_command
               SET trigger = ?1, action_type = ?2, parameters_json = ?3, enabled = ?4
               WHERE id = ?5"#,
            params![
                cmd.trigger.clone(),
                action_type_to_string(&cmd.action_type),
                parameters_json,
                cmd.enabled as i32,
                cmd.id.to_string()
            ],
        )
        .await
        .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// Delete a voice command by ID.
    ///
    /// # Arguments
    /// * `id` - The command ID to delete
    pub async fn delete_voice_command(&self, id: Uuid) -> Result<(), RegistryError> {
        // Check if command exists
        let exists = self.voice_command_exists(id).await?;
        if !exists {
            return Err(RegistryError::NotFound(id));
        }

        self.execute(
            "DELETE FROM voice_command WHERE id = ?1",
            params![id.to_string()],
        )
        .await
        .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// List all voice commands ordered by created_at.
    ///
    /// # Returns
    /// Vector of all voice commands
    pub async fn list_voice_commands(&self) -> Result<Vec<CommandDefinition>, RegistryError> {
        let mut rows = self
            .query(
                "SELECT id, trigger, action_type, parameters_json, enabled FROM voice_command ORDER BY created_at",
                (),
            )
            .await
            .map_err(|e| RegistryError::LoadError(e.to_string()))?;

        let mut commands = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| RegistryError::LoadError(e.to_string()))?
        {
            let id_str: String = row.get(0).map_err(|e| RegistryError::LoadError(e.to_string()))?;
            let trigger: String = row.get(1).map_err(|e| RegistryError::LoadError(e.to_string()))?;
            let action_type_str: String = row.get(2).map_err(|e| RegistryError::LoadError(e.to_string()))?;
            let parameters_json: String = row.get(3).map_err(|e| RegistryError::LoadError(e.to_string()))?;
            let enabled: i32 = row.get(4).map_err(|e| RegistryError::LoadError(e.to_string()))?;

            let id = Uuid::parse_str(&id_str)
                .map_err(|e| RegistryError::LoadError(format!("Invalid UUID: {}", e)))?;

            let parameters: HashMap<String, String> = serde_json::from_str(&parameters_json)
                .map_err(|e| RegistryError::LoadError(format!("Invalid parameters JSON: {}", e)))?;

            commands.push(CommandDefinition {
                id,
                trigger,
                action_type: string_to_action_type(&action_type_str),
                parameters,
                enabled: enabled != 0,
            });
        }

        Ok(commands)
    }

    /// Check if a voice command exists by ID.
    async fn voice_command_exists(&self, id: Uuid) -> Result<bool, RegistryError> {
        let mut rows = self
            .query(
                "SELECT 1 FROM voice_command WHERE id = ?1",
                params![id.to_string()],
            )
            .await
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?;

        Ok(rows
            .next()
            .await
            .map_err(|e| RegistryError::PersistenceError(e.to_string()))?
            .is_some())
    }
}

/// Convert ActionType to string for database storage
fn action_type_to_string(action_type: &ActionType) -> String {
    match action_type {
        ActionType::OpenApp => "open_app".to_string(),
        ActionType::TypeText => "type_text".to_string(),
        ActionType::SystemControl => "system_control".to_string(),
        ActionType::Custom => "custom".to_string(),
    }
}

/// Convert string to ActionType
fn string_to_action_type(s: &str) -> ActionType {
    match s {
        "open_app" => ActionType::OpenApp,
        "type_text" => ActionType::TypeText,
        "system_control" => ActionType::SystemControl,
        "custom" => ActionType::Custom,
        _ => ActionType::Custom, // Default to Custom for unknown types
    }
}

#[cfg(test)]
#[path = "voice_command_test.rs"]
mod tests;
