//! Tests for utils::path module

use std::path::PathBuf;
use utils::path::{
    expand_tilde, get_vibe_kanban_temp_dir, make_path_relative, normalize_macos_private_alias,
    VIBE_IMAGES_DIR,
};

#[test]
fn test_make_path_relative_already_relative() {
    // Test with relative path (should remain unchanged)
    assert_eq!(
        make_path_relative("src/main.rs", "/tmp/test-worktree"),
        "src/main.rs"
    );
}

#[test]
fn test_make_path_relative_absolute_path() {
    // Test with absolute path (should become relative if possible)
    let test_worktree = "/tmp/test-worktree";
    let absolute_path = format!("{test_worktree}/src/main.rs");
    let result = make_path_relative(&absolute_path, test_worktree);
    assert_eq!(result, "src/main.rs");
}

#[test]
fn test_make_path_relative_outside_worktree() {
    // Test with path outside worktree (should return original)
    assert_eq!(
        make_path_relative("/other/path/file.js", "/tmp/test-worktree"),
        "/other/path/file.js"
    );
}

#[test]
fn test_make_path_relative_empty_result() {
    // Test when path equals worktree (should return ".")
    let test_worktree = "/tmp/test-worktree";
    let result = make_path_relative(test_worktree, test_worktree);
    assert_eq!(result, ".");
}

#[cfg(target_os = "macos")]
#[test]
fn test_make_path_relative_macos_private_alias() {
    // Simulate a worktree under /var with a path reported under /private/var
    let worktree = "/var/folders/zz/abc123/T/vibe-kanban-dev/worktrees/vk-test";
    let path_under_private = format!(
        "/private/var{}/hello-world.txt",
        worktree.strip_prefix("/var").unwrap()
    );
    assert_eq!(
        make_path_relative(&path_under_private, worktree),
        "hello-world.txt"
    );

    // Also handle the inverse: worktree under /private and path under /var
    let worktree_private = format!("/private{worktree}");
    let path_under_var = format!("{worktree}/hello-world.txt");
    assert_eq!(
        make_path_relative(&path_under_var, &worktree_private),
        "hello-world.txt"
    );
}

#[test]
fn test_normalize_macos_private_alias_var() {
    if cfg!(target_os = "macos") {
        assert_eq!(
            normalize_macos_private_alias("/private/var"),
            PathBuf::from("/var")
        );
        assert_eq!(
            normalize_macos_private_alias("/private/var/test"),
            PathBuf::from("/var/test")
        );
    }
}

#[test]
fn test_normalize_macos_private_alias_tmp() {
    if cfg!(target_os = "macos") {
        assert_eq!(
            normalize_macos_private_alias("/private/tmp"),
            PathBuf::from("/tmp")
        );
        assert_eq!(
            normalize_macos_private_alias("/private/tmp/test"),
            PathBuf::from("/tmp/test")
        );
    }
}

#[test]
fn test_normalize_macos_private_alias_no_change() {
    // Paths that should not be changed
    assert_eq!(
        normalize_macos_private_alias("/var/test"),
        PathBuf::from("/var/test")
    );
    assert_eq!(
        normalize_macos_private_alias("/tmp/test"),
        PathBuf::from("/tmp/test")
    );
    assert_eq!(
        normalize_macos_private_alias("/home/user"),
        PathBuf::from("/home/user")
    );
}

#[test]
fn test_vibe_images_dir_constant() {
    assert_eq!(VIBE_IMAGES_DIR, ".vibe-images");
}

#[test]
fn test_get_vibe_kanban_temp_dir() {
    let temp_dir = get_vibe_kanban_temp_dir();

    // Should contain vibe-kanban in the path
    let path_str = temp_dir.to_string_lossy();
    assert!(
        path_str.contains("vibe-kanban") || path_str.contains("vibe-kanban-dev"),
        "Temp dir should contain vibe-kanban: {}",
        path_str
    );
}

#[test]
fn test_expand_tilde_home() {
    let result = expand_tilde("~/documents");
    // On Unix-like systems, this should expand to home directory
    assert!(!result.to_string_lossy().starts_with('~'));
}

#[test]
fn test_expand_tilde_no_tilde() {
    let result = expand_tilde("/absolute/path");
    assert_eq!(result, PathBuf::from("/absolute/path"));
}
