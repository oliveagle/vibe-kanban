//! Tests for utils::jwt module

use utils::jwt::{extract_expiration, extract_subject, TokenClaimsError};

fn create_test_jwt(exp: i64, sub: &str) -> String {
    // Create a simple JWT payload (base64 encoded JSON)
    let header = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        r#"{"alg":"none","typ":"JWT"}"#
    );
    let payload = format!(
        "{{\"exp\":{},\"sub\":\"{}\"}}",
        exp, sub
    );
    let payload = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &payload
    );
    format!("{}.{}.", header, payload)
}

fn create_test_jwt_without_exp(sub: &str) -> String {
    let header = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        r#"{"alg":"none","typ":"JWT"}"#
    );
    let payload = format!("{{\"sub\":\"{}\"}}", sub);
    let payload = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &payload
    );
    format!("{}.{}.", header, payload)
}

fn create_test_jwt_without_sub(exp: i64) -> String {
    let header = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        r#"{"alg":"none","typ":"JWT"}"#
    );
    let payload = format!("{{\"exp\":{}}}", exp);
    let payload = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        &payload
    );
    format!("{}.{}.", header, payload)
}

#[test]
fn test_extract_expiration_success() {
    // Use a timestamp in the future (year 2030)
    let exp = 1893456000i64;
    let token = create_test_jwt(exp, "550e8400-e29b-41d4-a716-446655440000");

    let result = extract_expiration(&token);
    assert!(result.is_ok());

    let datetime = result.unwrap();
    assert_eq!(datetime.timestamp(), exp);
}

#[test]
fn test_extract_expiration_missing() {
    let token = create_test_jwt_without_exp("550e8400-e29b-41d4-a716-446655440000");

    let result = extract_expiration(&token);
    assert!(matches!(result, Err(TokenClaimsError::MissingExpiration)));
}

#[test]
fn test_extract_expiration_invalid_timestamp() {
    // Create token with invalid timestamp (too large)
    let header = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        r#"{"alg":"none","typ":"JWT"}"#
    );
    let payload = r#"{"exp":999999999999999999}"#;
    let payload = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        payload
    );
    let token = format!("{}.{}.", header, payload);

    let result = extract_expiration(&token);
    assert!(matches!(result, Err(TokenClaimsError::InvalidExpiration(_))));
}

#[test]
fn test_extract_subject_success() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let token = create_test_jwt(1893456000, uuid_str);

    let result = extract_subject(&token);
    assert!(result.is_ok());

    let uuid = result.unwrap();
    assert_eq!(uuid.to_string(), uuid_str);
}

#[test]
fn test_extract_subject_missing() {
    let token = create_test_jwt_without_sub(1893456000);

    let result = extract_subject(&token);
    assert!(matches!(result, Err(TokenClaimsError::MissingSubject)));
}

#[test]
fn test_extract_subject_invalid_uuid() {
    let header = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        r#"{"alg":"none","typ":"JWT"}"#
    );
    let payload = r#"{"sub":"not-a-valid-uuid"}"#;
    let payload = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        payload
    );
    let token = format!("{}.{}.", header, payload);

    let result = extract_subject(&token);
    assert!(matches!(result, Err(TokenClaimsError::InvalidSubject(_))));
}

#[test]
fn test_extract_malformed_token() {
    let result = extract_expiration("not-a-valid-jwt");
    assert!(result.is_err());
}
