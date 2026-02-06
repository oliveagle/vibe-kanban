//! Tests for utils::approvals module

use chrono::Duration;
use serde_json::json;
use uuid::Uuid;
use utils::approvals::{
    ApprovalRequest, ApprovalResponse, ApprovalStatus, CreateApprovalRequest,
    APPROVAL_TIMEOUT_SECONDS,
};

#[test]
fn test_approval_timeout_constant() {
    assert_eq!(APPROVAL_TIMEOUT_SECONDS, 3600); // 1 hour
}

#[test]
fn test_approval_request_from_create() {
    let create = CreateApprovalRequest {
        tool_name: "test_tool".to_string(),
        tool_input: json!({"key": "value"}),
        tool_call_id: "call_123".to_string(),
    };

    let execution_process_id = Uuid::new_v4();
    let request = ApprovalRequest::from_create(create.clone(), execution_process_id);

    // Verify fields are correctly set
    assert_eq!(request.tool_name, "test_tool");
    assert_eq!(request.tool_input, json!({"key": "value"}));
    assert_eq!(request.tool_call_id, "call_123");
    assert_eq!(request.execution_process_id, execution_process_id);

    // Verify ID was generated (not empty)
    assert!(!request.id.is_empty());

    // Verify timestamps
    let now = chrono::Utc::now();
    let timeout_diff = request.timeout_at - request.created_at;
    assert_eq!(timeout_diff.num_seconds(), APPROVAL_TIMEOUT_SECONDS);
    assert!(request.created_at <= now);
}

#[test]
fn test_approval_request_id_unique() {
    let create = CreateApprovalRequest {
        tool_name: "test".to_string(),
        tool_input: json!(null),
        tool_call_id: "call_1".to_string(),
    };

    let exec_id = Uuid::new_v4();
    let request1 = ApprovalRequest::from_create(create.clone(), exec_id);
    let request2 = ApprovalRequest::from_create(create.clone(), exec_id);

    // IDs should be unique
    assert_ne!(request1.id, request2.id);
}

#[test]
fn test_approval_status_variants() {
    // Test Pending variant
    let pending = ApprovalStatus::Pending;
    let serialized = serde_json::to_string(&pending).unwrap();
    assert!(serialized.contains("pending"));

    // Test Approved variant
    let approved = ApprovalStatus::Approved;
    let serialized = serde_json::to_string(&approved).unwrap();
    assert!(serialized.contains("approved"));

    // Test Denied variant without reason
    let denied = ApprovalStatus::Denied { reason: None };
    let serialized = serde_json::to_string(&denied).unwrap();
    assert!(serialized.contains("denied"));

    // Test Denied variant with reason
    let denied_with_reason = ApprovalStatus::Denied {
        reason: Some("Not allowed".to_string()),
    };
    let serialized = serde_json::to_string(&denied_with_reason).unwrap();
    assert!(serialized.contains("denied"));
    assert!(serialized.contains("Not allowed"));

    // Test TimedOut variant
    let timed_out = ApprovalStatus::TimedOut;
    let serialized = serde_json::to_string(&timed_out).unwrap();
    assert!(serialized.contains("timed_out"));
}

#[test]
fn test_approval_status_deserialization() {
    // Deserialize Pending
    let pending: ApprovalStatus = serde_json::from_str(r#"{"status":"pending"}"#).unwrap();
    assert!(matches!(pending, ApprovalStatus::Pending));

    // Deserialize Approved
    let approved: ApprovalStatus = serde_json::from_str(r#"{"status":"approved"}"#).unwrap();
    assert!(matches!(approved, ApprovalStatus::Approved));

    // Deserialize Denied without reason
    let denied: ApprovalStatus = serde_json::from_str(r#"{"status":"denied"}"#).unwrap();
    assert!(matches!(denied, ApprovalStatus::Denied { reason: None }));

    // Deserialize Denied with reason
    let denied: ApprovalStatus =
        serde_json::from_str(r#"{"status":"denied","reason":"test reason"}"#).unwrap();
    match denied {
        ApprovalStatus::Denied { reason: Some(r) } => assert_eq!(r, "test reason"),
        _ => panic!("Expected Denied with reason"),
    }

    // Deserialize TimedOut
    let timed_out: ApprovalStatus = serde_json::from_str(r#"{"status":"timed_out"}"#).unwrap();
    assert!(matches!(timed_out, ApprovalStatus::TimedOut));
}

#[test]
fn test_approval_response_serialization() {
    let exec_id = Uuid::new_v4();
    let response = ApprovalResponse {
        execution_process_id: exec_id,
        status: ApprovalStatus::Approved,
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains(&exec_id.to_string()));
    assert!(serialized.contains("approved"));
}

#[test]
fn test_create_approval_request_serialization() {
    let request = CreateApprovalRequest {
        tool_name: "test_tool".to_string(),
        tool_input: json!({"arg": 123}),
        tool_call_id: "call_456".to_string(),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("test_tool"));
    assert!(serialized.contains("call_456"));
    assert!(serialized.contains("arg"));
}
