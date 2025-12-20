---
status: in-progress
created: 2025-12-20
completed: null
dependencies: []
review_round: 1
review_history:
  - round: 1
    date: 2025-12-20
    verdict: NEEDS_WORK
    failedCriteria: []
    concerns: ["**CRITICAL**: Store is not wired up to production code - no components use it", "Store exports are only imported in test file (appStore.test.ts)", "No production call sites found in src/*.tsx or src/*.ts files", "This is orphaned code with no integration into the application"]
---

# Spec: Create Zustand store for client state

## Description

Install Zustand and create the app store for managing global client-side state. This store holds UI state (overlay visibility, app status) and cached settings. Following the separation of concerns principle: NO server state in this store - that belongs in Tanstack Query.

## Acceptance Criteria

- [ ] `zustand` package installed and in package.json
- [ ] `src/stores/appStore.ts` created with typed store
- [ ] Store contains client state slices:
  - `overlayMode`: string | null (current overlay state)
  - `settingsCache`: Settings object (cached from Tauri Store)
  - `isSettingsLoaded`: boolean (hydration flag)
- [ ] Store actions defined:
  - `setOverlayMode(mode: string | null)`
  - `setSettings(settings: Settings)`
  - `updateSetting(key: string, value: any)`
- [ ] Selectors exported for optimized re-renders:
  - `useOverlayMode()` - returns only overlayMode
  - `useSettingsCache()` - returns only settings
- [ ] TypeScript types are strict with proper inference
- [ ] Store does NOT contain server state (recordings, recording state, etc.)
- [ ] No persist middleware (settings persist via Tauri Store, not localStorage)

## Test Cases

- [ ] Store initializes with default values
- [ ] `setOverlayMode('recording')` updates state correctly
- [ ] `setSettings(mockSettings)` updates settingsCache
- [ ] Selectors return only their slice (not full store)
- [ ] Multiple components using same selector share state

## Dependencies

None - this is a foundational spec.

## Preconditions

- Existing React + TypeScript project structure

## Implementation Notes

```typescript
// src/stores/appStore.ts
import { create } from 'zustand';

interface Settings {
  listening: { enabled: boolean; autoStartOnLaunch: boolean };
  audio: { selectedDevice: string | null };
  shortcuts: { distinguishLeftRight: boolean };
}

interface AppState {
  // Client state only - NO server state here
  overlayMode: string | null;
  settingsCache: Settings | null;
  isSettingsLoaded: boolean;

  // Actions
  setOverlayMode: (mode: string | null) => void;
  setSettings: (settings: Settings) => void;
  updateSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

export const useAppStore = create<AppState>((set) => ({
  overlayMode: null,
  settingsCache: null,
  isSettingsLoaded: false,

  setOverlayMode: (mode) => set({ overlayMode: mode }),
  setSettings: (settings) => set({ settingsCache: settings, isSettingsLoaded: true }),
  updateSetting: (key, value) => set((state) => ({
    settingsCache: state.settingsCache
      ? { ...state.settingsCache, [key]: value }
      : null
  })),
}));

// Optimized selectors
export const useOverlayMode = () => useAppStore((s) => s.overlayMode);
export const useSettingsCache = () => useAppStore((s) => s.settingsCache);
export const useIsSettingsLoaded = () => useAppStore((s) => s.isSettingsLoaded);
```

## Related Specs

- `event-bridge` - Updates store on UI state events
- `settings-zustand-hooks` - Uses store for settings access
- `app-providers-wiring` - Initializes settings into store

## Integration Points

- Production call site: Components via `useAppStore()` hook
- Connects to: event-bridge (receives updates), settings-zustand-hooks (primary consumer)

## Integration Test

- Test location: `src/stores/__tests__/appStore.test.ts`
- Verification: [ ] Unit tests pass for store actions and selectors

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `zustand` package installed and in package.json | PASS | package.json:34 |
| `src/stores/appStore.ts` created with typed store | PASS | appStore.ts:30 - useAppStore with AppState interface |
| Store contains overlayMode slice | PASS | appStore.ts:17 |
| Store contains settingsCache slice | PASS | appStore.ts:18 |
| Store contains isSettingsLoaded slice | PASS | appStore.ts:19 |
| Store action: setOverlayMode | PASS | appStore.ts:35 |
| Store action: setSettings | PASS | appStore.ts:37-38 |
| Store action: updateSetting | PASS | appStore.ts:40-45 |
| Selector: useOverlayMode | PASS | appStore.ts:50 |
| Selector: useSettingsCache | PASS | appStore.ts:51 |
| Selector: useIsSettingsLoaded | PASS | appStore.ts:52 |
| TypeScript types are strict with proper inference | PASS | AppState interface with generic updateSetting method |
| Store does NOT contain server state | PASS | Only client state (overlay, settings cache, hydration flag) |
| No persist middleware | PASS | No persistence configuration present |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Store initializes with default values | PASS | appStore.test.ts:18-24 (beforeEach) |
| setOverlayMode('recording') updates state correctly | PASS | appStore.test.ts:28-36 |
| setSettings(mockSettings) updates settingsCache | PASS | appStore.test.ts:51-60 |
| Selectors return only their slice | PASS | appStore.test.ts:91-112 |
| Multiple components using same selector share state | PASS | Implicit - Zustand's built-in behavior |

### Code Quality

**Strengths:**
- Clear separation of concerns: client state only, no server state
- Comprehensive JSDoc documentation explaining purpose and constraints
- Type-safe implementation with proper TypeScript generics
- Optimized selectors to prevent unnecessary re-renders
- Complete test coverage with all actions and selectors tested
- Handles edge cases (updateSetting with null cache)

**Concerns:**
- **CRITICAL**: Store is not wired up to production code - no components use it
- Store exports are only imported in test file (appStore.test.ts)
- No production call sites found in src/*.tsx or src/*.ts files
- This is orphaned code with no integration into the application

### Integration Analysis

**Production Usage Check:**
```bash
grep -rn "useAppStore|useOverlayMode|useSettingsCache|useIsSettingsLoaded" src/ --include="*.tsx" --include="*.ts"
```
Result: Only found in appStore.ts (definition) and appStore.test.ts (tests)

**What would break if this code was deleted?**

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| useAppStore | hook | NONE | TEST-ONLY |
| useOverlayMode | hook | NONE | TEST-ONLY |
| useSettingsCache | hook | NONE | TEST-ONLY |
| useIsSettingsLoaded | hook | NONE | TEST-ONLY |

**Verdict: All new code is TEST-ONLY with no production integration.**

### Automated Checks

**Build Warnings:** No warnings found
**Deferrals:** No TODO/FIXME/HACK comments found
**Tests:** All 8 tests pass

### Verdict

**NEEDS_WORK** - Store is not integrated into production code. The implementation is correct but the store is orphaned - no components or hooks use it. This violates review question #1: "Is the code wired up end-to-end?" and question #2: "What would break if this code was deleted?" (Answer: Nothing - it's TEST-ONLY code).

**Required fixes:**
1. According to spec Related Specs section, this should be used by:
   - `event-bridge` - Updates store on UI state events
   - `settings-zustand-hooks` - Uses store for settings access
   - `app-providers-wiring` - Initializes settings into store
2. At minimum, one production component/hook must import and use the store
3. Re-run review after integration is complete
