use super::*;
use crate::turso::TursoClient;
use libsql::params;
use tempfile::TempDir;

/// Test schema initialization creates all tables
#[tokio::test]
async fn test_schema_creates_all_tables() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");

    initialize_schema(&client).await.expect("Failed to initialize schema");

    // Verify all tables exist
    let tables = ["dictionary_entry", "window_context", "recording", "transcription", "voice_command", "schema_version"];

    for table in tables {
        let mut rows = client
            .query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
                params![table],
            )
            .await
            .expect("Query failed");

        assert!(
            rows.next().await.expect("Failed to get next").is_some(),
            "Table {} should exist",
            table
        );
    }
}

/// Test schema is idempotent (can be called multiple times)
#[tokio::test]
async fn test_schema_initialization_is_idempotent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");

    // Initialize twice
    initialize_schema(&client).await.expect("First init failed");
    initialize_schema(&client).await.expect("Second init failed");

    // Schema should still be valid
    let version = get_schema_version(&client).await.expect("Failed to get version");
    assert_eq!(version, SCHEMA_VERSION);
}

/// Test dictionary_entry table has correct constraints
#[tokio::test]
async fn test_dictionary_entry_unique_trigger() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");

    initialize_schema(&client).await.expect("Failed to initialize schema");

    // Insert first entry
    client
        .execute(
            "INSERT INTO dictionary_entry (id, trigger, expansion, created_at) VALUES (?1, ?2, ?3, ?4)",
            params!["id-1", "ty", "thank you", "2025-01-01T00:00:00Z"],
        )
        .await
        .expect("First insert should succeed");

    // Try to insert with same trigger - should fail
    let result = client
        .execute(
            "INSERT INTO dictionary_entry (id, trigger, expansion, created_at) VALUES (?1, ?2, ?3, ?4)",
            params!["id-2", "ty", "thanks", "2025-01-01T00:00:00Z"],
        )
        .await;

    assert!(result.is_err(), "Duplicate trigger should fail");
}

/// Test recording table has correct constraints
#[tokio::test]
async fn test_recording_unique_file_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");

    initialize_schema(&client).await.expect("Failed to initialize schema");

    // Insert first recording
    client
        .execute(
            "INSERT INTO recording (id, file_path, duration_secs, sample_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["id-1", "/path/to/file.wav", 10.5, 168000, "2025-01-01T00:00:00Z"],
        )
        .await
        .expect("First insert should succeed");

    // Try to insert with same file_path - should fail
    let result = client
        .execute(
            "INSERT INTO recording (id, file_path, duration_secs, sample_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["id-2", "/path/to/file.wav", 5.0, 80000, "2025-01-01T00:00:00Z"],
        )
        .await;

    assert!(result.is_err(), "Duplicate file_path should fail");
}

/// Test transcription foreign key cascade delete
#[tokio::test]
async fn test_transcription_cascade_delete() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");

    initialize_schema(&client).await.expect("Failed to initialize schema");

    // Insert recording
    client
        .execute(
            "INSERT INTO recording (id, file_path, duration_secs, sample_count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["rec-1", "/path/to/file.wav", 10.5, 168000, "2025-01-01T00:00:00Z"],
        )
        .await
        .expect("Insert recording failed");

    // Insert transcription linked to recording
    client
        .execute(
            "INSERT INTO transcription (id, recording_id, text, model_version, duration_ms, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params!["trans-1", "rec-1", "hello world", "v1.0", 500, "2025-01-01T00:00:00Z"],
        )
        .await
        .expect("Insert transcription failed");

    // Delete recording - should cascade to transcription
    client
        .execute("DELETE FROM recording WHERE id = ?1", params!["rec-1"])
        .await
        .expect("Delete recording failed");

    // Verify transcription is also deleted
    let mut rows = client
        .query("SELECT id FROM transcription WHERE id = ?1", params!["trans-1"])
        .await
        .expect("Query failed");

    assert!(
        rows.next().await.expect("Failed to get next").is_none(),
        "Transcription should be deleted with recording"
    );
}

/// Test voice_command table has correct constraints
#[tokio::test]
async fn test_voice_command_unique_trigger() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");

    initialize_schema(&client).await.expect("Failed to initialize schema");

    // Insert first command
    client
        .execute(
            "INSERT INTO voice_command (id, trigger, action_type, parameters_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["id-1", "open browser", "open_app", "{\"app\": \"Safari\"}", "2025-01-01T00:00:00Z"],
        )
        .await
        .expect("First insert should succeed");

    // Try to insert with same trigger - should fail
    let result = client
        .execute(
            "INSERT INTO voice_command (id, trigger, action_type, parameters_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["id-2", "open browser", "open_app", "{\"app\": \"Chrome\"}", "2025-01-01T00:00:00Z"],
        )
        .await;

    assert!(result.is_err(), "Duplicate trigger should fail");
}

/// Test transcription index exists for recording_id lookups
#[tokio::test]
async fn test_transcription_recording_id_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");

    initialize_schema(&client).await.expect("Failed to initialize schema");

    // Check index exists
    let mut rows = client
        .query(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_transcription_recording_id'",
            (),
        )
        .await
        .expect("Query failed");

    assert!(
        rows.next().await.expect("Failed to get next").is_some(),
        "Index idx_transcription_recording_id should exist"
    );
}
