---
status: in-progress
created: 2025-12-19
completed: null
dependencies: []
review_round: 1
review_history:
  - round: 1
    date: 2025-12-19
    verdict: NEEDS_WORK
    failedCriteria: ["Prompt user with clear guidance to enable Accessibility in System Settings", "Return appropriate error when permission not granted"]
    concerns: ["**Critical:** All code has dead_code warnings - nothing is wired up to production", "**Critical:** Duplicate implementation exists in text_input.rs instead of using this module", "Missing test coverage for open_accessibility_settings()", "The spec claims \"N/A (standalone module, will be integrated in replace-iokit-hid spec)\" but this violates the review requirement that code must be wired up end-to-end", "Module is exported but never imported or used anywhere"]
---

# Spec: Accessibility permission handling

## Description

Implement Accessibility permission checking and prompting for macOS. CGEventTap requires Accessibility permission (not Input Monitoring). This spec adds functions to check if permission is granted and guide the user to enable it.

## Acceptance Criteria

- [ ] Check permission with AXIsProcessTrusted() FFI binding
- [ ] Prompt user with clear guidance to enable Accessibility in System Settings
- [ ] Open System Settings to correct pane (Privacy & Security > Accessibility)
- [ ] Return appropriate error when permission not granted

## Test Cases

- [ ] When Accessibility is enabled, permission check returns true
- [ ] When Accessibility is disabled, permission check returns false
- [ ] System Settings opens to correct pane when requested

## Dependencies

None - this is foundational

## Preconditions

- macOS 10.15+ (Catalina or later)
- ApplicationServices framework available

## Implementation Notes

FFI bindings needed:
```rust
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}
```

To open System Settings:
```rust
// Open Accessibility pane
open("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
```

Note: Unlike Input Monitoring, Accessibility permission doesn't have a programmatic "request" API - the user must manually enable it.

File location: `src-tauri/src/keyboard_capture/permissions.rs` (new file)

## Related Specs

- cgeventtap-core.spec.md - uses permission check
- replace-iokit-hid.spec.md - integration

## Integration Points

- Production call site: N/A (standalone module, will be integrated in replace-iokit-hid spec)
- Connects to: ApplicationServices framework

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A

## Review

**Reviewed:** 2025-12-19
**Reviewer:** Claude

### Pre-Review Gates

#### 1. Build Warning Check
```
warning: function `check_accessibility_permission` is never used
warning: function `open_accessibility_settings` is never used
warning: struct `AccessibilityPermissionError` is never constructed
warning: associated function `new` is never used
warning: function `AXIsProcessTrusted` is never used
```
**STATUS: FAIL** - All new code has dead_code warnings indicating it's not wired up.

#### 2. Command Registration Check
**STATUS: N/A** - Spec does not add Tauri commands.

#### 3. Event Subscription Check
**STATUS: N/A** - Spec does not add events.

### Manual Review

#### 1. Is the code wired up end-to-end?

| Item | Status | Issue |
|------|--------|-------|
| New functions called from production code | FAIL | All functions only called in tests |
| New structs instantiated in production code | FAIL | AccessibilityPermissionError never constructed outside tests |
| Module integrated | PARTIAL | Module declared in mod.rs but exports never imported |

**Finding:** The permissions module is declared in `src-tauri/src/keyboard_capture/mod.rs:5` with `pub mod permissions;` but none of its exports are used anywhere in production code. The only usage found is in test files (`text_input_test.rs`).

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| check_accessibility_permission | fn | NONE | TEST-ONLY |
| open_accessibility_settings | fn | NONE | NO |
| AccessibilityPermissionError | struct | NONE | NO |
| AXIsProcessTrusted | fn | NONE | NO |

**STATUS: NEEDS_WORK** - All code is either TEST-ONLY or unreachable from production.

**Note:** Found duplicate implementation in `src-tauri/src/voice_commands/actions/text_input.rs:19` which defines its own `check_accessibility_permission()` FFI binding, indicating this spec's code is not integrated.

#### 3. Where does the data flow?

**STATUS: N/A** - No frontend-backend interaction in this spec. This is a backend-only module spec.

However, the spec states "Integration Points: Production call site: N/A (standalone module, will be integrated in replace-iokit-hid spec)" which acknowledges this is foundational code.

#### 4. Are there any deferrals?

No TODO/FIXME/HACK comments found in the new permissions.rs file.

**STATUS: PASS**

#### 5. Automated check results

See Pre-Review Gates section above.

#### 6. Frontend-Only Integration Check

**STATUS: N/A** - This is a backend-only spec.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Check permission with AXIsProcessTrusted() FFI binding | PASS | permissions.rs:12 - FFI binding defined and used in check_accessibility_permission() |
| Prompt user with clear guidance to enable Accessibility in System Settings | FAIL | AccessibilityPermissionError created with guidance message but never constructed in production code |
| Open System Settings to correct pane (Privacy & Security > Accessibility) | PASS | permissions.rs:31-40 - open_accessibility_settings() implemented with correct URL |
| Return appropriate error when permission not granted | FAIL | Error type exists but never used in production code |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| When Accessibility is enabled, permission check returns true | PASS | permissions.rs:75 - test_check_accessibility_permission_returns_bool |
| When Accessibility is disabled, permission check returns false | PASS | permissions.rs:75 - test_check_accessibility_permission_returns_bool (tests both states) |
| System Settings opens to correct pane when requested | MISSING | No test for open_accessibility_settings() |

**Note:** The test for opening System Settings is missing. While this is hard to test automatically, a manual verification or integration test should be documented.

### Code Quality

**Strengths:**
- Clean FFI bindings with proper safety documentation
- Clear error messages for users
- Appropriate use of Result types
- Good code comments explaining macOS-specific behavior
- Tests verify basic functionality

**Concerns:**
- **Critical:** All code has dead_code warnings - nothing is wired up to production
- **Critical:** Duplicate implementation exists in text_input.rs instead of using this module
- Missing test coverage for open_accessibility_settings()
- The spec claims "N/A (standalone module, will be integrated in replace-iokit-hid spec)" but this violates the review requirement that code must be wired up end-to-end
- Module is exported but never imported or used anywhere

### Verdict

**NEEDS_WORK** - Code exists but is completely orphaned with no production call sites.
