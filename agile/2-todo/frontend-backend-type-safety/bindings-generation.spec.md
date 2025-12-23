---
status: pending
created: 2025-12-23
completed: null
dependencies: ["type-annotations"]
---

# Spec: Create specta module and generate TypeScript bindings

## Description

Create a new `specta.rs` module that configures the tauri-specta builder with all commands and types, then generates TypeScript bindings to `src/lib/bindings.ts`. The bindings will be committed to the repo for CI and reviewability.

## Acceptance Criteria

- [ ] `src-tauri/src/specta.rs` module created with builder configuration
- [ ] `src-tauri/src/lib.rs` updated to include specta module
- [ ] TypeScript bindings generated at `src/lib/bindings.ts`
- [ ] Generated bindings export all command functions
- [ ] Generated bindings export all event payload types
- [ ] `cargo build` succeeds
- [ ] TypeScript compiles successfully

## Test Cases

- [ ] Bindings file is generated and non-empty
- [ ] All 33 commands are exported
- [ ] Event payload types have camelCase fields
- [ ] `tsc --noEmit` passes on generated bindings

## Dependencies

- type-annotations.spec.md (all types must have `#[derive(Type)]`)

## Preconditions

- All types and commands are annotated with specta attributes

## Implementation Notes

### Create `src-tauri/src/specta.rs`

```rust
use tauri_specta::{collect_commands, collect_types, Builder};

/// Build the specta TypeScript bindings
pub fn builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            // Recording commands
            crate::commands::start_recording,
            crate::commands::stop_recording,
            crate::commands::get_recording_state,
            crate::commands::list_recordings,
            crate::commands::delete_recording,
            // ... all other commands
        ])
        .types(collect_types![
            // Event payloads
            crate::events::RecordingStartedPayload,
            crate::events::TranscriptionCompletedPayload,
            // ... all other event types
        ])
}

/// Export TypeScript bindings
#[cfg(debug_assertions)]
pub fn export_typescript() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("../src/lib/bindings.ts");
    builder().export(tauri_specta::ts::ExportConfig::default(), path)?;
    Ok(())
}
```

### Update `src-tauri/src/lib.rs`

Add module declaration:
```rust
mod specta;
```

### Generate Bindings

Run export during development:
```bash
cargo test export_bindings --release
```

Or create a test to run it:
```rust
#[test]
fn export_bindings() {
    heycat_lib::specta::export_typescript()
        .expect("Failed to export TypeScript bindings");
}
```

## Related Specs

- type-annotations.spec.md (dependency)
- frontend-migration.spec.md (depends on this spec)

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (module declaration)
- Connects to: Frontend via generated `src/lib/bindings.ts`

## Integration Test

- Test location: `src-tauri/tests/export_bindings.rs`
- Verification: [ ] Integration test passes
