// WindowContext CRUD operations using Turso/libsql
//
// Provides database operations for window contexts using SQL queries.

use libsql::params;
use uuid::Uuid;

use super::client::TursoClient;
use crate::window_context::{OverrideMode, WindowContext, WindowContextStoreError, WindowMatcher};

impl TursoClient {
    /// Add a new window context.
    ///
    /// Generates a UUID for the id, serializes JSON arrays,
    /// and inserts into the database.
    ///
    /// # Arguments
    /// * `name` - Display name for the context
    /// * `matcher` - Window matching rules
    /// * `command_mode` - How to apply command overrides
    /// * `dictionary_mode` - How to apply dictionary overrides
    /// * `command_ids` - List of command UUIDs to use in this context
    /// * `dictionary_entry_ids` - List of dictionary entry IDs to use
    /// * `enabled` - Whether the context is active
    /// * `priority` - Priority for matching (higher = matched first)
    ///
    /// # Returns
    /// The created WindowContext with generated ID
    pub async fn add_window_context(
        &self,
        name: String,
        matcher: WindowMatcher,
        command_mode: OverrideMode,
        dictionary_mode: OverrideMode,
        command_ids: Vec<Uuid>,
        dictionary_entry_ids: Vec<String>,
        enabled: bool,
        priority: i32,
    ) -> Result<WindowContext, WindowContextStoreError> {
        let id = Uuid::new_v4();
        let created_at = chrono::Utc::now().to_rfc3339();

        // Serialize UUIDs as strings in JSON
        let command_ids_json = serde_json::to_string(&command_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;
        let dictionary_entry_ids_json = serde_json::to_string(&dictionary_entry_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        self.execute(
            r#"INSERT INTO window_context
               (id, name, matcher_app_name, matcher_title_pattern, matcher_bundle_id,
                command_mode, dictionary_mode, command_ids_json, dictionary_entry_ids_json,
                enabled, priority, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
            params![
                id.to_string(),
                name.clone(),
                matcher.app_name.clone(),
                matcher.title_pattern.clone(),
                matcher.bundle_id.clone(),
                override_mode_to_string(command_mode),
                override_mode_to_string(dictionary_mode),
                command_ids_json,
                dictionary_entry_ids_json,
                enabled as i32,
                priority,
                created_at
            ],
        )
        .await
        .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        Ok(WindowContext {
            id,
            name,
            matcher,
            command_mode,
            dictionary_mode,
            command_ids,
            dictionary_entry_ids,
            enabled,
            priority,
        })
    }

    /// Update an existing window context.
    ///
    /// # Arguments
    /// * `context` - The window context with updated values
    ///
    /// # Returns
    /// The updated WindowContext
    pub async fn update_window_context(
        &self,
        context: WindowContext,
    ) -> Result<WindowContext, WindowContextStoreError> {
        // Check if context exists
        let exists = self.window_context_exists(context.id).await?;
        if !exists {
            return Err(WindowContextStoreError::NotFound(context.id));
        }

        // Serialize UUIDs as strings in JSON
        let command_ids_json = serde_json::to_string(&context.command_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;
        let dictionary_entry_ids_json = serde_json::to_string(&context.dictionary_entry_ids)
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        self.execute(
            r#"UPDATE window_context
               SET name = ?1, matcher_app_name = ?2, matcher_title_pattern = ?3, matcher_bundle_id = ?4,
                   command_mode = ?5, dictionary_mode = ?6, command_ids_json = ?7, dictionary_entry_ids_json = ?8,
                   enabled = ?9, priority = ?10
               WHERE id = ?11"#,
            params![
                context.name.clone(),
                context.matcher.app_name.clone(),
                context.matcher.title_pattern.clone(),
                context.matcher.bundle_id.clone(),
                override_mode_to_string(context.command_mode),
                override_mode_to_string(context.dictionary_mode),
                command_ids_json,
                dictionary_entry_ids_json,
                context.enabled as i32,
                context.priority,
                context.id.to_string()
            ],
        )
        .await
        .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        Ok(context)
    }

    /// Delete a window context by ID.
    ///
    /// # Arguments
    /// * `id` - The context ID to delete
    pub async fn delete_window_context(&self, id: Uuid) -> Result<(), WindowContextStoreError> {
        // Check if context exists
        let exists = self.window_context_exists(id).await?;
        if !exists {
            return Err(WindowContextStoreError::NotFound(id));
        }

        self.execute(
            "DELETE FROM window_context WHERE id = ?1",
            params![id.to_string()],
        )
        .await
        .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    /// List all window contexts ordered by priority (descending).
    ///
    /// # Returns
    /// Vector of all window contexts, highest priority first
    pub async fn list_window_contexts(&self) -> Result<Vec<WindowContext>, WindowContextStoreError> {
        let mut rows = self
            .query(
                r#"SELECT id, name, matcher_app_name, matcher_title_pattern, matcher_bundle_id,
                          command_mode, dictionary_mode, command_ids_json, dictionary_entry_ids_json,
                          enabled, priority
                   FROM window_context
                   ORDER BY priority DESC"#,
                (),
            )
            .await
            .map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;

        let mut contexts = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?
        {
            let context = parse_window_context_row(&row)?;
            contexts.push(context);
        }

        Ok(contexts)
    }

    /// Get a window context by ID.
    ///
    /// # Arguments
    /// * `id` - The context ID to look up
    ///
    /// # Returns
    /// The window context if found
    pub async fn get_window_context(&self, id: Uuid) -> Result<Option<WindowContext>, WindowContextStoreError> {
        let mut rows = self
            .query(
                r#"SELECT id, name, matcher_app_name, matcher_title_pattern, matcher_bundle_id,
                          command_mode, dictionary_mode, command_ids_json, dictionary_entry_ids_json,
                          enabled, priority
                   FROM window_context
                   WHERE id = ?1"#,
                params![id.to_string()],
            )
            .await
            .map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;

        match rows
            .next()
            .await
            .map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?
        {
            Some(row) => {
                let context = parse_window_context_row(&row)?;
                Ok(Some(context))
            }
            None => Ok(None),
        }
    }

    /// Check if a window context exists by ID.
    async fn window_context_exists(&self, id: Uuid) -> Result<bool, WindowContextStoreError> {
        let mut rows = self
            .query(
                "SELECT 1 FROM window_context WHERE id = ?1",
                params![id.to_string()],
            )
            .await
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?;

        Ok(rows
            .next()
            .await
            .map_err(|e| WindowContextStoreError::PersistenceError(e.to_string()))?
            .is_some())
    }
}

/// Parse a database row into a WindowContext
fn parse_window_context_row(row: &libsql::Row) -> Result<WindowContext, WindowContextStoreError> {
    let id_str: String = row.get(0).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let name: String = row.get(1).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let matcher_app_name: String = row.get(2).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let matcher_title_pattern: Option<String> = row.get(3).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let matcher_bundle_id: Option<String> = row.get(4).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let command_mode_str: String = row.get(5).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let dictionary_mode_str: String = row.get(6).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let command_ids_json: String = row.get(7).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let dictionary_entry_ids_json: String = row.get(8).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let enabled: i32 = row.get(9).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;
    let priority: i32 = row.get(10).map_err(|e| WindowContextStoreError::LoadError(e.to_string()))?;

    let id = Uuid::parse_str(&id_str)
        .map_err(|e| WindowContextStoreError::LoadError(format!("Invalid UUID: {}", e)))?;

    let command_ids: Vec<Uuid> = serde_json::from_str(&command_ids_json)
        .map_err(|e| WindowContextStoreError::LoadError(format!("Invalid command_ids JSON: {}", e)))?;

    let dictionary_entry_ids: Vec<String> = serde_json::from_str(&dictionary_entry_ids_json)
        .map_err(|e| WindowContextStoreError::LoadError(format!("Invalid dictionary_entry_ids JSON: {}", e)))?;

    Ok(WindowContext {
        id,
        name,
        matcher: WindowMatcher {
            app_name: matcher_app_name,
            title_pattern: matcher_title_pattern,
            bundle_id: matcher_bundle_id,
        },
        command_mode: string_to_override_mode(&command_mode_str),
        dictionary_mode: string_to_override_mode(&dictionary_mode_str),
        command_ids,
        dictionary_entry_ids,
        enabled: enabled != 0,
        priority,
    })
}

/// Convert OverrideMode to string for database storage
fn override_mode_to_string(mode: OverrideMode) -> String {
    match mode {
        OverrideMode::Merge => "merge".to_string(),
        OverrideMode::Replace => "replace".to_string(),
    }
}

/// Convert string to OverrideMode
fn string_to_override_mode(s: &str) -> OverrideMode {
    match s {
        "replace" => OverrideMode::Replace,
        _ => OverrideMode::Merge,
    }
}

#[cfg(test)]
#[path = "window_context_test.rs"]
mod tests;
