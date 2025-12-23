---
status: pending
created: 2025-12-23
completed: null
dependencies: []
---

# Spec: Add tauri-specta dependencies to Cargo.toml

## Description

Add the tauri-specta and specta crates to enable TypeScript type generation from Rust types. This is the foundation spec that enables all subsequent type safety work.

## Acceptance Criteria

- [ ] specta dependency added with derive feature
- [ ] tauri-specta dependency added with derive and typescript features
- [ ] `cargo build` succeeds with new dependencies
- [ ] No conflicts with existing dependencies

## Test Cases

- [ ] `cargo check` passes after adding dependencies
- [ ] `cargo build --release` passes

## Dependencies

None (this is the foundation spec)

## Preconditions

- Cargo.toml is editable
- Internet access for crate download

## Implementation Notes

Add to `src-tauri/Cargo.toml`:

```toml
specta = { version = "=2.0.0-rc.20", features = ["derive"] }
tauri-specta = { version = "=2.0.0-rc.20", features = ["derive", "typescript"] }
```

Note: Using exact versions with `=` to ensure compatibility since specta is in pre-release.

## Related Specs

- type-annotations.spec.md - depends on this spec
- bindings-generation.spec.md - depends on this spec
- frontend-migration.spec.md - depends on this spec

## Integration Points

- Production call site: `src-tauri/Cargo.toml` (dependency declaration)
- Connects to: All Rust modules that will use specta attributes

## Integration Test

- Test location: N/A (dependency-only spec)
- Verification: [x] N/A - verified by cargo build
