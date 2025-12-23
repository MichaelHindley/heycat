---
discovery_phase: paths
---

# Feature: Block Cancel Key Propagation

**Created:** 2025-12-23
**Owner:** Michael
**Discovery Phase:** not_started

## Description

When users cancel a hotkey recording using double-escape, the Escape key events currently propagate to other applications. This causes unintended side effects like closing dialogs or exiting modes in terminals/editors. This feature will consume the Escape key events during recording so they don't reach other applications.

## BDD Scenarios

### User Persona
Any user who records hotkeys while other applications are focused. This includes users working in terminals, IDEs, browsers, or any application where Escape has special meaning.

### Problem Statement
When cancelling a recording with double-escape, the Escape key presses also reach other applications, causing unintended actions (e.g., closing dialogs, exiting modes in terminals/editors). Users expect the cancel action to be non-destructive to their workflow.

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] [High-level criterion 1]
- [ ] [High-level criterion 2]

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated
