---
discovery_phase: complete
---

# Feature: Dataflow Refactor

**Created:** 2025-12-20
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Refactor the frontend data management architecture to use consistent, type-safe patterns across the entire application. This includes implementing Zustand for global state management, Tanstack Query for server state (wrapping Tauri commands), and React Router for navigation. The goal is to eliminate pattern sprawl and create a maintainable foundation for future feature development.

## BDD Scenarios

### User Persona
Developers maintaining the application who need clear, consistent patterns for working with state management and data flow across the React frontend and Rust backend.

### Problem Statement
The current codebase suffers from unmaintainable state management in frontend components, pattern sprawl between frontend and backend, and lack of type-safe operations across the React frontend and Rust backend. This creates friction when adding new features and increases the risk of bugs. Stability is needed as more features are built across the entire data flow of the app.

```gherkin
Feature: Dataflow Refactor

  # Happy Paths - Read Operations
  Scenario: Fetch data from Rust backend via query hook
    Given I am developing a React component
    When I use a Tanstack Query hook wrapping a Tauri command
    Then I receive typed data matching the TypeScript interface
    And the data is cached for subsequent renders

  Scenario: Use cached data with stale-while-revalidate
    Given data has been previously fetched and cached
    When I access the same data from another component
    Then I receive the cached data immediately
    And a background refetch updates the cache

  Scenario: Initial page load fetches required data
    Given I navigate to a route via React Router
    When the component mounts
    Then all required queries are triggered
    And loading states are displayed until data arrives

  # Happy Paths - Mutation Operations
  Scenario: Mutate data via mutation hook
    Given I have a form or action that modifies data
    When I trigger a mutation via Tanstack Mutation hook
    Then the Tauri command is invoked with typed parameters
    And related query caches are invalidated on success

  Scenario: Optimistic update during mutation
    Given I trigger a mutation on an entity
    When the mutation is in progress
    Then the UI reflects the expected change immediately
    And the change persists when the mutation succeeds

  Scenario: Rollback optimistic update on failure
    Given an optimistic update has been applied
    When the mutation fails
    Then the UI reverts to the previous state
    And an error notification is displayed

  # Happy Paths - Global State
  Scenario: Access global state via Zustand store
    Given I need app-wide state in a component
    When I use a Zustand selector hook
    Then I receive the current state value
    And the component re-renders when that state changes

  Scenario: Refresh data on user action
    Given data is displayed from a previous fetch
    When the user triggers a refresh action
    Then the query refetches from the Rust backend
    And the UI updates with the new data

  # Error Scenarios
  Scenario: Handle Tauri command failure
    Given I invoke a Tauri command via query or mutation
    When the Rust backend returns an error
    Then the error is caught by the query/mutation
    And a toast notification displays the error message
    And error boundaries prevent component crashes

  Scenario: Handle type mismatch from backend
    Given the Rust backend returns data
    When the data does not match the expected TypeScript type
    Then a runtime validation error is raised
    And the error is logged for debugging

  Scenario: Handle IPC timeout
    Given a Tauri command is invoked
    When the communication times out
    Then the query retries with exponential backoff
    And after max retries, an error is displayed

  Scenario: Recover from invalid store state
    Given the Zustand store is in an inconsistent state
    When an action detects the invalid state
    Then the store is reset to a known good state
    And the user is notified of the recovery
```

### Out of Scope
- UI/UX redesign - visual components stay the same, only data layer changes
- New features - focus on refactoring existing patterns, no new functionality
- Database schema changes - storage layer and persistence unchanged

### Assumptions
- Tauri v2 APIs are stable - current Tauri command patterns will continue to work
- Can add dependencies - free to add Zustand, Tanstack Query, React Router if not present
- Incremental migration - can migrate component by component, not all at once
- Tests will need updates - existing test wiring will likely change significantly with the new patterns

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Zustand store implemented for global app state
- [ ] Tanstack Query configured with Tauri command wrappers
- [ ] React Router set up for page navigation
- [ ] All existing Tauri command calls migrated to query/mutation hooks
- [ ] Type safety enforced between TypeScript and Rust boundaries
- [ ] Error handling patterns standardized (error boundaries, toasts, retries)

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated
