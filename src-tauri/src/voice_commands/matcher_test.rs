use super::*;
use crate::voice_commands::registry::{ActionType, CommandDefinition};
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_registry() -> (CommandRegistry, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("commands.json");
    let registry = CommandRegistry::new(config_path);
    (registry, temp_dir)
}

fn create_command(trigger: &str) -> CommandDefinition {
    CommandDefinition {
        id: Uuid::new_v4(),
        trigger: trigger.to_string(),
        action_type: ActionType::OpenApp,
        parameters: HashMap::new(),
        enabled: true,
    }
}

#[test]
fn test_exact_match_open_slack() {
    let (mut registry, _temp) = create_test_registry();
    let cmd = create_command("open slack");
    registry.add(cmd.clone()).unwrap();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("open slack", &registry);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match, got {:?}", result),
    }
}

#[test]
fn test_fuzzy_match_typo() {
    let (mut registry, _temp) = create_test_registry();
    let cmd = create_command("open slack");
    registry.add(cmd.clone()).unwrap();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("opn slack", &registry);

    match result {
        MatchResult::Fuzzy { command, score, .. } => {
            assert_eq!(command.trigger, "open slack");
            assert!(score >= 0.8, "Score {} should be >= 0.8", score);
        }
        _ => panic!("Expected Fuzzy match, got {:?}", result),
    }
}

#[test]
fn test_case_insensitive_match() {
    let (mut registry, _temp) = create_test_registry();
    let cmd = create_command("open slack");
    registry.add(cmd.clone()).unwrap();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("OPEN SLACK", &registry);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match for case variation, got {:?}", result),
    }
}

#[test]
fn test_parameter_extraction() {
    let (mut registry, _temp) = create_test_registry();
    let mut cmd = create_command("type {text}");
    cmd.action_type = ActionType::TypeText;
    registry.add(cmd.clone()).unwrap();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("type hello world", &registry);

    match result {
        MatchResult::Exact { parameters, .. } => {
            assert_eq!(parameters.get("text"), Some(&"hello world".to_string()));
        }
        _ => panic!("Expected Exact match with parameters, got {:?}", result),
    }
}

#[test]
fn test_no_match_different_text() {
    let (mut registry, _temp) = create_test_registry();
    let cmd = create_command("open slack");
    registry.add(cmd.clone()).unwrap();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("xyz abc", &registry);

    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_ambiguous_similar_commands() {
    let (mut registry, _temp) = create_test_registry();
    // Use commands that are very similar to each other
    let cmd1 = create_command("open slack");
    let cmd2 = create_command("open slick");
    registry.add(cmd1).unwrap();
    registry.add(cmd2).unwrap();

    // Configure matcher with higher ambiguity delta to make the test more reliable
    let config = MatcherConfig {
        threshold: 0.7,
        ambiguity_delta: 0.15,
    };
    let matcher = CommandMatcher::with_config(config);
    // Input that's similar to both: "slaick" is between "slack" and "slick"
    let result = matcher.match_input("open slaik", &registry);

    match result {
        MatchResult::Ambiguous { candidates } => {
            assert!(candidates.len() >= 2, "Expected at least 2 ambiguous candidates");
        }
        _ => panic!("Expected Ambiguous result, got {:?}", result),
    }
}

#[test]
fn test_whitespace_normalization() {
    let (mut registry, _temp) = create_test_registry();
    let cmd = create_command("open slack");
    registry.add(cmd.clone()).unwrap();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("  open slack  ", &registry);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match with trimmed input, got {:?}", result),
    }
}

#[test]
fn test_disabled_command_not_matched() {
    let (mut registry, _temp) = create_test_registry();
    let mut cmd = create_command("open slack");
    cmd.enabled = false;
    registry.add(cmd.clone()).unwrap();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("open slack", &registry);

    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_empty_registry_returns_no_match() {
    let (registry, _temp) = create_test_registry();

    let matcher = CommandMatcher::new();
    let result = matcher.match_input("open slack", &registry);

    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_custom_threshold() {
    let (mut registry, _temp) = create_test_registry();
    let cmd = create_command("open slack");
    registry.add(cmd.clone()).unwrap();

    // Set a very high threshold that won't match fuzzy
    let config = MatcherConfig {
        threshold: 0.99,
        ambiguity_delta: 0.1,
    };
    let matcher = CommandMatcher::with_config(config);
    let result = matcher.match_input("opn slack", &registry);

    // With high threshold, fuzzy match shouldn't work
    assert!(matches!(result, MatchResult::NoMatch));
}

#[test]
fn test_match_result_serialization() {
    let result = MatchResult::Exact {
        command: MatchedCommand {
            id: Uuid::new_v4(),
            trigger: "open slack".to_string(),
        },
        parameters: HashMap::new(),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Exact"));
    assert!(json.contains("open slack"));
}

#[test]
fn test_best_match_selected_when_not_ambiguous() {
    let (mut registry, _temp) = create_test_registry();
    let cmd1 = create_command("open slack");
    let cmd2 = create_command("open zoom");
    registry.add(cmd1).unwrap();
    registry.add(cmd2).unwrap();

    let matcher = CommandMatcher::new();
    // "open slack" should match exactly, not be ambiguous with "open zoom"
    let result = matcher.match_input("open slack", &registry);

    match result {
        MatchResult::Exact { command, .. } => {
            assert_eq!(command.trigger, "open slack");
        }
        _ => panic!("Expected Exact match, got {:?}", result),
    }
}
