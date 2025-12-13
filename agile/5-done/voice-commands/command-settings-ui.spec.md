---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - command-registry
---

# Spec: Command Settings UI

## Description

React component for managing voice commands. Provides a visual interface to view, add, edit, and remove commands without manually editing JSON files. Integrates with the existing Sidebar as a new "Commands" tab.

## Acceptance Criteria

- [ ] Display list of all registered commands with trigger, action type, and enabled status
- [ ] Add new command with trigger phrase, action type selector, and parameter fields
- [ ] Edit existing commands (inline or modal)
- [ ] Delete commands with confirmation dialog
- [ ] Toggle command enabled/disabled status
- [ ] Form validation (unique triggers, required fields, valid parameters)
- [ ] Persist changes via Tauri commands (get_commands, add_command, remove_command)
- [ ] Empty state shown when no commands exist

## Test Cases

- [ ] Empty state displays helpful message when no commands
- [ ] Adding command shows it in the list immediately
- [ ] Editing command trigger updates the list
- [ ] Deleting command removes it after confirmation
- [ ] Duplicate trigger shows validation error
- [ ] Required fields prevent submission when empty
- [ ] Parameter fields change based on action type selection

## Dependencies

- command-registry (provides Tauri IPC commands)

## Preconditions

- Tauri commands exposed: get_commands, add_command, remove_command
- Sidebar component exists with tab system

## Implementation Notes

- Location: `src/components/CommandSettings/`
  - `CommandList.tsx` - List view with enable/disable toggles
  - `CommandEditor.tsx` - Form for add/edit with validation
  - `CommandSettings.tsx` - Container component
  - `CommandSettings.css` - Styling

- Tauri commands available:
  - `get_commands()` → `CommandDefinition[]`
  - `add_command(trigger, action_type, parameters, enabled)` → `CommandDefinition`
  - `remove_command(id)` → void

- Action types to support:
  - `open_app` → parameter: `app` (text input)
  - `type_text` → parameter: `text` (text input), `delay_ms` (number)
  - `system_control` → parameter: `control` (dropdown)
  - `workflow` → parameter: `workflow` (text)
  - `custom` → parameter: `script` (textarea)

## Related Specs

- command-registry.spec.md (backend persistence)
- disambiguation-ui.spec.md (similar React patterns)

## Integration Points

- Production call site: `src/components/Sidebar/Sidebar.tsx` (add Commands tab)
- Connects to: voice_commands Tauri module, existing Sidebar tab system

## Integration Test

- Test location: `src/components/CommandSettings/CommandSettings.test.tsx`
- Verification: [ ] Integration test passes
