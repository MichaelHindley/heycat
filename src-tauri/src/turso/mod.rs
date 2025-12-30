// Turso/libsql database module
//
// This module provides embedded SQLite database functionality using libsql.
// It provides a simple, embedded solution for all data persistence.

mod client;
mod dictionary;
pub mod events;
mod recording;
mod schema;
mod voice_command;
mod window_context;

// Recording types are used internally by TursoClient methods
#[allow(unused_imports)]
pub use recording::{RecordingRecord, RecordingStoreError, TranscriptionRecord, TranscriptionStoreError};

pub use client::TursoClient;
pub use schema::initialize_schema;
