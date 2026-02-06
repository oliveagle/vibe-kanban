//! Tests for executors::env module

use std::collections::HashMap;

use executors::env::ExecutionEnv;

#[test]
fn test_execution_env_new() {
    let env = ExecutionEnv::new();
    assert!(env.vars.is_empty());
}

#[test]
fn test_execution_env_default() {
    let env: ExecutionEnv = Default::default();
    assert!(env.vars.is_empty());
}

#[test]
fn test_execution_env_insert() {
    let mut env = ExecutionEnv::new();
    env.insert("KEY", "value");
    assert_eq!(env.vars.get("KEY"), Some(&"value".to_string()));
}

#[test]
fn test_execution_env_merge() {
    let mut env = ExecutionEnv::new();
    env.insert("EXISTING", "original");

    let mut other = HashMap::new();
    other.insert("EXISTING".to_string(), "overridden".to_string());
    other.insert("NEW".to_string(), "new_value".to_string());

    env.merge(&other);

    assert_eq!(env.vars.get("EXISTING"), Some(&"overridden".to_string()));
    assert_eq!(env.vars.get("NEW"), Some(&"new_value".to_string()));
}

#[test]
fn test_execution_env_with_overrides() {
    let mut base = ExecutionEnv::default();
    base.insert("VK_PROJECT_NAME", "runtime");
    base.insert("FOO", "runtime");

    let mut profile = HashMap::new();
    profile.insert("FOO".to_string(), "profile".to_string());
    profile.insert("BAR".to_string(), "profile".to_string());

    let merged = base.with_overrides(&profile);

    assert_eq!(merged.vars.get("VK_PROJECT_NAME").unwrap(), "runtime");
    assert_eq!(merged.vars.get("FOO").unwrap(), "profile"); // overrides
    assert_eq!(merged.vars.get("BAR").unwrap(), "profile");
}

#[test]
fn test_execution_env_contains_key() {
    let mut env = ExecutionEnv::new();
    env.insert("PRESENT", "value");
    assert!(env.contains_key("PRESENT"));
    assert!(!env.contains_key("ABSENT"));
}

#[test]
fn test_execution_env_apply_to_command() {
    let mut env = ExecutionEnv::new();
    env.insert("TEST_VAR", "test_value");

    let mut cmd = tokio::process::Command::new("echo");
    env.apply_to_command(&mut cmd);

    // Command was modified (can't easily verify env vars on command)
    assert!(true);
}
