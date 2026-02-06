//! Tests for utils::shell module

use std::ffi::OsString;
use utils::shell::{merge_paths, UnixShell};

#[test]
fn test_merge_paths_basic() {
    let primary = OsString::from("/usr/bin:/bin");
    let secondary = OsString::from("/usr/local/bin");
    let result = merge_paths(&primary, &secondary);

    let result_str = result.to_string_lossy();
    assert!(result_str.contains("/usr/bin"));
    assert!(result_str.contains("/bin"));
    assert!(result_str.contains("/usr/local/bin"));
}

#[test]
fn test_merge_paths_deduplication() {
    // Duplicate paths should be removed
    let primary = OsString::from("/usr/bin:/bin");
    let secondary = OsString::from("/usr/bin:/usr/local/bin");
    let result = merge_paths(&primary, &secondary);

    let result_str = result.to_string_lossy();
    // /usr/bin appears in both, should only appear once (from primary)
    let occurrences = result_str.matches("/usr/bin").count();
    assert_eq!(occurrences, 1, "Duplicate paths should be removed");
}

#[test]
fn test_merge_paths_empty_components() {
    // Empty components should be ignored
    let primary = OsString::from("/usr/bin::/bin");
    let secondary = OsString::from("");
    let result = merge_paths(&primary, &secondary);

    let result_str = result.to_string_lossy();
    assert!(result_str.contains("/usr/bin"));
    assert!(result_str.contains("/bin"));
}

#[test]
fn test_merge_paths_order_preserved() {
    // Primary paths should come first
    let primary = OsString::from("/first:/second");
    let secondary = OsString::from("/third:/fourth");
    let result = merge_paths(&primary, &secondary);

    let result_str = result.to_string_lossy();
    let first_pos = result_str.find("/first").unwrap();
    let second_pos = result_str.find("/second").unwrap();
    let third_pos = result_str.find("/third").unwrap();

    assert!(first_pos < second_pos);
    assert!(second_pos < third_pos);
}

#[test]
fn test_unix_shell_zsh() {
    let shell = UnixShell::Zsh;
    assert_eq!(shell.path(), std::path::PathBuf::from("/bin/zsh"));
    assert!(shell.login());
}

#[test]
fn test_unix_shell_bash() {
    let shell = UnixShell::Bash;
    assert_eq!(shell.path(), std::path::PathBuf::from("/bin/bash"));
    assert!(shell.login());
}

#[test]
fn test_unix_shell_sh() {
    let shell = UnixShell::Sh;
    assert_eq!(shell.path(), std::path::PathBuf::from("/bin/sh"));
    assert!(!shell.login());
}

#[test]
fn test_unix_shell_other() {
    let shell = UnixShell::Other("/usr/bin/fish".to_string());
    assert_eq!(shell.path(), std::path::PathBuf::from("/usr/bin/fish"));
    assert!(!shell.login());
}

#[test]
fn test_unix_shell_from_path() {
    use std::path::Path;

    assert!(matches!(
        UnixShell::from_path(Path::new("/bin/zsh")),
        Some(UnixShell::Zsh)
    ));
    assert!(matches!(
        UnixShell::from_path(Path::new("/bin/bash")),
        Some(UnixShell::Bash)
    ));
    assert!(matches!(
        UnixShell::from_path(Path::new("/bin/sh")),
        Some(UnixShell::Sh)
    ));
    assert!(matches!(
        UnixShell::from_path(Path::new("/usr/bin/fish")),
        Some(UnixShell::Other(_))
    ));
    assert!(UnixShell::from_path(Path::new("relative/path")).is_none());
}

#[test]
fn test_unix_shell_get_shell_command() {
    let shell = UnixShell::Bash;
    let (program, arg) = shell.get_shell_command();
    assert_eq!(program, "/bin/bash");
    assert_eq!(arg, "-c");
}
