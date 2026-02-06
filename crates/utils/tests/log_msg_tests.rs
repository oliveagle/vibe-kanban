//! Tests for utils::log_msg module

use axum::extract::ws::Message;
use json_patch::Patch;
use utils::log_msg::{
    LogMsg, EV_FINISHED, EV_JSON_PATCH, EV_SESSION_ID, EV_STDERR, EV_STDOUT,
};

#[test]
fn test_log_msg_name_stdout() {
    let msg = LogMsg::Stdout("test".to_string());
    assert_eq!(msg.name(), EV_STDOUT);
}

#[test]
fn test_log_msg_name_stderr() {
    let msg = LogMsg::Stderr("error".to_string());
    assert_eq!(msg.name(), EV_STDERR);
}

#[test]
fn test_log_msg_name_json_patch() {
    let patch = serde_json::from_str::<Patch>("[]").unwrap();
    let msg = LogMsg::JsonPatch(patch);
    assert_eq!(msg.name(), EV_JSON_PATCH);
}

#[test]
fn test_log_msg_name_session_id() {
    let msg = LogMsg::SessionId("sess-123".to_string());
    assert_eq!(msg.name(), EV_SESSION_ID);
}

#[test]
fn test_log_msg_name_finished() {
    let msg = LogMsg::Finished;
    assert_eq!(msg.name(), EV_FINISHED);
}

#[test]
fn test_log_msg_approx_bytes_stdout() {
    let content = "hello world";
    let msg = LogMsg::Stdout(content.to_string());
    let bytes = msg.approx_bytes();
    // Should be: EV_STDOUT.len() + content.len() + 8 (overhead)
    assert_eq!(bytes, EV_STDOUT.len() + content.len() + 8);
}

#[test]
fn test_log_msg_approx_bytes_stderr() {
    let content = "error message";
    let msg = LogMsg::Stderr(content.to_string());
    let bytes = msg.approx_bytes();
    assert_eq!(bytes, EV_STDERR.len() + content.len() + 8);
}

#[test]
fn test_log_msg_approx_bytes_session_id() {
    let content = "session-123";
    let msg = LogMsg::SessionId(content.to_string());
    let bytes = msg.approx_bytes();
    assert_eq!(bytes, EV_SESSION_ID.len() + content.len() + 8);
}

#[test]
fn test_log_msg_approx_bytes_finished() {
    let msg = LogMsg::Finished;
    let bytes = msg.approx_bytes();
    assert_eq!(bytes, EV_FINISHED.len() + 8);
}

#[test]
fn test_log_msg_approx_bytes_json_patch() {
    let patch_json = r#"[{"op":"add","path":"/test","value":123}]"#;
    let patch = serde_json::from_str::<Patch>(patch_json).unwrap();
    let msg = LogMsg::JsonPatch(patch);
    let bytes = msg.approx_bytes();
    // Should account for JSON length
    assert!(bytes > EV_JSON_PATCH.len() + 8);
}

#[test]
fn test_log_msg_to_ws_message() {
    let msg = LogMsg::Stdout("test".to_string());
    let result = msg.to_ws_message();
    assert!(result.is_ok());

    if let Ok(Message::Text(text)) = result {
        let text_str = text.as_ref();
        assert!(text_str.contains("Stdout"));
        assert!(text_str.contains("test"));
    } else {
        panic!("Expected Text message");
    }
}

#[test]
fn test_log_msg_to_ws_message_unchecked() {
    let msg = LogMsg::Stdout("test".to_string());
    let message = msg.to_ws_message_unchecked();

    if let Message::Text(text) = message {
        let text_str = text.as_ref();
        assert!(text_str.contains("Stdout"));
        assert!(text_str.contains("test"));
    } else {
        panic!("Expected Text message");
    }
}

#[test]
fn test_log_msg_to_ws_message_unchecked_finished() {
    let msg = LogMsg::Finished;
    let message = msg.to_ws_message_unchecked();

    if let Message::Text(text) = message {
        assert_eq!(text.as_ref(), r#"{"finished":true}"#);
    } else {
        panic!("Expected Text message");
    }
}

#[test]
fn test_log_msg_to_sse_event_stdout() {
    let msg = LogMsg::Stdout("test data".to_string());
    let event = msg.to_sse_event();

    assert_eq!(event.event.unwrap(), EV_STDOUT);
    assert_eq!(event.data.unwrap(), "test data");
}

#[test]
fn test_log_msg_to_sse_event_stderr() {
    let msg = LogMsg::Stderr("error data".to_string());
    let event = msg.to_sse_event();

    assert_eq!(event.event.unwrap(), EV_STDERR);
    assert_eq!(event.data.unwrap(), "error data");
}

#[test]
fn test_log_msg_to_sse_event_session_id() {
    let msg = LogMsg::SessionId("sess-123".to_string());
    let event = msg.to_sse_event();

    assert_eq!(event.event.unwrap(), EV_SESSION_ID);
    assert_eq!(event.data.unwrap(), "sess-123");
}

#[test]
fn test_log_msg_to_sse_event_finished() {
    let msg = LogMsg::Finished;
    let event = msg.to_sse_event();

    assert_eq!(event.event.unwrap(), EV_FINISHED);
    assert_eq!(event.data.unwrap(), "");
}

#[test]
fn test_log_msg_to_sse_event_json_patch() {
    let patch_json = r#"[{"op":"add","path":"/test","value":123}]"#;
    let patch = serde_json::from_str::<Patch>(patch_json).unwrap();
    let msg = LogMsg::JsonPatch(patch);
    let event = msg.to_sse_event();

    assert_eq!(event.event.unwrap(), EV_JSON_PATCH);
    let data = event.data.unwrap();
    assert!(data.contains("op"));
    assert!(data.contains("add"));
}

#[test]
fn test_event_constants() {
    assert_eq!(EV_STDOUT, "stdout");
    assert_eq!(EV_STDERR, "stderr");
    assert_eq!(EV_JSON_PATCH, "json_patch");
    assert_eq!(EV_SESSION_ID, "session_id");
    assert_eq!(EV_FINISHED, "finished");
}
