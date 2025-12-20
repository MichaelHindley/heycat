# Bug: Listening Settings Not Persisting

**Created:** 2025-12-20
**Owner:** Michael
**Severity:** major

## Description

The "Auto Start Listening" setting is not being respected on app startup. Even when the setting is turned off, the app still starts with listening enabled.

## Steps to Reproduce

1. Open the app and navigate to Settings → General Settings
2. Turn off "Auto Start Listening"
3. Close the app completely
4. Reopen the app

## Expected Behavior

The app should start with listening disabled (since "Auto Start Listening" was turned off).

## Actual Behavior

The app starts with listening enabled, ignoring the saved "Auto Start Listening" preference.

## Environment

- OS: macOS
- App Version: Current development build

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Root cause documented
