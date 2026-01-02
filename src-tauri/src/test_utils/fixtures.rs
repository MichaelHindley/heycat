//! Test data fixtures and helpers.

use crate::model::{ModelManifest, ModelType};
use std::path::PathBuf;
use std::sync::Once;

/// Global lock for model directory operations to prevent test races.
pub static MODEL_DIR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Ensure model files exist for tests (called once per test run).
static INIT_MODEL_FILES: Once = Once::new();

/// Get the path to models directory in the git repo (for tests).
pub fn get_test_models_dir(model_type: ModelType) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("Failed to get parent of manifest dir")
        .join("models")
        .join(model_type.dir_name())
}

/// Ensure model files exist in repo - fails if not present.
pub fn ensure_test_model_files() {
    INIT_MODEL_FILES.call_once(|| {
        let _lock = MODEL_DIR_LOCK.lock().unwrap();

        // Verify TDT model exists in repo
        let tdt_model_dir = get_test_models_dir(ModelType::ParakeetTDT);
        let tdt_manifest = ModelManifest::tdt();
        for file in &tdt_manifest.files {
            let file_path = tdt_model_dir.join(&file.name);
            assert!(
                file_path.exists(),
                "TDT model file missing from repo: {:?}. Run 'git lfs pull'.",
                file_path
            );
        }
    });
}
