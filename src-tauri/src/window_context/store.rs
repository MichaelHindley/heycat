// Window context store error types
//
// The WindowContextStore struct was removed as SpacetimeDB is now used for persistence.
// This error type is kept for API compatibility with SpacetimeDB client.

use uuid::Uuid;

/// Error types for window context store operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum WindowContextStoreError {
    /// Context with this ID already exists
    #[allow(dead_code)]
    #[error("Context with ID {0} already exists")]
    DuplicateId(Uuid),
    /// Context not found
    #[error("Context with ID {0} not found")]
    NotFound(Uuid),
    /// Invalid regex pattern
    #[allow(dead_code)]
    #[error("Invalid title pattern regex: {0}")]
    InvalidPattern(String),
    /// Failed to persist contexts
    #[error("Failed to persist contexts: {0}")]
    PersistenceError(String),
    /// Failed to load contexts
    #[error("Failed to load contexts: {0}")]
    LoadError(String),
}
