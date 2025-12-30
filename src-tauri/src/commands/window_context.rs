// Window context Tauri commands
// Exposes window context functionality to the frontend
//
// This file contains Tauri-specific wrappers and is excluded from coverage.
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::turso::{events as turso_events, TursoClient};
use crate::window_context::{
    get_active_window, get_running_applications, ActiveWindowInfo, OverrideMode,
    RunningApplication, WindowContext, WindowContextStoreError, WindowMatcher,
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use uuid::Uuid;

/// Type alias for Turso client state
pub type TursoClientState = Arc<TursoClient>;

/// Map WindowContextStoreError to user-friendly error messages
fn to_user_error(error: WindowContextStoreError) -> String {
    match error {
        WindowContextStoreError::NotFound(id) => format!("Context with ID '{}' not found", id),
        WindowContextStoreError::DuplicateId(id) => {
            format!("Context with ID '{}' already exists", id)
        }
        WindowContextStoreError::InvalidPattern(msg) => format!("Invalid pattern: {}", msg),
        WindowContextStoreError::PersistenceError(msg) => format!("Failed to save contexts: {}", msg),
        WindowContextStoreError::LoadError(msg) => format!("Failed to load contexts: {}", msg),
    }
}

/// Get information about the currently active window
///
/// Returns the frontmost application's name, bundle ID, window title, and process ID.
/// Useful for testing window detection and context matching.
#[tauri::command]
pub fn get_active_window_info() -> Result<ActiveWindowInfo, String> {
    get_active_window()
}

/// List all running user-visible applications
///
/// Returns applications that have a user interface (activationPolicy == .regular).
/// Background helpers, agents, and daemons are filtered out.
/// Results are sorted alphabetically by application name.
#[tauri::command]
pub fn list_running_applications() -> Vec<RunningApplication> {
    get_running_applications()
}

/// List all window contexts
///
/// Returns all contexts from Turso database.
#[tauri::command]
pub async fn list_window_contexts(
    turso_client: State<'_, TursoClientState>,
) -> Result<Vec<WindowContext>, String> {
    turso_client
        .list_window_contexts()
        .await
        .map_err(to_user_error)
}

/// Add a new window context
///
/// Creates a new context with the given parameters, generates a unique ID,
/// persists to Turso, and emits a window_contexts_updated event.
#[tauri::command]
pub async fn add_window_context(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    name: String,
    app_name: String,
    title_pattern: Option<String>,
    bundle_id: Option<String>,
    command_mode: Option<String>,
    dictionary_mode: Option<String>,
    command_ids: Option<Vec<String>>,
    dictionary_entry_ids: Option<Vec<String>>,
    enabled: Option<bool>,
    priority: Option<i32>,
) -> Result<WindowContext, String> {
    // Validate: name cannot be empty
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // Validate: app_name cannot be empty
    if app_name.trim().is_empty() {
        return Err("App name cannot be empty".to_string());
    }

    let matcher = WindowMatcher {
        app_name,
        title_pattern,
        bundle_id,
    };

    let command_mode_val = parse_override_mode(command_mode.as_deref());
    let dictionary_mode_val = parse_override_mode(dictionary_mode.as_deref());

    let command_ids_val: Vec<Uuid> = command_ids
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();
    let dictionary_entry_ids_val = dictionary_entry_ids.clone().unwrap_or_default();
    let enabled_val = enabled.unwrap_or(true);
    let priority_val = priority.unwrap_or(0);

    // Add context to Turso
    let context = turso_client
        .add_window_context(
            name.clone(),
            matcher.clone(),
            command_mode_val,
            dictionary_mode_val,
            command_ids_val,
            dictionary_entry_ids_val,
            enabled_val,
            priority_val,
        )
        .await
        .map_err(to_user_error)?;

    // Emit window_contexts_updated event
    turso_events::emit_window_contexts_updated(&app_handle, "add", &context.id.to_string());

    crate::info!("Added window context: {} ({})", context.name, context.id);
    Ok(context)
}

/// Update an existing window context
///
/// Updates the context with the given ID, persists to Turso,
/// and emits a window_contexts_updated event.
#[tauri::command]
pub async fn update_window_context(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    id: String,
    name: String,
    app_name: String,
    title_pattern: Option<String>,
    bundle_id: Option<String>,
    command_mode: Option<String>,
    dictionary_mode: Option<String>,
    command_ids: Option<Vec<String>>,
    dictionary_entry_ids: Option<Vec<String>>,
    enabled: Option<bool>,
    priority: Option<i32>,
) -> Result<(), String> {
    // Validate: name cannot be empty
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // Validate: app_name cannot be empty
    if app_name.trim().is_empty() {
        return Err("App name cannot be empty".to_string());
    }

    let uuid = Uuid::parse_str(&id).map_err(|_| format!("Invalid UUID: {}", id))?;

    let matcher = WindowMatcher {
        app_name,
        title_pattern,
        bundle_id,
    };

    let command_mode_val = parse_override_mode(command_mode.as_deref());
    let dictionary_mode_val = parse_override_mode(dictionary_mode.as_deref());

    let command_ids_val: Vec<Uuid> = command_ids
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .collect();

    let context = WindowContext {
        id: uuid,
        name,
        matcher,
        command_mode: command_mode_val,
        dictionary_mode: dictionary_mode_val,
        command_ids: command_ids_val,
        dictionary_entry_ids: dictionary_entry_ids.unwrap_or_default(),
        enabled: enabled.unwrap_or(true),
        priority: priority.unwrap_or(0),
    };

    // Update context in Turso
    turso_client
        .update_window_context(context)
        .await
        .map_err(to_user_error)?;

    // Emit window_contexts_updated event
    turso_events::emit_window_contexts_updated(&app_handle, "update", &id);

    crate::info!("Updated window context: {}", id);
    Ok(())
}

/// Delete a window context
///
/// Removes the context with the given ID, persists to Turso,
/// and emits a window_contexts_updated event.
#[tauri::command]
pub async fn delete_window_context(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|_| format!("Invalid UUID: {}", id))?;

    // Delete context from Turso
    turso_client
        .delete_window_context(uuid)
        .await
        .map_err(to_user_error)?;

    // Emit window_contexts_updated event
    turso_events::emit_window_contexts_updated(&app_handle, "delete", &id);

    crate::info!("Deleted window context: {}", id);
    Ok(())
}

/// Parse override mode from string
fn parse_override_mode(mode: Option<&str>) -> OverrideMode {
    match mode {
        Some("replace") => OverrideMode::Replace,
        _ => OverrideMode::Merge,
    }
}
