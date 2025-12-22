---
last-updated: 2025-12-22
status: active
---

# Technical Guidance: Add Spacetime Database

## Architecture Overview

### High-Level Design

SpacetimeDB will be integrated as a **sidecar process** that runs alongside heycat, providing persistent data storage with real-time sync capabilities. The architecture follows this pattern:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              heycat (Tauri App)                             │
│                                                                             │
│  ┌─────────────────┐      ┌─────────────────────────────────────────────┐  │
│  │  React Frontend │ ←──→ │             Rust Backend                    │  │
│  │                 │ IPC  │                                             │  │
│  │  • Queries via  │      │  ┌─────────────────────────────────────┐   │  │
│  │    Tauri invoke │      │  │   SpacetimeDB Client (SDK)          │   │  │
│  │  • Real-time    │      │  │                                     │   │  │
│  │    updates via  │      │  │   • DbConnection (WebSocket)        │   │  │
│  │    events       │      │  │   • Subscription cache              │   │  │
│  └────────┬────────┘      │  │   • Reducer invocations             │   │  │
│           │               │  └─────────────┬───────────────────────┘   │  │
│           │               │                │ WebSocket (localhost)     │  │
│           │               └────────────────┼───────────────────────────┘  │
│           │                                │                               │
└───────────┼────────────────────────────────┼───────────────────────────────┘
            │                                │
            │ Tauri Events                   │
            │ (state sync)                   ▼
            │                    ┌─────────────────────────┐
            │                    │  SpacetimeDB Server     │
            │                    │  (Sidecar Process)      │
            │                    │                         │
            │                    │  • Module: heycat-data  │
            │                    │  • Tables & Reducers    │
            │                    │  • WAL persistence      │
            └────────────────────│  • Port: 3000 (local)   │
                                 └─────────────────────────┘
```

### Layers Involved

| Layer | Current State | SpacetimeDB Integration |
|-------|---------------|-------------------------|
| **Frontend (React)** | Uses Tanstack Query for Tauri commands | No direct changes; continues using Tauri commands |
| **Backend (Rust)** | Uses Tauri Store (settings.json), file-based storage | New SpacetimeDB client module, commands wrap SDK |
| **Persistence** | JSON files, WAV files | SpacetimeDB tables + WAL, WAV files remain on disk |
| **Event System** | `app_handle.emit()` to frontend | SDK subscriptions trigger emit() calls |

### Integration Pattern

SpacetimeDB integrates at the **Rust backend layer** only. The frontend remains unchanged:

1. **Frontend** continues calling `invoke("list_recordings")`, `invoke("get_settings")`, etc.
2. **Tauri commands** (in `commands/`) delegate to SpacetimeDB client instead of file I/O
3. **SpacetimeDB SDK** manages connection, caching, and subscriptions
4. **Event Bridge** receives SpacetimeDB change notifications and emits to frontend

This preserves the existing architecture pattern: **Commands return data, Events push state changes**.

### Data Model

SpacetimeDB module (`heycat-data`) will define these tables:

| Table | Primary Key | Fields | Notes |
|-------|-------------|--------|-------|
| `recordings` | `id: u64` (auto-inc) | file_path, duration_ms, sample_rate, created_at, transcription, transcription_model | Audio files remain on disk; metadata only |
| `settings` | `key: String` | value (JSON string), updated_at | Replaces settings.json |
| `dictionary_entries` | `id: u64` (auto-inc) | trigger, replacement, enabled | Replaces dictionary.json |
| `voice_commands` | `id: u64` (auto-inc) | phrase, action_type, action_payload (JSON), enabled | User-defined voice commands |

### Sidecar Management

SpacetimeDB runs as a Tauri sidecar, managed in `lib.rs`:

```rust
// In setup(), spawn SpacetimeDB sidecar
let (mut rx, child) = app
    .shell()
    .sidecar("spacetimedb")
    .expect("failed to create sidecar command")
    .args(["start", "--listen-addr", "127.0.0.1:3000", "--data-dir", data_dir])
    .spawn()
    .expect("failed to spawn spacetimedb");

// Store child handle for cleanup
app.manage(Arc::new(Mutex::new(Some(child))));

// Wait for server to be ready, then connect SDK
```

### SDK Connection Lifecycle

```rust
// New module: src-tauri/src/spacetime/mod.rs
pub struct SpacetimeConnection {
    conn: DbConnection,
    // Cached client-side view from subscriptions
}

impl SpacetimeConnection {
    pub async fn connect(app_handle: AppHandle) -> Result<Self, Error> {
        let conn = DbConnection::builder()
            .with_uri("ws://127.0.0.1:3000")
            .with_module_name("heycat-data")
            .on_connect(|ctx| {
                // Subscribe to all tables
                ctx.subscription_builder()
                    .on_applied(|_| info!("Subscriptions applied"))
                    .subscribe(vec![
                        "SELECT * FROM recordings",
                        "SELECT * FROM settings",
                        "SELECT * FROM dictionary_entries",
                        "SELECT * FROM voice_commands",
                    ]);
            })
            .on_disconnect(|ctx, err| {
                error!("SpacetimeDB disconnected: {:?}", err);
            })
            .build()?;

        // Use run_threaded() for background processing
        conn.run_threaded();

        Ok(Self { conn })
    }
}
```

### Constraints & Non-Functional Requirements

| Requirement | Approach |
|-------------|----------|
| **Startup latency** | Sidecar spawns async; app functions with cached/empty state until ready |
| **Offline resilience** | SDK cache provides read access; writes queue until reconnected |
| **Resource usage** | Single SpacetimeDB process (~50-100MB); WAL files for persistence |
| **Platform support** | Sidecar binaries for macOS (arm64, x64), Windows, Linux |
| **Data migration** | One-time migration from existing JSON files on first run |

---

## Complete Data Flow Diagram

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    heycat Application                                        │
│                                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────────────────────┐ │
│  │                              FRONTEND (React + TypeScript)                              │ │
│  │                                                                                         │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │ │
│  │  │   Dashboard     │  │   Recordings    │  │   Settings      │  │   Dictionary    │   │ │
│  │  │   Page          │  │   List          │  │   Page          │  │   Manager       │   │ │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘   │ │
│  │           │                    │                    │                    │            │ │
│  │           └────────────────────┴────────────────────┴────────────────────┘            │ │
│  │                                         │                                              │ │
│  │                                         ▼                                              │ │
│  │                          ┌──────────────────────────────┐                              │ │
│  │                          │        Hooks Layer           │                              │ │
│  │                          │  useRecordings()             │                              │ │
│  │                          │  useSettings()               │                              │ │
│  │                          │  useDictionary()             │                              │ │
│  │                          │  useVoiceCommands()          │                              │ │
│  │                          └──────────────┬───────────────┘                              │ │
│  │                                         │                                              │ │
│  │           ┌─────────────────────────────┼─────────────────────────────┐                │ │
│  │           │                             │                             │                │ │
│  │           ▼                             ▼                             ▼                │ │
│  │  ┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐          │ │
│  │  │ Tanstack Query  │         │  Zustand Store  │         │  Event Bridge   │          │ │
│  │  │                 │         │                 │         │                 │          │ │
│  │  │ queryFn:        │         │ • overlayMode   │         │ listen() for:   │          │ │
│  │  │ invoke(cmd)     │         │ • appStatus     │         │ • data_changed  │          │ │
│  │  │                 │         │ • settings      │         │ • sync_complete │          │ │
│  │  └────────┬────────┘         └────────┬────────┘         └────────┬────────┘          │ │
│  │           │                           │                           ▲                   │ │
│  └───────────┼───────────────────────────┼───────────────────────────┼───────────────────┘ │
│              │                           │                           │                     │
│              │ invoke()                  │                           │ emit()              │
│              │                           │                           │                     │
│  ════════════╪═══════════════════════════╪═══════════════════════════╪═════════════════════│
│              │           TAURI IPC BOUNDARY (Security Enforced)      │                     │
│  ════════════╪═══════════════════════════╪═══════════════════════════╪═════════════════════│
│              │                           │                           │                     │
│              ▼                           │                           │                     │
│  ┌────────────────────────────────────────────────────────────────────────────────────────┐ │
│  │                              BACKEND (Rust + Tauri)                                    │ │
│  │                                                                                        │ │
│  │  ┌──────────────────────────────────────────────────────────────────────────────────┐ │ │
│  │  │                          Tauri Command Handlers                                   │ │ │
│  │  │                          (src-tauri/src/commands/)                                │ │ │
│  │  │                                                                                   │ │ │
│  │  │  list_recordings()  save_setting()  add_dictionary_entry()  get_voice_commands() │ │ │
│  │  │         │                 │                  │                      │             │ │ │
│  │  └─────────┼─────────────────┼──────────────────┼──────────────────────┼─────────────┘ │ │
│  │            │                 │                  │                      │               │ │
│  │            └─────────────────┴──────────────────┴──────────────────────┘               │ │
│  │                                         │                                              │ │
│  │                                         ▼                                              │ │
│  │  ┌──────────────────────────────────────────────────────────────────────────────────┐ │ │
│  │  │                      SpacetimeDB Client (src-tauri/src/spacetime/)               │ │ │
│  │  │                                                                                   │ │ │
│  │  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐                   │ │ │
│  │  │  │ DbConnection    │  │ Subscription    │  │ Identity        │                   │ │ │
│  │  │  │                 │  │ Cache           │  │ Credentials     │                   │ │ │
│  │  │  │ ws://127.0.0.1  │  │                 │  │                 │                   │ │ │
│  │  │  │ :3000           │  │ recordings[]    │  │ ~/.spacetimedb_ │                   │ │ │
│  │  │  │                 │  │ settings[]      │  │ client_creds/   │                   │ │ │
│  │  │  └────────┬────────┘  │ dictionary[]    │  └─────────────────┘                   │ │ │
│  │  │           │           │ commands[]      │                                        │ │ │
│  │  │           │           └─────────────────┘                                        │ │ │
│  │  └───────────┼──────────────────────────────────────────────────────────────────────┘ │ │
│  │              │                                                                        │ │
│  │              │ WebSocket (127.0.0.1:3000 ONLY)                                        │ │
│  │              │                                                                        │ │
│  └──────────────┼────────────────────────────────────────────────────────────────────────┘ │
│                 │                                                                          │
│  ═══════════════╪══════════════════════════════════════════════════════════════════════════│
│                 │              PROCESS BOUNDARY (Sidecar)                                  │
│  ═══════════════╪══════════════════════════════════════════════════════════════════════════│
│                 │                                                                          │
│                 ▼                                                                          │
│  ┌────────────────────────────────────────────────────────────────────────────────────────┐│
│  │                         SpacetimeDB Server (Sidecar Process)                           ││
│  │                         --listen-addr=127.0.0.1:3000                                   ││
│  │                                                                                        ││
│  │  ┌──────────────────────────────────────────────────────────────────────────────────┐ ││
│  │  │                          heycat-data Module (WASM)                                │ ││
│  │  │                                                                                   │ ││
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                 │ ││
│  │  │  │ recordings  │ │  settings   │ │ dictionary  │ │  commands   │  ◄── Tables     │ ││
│  │  │  │ table       │ │  table      │ │ table       │ │  table      │                 │ ││
│  │  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘                 │ ││
│  │  │                                                                                   │ ││
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                 │ ││
│  │  │  │ add_        │ │ update_     │ │ add_        │ │ delete_     │  ◄── Reducers   │ ││
│  │  │  │ recording() │ │ setting()   │ │ entry()     │ │ command()   │                 │ ││
│  │  │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘                 │ ││
│  │  │                                                                                   │ ││
│  │  │  All reducers validate: ctx.sender == record.owner_identity                       │ ││
│  │  └──────────────────────────────────────────────────────────────────────────────────┘ ││
│  │                                         │                                             ││
│  │                                         ▼                                             ││
│  │                          ┌──────────────────────────────┐                             ││
│  │                          │   Write-Ahead Log (WAL)      │                             ││
│  │                          │   ~/.local/share/heycat/     │                             ││
│  │                          │   spacetimedb/               │                             ││
│  │                          └──────────────────────────────┘                             ││
│  └────────────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                            │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Write Path (Frontend → SpacetimeDB)

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   User       │    │   React      │    │   Tauri      │    │  SpacetimeDB │    │  SpacetimeDB │
│   Action     │    │   Hook       │    │   Command    │    │   Client     │    │   Server     │
└──────┬───────┘    └──────┬───────┘    └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
       │                   │                   │                   │                   │
       │ Click "Add"       │                   │                   │                   │
       │──────────────────>│                   │                   │                   │
       │                   │                   │                   │                   │
       │                   │ invoke("add_      │                   │                   │
       │                   │ dictionary_entry")│                   │                   │
       │                   │──────────────────>│                   │                   │
       │                   │                   │                   │                   │
       │                   │                   │ client.reducers   │                   │
       │                   │                   │ .add_entry()      │                   │
       │                   │                   │──────────────────>│                   │
       │                   │                   │                   │                   │
       │                   │                   │                   │ WebSocket         │
       │                   │                   │                   │ Reducer call      │
       │                   │                   │                   │──────────────────>│
       │                   │                   │                   │                   │
       │                   │                   │                   │                   │ Validate
       │                   │                   │                   │                   │ owner_identity
       │                   │                   │                   │                   │
       │                   │                   │                   │                   │ Insert row
       │                   │                   │                   │                   │ with owner
       │                   │                   │                   │                   │
       │                   │                   │                   │   Subscription    │
       │                   │                   │                   │<──────────────────│
       │                   │                   │                   │   update          │
       │                   │                   │                   │                   │
       │                   │                   │ on_insert         │                   │
       │                   │                   │ callback          │                   │
       │                   │                   │<──────────────────│                   │
       │                   │                   │                   │                   │
       │                   │                   │ app_handle        │                   │
       │                   │                   │ .emit()           │                   │
       │                   │                   │──────┐            │                   │
       │                   │                   │      │            │                   │
       │                   │ Event:            │      │            │                   │
       │                   │ dictionary_updated│<─────┘            │                   │
       │                   │<──────────────────│                   │                   │
       │                   │                   │                   │                   │
       │                   │ invalidateQueries │                   │                   │
       │                   │ (via Event Bridge)│                   │                   │
       │                   │                   │                   │                   │
       │   UI Updated      │                   │                   │                   │
       │<──────────────────│                   │                   │                   │
       │                   │                   │                   │                   │
```

### Read Path (Query Data)

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   React      │    │   Tanstack   │    │   Tauri      │    │  SpacetimeDB │
│   Component  │    │   Query      │    │   Command    │    │   Client     │
└──────┬───────┘    └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
       │                   │                   │                   │
       │ useRecordings()   │                   │                   │
       │──────────────────>│                   │                   │
       │                   │                   │                   │
       │                   │ Check cache       │                   │
       │                   │                   │                   │
       │                   │ Cache miss?       │                   │
       │                   │ invoke("list_     │                   │
       │                   │ recordings")      │                   │
       │                   │──────────────────>│                   │
       │                   │                   │                   │
       │                   │                   │ Read from         │
       │                   │                   │ subscription      │
       │                   │                   │ cache (local)     │
       │                   │                   │──────────────────>│
       │                   │                   │                   │
       │                   │                   │   Vec<Recording>  │
       │                   │                   │<──────────────────│
       │                   │                   │                   │
       │                   │ Filter by         │                   │
       │                   │ owner_identity    │                   │
       │                   │<──────────────────│                   │
       │                   │                   │                   │
       │                   │ Cache result      │                   │
       │                   │                   │                   │
       │   data            │                   │                   │
       │<──────────────────│                   │                   │
       │                   │                   │                   │

Note: Most reads come from the SDK's subscription cache (in-memory).
      No round-trip to SpacetimeDB server needed for cached data.
```

### Real-Time Update Path (Server → Frontend)

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  SpacetimeDB │    │  SpacetimeDB │    │   Tauri      │    │   Event      │    │   React      │
│   Server     │    │   Client     │    │   Backend    │    │   Bridge     │    │   Component  │
└──────┬───────┘    └──────┬───────┘    └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
       │                   │                   │                   │                   │
       │ Data changes      │                   │                   │                   │
       │ (from reducer)    │                   │                   │                   │
       │                   │                   │                   │                   │
       │ WebSocket push    │                   │                   │                   │
       │──────────────────>│                   │                   │                   │
       │                   │                   │                   │                   │
       │                   │ on_insert/        │                   │                   │
       │                   │ on_update/        │                   │                   │
       │                   │ on_delete         │                   │                   │
       │                   │ callback          │                   │                   │
       │                   │──────────────────>│                   │                   │
       │                   │                   │                   │                   │
       │                   │                   │ app_handle.emit   │                   │
       │                   │                   │ ("recording_      │                   │
       │                   │                   │  updated")        │                   │
       │                   │                   │──────────────────>│                   │
       │                   │                   │                   │                   │
       │                   │                   │                   │ listen() handler  │
       │                   │                   │                   │                   │
       │                   │                   │                   │ queryClient.      │
       │                   │                   │                   │ invalidateQueries │
       │                   │                   │                   │──────────────────>│
       │                   │                   │                   │                   │
       │                   │                   │                   │                   │ Re-render
       │                   │                   │                   │                   │ with new
       │                   │                   │                   │                   │ data
       │                   │                   │                   │                   │
```

### Application Startup Sequence

```
┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
│  Tauri  │   │ Sidecar │   │ Spacetime│  │ SDK     │   │  Event  │   │ Frontend│
│  App    │   │ Process │   │ Server   │  │ Client  │   │  Bridge │   │  React  │
└────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘
     │             │             │             │             │             │
     │ 1. setup()  │             │             │             │             │
     │ spawn       │             │             │             │             │
     │ sidecar     │             │             │             │             │
     │────────────>│             │             │             │             │
     │             │             │             │             │             │
     │             │ 2. Start    │             │             │             │
     │             │ --listen-   │             │             │             │
     │             │ addr=       │             │             │             │
     │             │ 127.0.0.1   │             │             │             │
     │             │────────────>│             │             │             │
     │             │             │             │             │             │
     │             │             │ 3. Ready    │             │             │
     │             │             │ (port open) │             │             │
     │             │<────────────│             │             │             │
     │             │             │             │             │             │
     │ 4. Load credentials       │             │             │             │
     │ from ~/.spacetimedb/      │             │             │             │
     │             │             │             │             │             │
     │ 5. Connect  │             │             │             │             │
     │ with        │             │             │             │             │
     │ identity    │             │             │             │             │
     │─────────────────────────────────────────>│             │             │
     │             │             │             │             │             │
     │             │             │ 6. Validate │             │             │
     │             │             │ identity    │             │             │
     │             │             │<────────────│             │             │
     │             │             │             │             │             │
     │             │             │ 7. Accept   │             │             │
     │             │             │────────────>│             │             │
     │             │             │             │             │             │
     │             │             │             │ 8. Subscribe│             │
     │             │             │             │ to tables   │             │
     │             │             │<────────────│             │             │
     │             │             │             │             │             │
     │             │             │ 9. Send     │             │             │
     │             │             │ initial     │             │             │
     │             │             │ data        │             │             │
     │             │             │────────────>│             │             │
     │             │             │             │             │             │
     │             │             │             │ 10. Cache   │             │
     │             │             │             │ populated   │             │
     │             │             │             │             │             │
     │ 11. app.manage(client)    │             │             │             │
     │             │             │             │             │             │
     │ 12. Setup Event Bridge    │             │             │             │
     │─────────────────────────────────────────────────────>│             │
     │             │             │             │             │             │
     │ 13. Render  │             │             │             │             │
     │ frontend    │             │             │             │             │
     │────────────────────────────────────────────────────────────────────>│
     │             │             │             │             │             │
     │             │             │             │             │             │ 14. Ready
     │             │             │             │             │             │ (queries
     │             │             │             │             │             │  work)
     │             │             │             │             │             │
```

### Hotkey/Background Trigger Path

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Global     │    │   Hotkey     │    │   Recording  │    │  SpacetimeDB │    │   Frontend   │
│   Hotkey     │    │   Handler    │    │   Manager    │    │   Client     │    │   (via emit) │
└──────┬───────┘    └──────┬───────┘    └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
       │                   │                   │                   │                   │
       │ Cmd+Shift+R       │                   │                   │                   │
       │──────────────────>│                   │                   │                   │
       │                   │                   │                   │                   │
       │                   │ handle_toggle()   │                   │                   │
       │                   │──────────────────>│                   │                   │
       │                   │                   │                   │                   │
       │                   │                   │ Start recording   │                   │
       │                   │                   │ (audio capture)   │                   │
       │                   │                   │                   │                   │
       │                   │                   │ ... recording ... │                   │
       │                   │                   │                   │                   │
       │ Cmd+Shift+R       │                   │                   │                   │
       │──────────────────>│                   │                   │                   │
       │                   │                   │                   │                   │
       │                   │ handle_toggle()   │                   │                   │
       │                   │──────────────────>│                   │                   │
       │                   │                   │                   │                   │
       │                   │                   │ Stop & save       │                   │
       │                   │                   │ WAV file          │                   │
       │                   │                   │                   │                   │
       │                   │                   │ Transcribe        │                   │
       │                   │                   │ (Parakeet)        │                   │
       │                   │                   │                   │                   │
       │                   │                   │ Save metadata     │                   │
       │                   │                   │ to SpacetimeDB    │                   │
       │                   │                   │──────────────────>│                   │
       │                   │                   │                   │                   │
       │                   │                   │                   │ add_recording()   │
       │                   │                   │                   │ reducer           │
       │                   │                   │                   │                   │
       │                   │                   │                   │ on_insert         │
       │                   │                   │                   │ callback          │
       │                   │                   │                   │                   │
       │                   │                   │ emit("recording_  │                   │
       │                   │                   │ completed")       │                   │
       │                   │                   │──────────────────────────────────────>│
       │                   │                   │                   │                   │
       │                   │                   │                   │                   │ UI shows
       │                   │                   │                   │                   │ new recording
       │                   │                   │                   │                   │

Note: This path works WITHOUT frontend involvement. The hotkey triggers
      backend directly, data flows to SpacetimeDB, and frontend updates
      via Event Bridge. This is why the IPC-only pattern is critical.
```

---

## Security Architecture

### Threat Model

SpacetimeDB must be isolated from:
1. **Network access** - No external machines can connect
2. **Other host processes** - Only the heycat backend can access the database
3. **Frontend/WebView** - No direct database access from JavaScript

### Security Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                     FRONTEND (React)                            │
│                                                                 │
│  invoke("list_recordings") ──┐                                  │
│  invoke("save_setting")   ───┼──► Tauri IPC (ONLY access path)  │
│  listen("recording_saved") ◄─┘                                  │
│                                                                 │
│  ✗ Cannot import SpacetimeDB SDK                                │
│  ✗ Cannot access ws://127.0.0.1:3000                            │
│  ✗ Has no database credentials                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                    ══════════╪══════════ IPC Boundary (enforced by Tauri)
                              │
┌─────────────────────────────────────────────────────────────────┐
│                     BACKEND (Rust)                              │
│                                                                 │
│  Tauri command handlers (src-tauri/src/commands/)               │
│         │                                                       │
│         ▼                                                       │
│  SpacetimeDbClient (Arc<...>)                                   │
│  ├─ Identity credentials (from ~/.spacetimedb_client_credentials)│
│  └─ WebSocket to 127.0.0.1:3000                                 │
│                                                                 │
│  ✓ Only component with database access                          │
│  ✓ Validates all requests before DB operations                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                    ══════════╪══════════ Process Boundary
                              │
┌─────────────────────────────────────────────────────────────────┐
│  SpacetimeDB Sidecar                                            │
│  --listen-addr=127.0.0.1:3000  (LOCALHOST ONLY)                 │
│                                                                 │
│  ✗ Not reachable from network (bound to loopback)               │
│  ✓ Validates identity on connection                             │
└─────────────────────────────────────────────────────────────────┘
```

### 1. Localhost-Only Binding

SpacetimeDB sidecar MUST bind to `127.0.0.1` only:

```rust
// In lib.rs setup() - CRITICAL: localhost binding
let (mut rx, child) = app
    .shell()
    .sidecar("spacetimedb")
    .args([
        "start",
        "--listen-addr", "127.0.0.1:3000",  // NEVER use 0.0.0.0
        "--data-dir", &data_dir,
    ])
    .spawn()?;
```

**Why this matters:** Binding to `0.0.0.0` would expose the database to the entire network.

### 2. Identity Credentials

Backend connects with identity credentials that other processes don't have:

```rust
// Load or create identity credentials
let credentials = spacetimedb_sdk::credentials::File::load()
    .unwrap_or_else(|_| {
        let creds = spacetimedb_sdk::credentials::Credentials::new();
        let _ = spacetimedb_sdk::credentials::File::save(&creds);
        creds
    });

let spacetime_client = DbConnection::builder()
    .with_uri("ws://127.0.0.1:3000")
    .with_module_name("heycat-data")
    .with_credentials(credentials)  // Identity for this backend instance
    .build()?;
```

Credentials are stored in `~/.spacetimedb_client_credentials/` - only the heycat process has access.

### 3. Tauri Capability Restrictions

Restrict sidecar arguments via capabilities to prevent bypass:

```json
// src-tauri/capabilities/default.json
{
  "identifier": "shell:allow-execute",
  "allow": [{
    "name": "binaries/spacetimedb",
    "sidecar": true,
    "args": [
      "start",
      "--listen-addr",
      { "validator": "^127\\.0\\.0\\.1:[0-9]{1,5}$" },
      "--data-dir",
      { "validator": ".*" }
    ]
  }]
}
```

This ensures even if code is compromised, sidecar can only bind to localhost.

### 4. IPC-Only Frontend Access

Frontend MUST use Tauri commands - never direct DB access:

```rust
// CORRECT - Frontend calls this command
#[tauri::command]
pub async fn list_recordings(
    client: State<'_, Arc<SpacetimeDbClient>>,
) -> Result<Vec<Recording>, String> {
    client.query_recordings().await.map_err(|e| e.to_string())
}

// WRONG - Never expose connection or credentials
// invoke("get_db_connection")  // NEVER DO THIS
// invoke("get_db_credentials") // NEVER DO THIS
```

### Security Guarantees

| Threat | Mitigation |
|--------|------------|
| Network access to SpacetimeDB | Bound to 127.0.0.1 only |
| Frontend direct DB access | IPC commands only, SDK not exposed |
| Malicious sidecar args | Capability regex validates localhost |
| Credential exposure | Credentials stored in backend only |
| WebView compromise | Can only call commands, not access DB |
| Other host processes | Need identity credentials to connect |

---

## Multi-User Architecture (Future-Ready)

### Design Principle

Architecture supports future multi-user collaboration while maintaining local security:
- Each user instance has unique identity
- Data ownership tracked via `owner_identity` field
- Reducers validate ownership before mutations
- Future cloud sync uses same identity system

### Current Phase: Local-Only with Identity

```
┌─────────────────────────────────────────────────────────────────┐
│                     LOCAL INSTANCE                              │
│                                                                 │
│  ┌─────────────────┐      ┌─────────────────────────────────┐  │
│  │  React Frontend │──IPC─│  Tauri Backend                  │  │
│  │  (no DB access) │      │                                 │  │
│  └─────────────────┘      │  SpacetimeDbClient              │  │
│                           │  ├─ identity: UserCredentials   │  │
│                           │  └─ ws://127.0.0.1:3000         │  │
│                           └─────────────┬───────────────────┘  │
│                                         │                       │
│                           ┌─────────────▼───────────────────┐  │
│                           │  SpacetimeDB Sidecar            │  │
│                           │  --listen-addr=127.0.0.1:3000   │  │
│                           │  (validates identity on connect)│  │
│                           └─────────────────────────────────┘  │
│                                                                 │
│  ✗ Other processes on host CANNOT connect (need credentials)   │
└─────────────────────────────────────────────────────────────────┘
```

### Future Phase: Multi-Instance Collaboration

```
┌─────────────────────┐     ┌─────────────────────┐
│  User A's Instance  │     │  User B's Instance  │
│  (Local Sidecar)    │     │  (Local Sidecar)    │
│         │           │     │         │           │
│         ▼           │     │         ▼           │
│  Local SpacetimeDB  │     │  Local SpacetimeDB  │
└─────────┬───────────┘     └─────────┬───────────┘
          │                           │
          └───────────┬───────────────┘
                      │ (Future: Sync via SpacetimeDB Cloud)
                      ▼
          ┌───────────────────────┐
          │  SpacetimeDB Cloud    │
          │  (Maincloud or        │
          │   Self-hosted Server) │
          └───────────────────────┘
```

### Data Model with Ownership

All tables include `owner_identity` for multi-user support:

| Table | Primary Key | Ownership Field | Notes |
|-------|-------------|-----------------|-------|
| `recordings` | `id: u64` (auto-inc) | `owner_identity: Identity` | Each user owns their recordings |
| `settings` | `key: String` | `owner_identity: Identity` | Per-user settings |
| `dictionary_entries` | `id: u64` (auto-inc) | `owner_identity: Identity` | Personal dictionary |
| `voice_commands` | `id: u64` (auto-inc) | `owner_identity: Identity` | User-defined commands |

### Reducer Authorization

All mutations validate ownership:

```rust
// In heycat-data module (SpacetimeDB WASM module)
#[reducer]
pub fn delete_recording(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let recording = ctx.db.recordings().id().find(id)
        .ok_or("Recording not found")?;

    // Validate ownership
    if recording.owner_identity != ctx.sender {
        return Err("Not authorized to delete this recording".to_string());
    }

    ctx.db.recordings().id().delete(id);
    Ok(())
}

#[reducer]
pub fn add_recording(ctx: &ReducerContext, file_path: String, ...) {
    ctx.db.recordings().insert(Recording {
        id: 0,  // auto-inc
        owner_identity: ctx.sender,  // Automatically set to caller
        file_path,
        // ...
    });
}
```

---

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Sidecar deployment (not embedded) | SpacetimeDB doesn't support in-process embedding; sidecar is the only option for local-first | 2025-12-22 |
| All data types in SpacetimeDB | Recordings, settings, dictionary, voice commands - consistent data layer | 2025-12-22 |
| Auto-start with app | Seamless UX; no manual server management for users | 2025-12-22 |
| Frontend unchanged | Keep React layer simple; SpacetimeDB is a backend concern | 2025-12-22 |
| Audio files on disk | Binary blobs stay in filesystem; only metadata in SpacetimeDB | 2025-12-22 |
| Localhost-only binding | Bind to 127.0.0.1 to prevent network access from other machines | 2025-12-22 |
| Identity credentials for connection | Prevents other host processes from connecting to SpacetimeDB | 2025-12-22 |
| IPC-only frontend access | Frontend uses Tauri commands only; no direct SDK access | 2025-12-22 |
| owner_identity in data model | Future multi-user support; each record tracks its owner | 2025-12-22 |
| Reducer-based authorization | Reducers validate ownership before mutations | 2025-12-22 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-22 | SpacetimeDB has no explicit offline-first support | Writes require server connection; need graceful degradation |
| 2025-12-22 | SDK uses `run_threaded()` for background processing | Fits well with Tauri's async model |
| 2025-12-22 | Tauri already has `tauri-plugin-shell` installed | Sidecar infrastructure ready to use |
| 2025-12-22 | SpacetimeDB client cache is read-only mirror | All mutations go through reducers; good for consistency |
| 2025-12-22 | SpacetimeDB `--listen-addr` controls network binding | Use 127.0.0.1 to restrict to localhost only |
| 2025-12-22 | SpacetimeDB SDK supports identity credentials | Stored in ~/.spacetimedb_client_credentials/ |
| 2025-12-22 | Tauri capabilities can restrict sidecar args with regex | Enforces localhost binding even if code compromised |
| 2025-12-22 | heycat follows IPC-only pattern (like dictionary) | Security model naturally extends to SpacetimeDB |

## Open Questions

- [ ] How to handle first-time module deployment? Does sidecar need to `spacetime publish` on first run?
- [ ] What happens if SpacetimeDB sidecar crashes? Auto-restart strategy?
- [ ] How to bundle SpacetimeDB binary for each platform (macOS arm64/x64, Windows, Linux)?
- [ ] Should migrations run as reducers or via CLI before SDK connects?
- [ ] How to test SpacetimeDB integration in CI? Mock server or actual sidecar?

## Files to Modify

### New Files
- `src-tauri/src/spacetime/mod.rs` - SpacetimeDB client connection, lifecycle
- `src-tauri/src/spacetime/subscriptions.rs` - Table subscriptions, cache management
- `src-tauri/src/spacetime/migration.rs` - One-time data migration from JSON files
- `heycat-data/` - SpacetimeDB module (Rust, compiles to WASM)
- `heycat-data/src/lib.rs` - Tables, reducers, module entry point
- `src-tauri/binaries/` - SpacetimeDB sidecar binaries per platform

### Modified Files
- `src-tauri/src/lib.rs` - Sidecar spawn in setup(), SDK connection management
- `src-tauri/src/commands/mod.rs` - Delegate to SpacetimeDB instead of file I/O
- `src-tauri/src/commands/dictionary.rs` - Use SpacetimeDB reducers
- `src-tauri/src/dictionary/store.rs` - Backed by SpacetimeDB table
- `src-tauri/src/voice_commands/mod.rs` - Backed by SpacetimeDB table
- `src-tauri/tauri.conf.json` - Add `externalBin` for SpacetimeDB sidecar
- `src-tauri/capabilities/default.json` - Shell permissions for sidecar
- `src-tauri/Cargo.toml` - Add `spacetimedb-sdk` dependency

### Migration/Removal Candidates
- `src-tauri/src/dictionary/store.rs` - May become thin wrapper over SpacetimeDB
- Settings via Tauri Store - Migrated to SpacetimeDB `settings` table

## References

- [SpacetimeDB Rust SDK Reference](https://spacetimedb.com/docs/sdks/rust)
- [SpacetimeDB Rust Quickstart](https://spacetimedb.com/docs/sdks/rust/quickstart)
- [SpacetimeDB Self-Hosting](https://spacetimedb.com/docs/deploying/spacetimedb-standalone)
- [Tauri v2 Sidecar Documentation](https://v2.tauri.app/develop/sidecar/)
- [SpacetimeDB GitHub](https://github.com/clockworklabs/SpacetimeDB)
- [spacetimedb-sdk crate](https://crates.io/crates/spacetimedb-sdk)
