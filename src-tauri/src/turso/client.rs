// TursoClient - Embedded SQLite database using libsql
//
// This client wraps libsql::Database to provide async-compatible
// database operations for all heycat data tables.

use libsql::{Builder, Connection, Database};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Error types for Turso operations
#[derive(Debug)]
pub enum TursoError {
    /// Database connection or initialization error
    Connection(String),
    /// Query execution error
    Query(String),
    /// Row not found (will be used by CRUD migration specs)
    #[allow(dead_code)]
    NotFound(String),
    /// Constraint violation (e.g., unique constraint)
    Constraint(String),
}

impl std::fmt::Display for TursoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TursoError::Connection(msg) => write!(f, "Database connection error: {}", msg),
            TursoError::Query(msg) => write!(f, "Query error: {}", msg),
            TursoError::NotFound(msg) => write!(f, "Not found: {}", msg),
            TursoError::Constraint(msg) => write!(f, "Constraint violation: {}", msg),
        }
    }
}

impl std::error::Error for TursoError {}

impl From<libsql::Error> for TursoError {
    fn from(err: libsql::Error) -> Self {
        let msg = err.to_string();
        if msg.contains("UNIQUE constraint failed") {
            TursoError::Constraint(msg)
        } else {
            TursoError::Query(msg)
        }
    }
}

/// Database directory name within the app data directory
const DB_DIR: &str = "turso";
/// Database file name
const DB_FILE: &str = "heycat.db";

/// TursoClient wraps libsql::Database for embedded SQLite operations.
///
/// The client uses an Arc<Mutex<Connection>> internally to ensure
/// thread-safe access to the database connection. This allows the
/// client to be cloned and shared across multiple Tauri commands.
#[derive(Clone)]
pub struct TursoClient {
    #[allow(dead_code)]
    db: Arc<Database>,
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl TursoClient {
    /// Create a new TursoClient synchronously (blocking).
    ///
    /// This is useful for initialization in synchronous contexts like Tauri's setup closure.
    /// It internally creates a tokio runtime to run the async initialization.
    ///
    /// # Arguments
    /// * `data_dir` - The base data directory (e.g., ~/.local/share/heycat/)
    ///
    /// # Returns
    /// A Result containing the TursoClient or a TursoError on failure.
    pub fn new_blocking(data_dir: PathBuf) -> Result<Self, TursoError> {
        // Use the current tokio runtime if available, otherwise create one
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're already in a tokio runtime, use block_in_place
                tokio::task::block_in_place(|| {
                    handle.block_on(Self::new(data_dir))
                })
            }
            Err(_) => {
                // No runtime, create a temporary one
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| TursoError::Connection(format!("Failed to create runtime: {}", e)))?;
                rt.block_on(Self::new(data_dir))
            }
        }
    }

    /// Create a new TursoClient with the database at the specified directory.
    ///
    /// The database file will be created at `{data_dir}/turso/heycat.db`.
    /// This function creates the directory if it doesn't exist and initializes
    /// the SQLite database.
    ///
    /// # Arguments
    /// * `data_dir` - The base data directory (e.g., ~/.local/share/heycat/)
    ///
    /// # Returns
    /// A Result containing the TursoClient or a TursoError on failure.
    pub async fn new(data_dir: PathBuf) -> Result<Self, TursoError> {
        // Create the turso subdirectory
        let db_dir = data_dir.join(DB_DIR);
        std::fs::create_dir_all(&db_dir).map_err(|e| {
            TursoError::Connection(format!("Failed to create database directory: {}", e))
        })?;

        let db_path = db_dir.join(DB_FILE);
        let db_path_str = db_path.to_string_lossy().to_string();

        crate::info!("Opening Turso database at: {}", db_path_str);

        // Build the embedded database
        let db = Builder::new_local(&db_path_str)
            .build()
            .await
            .map_err(|e| TursoError::Connection(format!("Failed to open database: {}", e)))?;

        // Get a connection
        let conn = db
            .connect()
            .map_err(|e| TursoError::Connection(format!("Failed to connect: {}", e)))?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", ())
            .await
            .map_err(|e| TursoError::Query(format!("Failed to enable foreign keys: {}", e)))?;

        Ok(Self {
            db: Arc::new(db),
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Get the path to the database file.
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Execute a SQL query that doesn't return rows.
    ///
    /// # Arguments
    /// * `sql` - The SQL statement to execute
    /// * `params` - Parameters for the SQL statement
    pub async fn execute(
        &self,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<u64, TursoError> {
        let conn = self.conn.lock().await;
        conn.execute(sql, params)
            .await
            .map_err(TursoError::from)
    }

    /// Execute a SQL query and return all rows.
    ///
    /// # Arguments
    /// * `sql` - The SQL query to execute
    /// * `params` - Parameters for the SQL query
    pub async fn query(
        &self,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<libsql::Rows, TursoError> {
        let conn = self.conn.lock().await;
        conn.query(sql, params)
            .await
            .map_err(TursoError::from)
    }

    /// Check if the database connection is valid.
    /// Note: Currently only used in tests - will be used for health checks
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        // Try a simple query to verify connection
        match self.query("SELECT 1", ()).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

// Note: Database is closed automatically when TursoClient is dropped
// (Arc<Database> handles cleanup when reference count reaches zero)

#[cfg(test)]
#[path = "client_test.rs"]
mod tests;
