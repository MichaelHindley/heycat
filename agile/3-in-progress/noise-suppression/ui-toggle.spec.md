---
status: in-progress
created: 2025-12-23
completed: null
dependencies: [pipeline-integration]
---

# Spec: Add settings UI toggle to enable/disable noise suppression

## Description

Add a toggle switch in the Audio Settings panel to let users enable or disable noise suppression. Currently noise suppression is always-on; this gives users control when they prefer raw audio (e.g., quiet environments, external processing, or debugging).

The toggle will:
1. Add `noiseSuppression: boolean` to `AudioSettings` type (default: `true`)
2. Display a Toggle component in the Audio Input section of AudioSettings
3. Persist the setting via `useSettings` hook (Zustand + Tauri Store)
4. Backend reads setting and bypasses denoiser when disabled

## Acceptance Criteria

- [ ] Toggle appears in Audio Settings under "Audio Input" section
- [ ] Toggle defaults to ON (enabled) for new installations
- [ ] Toggle state persists across app restarts
- [ ] Backend respects the setting (denoiser bypassed when OFF)
- [ ] Toast notification confirms setting change

## Test Cases

- [ ] `AudioSettings.test.tsx`: Toggle renders and is checked by default
- [ ] `AudioSettings.test.tsx`: Clicking toggle updates settings state
- [ ] `useSettings.test.ts`: `updateNoiseSuppression(false)` persists to store
- [ ] Backend: `cargo test` - audio capture respects `noise_suppression` setting

## Dependencies

- `pipeline-integration` (denoiser must be integrated before we can toggle it)

## Preconditions

- Noise suppression pipeline is working (denoiser integrated into cpal_backend)
- Toggle component exists in `src/components/ui/Toggle.tsx`

## Implementation Notes

### Frontend Changes

1. **`src/types/audio.ts`** - Add field to `AudioSettings`:
   ```typescript
   noiseSuppression: boolean;
   ```

2. **`src/hooks/useSettings.ts`**:
   - Add `updateNoiseSuppression` method
   - Load `audio.noiseSuppression` in `initializeSettings()`

3. **`src/pages/components/AudioSettings.tsx`**:
   - Import `Toggle` component
   - Add toggle row below Audio Level Meter
   - Call `updateNoiseSuppression` on change

### Backend Changes

4. **`src-tauri/src/audio/cpal_backend.rs`**:
   - Read `audio.noiseSuppression` from Tauri Store on capture start
   - Skip denoiser processing when setting is `false`

## Related Specs

- [pipeline-integration.spec.md](./pipeline-integration.spec.md) - Denoiser integration

## Integration Points

- Production call site: `src/pages/components/AudioSettings.tsx` (Toggle component)
- Connects to: `useSettings` hook, Tauri Store, `cpal_backend.rs`

## Integration Test

- Test location: `src/pages/components/AudioSettings.test.tsx`
- Verification: [ ] Integration test passes
