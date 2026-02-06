//! Tests for utils::port_file module

use std::env;
use utils::port_file::{read_port_file, write_port_file};

#[tokio::test]
async fn test_write_and_read_port_file() {
    let test_port: u16 = 54321;

    // Write port file
    let path = write_port_file(test_port).await.expect("Failed to write port file");

    // Verify file exists
    assert!(path.exists(), "Port file should exist");

    // Read port file
    let read_port = read_port_file("vibe-kanban").await.expect("Failed to read port file");

    assert_eq!(read_port, test_port, "Read port should match written port");

    // Clean up
    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_read_port_file_not_found() {
    // Try to read from a non-existent app name
    let result = read_port_file("non-existent-app-12345").await;
    assert!(result.is_err(), "Should fail for non-existent port file");
}

#[tokio::test]
async fn test_write_port_file_creates_directory() {
    let test_port: u16 = 54322;

    // Write port file
    let path = write_port_file(test_port).await.expect("Failed to write port file");

    // Verify parent directory exists
    if let Some(parent) = path.parent() {
        assert!(parent.exists(), "Parent directory should exist");
    }

    // Clean up
    let _ = tokio::fs::remove_file(&path).await;
}

#[tokio::test]
async fn test_read_port_file_invalid_content() {
    let temp_dir = env::temp_dir().join("vibe-kanban");
    let port_file = temp_dir.join("vibe-kanban.port");

    // Ensure directory exists
    tokio::fs::create_dir_all(&temp_dir).await.ok();

    // Write invalid content
    tokio::fs::write(&port_file, "not-a-number").await.expect("Failed to write test file");

    // Try to read - should fail
    let result = read_port_file("vibe-kanban").await;
    assert!(result.is_err(), "Should fail for invalid port content");

    // Clean up
    let _ = tokio::fs::remove_file(&port_file).await;
}

#[tokio::test]
async fn test_read_port_file_whitespace() {
    let temp_dir = env::temp_dir().join("vibe-kanban");
    let port_file = temp_dir.join("vibe-kanban.port");

    // Ensure directory exists
    tokio::fs::create_dir_all(&temp_dir).await.ok();

    // Write port with whitespace
    tokio::fs::write(&port_file, "  8080  \n").await.expect("Failed to write test file");

    // Should trim whitespace and parse correctly
    let port = read_port_file("vibe-kanban").await.expect("Should read port with whitespace");
    assert_eq!(port, 8080);

    // Clean up
    let _ = tokio::fs::remove_file(&port_file).await;
}
