//! Tests for utils::git module

use utils::git::is_valid_branch_prefix;

#[test]
fn test_valid_prefixes_empty() {
    // Empty prefix should be valid
    assert!(is_valid_branch_prefix(""));
}

#[test]
fn test_valid_prefixes_simple() {
    assert!(is_valid_branch_prefix("vk"));
    assert!(is_valid_branch_prefix("feature"));
    assert!(is_valid_branch_prefix("hotfix-123"));
}

#[test]
fn test_valid_prefixes_with_dots() {
    assert!(is_valid_branch_prefix("foo.bar"));
    assert!(is_valid_branch_prefix("v1.0"));
}

#[test]
fn test_valid_prefixes_with_underscores() {
    assert!(is_valid_branch_prefix("foo_bar"));
    assert!(is_valid_branch_prefix("feature_branch"));
}

#[test]
fn test_valid_prefixes_with_hyphens() {
    assert!(is_valid_branch_prefix("FOO-Bar"));
    assert!(is_valid_branch_prefix("feature-branch"));
}

#[test]
fn test_invalid_prefixes_with_slash() {
    // Should not contain slash
    assert!(!is_valid_branch_prefix("foo/bar"));
    assert!(!is_valid_branch_prefix("/foo"));
    assert!(!is_valid_branch_prefix("foo/"));
}

#[test]
fn test_invalid_prefixes_with_double_dot() {
    assert!(!is_valid_branch_prefix("foo..bar"));
}

#[test]
fn test_invalid_prefixes_with_at_sign() {
    assert!(!is_valid_branch_prefix("foo@{"));
}

#[test]
fn test_invalid_prefixes_with_lock() {
    assert!(!is_valid_branch_prefix("foo.lock"));
}

#[test]
fn test_invalid_prefixes_with_space() {
    assert!(!is_valid_branch_prefix("foo bar"));
}

#[test]
fn test_invalid_prefixes_with_special_chars() {
    assert!(!is_valid_branch_prefix("foo?"));
    assert!(!is_valid_branch_prefix("foo*"));
    assert!(!is_valid_branch_prefix("foo~"));
    assert!(!is_valid_branch_prefix("foo^"));
    assert!(!is_valid_branch_prefix("foo:"));
    assert!(!is_valid_branch_prefix("foo["));
}

#[test]
fn test_invalid_prefixes_leading_dot() {
    assert!(!is_valid_branch_prefix(".foo"));
}
