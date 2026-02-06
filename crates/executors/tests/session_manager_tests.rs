use std::sync::Arc;
use executors::executors::acp::SessionManager;

#[tokio::test]
async fn test_session_manager_new() {
    let manager = SessionManager::new("test_namespace_new").await;
    assert!(manager.is_ok());
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_new")
    ).await;
}

#[tokio::test]
async fn test_session_manager_append_and_read() {
    let manager = SessionManager::new("test_namespace_append").await.unwrap();
    let session_id = "test-session-001";
    
    // Append some lines
    manager.append_raw_line(session_id, r#"{"user": "Hello"}"#).await.unwrap();
    manager.append_raw_line(session_id, r#"{"assistant": "Hi there"}"#).await.unwrap();
    
    // Read back
    let content = manager.read_session_raw(session_id).await.unwrap();
    assert!(content.contains(r#"{"user": "Hello"}"#));
    assert!(content.contains(r#"{"assistant": "Hi there"}"#));
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_append")
    ).await;
}

#[tokio::test]
async fn test_session_manager_fork() {
    let manager = SessionManager::new("test_namespace_fork").await.unwrap();
    let old_id = "original-session";
    let new_id = "forked-session";
    
    // Add content to original
    manager.append_raw_line(old_id, r#"{"user": "Original"}"#).await.unwrap();
    
    // Fork
    manager.fork_session(old_id, new_id).await.unwrap();
    
    // Verify forked content
    let forked_content = manager.read_session_raw(new_id).await.unwrap();
    assert!(forked_content.contains("Original"));
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_fork")
    ).await;
}

#[tokio::test]
async fn test_session_manager_delete() {
    let manager = SessionManager::new("test_namespace_delete").await.unwrap();
    let session_id = "session-to-delete";
    
    // Create session
    manager.append_raw_line(session_id, r#"{"user": "Test"}"#).await.unwrap();
    
    // Verify it exists
    let path = manager.session_file_path(session_id);
    assert!(path.exists());
    
    // Delete
    manager.delete_session(session_id).await.unwrap();
    
    // Verify it's gone
    assert!(!path.exists());
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_delete")
    ).await;
}

#[tokio::test]
async fn test_session_manager_generate_resume_prompt() {
    let manager = SessionManager::new("test_namespace_resume").await.unwrap();
    let session_id = "resume-session";
    
    // Add some history
    manager.append_raw_line(session_id, r#"{"user": "Do something"}"#).await.unwrap();
    manager.append_raw_line(session_id, r#"{"assistant": "Done"}"#).await.unwrap();
    
    // Generate resume prompt
    let prompt = manager.generate_resume_prompt(session_id, "Continue working").await.unwrap();
    
    assert!(prompt.contains("RESUME CONTEXT FOR CONTINUING TASK"));
    assert!(prompt.contains("Do something"));
    assert!(prompt.contains("Continue working"));
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_resume")
    ).await;
}

#[tokio::test]
async fn test_session_manager_concurrent_writes() {
    let manager = SessionManager::new("test_namespace_concurrent").await.unwrap();
    let session_id = "concurrent-session";
    let manager = Arc::new(manager);
    let mut handles = vec![];
    
    // Spawn multiple concurrent writes
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let session_id = session_id.to_string();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let line = format!(r#"{{"task": "{}-{}"}}"#, i, j);
                manager_clone.append_raw_line(&session_id, &line).await.unwrap();
            }
        });
        handles.push(handle);
    }
    
    // Wait for all
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all lines were written
    let content = manager.read_session_raw(session_id).await.unwrap();
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 100);
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_concurrent")
    ).await;
}

#[test]
fn test_normalize_session_event() {
    // Test user event
    let user_event = r#"{"User": "Hello"}"#;
    let normalized = SessionManager::normalize_session_event(user_event);
    assert!(normalized.is_some());
    assert!(normalized.unwrap().contains("user"));

    // Test message event
    let msg_event = r#"{"Message": {"Text": {"text": "Hello"}}}"#;
    let normalized = SessionManager::normalize_session_event(msg_event);
    assert!(normalized.is_some());

    // Test events that should be filtered out
    let session_start = r#"{"SessionStart": "id123"}"#;
    assert!(SessionManager::normalize_session_event(session_start).is_none());
    
    let done_event = r#"{"Done": "success"}"#;
    assert!(SessionManager::normalize_session_event(done_event).is_none());
    
    let error_event = r#"{"Error": "something went wrong"}"#;
    assert!(SessionManager::normalize_session_event(error_event).is_none());
}

#[tokio::test]
async fn test_session_manager_read_nonexistent() {
    let manager = SessionManager::new("test_namespace_nonexistent").await.unwrap();
    
    // Reading a non-existent session should return empty string
    let content = manager.read_session_raw("non-existent-session").await.unwrap();
    assert!(content.is_empty());
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_nonexistent")
    ).await;
}

#[tokio::test]
async fn test_session_manager_delete_nonexistent() {
    let manager = SessionManager::new("test_namespace_delete_nonexist").await.unwrap();
    
    // Deleting a non-existent session should not error
    let result = manager.delete_session("non-existent-session").await;
    assert!(result.is_ok());
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(
        dirs::home_dir()
            .unwrap()
            .join(".vibe-kanban")
            .join("test_namespace_delete_nonexist")
    ).await;
}
