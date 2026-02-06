//! Tests for server::error module

use axum::response::IntoResponse;
use http::StatusCode;
use server::error::ApiError;

#[test]
fn test_api_error_from_str() {
    let err: ApiError = "test error message".into();
    match err {
        ApiError::BadRequest(msg) => assert_eq!(msg, "test error message"),
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_api_error_unauthorized() {
    let err = ApiError::Unauthorized;
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_api_error_bad_request() {
    let err = ApiError::BadRequest("Invalid input".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_api_error_conflict() {
    let err = ApiError::Conflict("Resource exists".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[test]
fn test_api_error_forbidden() {
    let err = ApiError::Forbidden("Access denied".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
