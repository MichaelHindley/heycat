use crate::turso::{initialize_schema, TursoClient};
use crate::voice_commands::registry::{ActionType, CommandDefinition, RegistryError};
use std::collections::HashMap;
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

fn make_command(trigger: &str, action_type: ActionType) -> CommandDefinition {
    CommandDefinition {
        id: Uuid::new_v4(),
        trigger: trigger.to_string(),
        action_type,
        parameters: HashMap::new(),
        enabled: true,
    }
}

#[tokio::test]
async fn test_add_voice_command() {
    let (client, _temp) = setup_client().await;

    let cmd = make_command("open slack", ActionType::OpenApp);
    client
        .add_voice_command(&cmd)
        .await
        .expect("Failed to add command");

    let commands = client.list_voice_commands().await.expect("Failed to list");
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].id, cmd.id);
    assert_eq!(commands[0].trigger, "open slack");
    assert!(matches!(commands[0].action_type, ActionType::OpenApp));
    assert!(commands[0].enabled);
}

#[tokio::test]
async fn test_add_voice_command_with_parameters() {
    let (client, _temp) = setup_client().await;

    let mut params = HashMap::new();
    params.insert("app_name".to_string(), "Slack".to_string());
    params.insert("path".to_string(), "/Applications/Slack.app".to_string());

    let cmd = CommandDefinition {
        id: Uuid::new_v4(),
        trigger: "launch slack".to_string(),
        action_type: ActionType::OpenApp,
        parameters: params,
        enabled: true,
    };

    client
        .add_voice_command(&cmd)
        .await
        .expect("Failed to add command");

    let commands = client.list_voice_commands().await.expect("Failed to list");
    assert_eq!(commands[0].parameters.get("app_name"), Some(&"Slack".to_string()));
    assert_eq!(commands[0].parameters.get("path"), Some(&"/Applications/Slack.app".to_string()));
}

#[tokio::test]
async fn test_add_voice_command_empty_trigger_fails() {
    let (client, _temp) = setup_client().await;

    let cmd = make_command("  ", ActionType::Custom);
    let result = client.add_voice_command(&cmd).await;

    assert!(matches!(result.err().unwrap(), RegistryError::EmptyTrigger));
}

#[tokio::test]
async fn test_list_voice_commands_empty() {
    let (client, _temp) = setup_client().await;

    let commands = client.list_voice_commands().await.expect("Failed to list");
    assert!(commands.is_empty());
}

#[tokio::test]
async fn test_list_voice_commands_ordered() {
    let (client, _temp) = setup_client().await;

    let cmd1 = make_command("first command", ActionType::Custom);
    let cmd2 = make_command("second command", ActionType::TypeText);
    let cmd3 = make_command("third command", ActionType::SystemControl);

    client.add_voice_command(&cmd1).await.expect("Failed to add 1");
    client.add_voice_command(&cmd2).await.expect("Failed to add 2");
    client.add_voice_command(&cmd3).await.expect("Failed to add 3");

    let commands = client.list_voice_commands().await.expect("Failed to list");
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].trigger, "first command");
    assert_eq!(commands[1].trigger, "second command");
    assert_eq!(commands[2].trigger, "third command");
}

#[tokio::test]
async fn test_update_voice_command() {
    let (client, _temp) = setup_client().await;

    let cmd = make_command("old trigger", ActionType::Custom);
    client.add_voice_command(&cmd).await.expect("Failed to add");

    let mut updated = cmd.clone();
    updated.trigger = "new trigger".to_string();
    updated.action_type = ActionType::TypeText;
    updated.enabled = false;
    updated.parameters.insert("text".to_string(), "hello".to_string());

    client
        .update_voice_command(&updated)
        .await
        .expect("Failed to update");

    let commands = client.list_voice_commands().await.expect("Failed to list");
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].trigger, "new trigger");
    assert!(matches!(commands[0].action_type, ActionType::TypeText));
    assert!(!commands[0].enabled);
    assert_eq!(commands[0].parameters.get("text"), Some(&"hello".to_string()));
}

#[tokio::test]
async fn test_update_voice_command_not_found() {
    let (client, _temp) = setup_client().await;

    let cmd = make_command("test", ActionType::Custom);
    let result = client.update_voice_command(&cmd).await;

    match result.err().unwrap() {
        RegistryError::NotFound(id) => assert_eq!(id, cmd.id),
        other => panic!("Expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_update_voice_command_empty_trigger_fails() {
    let (client, _temp) = setup_client().await;

    let cmd = make_command("original", ActionType::Custom);
    client.add_voice_command(&cmd).await.expect("Failed to add");

    let mut updated = cmd.clone();
    updated.trigger = "   ".to_string();

    let result = client.update_voice_command(&updated).await;
    assert!(matches!(result.err().unwrap(), RegistryError::EmptyTrigger));
}

#[tokio::test]
async fn test_update_voice_command_trigger_conflict() {
    let (client, _temp) = setup_client().await;

    let cmd1 = make_command("trigger one", ActionType::Custom);
    let cmd2 = make_command("trigger two", ActionType::Custom);

    client.add_voice_command(&cmd1).await.expect("Failed to add 1");
    client.add_voice_command(&cmd2).await.expect("Failed to add 2");

    // Try to update cmd1 to use cmd2's trigger
    let mut updated = cmd1.clone();
    updated.trigger = "trigger two".to_string();

    let result = client.update_voice_command(&updated).await;
    assert!(result.is_err());
    match result.err().unwrap() {
        RegistryError::PersistenceError(msg) => {
            assert!(msg.contains("already exists"));
        }
        other => panic!("Expected PersistenceError, got {:?}", other),
    }
}

#[tokio::test]
async fn test_delete_voice_command() {
    let (client, _temp) = setup_client().await;

    let cmd = make_command("to delete", ActionType::Custom);
    client.add_voice_command(&cmd).await.expect("Failed to add");

    client
        .delete_voice_command(cmd.id)
        .await
        .expect("Failed to delete");

    let commands = client.list_voice_commands().await.expect("Failed to list");
    assert!(commands.is_empty());
}

#[tokio::test]
async fn test_delete_voice_command_not_found() {
    let (client, _temp) = setup_client().await;

    let id = Uuid::new_v4();
    let result = client.delete_voice_command(id).await;

    match result.err().unwrap() {
        RegistryError::NotFound(found_id) => assert_eq!(found_id, id),
        other => panic!("Expected NotFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_action_type_roundtrip() {
    let (client, _temp) = setup_client().await;

    // Test all action types serialize/deserialize correctly
    let action_types = [
        ActionType::OpenApp,
        ActionType::TypeText,
        ActionType::SystemControl,
        ActionType::Custom,
    ];

    for action_type in action_types {
        let cmd = CommandDefinition {
            id: Uuid::new_v4(),
            trigger: format!("test {:?}", action_type),
            action_type: action_type.clone(),
            parameters: HashMap::new(),
            enabled: true,
        };
        client.add_voice_command(&cmd).await.expect("Failed to add");
    }

    let commands = client.list_voice_commands().await.expect("Failed to list");
    assert_eq!(commands.len(), 4);
    assert!(matches!(commands[0].action_type, ActionType::OpenApp));
    assert!(matches!(commands[1].action_type, ActionType::TypeText));
    assert!(matches!(commands[2].action_type, ActionType::SystemControl));
    assert!(matches!(commands[3].action_type, ActionType::Custom));
}
