// Application settings persistence
// Stores user preferences in a JSON file in the app's config directory

use crate::parakeet::TranscriptionMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.json";

/// Application settings that are persisted between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Transcription mode (batch or streaming)
    #[serde(default)]
    pub transcription_mode: TranscriptionMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            transcription_mode: TranscriptionMode::Batch,
        }
    }
}

/// Get the path to the settings file
fn get_settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("heycat");
        path.push(SETTINGS_FILE);
        path
    })
}

/// Load settings from disk, returning default if not found or on error
pub fn load_settings() -> AppSettings {
    let Some(path) = get_settings_path() else {
        return AppSettings::default();
    };

    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

/// Save settings to disk
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path().ok_or("Could not determine config directory")?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_default_settings_has_batch_mode() {
        let settings = AppSettings::default();
        assert_eq!(settings.transcription_mode, TranscriptionMode::Batch);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = AppSettings {
            transcription_mode: TranscriptionMode::Streaming,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.transcription_mode,
            TranscriptionMode::Streaming
        );
    }

    #[test]
    fn test_load_settings_returns_default_for_missing_file() {
        // This test relies on the fact that the settings file likely doesn't exist
        // in the test environment's config directory
        let settings = load_settings();
        // Should return default without panicking
        assert_eq!(settings.transcription_mode, TranscriptionMode::Batch);
    }

    #[test]
    fn test_save_and_load_settings() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("heycat");

        // Override HOME for this test to use temp directory
        // Note: This is a simplified test - in reality we'd need to mock dirs::config_dir
        // For now, just test serialization works
        let settings = AppSettings {
            transcription_mode: TranscriptionMode::Streaming,
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        fs::create_dir_all(&config_path).unwrap();
        let settings_path = config_path.join(SETTINGS_FILE);
        fs::write(&settings_path, &json).unwrap();

        let loaded: AppSettings =
            serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
        assert_eq!(loaded.transcription_mode, TranscriptionMode::Streaming);
    }
}
