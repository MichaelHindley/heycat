use super::*;
use std::path::PathBuf;

fn create_test_worktree_context(identifier: &str) -> WorktreeContext {
    WorktreeContext {
        identifier: identifier.to_string(),
        gitdir_path: PathBuf::from("/tmp/test/.git/worktrees/test"),
    }
}

// ==================== Path Resolution Tests ====================

#[test]
fn test_get_data_dir_without_worktree_returns_standard_path() {
    let result = get_data_dir(None);
    assert!(result.is_ok());
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.ends_with("heycat") || path_str.ends_with("heycat/") || path_str.ends_with("heycat\\"),
        "Expected path to end with 'heycat', got: {}",
        path_str
    );
}

#[test]
fn test_get_data_dir_with_worktree_returns_worktree_specific_path() {
    let ctx = create_test_worktree_context("feature-branch");
    let result = get_data_dir(Some(&ctx));
    assert!(result.is_ok());
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.ends_with("heycat-feature-branch"),
        "Expected path to end with 'heycat-feature-branch', got: {}",
        path_str
    );
}

#[test]
fn test_get_config_dir_without_worktree_returns_standard_path() {
    let result = get_config_dir(None);
    assert!(result.is_ok());
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.ends_with("heycat") || path_str.ends_with("heycat/") || path_str.ends_with("heycat\\"),
        "Expected path to end with 'heycat', got: {}",
        path_str
    );
}

#[test]
fn test_get_config_dir_with_worktree_returns_worktree_specific_path() {
    let ctx = create_test_worktree_context("my-worktree");
    let result = get_config_dir(Some(&ctx));
    assert!(result.is_ok());
    let path = result.unwrap();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.ends_with("heycat-my-worktree"),
        "Expected path to end with 'heycat-my-worktree', got: {}",
        path_str
    );
}

#[test]
fn test_get_models_dir_without_worktree_returns_standard_path() {
    let result = get_models_dir(None);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(
        path.ends_with("heycat/models") || path.ends_with("heycat\\models"),
        "Expected path to end with 'heycat/models', got: {:?}",
        path
    );
}

#[test]
fn test_get_models_dir_with_worktree_returns_worktree_specific_path() {
    let ctx = create_test_worktree_context("test-wt");
    let result = get_models_dir(Some(&ctx));
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(
        path.ends_with("heycat-test-wt/models") || path.ends_with("heycat-test-wt\\models"),
        "Expected path to end with 'heycat-test-wt/models', got: {:?}",
        path
    );
}

#[test]
fn test_get_recordings_dir_without_worktree_returns_standard_path() {
    let result = get_recordings_dir(None);
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(
        path.ends_with("heycat/recordings") || path.ends_with("heycat\\recordings"),
        "Expected path to end with 'heycat/recordings', got: {:?}",
        path
    );
}

#[test]
fn test_get_recordings_dir_with_worktree_returns_worktree_specific_path() {
    let ctx = create_test_worktree_context("feature-xyz");
    let result = get_recordings_dir(Some(&ctx));
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(
        path.ends_with("heycat-feature-xyz/recordings") || path.ends_with("heycat-feature-xyz\\recordings"),
        "Expected path to end with 'heycat-feature-xyz/recordings', got: {:?}",
        path
    );
}

// ==================== Path Consistency Tests ====================

#[test]
fn test_path_resolution_is_consistent_across_calls() {
    let ctx = create_test_worktree_context("consistent-test");

    let path1 = get_data_dir(Some(&ctx)).unwrap();
    let path2 = get_data_dir(Some(&ctx)).unwrap();

    assert_eq!(path1, path2, "Same worktree context should produce same path");
}

#[test]
fn test_different_worktree_identifiers_produce_different_paths() {
    let ctx1 = create_test_worktree_context("feature-a");
    let ctx2 = create_test_worktree_context("feature-b");

    let path1 = get_data_dir(Some(&ctx1)).unwrap();
    let path2 = get_data_dir(Some(&ctx2)).unwrap();

    assert_ne!(path1, path2, "Different worktree contexts should produce different paths");
}

// ==================== Directory Creation Tests ====================

#[test]
fn test_ensure_dir_exists_creates_directory_if_missing() {
    let temp_dir = std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
    assert!(!temp_dir.exists());

    let result = ensure_dir_exists(&temp_dir);
    assert!(result.is_ok());
    assert!(temp_dir.exists());

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ensure_dir_exists_succeeds_if_already_exists() {
    let temp_dir = std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    assert!(temp_dir.exists());

    let result = ensure_dir_exists(&temp_dir);
    assert!(result.is_ok());
    assert!(temp_dir.exists());

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}
