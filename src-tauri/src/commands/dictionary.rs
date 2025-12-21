// Dictionary Tauri commands for CRUD operations
// Exposes DictionaryStore to the frontend and emits dictionary_updated events on mutations
//
// This file contains Tauri-specific wrappers and is excluded from coverage.
#![cfg_attr(coverage_nightly, coverage(off))]

use crate::dictionary::{DictionaryEntry, DictionaryError, DictionaryStore};
use crate::events::dictionary_events::{self, DictionaryUpdatedPayload};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

/// Type alias for dictionary store state
pub type DictionaryStoreState = Mutex<DictionaryStore>;

/// Helper macro to emit events with error logging
macro_rules! emit_or_warn {
    ($handle:expr, $event:expr, $payload:expr) => {
        if let Err(e) = $handle.emit($event, $payload) {
            crate::warn!("Failed to emit event '{}': {}", $event, e);
        }
    };
}

/// Map DictionaryError to user-friendly error messages
fn to_user_error(error: DictionaryError) -> String {
    match error {
        DictionaryError::NotFound(id) => format!("Entry with ID '{}' not found", id),
        DictionaryError::DuplicateId(id) => format!("Entry with ID '{}' already exists", id),
        DictionaryError::PersistenceError(msg) => format!("Failed to save dictionary: {}", msg),
        DictionaryError::LoadError(msg) => format!("Failed to load dictionary: {}", msg),
    }
}

/// List all dictionary entries
///
/// Returns all entries from the dictionary store.
#[tauri::command]
pub fn list_dictionary_entries(
    store: State<'_, DictionaryStoreState>,
) -> Result<Vec<DictionaryEntry>, String> {
    let store = store.lock().map_err(|_| "Failed to access dictionary store".to_string())?;
    Ok(store.list().into_iter().cloned().collect())
}

/// Add a new dictionary entry
///
/// Creates a new entry with the given trigger and expansion, generates a unique ID,
/// persists to storage, and emits a dictionary_updated event.
///
/// # Arguments
/// * `trigger` - The trigger word/phrase (e.g., "brb")
/// * `expansion` - The expansion text (e.g., "be right back")
///
/// # Returns
/// The newly created DictionaryEntry with its generated ID
#[tauri::command]
pub fn add_dictionary_entry(
    app_handle: AppHandle,
    store: State<'_, DictionaryStoreState>,
    trigger: String,
    expansion: String,
) -> Result<DictionaryEntry, String> {
    // Validate: trigger cannot be empty
    if trigger.trim().is_empty() {
        return Err("Trigger cannot be empty".to_string());
    }

    let mut store = store.lock().map_err(|_| "Failed to access dictionary store".to_string())?;
    let entry = store.add(trigger, expansion).map_err(to_user_error)?;

    // Emit dictionary_updated event
    emit_or_warn!(
        app_handle,
        dictionary_events::DICTIONARY_UPDATED,
        DictionaryUpdatedPayload {
            action: "add".to_string(),
            entry_id: entry.id.clone(),
        }
    );

    crate::info!("Added dictionary entry: {} -> {}", entry.trigger, entry.expansion);
    Ok(entry)
}

/// Update an existing dictionary entry
///
/// Updates the trigger and expansion for the entry with the given ID,
/// persists to storage, and emits a dictionary_updated event.
///
/// # Arguments
/// * `id` - The unique ID of the entry to update
/// * `trigger` - The new trigger word/phrase
/// * `expansion` - The new expansion text
#[tauri::command]
pub fn update_dictionary_entry(
    app_handle: AppHandle,
    store: State<'_, DictionaryStoreState>,
    id: String,
    trigger: String,
    expansion: String,
) -> Result<(), String> {
    // Validate: trigger cannot be empty
    if trigger.trim().is_empty() {
        return Err("Trigger cannot be empty".to_string());
    }

    let mut store = store.lock().map_err(|_| "Failed to access dictionary store".to_string())?;
    store.update(id.clone(), trigger, expansion).map_err(to_user_error)?;

    // Emit dictionary_updated event
    emit_or_warn!(
        app_handle,
        dictionary_events::DICTIONARY_UPDATED,
        DictionaryUpdatedPayload {
            action: "update".to_string(),
            entry_id: id.clone(),
        }
    );

    crate::info!("Updated dictionary entry: {}", id);
    Ok(())
}

/// Delete a dictionary entry
///
/// Removes the entry with the given ID, persists to storage,
/// and emits a dictionary_updated event.
///
/// # Arguments
/// * `id` - The unique ID of the entry to delete
#[tauri::command]
pub fn delete_dictionary_entry(
    app_handle: AppHandle,
    store: State<'_, DictionaryStoreState>,
    id: String,
) -> Result<(), String> {
    let mut store = store.lock().map_err(|_| "Failed to access dictionary store".to_string())?;
    store.delete(&id).map_err(to_user_error)?;

    // Emit dictionary_updated event
    emit_or_warn!(
        app_handle,
        dictionary_events::DICTIONARY_UPDATED,
        DictionaryUpdatedPayload {
            action: "delete".to_string(),
            entry_id: id.clone(),
        }
    );

    crate::info!("Deleted dictionary entry: {}", id);
    Ok(())
}

#[cfg(test)]
mod tests;
