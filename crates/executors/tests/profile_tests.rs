//! Tests for executors::profile module

use executors::profile::{
    canonical_variant_key, ExecutorConfig, ExecutorProfileId,
};
use executors::executors::BaseCodingAgent;

#[test]
fn test_canonical_variant_key_default() {
    assert_eq!(canonical_variant_key("DEFAULT"), "DEFAULT");
    assert_eq!(canonical_variant_key("default"), "DEFAULT");
    assert_eq!(canonical_variant_key("Default"), "DEFAULT");
}

#[test]
fn test_canonical_variant_key_snake_case() {
    assert_eq!(canonical_variant_key("my_variant"), "MY_VARIANT");
    assert_eq!(canonical_variant_key("myVariant"), "MY_VARIANT");
    assert_eq!(canonical_variant_key("MyVariant"), "MY_VARIANT");
    assert_eq!(canonical_variant_key("my-variant"), "MY_VARIANT");
}

#[test]
fn test_executor_profile_id_new() {
    let id = ExecutorProfileId::new(BaseCodingAgent::Claude);
    assert_eq!(id.executor, BaseCodingAgent::Claude);
    assert!(id.variant.is_none());
}

#[test]
fn test_executor_profile_id_with_variant() {
    let id = ExecutorProfileId::with_variant(BaseCodingAgent::Opencode, "PLAN".to_string());
    assert_eq!(id.executor, BaseCodingAgent::Opencode);
    assert_eq!(id.variant, Some("PLAN".to_string()));
}

#[test]
fn test_executor_profile_id_cache_key_no_variant() {
    let id = ExecutorProfileId::new(BaseCodingAgent::Claude);
    assert_eq!(id.cache_key(), "CLAUDE");
}

#[test]
fn test_executor_profile_id_cache_key_with_variant() {
    let id = ExecutorProfileId::with_variant(BaseCodingAgent::Claude, "PLAN".to_string());
    assert_eq!(id.cache_key(), "CLAUDE:PLAN");
}

#[test]
fn test_executor_profile_id_display_no_variant() {
    let id = ExecutorProfileId::new(BaseCodingAgent::Claude);
    assert_eq!(format!("{}", id), "CLAUDE");
}

#[test]
fn test_executor_profile_id_display_with_variant() {
    let id = ExecutorProfileId::with_variant(BaseCodingAgent::Claude, "PLAN".to_string());
    assert_eq!(format!("{}", id), "CLAUDE:PLAN");
}

#[test]
fn test_executor_config_new_with_default() {
    use executors::executors::{CodingAgent, StandardCodingAgentExecutor};

    let default_config = CodingAgent::Standard(StandardCodingAgentExecutor::Opencode);
    let config = ExecutorConfig::new_with_default(default_config.clone());

    assert!(config.configurations.contains_key("DEFAULT"));
    assert_eq!(config.get_default(), Some(&default_config));
}

#[test]
fn test_executor_config_set_variant() {
    use executors::executors::{CodingAgent, StandardCodingAgentExecutor};

    let default_config = CodingAgent::Standard(StandardCodingAgentExecutor::Opencode);
    let mut config = ExecutorConfig::new_with_default(default_config);

    let variant_config = CodingAgent::Standard(StandardCodingAgentExecutor::Claude);
    let result = config.set_variant("CUSTOM".to_string(), variant_config.clone());

    assert!(result.is_ok());
    assert_eq!(config.get_variant("CUSTOM"), Some(&variant_config));
}

#[test]
fn test_executor_config_set_default_variant_fails() {
    use executors::executors::{CodingAgent, StandardCodingAgentExecutor};

    let default_config = CodingAgent::Standard(StandardCodingAgentExecutor::Opencode);
    let mut config = ExecutorConfig::new_with_default(default_config.clone());

    let result = config.set_variant("DEFAULT".to_string(), default_config);

    assert!(result.is_err());
}

#[test]
fn test_executor_config_variant_names() {
    use executors::executors::{CodingAgent, StandardCodingAgentExecutor};

    let default_config = CodingAgent::Standard(StandardCodingAgentExecutor::Opencode);
    let mut config = ExecutorConfig::new_with_default(default_config);

    let variant_config = CodingAgent::Standard(StandardCodingAgentExecutor::Claude);
    config.set_variant("PLAN".to_string(), variant_config).unwrap();

    let names = config.variant_names();
    assert_eq!(names.len(), 1);
    assert!(names.contains(&&"PLAN".to_string()));
    assert!(!names.contains(&&"DEFAULT".to_string()));
}
