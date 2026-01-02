---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Frontend Tauri Integration

## Hooks-Only Enforcement

All `invoke()` calls must be in custom hooks, never in components directly. This centralizes backend communication and enables:
- Consistent error handling
- Cache management via Tanstack Query
- Type safety through typed hooks

```typescript
// GOOD: invoke in hook
export function useAudioDevices(): UseAudioDevicesResult {
  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.listAudioDevices,
    queryFn: () => invoke<AudioInputDevice[]>("list_audio_devices"),
  });
  // ...
}

// BAD: invoke in component
function DeviceSelector() {
  const [devices, setDevices] = useState([]);
  useEffect(() => {
    invoke("list_audio_devices").then(setDevices);  // Don't do this
  }, []);
}
```

## Typed invoke<T>() Usage

Always specify the return type generic when calling `invoke`:

```typescript
import { invoke } from "@tauri-apps/api/core";

// GOOD: Explicit return type
const devices = await invoke<AudioInputDevice[]>("list_audio_devices");
const state = await invoke<RecordingStateResponse>("get_recording_state");
const response = await invoke<PaginatedRecordingsResponse>("list_recordings", {
  limit: 20,
  offset: 0,
});

// BAD: Missing type annotation
const devices = await invoke("list_audio_devices");  // Returns unknown
```

## Query Key Conventions

Use the centralized `queryKeys` object from `src/lib/queryKeys.ts`:

```typescript
import { queryKeys } from "../lib/queryKeys";

// Convention: ['tauri', 'command_name', ...args]
export const queryKeys = {
  tauri: {
    listRecordings: (limit?: number, offset?: number) =>
      ["tauri", "list_recordings", { limit, offset }] as const,
    getRecordingState: ["tauri", "get_recording_state"] as const,
    listAudioDevices: ["tauri", "list_audio_devices"] as const,
    checkModelStatus: (type: string) =>
      ["tauri", "check_parakeet_model_status", type] as const,
  },
  dictionary: {
    all: ["dictionary"] as const,
    list: () => [...queryKeys.dictionary.all, "list"] as const,
  },
} as const;
```

Usage in hooks:

```typescript
// Query with static key
const { data } = useQuery({
  queryKey: queryKeys.tauri.listAudioDevices,
  queryFn: () => invoke<AudioInputDevice[]>("list_audio_devices"),
});

// Query with parameterized key
const { data } = useQuery({
  queryKey: queryKeys.tauri.listRecordings(limit, offset),
  queryFn: () => invoke<PaginatedRecordingsResponse>("list_recordings", { limit, offset }),
});

// Invalidation
queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listAudioDevices });
```

## Hook Structure Pattern

Follow this pattern for Tauri data hooks:

```typescript
export interface UseAudioDevicesResult {
  devices: AudioInputDevice[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => void;
}

export function useAudioDevices(): UseAudioDevicesResult {
  const queryClient = useQueryClient();

  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.listAudioDevices,
    queryFn: () => invoke<AudioInputDevice[]>("list_audio_devices"),
  });

  return {
    devices: data ?? [],
    isLoading,
    error: error instanceof Error ? error : error ? new Error(String(error)) : null,
    refetch: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listAudioDevices });
    },
  };
}
```

## Mutation Hooks

For commands that modify state, use `useMutation`:

```typescript
export function useStopRecording() {
  return useMutation({
    mutationFn: () => invoke("stop_recording"),
    // No onSuccess invalidation - Event Bridge handles this
  });
}

export function useStartRecording() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (deviceName?: string) =>
      invoke("start_recording", { deviceName }),
    onMutate: async () => {
      // Optimistic update
      await queryClient.cancelQueries({ queryKey: queryKeys.tauri.getRecordingState });
      const previousState = queryClient.getQueryData(queryKeys.tauri.getRecordingState);
      queryClient.setQueryData(queryKeys.tauri.getRecordingState, { state: "Recording" });
      return { previousState };
    },
    onError: (_error, _variables, context) => {
      // Rollback on error
      if (context?.previousState) {
        queryClient.setQueryData(queryKeys.tauri.getRecordingState, context.previousState);
      }
    },
  });
}
```

## Anti-Patterns

### Direct invoke in components

```typescript
// BAD: invoke in component body
function RecordingButton() {
  const handleClick = async () => {
    await invoke("start_recording");  // Should use hook
  };
}

// GOOD: Use hook
function RecordingButton() {
  const { startRecording } = useRecording();
  const handleClick = () => startRecording();
}
```

### Hardcoded query keys

```typescript
// BAD: Hardcoded string
const { data } = useQuery({
  queryKey: ["tauri", "list_audio_devices"],  // Fragile
  queryFn: () => invoke("list_audio_devices"),
});

// GOOD: Centralized query key
const { data } = useQuery({
  queryKey: queryKeys.tauri.listAudioDevices,
  queryFn: () => invoke<AudioInputDevice[]>("list_audio_devices"),
});
```

### Missing error type conversion

```typescript
// BAD: Error might not be Error type
return {
  error: error,  // Could be unknown type
};

// GOOD: Convert to Error | null
return {
  error: error instanceof Error ? error : error ? new Error(String(error)) : null,
};
```

### onSuccess invalidation (use Event Bridge instead)

```typescript
// BAD: Duplicates Event Bridge logic
useMutation({
  mutationFn: () => invoke("delete_recording", { filePath }),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listRecordings() });
  },
});

// GOOD: Let Event Bridge handle invalidation
useMutation({
  mutationFn: () => invoke("delete_recording", { filePath }),
  // Event Bridge invalidates on recordings_updated event
});
```
