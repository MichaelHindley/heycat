use super::*;
use libsql::params;
use tempfile::TempDir;

/// Test creating a TursoClient and verifying connection
#[tokio::test]
async fn test_client_creates_database_and_connects() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();

    // Create client
    let client = TursoClient::new(data_dir.clone())
        .await
        .expect("Failed to create TursoClient");

    // Verify database file exists
    let expected_db_path = data_dir.join("turso").join("heycat.db");
    assert!(expected_db_path.exists(), "Database file should exist");
    assert_eq!(client.db_path(), &expected_db_path);

    // Verify connection works
    assert!(client.is_connected().await, "Client should be connected");

    // Verify we can execute queries
    client
        .execute(
            "CREATE TABLE test (id TEXT PRIMARY KEY, value TEXT)",
            (),
        )
        .await
        .expect("Should create table");

    client
        .execute(
            "INSERT INTO test (id, value) VALUES (?1, ?2)",
            params!["test-1", "hello"],
        )
        .await
        .expect("Should insert row");

    let mut rows = client
        .query("SELECT value FROM test WHERE id = ?1", params!["test-1"])
        .await
        .expect("Should query row");

    let row = rows.next().await.expect("Should get next").expect("Should have row");
    let value: String = row.get(0).expect("Should get value");
    assert_eq!(value, "hello");
}

/// Test that foreign keys are enabled
#[tokio::test]
async fn test_foreign_keys_enabled() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();

    let client = TursoClient::new(data_dir)
        .await
        .expect("Failed to create TursoClient");

    // Create parent table
    client
        .execute(
            "CREATE TABLE parent (id TEXT PRIMARY KEY)",
            (),
        )
        .await
        .expect("Should create parent table");

    // Create child table with foreign key
    client
        .execute(
            "CREATE TABLE child (id TEXT PRIMARY KEY, parent_id TEXT REFERENCES parent(id))",
            (),
        )
        .await
        .expect("Should create child table");

    // Attempt to insert child with non-existent parent should fail
    let result = client
        .execute(
            "INSERT INTO child (id, parent_id) VALUES (?1, ?2)",
            params!["child-1", "nonexistent"],
        )
        .await;

    assert!(result.is_err(), "Foreign key constraint should be enforced");
}

/// Test that unique constraint violations are properly detected
#[tokio::test]
async fn test_unique_constraint_detection() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let data_dir = temp_dir.path().to_path_buf();

    let client = TursoClient::new(data_dir)
        .await
        .expect("Failed to create TursoClient");

    client
        .execute(
            "CREATE TABLE unique_test (id TEXT PRIMARY KEY, trigger TEXT UNIQUE)",
            (),
        )
        .await
        .expect("Should create table");

    // First insert should succeed
    client
        .execute(
            "INSERT INTO unique_test (id, trigger) VALUES (?1, ?2)",
            params!["id-1", "hello"],
        )
        .await
        .expect("First insert should succeed");

    // Second insert with same trigger should fail with Constraint error
    let result = client
        .execute(
            "INSERT INTO unique_test (id, trigger) VALUES (?1, ?2)",
            params!["id-2", "hello"],
        )
        .await;

    match result {
        Err(TursoError::Constraint(_)) => (), // Expected
        Err(other) => panic!("Expected Constraint error, got: {:?}", other),
        Ok(_) => panic!("Should have failed with unique constraint violation"),
    }
}
