use super::*;

/// Test payload structures serialize correctly with camelCase
#[test]
fn test_voice_commands_payload_serialization() {
    let payload = VoiceCommandsUpdatedPayload {
        action: "add".to_string(),
        command_id: "cmd-123".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"action\":\"add\""));
    assert!(json.contains("\"commandId\":\"cmd-123\""));
    // Verify camelCase (not snake_case)
    assert!(!json.contains("command_id"));
}

#[test]
fn test_recordings_payload_serialization() {
    let payload = RecordingsUpdatedPayload {
        change_type: "delete".to_string(),
        recording_id: Some("rec-456".to_string()),
        timestamp: "2025-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"changeType\":\"delete\""));
    assert!(json.contains("\"recordingId\":\"rec-456\""));
    assert!(json.contains("\"timestamp\":\"2025-01-01T00:00:00Z\""));
    // Verify camelCase
    assert!(!json.contains("change_type"));
    assert!(!json.contains("recording_id"));
}

#[test]
fn test_transcriptions_payload_serialization() {
    let payload = TranscriptionsUpdatedPayload {
        change_type: "add".to_string(),
        transcription_id: Some("trans-789".to_string()),
        recording_id: Some("rec-456".to_string()),
        timestamp: "2025-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"changeType\":\"add\""));
    assert!(json.contains("\"transcriptionId\":\"trans-789\""));
    assert!(json.contains("\"recordingId\":\"rec-456\""));
    // Verify camelCase
    assert!(!json.contains("change_type"));
    assert!(!json.contains("transcription_id"));
    assert!(!json.contains("recording_id"));
}

#[test]
fn test_recordings_payload_with_none_id() {
    let payload = RecordingsUpdatedPayload {
        change_type: "sync".to_string(),
        recording_id: None,
        timestamp: "2025-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"recordingId\":null"));
}

#[test]
fn test_transcriptions_payload_with_none_ids() {
    let payload = TranscriptionsUpdatedPayload {
        change_type: "sync".to_string(),
        transcription_id: None,
        recording_id: None,
        timestamp: "2025-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("\"transcriptionId\":null"));
    assert!(json.contains("\"recordingId\":null"));
}
