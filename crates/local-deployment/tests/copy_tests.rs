//! Tests for local_deployment::copy module

use std::fs;

use tempfile::TempDir;

use local_deployment::copy::copy_project_files_impl;

#[test]
fn test_copy_project_files_mixed_patterns() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    fs::write(source_dir.path().join(".env"), "secret").unwrap();
    fs::write(source_dir.path().join("config.json"), "{}").unwrap();

    let src_dir = source_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "code").unwrap();
    fs::write(src_dir.join("lib.rs"), "lib").unwrap();

    let config_dir = source_dir.path().join("config");
    fs::create_dir(&config_dir).unwrap();
    fs::write(config_dir.join("app.toml"), "config").unwrap();

    copy_project_files_impl(
        source_dir.path(),
        target_dir.path(),
        ".env, *.json, src, config",
    )
    .unwrap();

    assert!(target_dir.path().join(".env").exists());
    assert!(target_dir.path().join("config.json").exists());
    assert!(target_dir.path().join("src/main.rs").exists());
    assert!(target_dir.path().join("src/lib.rs").exists());
    assert!(target_dir.path().join("config/app.toml").exists());
}

#[test]
fn test_copy_project_files_nonexistent_pattern_ok() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    let result =
        copy_project_files_impl(source_dir.path(), target_dir.path(), "nonexistent.txt");

    assert!(result.is_ok());
    assert!(!target_dir.path().join("nonexistent.txt").exists());
}

#[test]
fn test_copy_project_files_empty_pattern_ok() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    let result = copy_project_files_impl(source_dir.path(), target_dir.path(), "");

    assert!(result.is_ok());
    assert_eq!(fs::read_dir(target_dir.path()).unwrap().count(), 0);
}

#[test]
fn test_copy_project_files_whitespace_handling() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    fs::write(source_dir.path().join("test.txt"), "content").unwrap();

    copy_project_files_impl(source_dir.path(), target_dir.path(), "  test.txt  ,  ").unwrap();

    assert!(target_dir.path().join("test.txt").exists());
}

#[test]
fn test_copy_project_files_nested_directory() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    let config_dir = source_dir.path().join("config");
    fs::create_dir(&config_dir).unwrap();
    fs::write(config_dir.join("app.json"), "{}").unwrap();

    let nested_dir = config_dir.join("nested");
    fs::create_dir(&nested_dir).unwrap();
    fs::write(nested_dir.join("deep.txt"), "deep").unwrap();

    copy_project_files_impl(source_dir.path(), target_dir.path(), "config").unwrap();

    assert!(target_dir.path().join("config/app.json").exists());
    assert!(target_dir.path().join("config/nested/deep.txt").exists());
}

#[test]
fn test_copy_project_files_outside_source_skips_without_copying() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Create file outside of source directory (one level up)
    let parent_dir = source_dir.path().parent().unwrap().to_path_buf();
    let outside_file = parent_dir.join("secret.txt");
    fs::write(&outside_file, "secret").unwrap();

    // Pattern referencing parent directory should resolve to outside_file and be rejected
    let result = copy_project_files_impl(source_dir.path(), target_dir.path(), "../secret.txt");

    assert!(result.is_ok());
    assert_eq!(fs::read_dir(target_dir.path()).unwrap().count(), 0);
}

#[test]
fn test_copy_project_files_recursive_glob_extension_filter() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Create nested directory structure with YAML files
    let config_dir = source_dir.path().join("config");
    fs::create_dir(&config_dir).unwrap();
    fs::write(config_dir.join("app.yml"), "app: config").unwrap();
    fs::write(config_dir.join("db.json"), "{}").unwrap();

    let nested_dir = config_dir.join("nested");
    fs::create_dir(&nested_dir).unwrap();
    fs::write(nested_dir.join("settings.yml"), "settings: value").unwrap();
    fs::write(nested_dir.join("other.txt"), "text").unwrap();

    let deep_dir = nested_dir.join("deep");
    fs::create_dir(&deep_dir).unwrap();
    fs::write(deep_dir.join("deep.yml"), "deep: config").unwrap();

    // Copy all YAML files recursively
    copy_project_files_impl(source_dir.path(), target_dir.path(), "config/**/*.yml").unwrap();

    // Verify only YAML files are copied
    assert!(target_dir.path().join("config/app.yml").exists());
    assert!(
        target_dir
            .path()
            .join("config/nested/settings.yml")
            .exists()
    );
    assert!(
        target_dir
            .path()
            .join("config/nested/deep/deep.yml")
            .exists()
    );

    // Verify non-YAML files are not copied
    assert!(!target_dir.path().join("config/db.json").exists());
    assert!(!target_dir.path().join("config/nested/other.txt").exists());
}

#[test]
fn test_copy_project_files_duplicate_patterns_ok() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Create source files
    let src_dir = source_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "lib code").unwrap();
    fs::write(src_dir.join("main.rs"), "main code").unwrap();

    // Copy with overlapping patterns: glob and specific file
    copy_project_files_impl(source_dir.path(), target_dir.path(), "src/*.rs, src/lib.rs")
        .unwrap();

    // Verify file exists once (deduplication works)
    let target_file = target_dir.path().join("src/lib.rs");
    assert!(target_file.exists());
    assert_eq!(fs::read_to_string(target_file).unwrap(), "lib code");

    // Verify other file from glob is also copied
    assert!(target_dir.path().join("src/main.rs").exists());
}

#[test]
fn test_copy_project_files_single_file_path() {
    let source_dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Create source file
    let src_dir = source_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), "library code").unwrap();

    // Copy single file by exact path (exercises fast path)
    copy_project_files_impl(source_dir.path(), target_dir.path(), "src/lib.rs").unwrap();

    // Verify file is copied
    let target_file = target_dir.path().join("src/lib.rs");
    assert!(target_file.exists());
    assert_eq!(fs::read_to_string(target_file).unwrap(), "library code");
}

#[cfg(unix)]
#[test]
fn test_symlink_loop_is_skipped() {
    use std::os::unix::fs::symlink;
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();

    let loop_dir = src.path().join("loop");
    std::fs::create_dir(&loop_dir).unwrap();
    symlink(".", loop_dir.join("self")).unwrap(); // loop/self -> loop

    copy_project_files_impl(src.path(), dst.path(), "loop").unwrap();

    assert_eq!(std::fs::read_dir(dst.path()).unwrap().count(), 0);
}

#[test]
fn test_normalize_pattern_backslashes() {
    // Test pattern normalization
    let pattern = "src\\**\\*.rs";
    let normalized = pattern.replace('\\', "/");
    assert_eq!(normalized, "src/**/*.rs");
}
