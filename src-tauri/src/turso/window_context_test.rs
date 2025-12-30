use crate::turso::{initialize_schema, TursoClient};
use crate::window_context::{OverrideMode, WindowContext, WindowContextStoreError, WindowMatcher};
use tempfile::TempDir;
use uuid::Uuid;

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

fn make_matcher(app_name: &str) -> WindowMatcher {
    WindowMatcher {
        app_name: app_name.to_string(),
        title_pattern: None,
        bundle_id: None,
    }
}

#[tokio::test]
async fn test_add_window_context() {
    let (client, _temp) = setup_client().await;

    let context = client
        .add_window_context(
            "Test Context".to_string(),
            make_matcher("Safari"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .await
        .expect("Failed to add context");

    assert!(!context.id.is_nil());
    assert_eq!(context.name, "Test Context");
    assert_eq!(context.matcher.app_name, "Safari");
    assert!(context.enabled);
    assert_eq!(context.priority, 0);
}

#[tokio::test]
async fn test_add_window_context_with_full_matcher() {
    let (client, _temp) = setup_client().await;

    let matcher = WindowMatcher {
        app_name: "Chrome".to_string(),
        title_pattern: Some("GitHub.*".to_string()),
        bundle_id: Some("com.google.Chrome".to_string()),
    };

    let context = client
        .add_window_context(
            "Chrome GitHub".to_string(),
            matcher,
            OverrideMode::Replace,
            OverrideMode::Merge,
            vec![Uuid::new_v4(), Uuid::new_v4()],
            vec!["dict1".to_string(), "dict2".to_string()],
            false,
            10,
        )
        .await
        .expect("Failed to add context");

    assert_eq!(context.matcher.app_name, "Chrome");
    assert_eq!(context.matcher.title_pattern, Some("GitHub.*".to_string()));
    assert_eq!(context.matcher.bundle_id, Some("com.google.Chrome".to_string()));
    assert!(matches!(context.command_mode, OverrideMode::Replace));
    assert!(matches!(context.dictionary_mode, OverrideMode::Merge));
    assert_eq!(context.command_ids.len(), 2);
    assert_eq!(context.dictionary_entry_ids, vec!["dict1", "dict2"]);
    assert!(!context.enabled);
    assert_eq!(context.priority, 10);
}

#[tokio::test]
async fn test_list_window_contexts_empty() {
    let (client, _temp) = setup_client().await;

    let contexts = client.list_window_contexts().await.expect("Failed to list");
    assert!(contexts.is_empty());
}

#[tokio::test]
async fn test_list_window_contexts_ordered_by_priority() {
    let (client, _temp) = setup_client().await;

    // Add contexts with different priorities
    client
        .add_window_context(
            "Low Priority".to_string(),
            make_matcher("App1"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .await
        .expect("Failed to add low priority");

    client
        .add_window_context(
            "High Priority".to_string(),
            make_matcher("App2"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            100,
        )
        .await
        .expect("Failed to add high priority");

    client
        .add_window_context(
            "Medium Priority".to_string(),
            make_matcher("App3"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            50,
        )
        .await
        .expect("Failed to add medium priority");

    let contexts = client.list_window_contexts().await.expect("Failed to list");
    assert_eq!(contexts.len(), 3);
    // Should be ordered by priority DESC
    assert_eq!(contexts[0].name, "High Priority");
    assert_eq!(contexts[1].name, "Medium Priority");
    assert_eq!(contexts[2].name, "Low Priority");
}

#[tokio::test]
async fn test_get_window_context() {
    let (client, _temp) = setup_client().await;

    let context = client
        .add_window_context(
            "Test Get".to_string(),
            make_matcher("App"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .await
        .expect("Failed to add");

    let fetched = client
        .get_window_context(context.id)
        .await
        .expect("Failed to get")
        .expect("Context should exist");

    assert_eq!(fetched.id, context.id);
    assert_eq!(fetched.name, "Test Get");
}

#[tokio::test]
async fn test_get_window_context_not_found() {
    let (client, _temp) = setup_client().await;

    let result = client
        .get_window_context(Uuid::new_v4())
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_window_context() {
    let (client, _temp) = setup_client().await;

    let context = client
        .add_window_context(
            "Original".to_string(),
            make_matcher("App"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .await
        .expect("Failed to add");

    let updated = WindowContext {
        id: context.id,
        name: "Updated".to_string(),
        matcher: WindowMatcher {
            app_name: "NewApp".to_string(),
            title_pattern: Some(".*pattern.*".to_string()),
            bundle_id: Some("com.new.app".to_string()),
        },
        command_mode: OverrideMode::Replace,
        dictionary_mode: OverrideMode::Replace,
        command_ids: vec![Uuid::new_v4()],
        dictionary_entry_ids: vec!["entry1".to_string()],
        enabled: false,
        priority: 99,
    };

    client
        .update_window_context(updated.clone())
        .await
        .expect("Failed to update");

    let fetched = client
        .get_window_context(context.id)
        .await
        .expect("Failed to get")
        .expect("Context should exist");

    assert_eq!(fetched.name, "Updated");
    assert_eq!(fetched.matcher.app_name, "NewApp");
    assert_eq!(fetched.matcher.title_pattern, Some(".*pattern.*".to_string()));
    assert!(matches!(fetched.command_mode, OverrideMode::Replace));
    assert!(!fetched.enabled);
    assert_eq!(fetched.priority, 99);
}

#[tokio::test]
async fn test_update_window_context_not_found() {
    let (client, _temp) = setup_client().await;

    let context = WindowContext {
        id: Uuid::new_v4(),
        name: "Nonexistent".to_string(),
        matcher: make_matcher("App"),
        command_mode: OverrideMode::Merge,
        dictionary_mode: OverrideMode::Merge,
        command_ids: vec![],
        dictionary_entry_ids: vec![],
        enabled: true,
        priority: 0,
    };

    let result = client.update_window_context(context.clone()).await;

    match result.err().unwrap() {
        WindowContextStoreError::NotFound(id) => assert_eq!(id, context.id),
        other => panic!("Expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_delete_window_context() {
    let (client, _temp) = setup_client().await;

    let context = client
        .add_window_context(
            "To Delete".to_string(),
            make_matcher("App"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .await
        .expect("Failed to add");

    client
        .delete_window_context(context.id)
        .await
        .expect("Failed to delete");

    let result = client
        .get_window_context(context.id)
        .await
        .expect("Query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_window_context_not_found() {
    let (client, _temp) = setup_client().await;

    let id = Uuid::new_v4();
    let result = client.delete_window_context(id).await;

    match result.err().unwrap() {
        WindowContextStoreError::NotFound(found_id) => assert_eq!(found_id, id),
        other => panic!("Expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_json_serialization_roundtrip() {
    let (client, _temp) = setup_client().await;

    let command_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let dictionary_entry_ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];

    let context = client
        .add_window_context(
            "JSON Test".to_string(),
            make_matcher("App"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            command_ids.clone(),
            dictionary_entry_ids.clone(),
            true,
            0,
        )
        .await
        .expect("Failed to add");

    let fetched = client
        .get_window_context(context.id)
        .await
        .expect("Failed to get")
        .expect("Context should exist");

    assert_eq!(fetched.command_ids, command_ids);
    assert_eq!(fetched.dictionary_entry_ids, dictionary_entry_ids);
}

#[tokio::test]
async fn test_override_mode_roundtrip() {
    let (client, _temp) = setup_client().await;

    // Test Merge mode
    let ctx1 = client
        .add_window_context(
            "Merge".to_string(),
            make_matcher("App1"),
            OverrideMode::Merge,
            OverrideMode::Merge,
            vec![],
            vec![],
            true,
            0,
        )
        .await
        .expect("Failed to add merge context");

    // Test Replace mode
    let ctx2 = client
        .add_window_context(
            "Replace".to_string(),
            make_matcher("App2"),
            OverrideMode::Replace,
            OverrideMode::Replace,
            vec![],
            vec![],
            true,
            0,
        )
        .await
        .expect("Failed to add replace context");

    let contexts = client.list_window_contexts().await.expect("Failed to list");

    let merge_ctx = contexts.iter().find(|c| c.name == "Merge").unwrap();
    let replace_ctx = contexts.iter().find(|c| c.name == "Replace").unwrap();

    assert!(matches!(merge_ctx.command_mode, OverrideMode::Merge));
    assert!(matches!(merge_ctx.dictionary_mode, OverrideMode::Merge));
    assert!(matches!(replace_ctx.command_mode, OverrideMode::Replace));
    assert!(matches!(replace_ctx.dictionary_mode, OverrideMode::Replace));
}
