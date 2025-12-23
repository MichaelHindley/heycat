# Feature: Full Type Safety Between Frontend and Backend

**Created:** 2025-12-23
**Owner:** michael
**Discovery Phase:** complete

## Description

The codebase has inconsistent serde configuration causing camelCase/snake_case mismatches between the Rust backend and TypeScript frontend. Some structs have `#[serde(rename_all = "camelCase")]` (listening, dictionary, model events) while others don't (recording, transcription, command events). The frontend has adapted to match snake_case in some places (e.g., `duration_ms` in eventBridge.ts). There's no compile-time type safety - types are manually duplicated.

This feature adds **tauri-specta v2** for automatic TypeScript generation from Rust types, providing compile-time type safety between frontend and backend.

## BDD Scenarios

### User Persona
Developer maintaining the heycat codebase

### Problem Statement
Type mismatches between frontend TypeScript and backend Rust cause runtime bugs that are only discovered through testing or production issues. Developers must manually keep types in sync across the IPC boundary.

```gherkin
Feature: Full Type Safety Between Frontend and Backend

  Scenario: Consistent JSON serialization
    Given a Rust struct that crosses the IPC boundary
    When it is serialized to JSON for the frontend
    Then all field names should be camelCase
    And the TypeScript types should match exactly

  Scenario: Compile-time type checking
    Given a TypeScript invoke() call to a Tauri command
    When the command signature changes in Rust
    Then TypeScript compilation should fail
    And the error should indicate the type mismatch

  Scenario: Event payload type safety
    Given a backend event emitted to the frontend
    When the frontend listens for that event
    Then the payload type should be generated from Rust
    And TypeScript should enforce the correct field names
```

### Out of Scope
- Runtime validation (Zod schemas)
- Changes to command semantics
- Performance optimization

### Assumptions
- tauri-specta v2 is compatible with our Tauri v2 setup
- Generated types can coexist with manual types during migration

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] All Rust types that cross IPC boundary use `#[serde(rename_all = "camelCase")]`
- [ ] tauri-specta generates TypeScript bindings from Rust types
- [ ] Frontend imports types from generated bindings.ts
- [ ] TypeScript compilation catches type mismatches
- [ ] Generated bindings are committed to the repo

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated
