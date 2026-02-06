//! Tests for services::file_ranker module

use chrono::Utc;
use std::collections::HashMap;

use services::services::file_ranker::{FileRanker, FileStat};

#[test]
fn test_file_ranker_new() {
    let ranker = FileRanker::new();
    // Just verify it can be created
    assert!(true);
}

#[test]
fn test_file_ranker_default() {
    let ranker: FileRanker = Default::default();
    assert!(true);
}

#[test]
fn test_file_stat_creation() {
    let stat = FileStat {
        last_index: 0,
        commit_count: 5,
        last_time: Utc::now(),
    };
    assert_eq!(stat.last_index, 0);
    assert_eq!(stat.commit_count, 5);
}

#[test]
fn test_file_stats_type() {
    let mut stats: HashMap<String, FileStat> = HashMap::new();
    stats.insert(
        "test.rs".to_string(),
        FileStat {
            last_index: 0,
            commit_count: 10,
            last_time: Utc::now(),
        },
    );
    assert_eq!(stats.len(), 1);
    assert!(stats.contains_key("test.rs"));
}
