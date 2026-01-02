---
paths: "src-tauri/src/**/*.rs, src/**/*.ts, src/**/*.tsx"
---

# Cross-Stack Type Contracts

## Rust: serde(rename_all = "camelCase")

All Rust structs sent to the frontend must use camelCase serialization:

```rust
// In src-tauri/src/audio/error.rs
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AudioDeviceError {
    DeviceNotFound { device_name: String },  // → { type: "deviceNotFound", deviceName: "..." }
    NoDevicesAvailable,                       // → { type: "noDevicesAvailable" }
    CaptureError { message: String },         // → { type: "captureError", message: "..." }
}

// In src-tauri/src/events.rs
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionsUpdatedPayload {
    pub change_type: String,      // → changeType
    pub transcription_id: String, // → transcriptionId
    pub recording_id: String,     // → recordingId
}
```

## TypeScript: Mirror Types

Frontend types must mirror the Rust struct shapes in camelCase:

```typescript
// In src/types/audio.ts - mirrors AudioDeviceError
export type AudioDeviceError =
  | { type: "deviceNotFound"; deviceName: string }
  | { type: "noDevicesAvailable" }
  | { type: "deviceDisconnected" }
  | { type: "captureError"; message: string };

// In src/lib/eventBridge.ts - mirrors Rust payloads
export interface TranscriptionsUpdatedPayload {
  changeType: string;
  transcriptionId: string | null;
  recordingId: string | null;
  timestamp: string;
}
```

## invoke<T>() Generic Usage

Always specify the return type when calling Tauri commands:

```typescript
import { invoke } from "@tauri-apps/api/core";

// GOOD: Explicit return type
const devices = await invoke<AudioInputDevice[]>("list_audio_devices");
const command = await invoke<CommandDto>("update_command", { input });
const response = await invoke<PaginatedRecordingsResponse>("list_recordings", {
  limit: 20,
  offset: 0,
});

// BAD: Missing type annotation
const devices = await invoke("list_audio_devices");  // Returns unknown
```

## Event Payload Exports

Export event payload types from `eventBridge.ts` for reuse:

```typescript
// In src/lib/eventBridge.ts
export interface TranscriptionCompletedPayload {
  text: string;
  duration_ms: number;
}

export interface KeyBlockingUnavailablePayload {
  reason: string;
  timestamp: string;
}

export interface RecordingsUpdatedPayload {
  changeType: string;
  recordingId: string | null;
  timestamp: string;
}
```

Usage in listeners:

```typescript
await listen<TranscriptionCompletedPayload>(eventNames.TRANSCRIPTION_COMPLETED, (event) => {
  const { text, duration_ms } = event.payload;
});
```

## Type Pair Examples

### Audio Device
```rust
// Rust: src-tauri/src/audio/device.rs
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInputDevice {
    pub name: String,
    pub is_default: bool,
}
```

```typescript
// TypeScript: src/types/audio.ts
export interface AudioInputDevice {
  name: string;
  isDefault: boolean;
}
```

### Paginated Response
```rust
// Rust: src-tauri/src/commands/logic.rs
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedRecordingsResponse {
    pub recordings: Vec<RecordingInfo>,
    pub total_count: usize,
    pub has_more: bool,
}
```

```typescript
// TypeScript: src/pages/Recordings/types.ts
export interface PaginatedRecordingsResponse {
  recordings: RecordingInfo[];
  totalCount: number;
  hasMore: boolean;
}
```

## Anti-Patterns

### Missing serde rename on Rust structs

```rust
// BAD: snake_case goes to frontend
#[derive(Serialize)]
pub struct MyPayload {
    pub file_name: String,  // Serializes as "file_name"
}

// GOOD: camelCase for frontend
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyPayload {
    pub file_name: String,  // Serializes as "fileName"
}
```

### Untyped invoke calls

```typescript
// BAD: No type safety
const result = await invoke("get_data");
result.someField;  // TypeScript can't check this

// GOOD: Typed response
const result = await invoke<MyDataType>("get_data");
result.someField;  // TypeScript validates this
```

### Mismatched casing in TypeScript types

```typescript
// BAD: Using snake_case in TypeScript
interface MyPayload {
  file_name: string;  // Doesn't match camelCase from Rust
}

// GOOD: Match Rust's serialized output
interface MyPayload {
  fileName: string;
}
```
