//! Tests for utils::text module

use uuid::Uuid;
use utils::text::{git_branch_id, short_uuid, truncate_to_char_boundary};

#[test]
fn test_git_branch_id_lowercase() {
    // Should convert to lowercase
    assert_eq!(git_branch_id("HELLO"), "hello");
    assert_eq!(git_branch_id("HelloWorld"), "helloworld");
}

#[test]
fn test_git_branch_id_replace_non_alphanumeric() {
    // Replace non-alphanumerics with hyphens
    assert_eq!(git_branch_id("hello world"), "hello-world");
    assert_eq!(git_branch_id("hello_world"), "hello-world");
    assert_eq!(git_branch_id("hello.world"), "hello-world");
    assert_eq!(git_branch_id("hello@world"), "hello-world");
}

#[test]
fn test_git_branch_id_trim_hyphens() {
    // Trim leading/trailing hyphens
    assert_eq!(git_branch_id("-hello-"), "hello");
    assert_eq!(git_branch_id("--hello-world--"), "hello-world");
}

#[test]
fn test_git_branch_id_truncate_to_16_chars() {
    // Should truncate to 16 characters
    let long = "this-is-a-very-long-branch-name";
    let result = git_branch_id(long);
    assert!(result.len() <= 16, "Result should be <= 16 chars: {}", result);
    assert_eq!(result, "this-is-a-very-l");
}

#[test]
fn test_git_branch_id_trim_trailing_hyphen_after_truncate() {
    // Should trim trailing hyphens after truncation
    let result = git_branch_id("feature-branch-name");
    assert!(!result.ends_with('-'), "Result should not end with hyphen: {}", result);
}

#[test]
fn test_git_branch_id_empty() {
    assert_eq!(git_branch_id(""), "");
}

#[test]
fn test_short_uuid_format() {
    let uuid = Uuid::new_v4();
    let short = short_uuid(&uuid);

    // Should be exactly 4 characters
    assert_eq!(short.len(), 4);

    // Should be hexadecimal
    assert!(short.chars().all(|c| c.is_ascii_hexdigit()), "Short UUID should be hex: {}", short);
}

#[test]
fn test_short_uuid_consistency() {
    // Same UUID should produce same short ID
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let short1 = short_uuid(&uuid);
    let short2 = short_uuid(&uuid);
    assert_eq!(short1, short2);
}

#[test]
fn test_truncate_to_char_boundary_within_limit() {
    let input = "hello world";
    assert_eq!(truncate_to_char_boundary(input, 20), input);
    assert_eq!(truncate_to_char_boundary(input, 11), input);
}

#[test]
fn test_truncate_to_char_boundary_truncate() {
    let input = "hello world";
    assert_eq!(truncate_to_char_boundary(input, 5), "hello");
    assert_eq!(truncate_to_char_boundary(input, 7), "hello w");
}

#[test]
fn test_truncate_to_char_boundary_unicode() {
    // Each fire emoji is 4 bytes
    let input = "🔥🔥🔥";
    assert_eq!(truncate_to_char_boundary(input, 5), "🔥");
    assert_eq!(truncate_to_char_boundary(input, 4), "🔥");
    assert_eq!(truncate_to_char_boundary(input, 3), "");
    assert_eq!(truncate_to_char_boundary(input, 0), "");
}

#[test]
fn test_truncate_to_char_boundary_chinese() {
    let input = "中文字符";
    // Each Chinese character is 3 bytes in UTF-8
    assert_eq!(truncate_to_char_boundary(input, 3), "中");
    assert_eq!(truncate_to_char_boundary(input, 6), "中文");
    assert_eq!(truncate_to_char_boundary(input, 12), "中文字符");
}

#[test]
fn test_truncate_to_char_boundary_empty() {
    assert_eq!(truncate_to_char_boundary("", 10), "");
}
