use std::sync::Arc;
use tokio::sync::broadcast;
use workspace_utils::msg_store::MsgStore;
use workspace_utils::log_msg::LogMsg;

#[tokio::test]
async fn test_msg_store_push_and_get_history() {
    let store = MsgStore::new();
    
    // Push some messages
    store.push_stdout("Hello").await;
    store.push_stdout("World").await;
    store.push_stderr("Error message").await;
    
    // Get history
    let history = store.get_history().await;
    
    assert_eq!(history.len(), 3);
    assert!(matches!(&history[0], LogMsg::Stdout(s) if s == "Hello"));
    assert!(matches!(&history[1], LogMsg::Stdout(s) if s == "World"));
    assert!(matches!(&history[2], LogMsg::Stderr(s) if s == "Error message"));
}

#[tokio::test]
async fn test_msg_store_receiver() {
    let store = MsgStore::new();
    let mut rx = store.get_receiver();
    
    // Push a message
    store.push_stdout("Test message").await;
    
    // Should receive the message
    let received = rx.recv().await.unwrap();
    assert!(matches!(received, LogMsg::Stdout(s) if s == "Test message"));
}

#[tokio::test]
async fn test_msg_store_concurrent_push() {
    let store = Arc::new(MsgStore::new());
    let mut handles = vec![];
    
    // Spawn multiple tasks pushing concurrently
    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                store_clone.push_stdout(format!("Task {} Message {}", i, j)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all messages were stored
    let history = store.get_history().await;
    assert_eq!(history.len(), 100);
}

#[tokio::test]
async fn test_msg_store_session_id() {
    let store = MsgStore::new();
    
    store.push_session_id("test-session-123".to_string()).await;
    
    let history = store.get_history().await;
    assert_eq!(history.len(), 1);
    assert!(matches!(&history[0], LogMsg::SessionId(s) if s == "test-session-123"));
}

#[tokio::test]
async fn test_msg_store_finished() {
    let store = MsgStore::new();
    
    store.push_stdout("Before").await;
    store.push_finished().await;
    store.push_stdout("After").await;
    
    let history = store.get_history().await;
    assert_eq!(history.len(), 3);
    assert!(matches!(&history[1], LogMsg::Finished));
}

#[tokio::test]
async fn test_msg_store_multiple_receivers() {
    let store = MsgStore::new();
    let mut rx1 = store.get_receiver();
    let mut rx2 = store.get_receiver();
    
    store.push_stdout("Broadcast message").await;
    
    // Both receivers should get the message
    let msg1 = rx1.recv().await.unwrap();
    let msg2 = rx2.recv().await.unwrap();
    
    assert!(matches!(msg1, LogMsg::Stdout(s) if s == "Broadcast message"));
    assert!(matches!(msg2, LogMsg::Stdout(s) if s == "Broadcast message"));
}

#[tokio::test]
async fn test_msg_store_history_ordering() {
    let store = MsgStore::new();
    
    for i in 0..5 {
        store.push_stdout(format!("Message {}", i)).await;
    }
    
    let history = store.get_history().await;
    assert_eq!(history.len(), 5);
    
    for (i, msg) in history.iter().enumerate() {
        let expected = format!("Message {}", i);
        assert!(matches!(msg, LogMsg::Stdout(s) if s == &expected));
    }
}
