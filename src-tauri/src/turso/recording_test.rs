use crate::audio::StopReason;
use crate::turso::{initialize_schema, TursoClient};
use tempfile::TempDir;

async fn setup_client() -> (TursoClient, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let client = TursoClient::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create client");
    initialize_schema(&client)
        .await
        .expect("Failed to initialize schema");
    (client, temp_dir)
}

// ============================================================
// Recording Tests
// ============================================================

#[tokio::test]
async fn test_add_recording() {
    let (client, _temp) = setup_client().await;

    let recording = client
        .add_recording(
            "rec-1".to_string(),
            "/path/to/recording.wav".to_string(),
            5.5,
            88200,
            Some(StopReason::SilenceAfterSpeech),
            Some("Safari".to_string()),
            Some("com.apple.Safari".to_string()),
            Some("Google - Safari".to_string()),
        )
        .await
        .expect("Failed to add recording");

    assert_eq!(recording.id, "rec-1");
    assert_eq!(recording.file_path, "/path/to/recording.wav");
    assert!((recording.duration_secs - 5.5).abs() < 0.001);
    assert_eq!(recording.sample_count, 88200);
    assert!(matches!(recording.stop_reason, Some(StopReason::SilenceAfterSpeech)));
    assert_eq!(recording.active_window_app_name, Some("Safari".to_string()));
}

#[tokio::test]
async fn test_add_recording_minimal() {
    let (client, _temp) = setup_client().await;

    let recording = client
        .add_recording(
            "rec-2".to_string(),
            "/path/to/minimal.wav".to_string(),
            1.0,
            16000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add recording");

    assert_eq!(recording.id, "rec-2");
    assert!(recording.stop_reason.is_none());
    assert!(recording.active_window_app_name.is_none());
}

#[tokio::test]
async fn test_list_recordings_empty() {
    let (client, _temp) = setup_client().await;

    let recordings = client.list_recordings().await.expect("Failed to list");
    assert!(recordings.is_empty());
}

#[tokio::test]
async fn test_list_recordings_ordered_by_date() {
    let (client, _temp) = setup_client().await;

    // Add recordings (newest should be listed first)
    client
        .add_recording(
            "rec-old".to_string(),
            "/path/old.wav".to_string(),
            1.0,
            16000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add old");

    // Small delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    client
        .add_recording(
            "rec-new".to_string(),
            "/path/new.wav".to_string(),
            2.0,
            32000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add new");

    let recordings = client.list_recordings().await.expect("Failed to list");
    assert_eq!(recordings.len(), 2);
    // Should be ordered by created_at DESC (newest first)
    assert_eq!(recordings[0].id, "rec-new");
    assert_eq!(recordings[1].id, "rec-old");
}

#[tokio::test]
async fn test_get_recording_by_path() {
    let (client, _temp) = setup_client().await;

    client
        .add_recording(
            "rec-find".to_string(),
            "/path/to/find.wav".to_string(),
            3.0,
            48000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add");

    let found = client
        .get_recording_by_path("/path/to/find.wav")
        .await
        .expect("Failed to get")
        .expect("Recording should exist");

    assert_eq!(found.id, "rec-find");
}

#[tokio::test]
async fn test_get_recording_by_path_not_found() {
    let (client, _temp) = setup_client().await;

    let result = client
        .get_recording_by_path("/nonexistent.wav")
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_recording_by_path() {
    let (client, _temp) = setup_client().await;

    client
        .add_recording(
            "rec-delete".to_string(),
            "/path/to/delete.wav".to_string(),
            1.0,
            16000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add");

    client
        .delete_recording_by_path("/path/to/delete.wav")
        .await
        .expect("Failed to delete");

    let result = client
        .get_recording_by_path("/path/to/delete.wav")
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_recording_not_found() {
    let (client, _temp) = setup_client().await;

    let result = client.delete_recording_by_path("/nonexistent.wav").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_stop_reason_roundtrip() {
    let (client, _temp) = setup_client().await;

    let stop_reasons = [
        StopReason::BufferFull,
        StopReason::LockError,
        StopReason::StreamError,
        StopReason::ResampleOverflow,
        StopReason::SilenceAfterSpeech,
        StopReason::NoSpeechTimeout,
    ];

    for (i, stop_reason) in stop_reasons.iter().enumerate() {
        client
            .add_recording(
                format!("rec-{}", i),
                format!("/path/{}.wav", i),
                1.0,
                16000,
                Some(stop_reason.clone()),
                None,
                None,
                None,
            )
            .await
            .expect("Failed to add");
    }

    let recordings = client.list_recordings().await.expect("Failed to list");
    assert_eq!(recordings.len(), 6);

    // Check that stop reasons roundtrip correctly
    for recording in &recordings {
        assert!(recording.stop_reason.is_some());
    }
}

// ============================================================
// Transcription Tests
// ============================================================

#[tokio::test]
async fn test_add_transcription() {
    let (client, _temp) = setup_client().await;

    // First add a recording (foreign key constraint)
    client
        .add_recording(
            "rec-1".to_string(),
            "/path/recording.wav".to_string(),
            5.0,
            80000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add recording");

    let transcription = client
        .add_transcription(
            "trans-1".to_string(),
            "rec-1".to_string(),
            "Hello world".to_string(),
            Some("en".to_string()),
            "parakeet-tdt".to_string(),
            250,
        )
        .await
        .expect("Failed to add transcription");

    assert_eq!(transcription.id, "trans-1");
    assert_eq!(transcription.recording_id, "rec-1");
    assert_eq!(transcription.text, "Hello world");
    assert_eq!(transcription.language, Some("en".to_string()));
    assert_eq!(transcription.model_version, "parakeet-tdt");
    assert_eq!(transcription.duration_ms, 250);
}

#[tokio::test]
async fn test_list_transcriptions() {
    let (client, _temp) = setup_client().await;

    // Add a recording
    client
        .add_recording(
            "rec-1".to_string(),
            "/path/recording.wav".to_string(),
            5.0,
            80000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add recording");

    // Add transcriptions
    client
        .add_transcription(
            "trans-1".to_string(),
            "rec-1".to_string(),
            "First".to_string(),
            None,
            "parakeet-tdt".to_string(),
            100,
        )
        .await
        .expect("Failed to add transcription 1");

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    client
        .add_transcription(
            "trans-2".to_string(),
            "rec-1".to_string(),
            "Second".to_string(),
            None,
            "parakeet-tdt".to_string(),
            150,
        )
        .await
        .expect("Failed to add transcription 2");

    let transcriptions = client.list_transcriptions().await.expect("Failed to list");
    assert_eq!(transcriptions.len(), 2);
    // Should be ordered by created_at DESC
    assert_eq!(transcriptions[0].text, "Second");
    assert_eq!(transcriptions[1].text, "First");
}

#[tokio::test]
async fn test_get_transcriptions_by_recording() {
    let (client, _temp) = setup_client().await;

    // Add two recordings
    client
        .add_recording(
            "rec-1".to_string(),
            "/path/rec1.wav".to_string(),
            5.0,
            80000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add recording 1");

    client
        .add_recording(
            "rec-2".to_string(),
            "/path/rec2.wav".to_string(),
            3.0,
            48000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add recording 2");

    // Add transcriptions to rec-1
    client
        .add_transcription(
            "trans-1a".to_string(),
            "rec-1".to_string(),
            "Rec 1 trans a".to_string(),
            None,
            "parakeet-tdt".to_string(),
            100,
        )
        .await
        .expect("Failed to add");

    client
        .add_transcription(
            "trans-1b".to_string(),
            "rec-1".to_string(),
            "Rec 1 trans b".to_string(),
            None,
            "parakeet-tdt".to_string(),
            110,
        )
        .await
        .expect("Failed to add");

    // Add transcription to rec-2
    client
        .add_transcription(
            "trans-2".to_string(),
            "rec-2".to_string(),
            "Rec 2 trans".to_string(),
            None,
            "parakeet-tdt".to_string(),
            90,
        )
        .await
        .expect("Failed to add");

    // Get transcriptions for rec-1
    let rec1_trans = client
        .get_transcriptions_by_recording("rec-1")
        .await
        .expect("Failed to get");
    assert_eq!(rec1_trans.len(), 2);

    // Get transcriptions for rec-2
    let rec2_trans = client
        .get_transcriptions_by_recording("rec-2")
        .await
        .expect("Failed to get");
    assert_eq!(rec2_trans.len(), 1);
    assert_eq!(rec2_trans[0].text, "Rec 2 trans");
}

#[tokio::test]
async fn test_cascade_delete() {
    let (client, _temp) = setup_client().await;

    // Add a recording
    client
        .add_recording(
            "rec-cascade".to_string(),
            "/path/cascade.wav".to_string(),
            5.0,
            80000,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add recording");

    // Add transcriptions
    client
        .add_transcription(
            "trans-cascade-1".to_string(),
            "rec-cascade".to_string(),
            "First".to_string(),
            None,
            "parakeet-tdt".to_string(),
            100,
        )
        .await
        .expect("Failed to add transcription");

    client
        .add_transcription(
            "trans-cascade-2".to_string(),
            "rec-cascade".to_string(),
            "Second".to_string(),
            None,
            "parakeet-tdt".to_string(),
            100,
        )
        .await
        .expect("Failed to add transcription");

    // Verify transcriptions exist
    let trans_before = client
        .get_transcriptions_by_recording("rec-cascade")
        .await
        .expect("Failed to get");
    assert_eq!(trans_before.len(), 2);

    // Delete the recording
    client
        .delete_recording_by_path("/path/cascade.wav")
        .await
        .expect("Failed to delete recording");

    // Verify transcriptions are also deleted (CASCADE)
    let trans_after = client
        .get_transcriptions_by_recording("rec-cascade")
        .await
        .expect("Failed to get");
    assert!(trans_after.is_empty());
}
