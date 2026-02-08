//! Tests for utils::diff module

use utils::diff::{
    compute_line_change_counts, concatenate_diff_hunks, create_unified_diff,
    extract_unified_diff_hunks, normalize_unified_diff, DiffChangeKind,
};

#[test]
fn test_extract_unified_diff_hunks_basic() {
    let diff = r#"@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3
"#;
    let hunks = extract_unified_diff_hunks(diff);
    assert_eq!(hunks.len(), 1);
    assert!(hunks[0].contains("@@ -1,3 +1,3 @@"));
    assert!(hunks[0].contains("line2_modified"));
}

#[test]
fn test_extract_unified_diff_hunks_multiple() {
    let diff = r#"@@ -1,2 +1,2 @@
 line1
-line2
+line2_new
@@ -10,3 +10,3 @@
 line10
-line11
+line11_new
 line12
"#;
    let hunks = extract_unified_diff_hunks(diff);
    assert_eq!(hunks.len(), 2);
}

#[test]
fn test_extract_unified_diff_hunks_no_headers() {
    // Diff without @@ headers should create a single hunk
    let diff = r#" line1
-line2
+line2_new
 line3
"#;
    let hunks = extract_unified_diff_hunks(diff);
    assert_eq!(hunks.len(), 1);
    assert!(hunks[0].contains("@@")); // Should have generated header
}

#[test]
fn test_extract_unified_diff_hunks_empty() {
    let hunks = extract_unified_diff_hunks("");
    assert!(hunks.is_empty());
}

#[test]
fn test_concatenate_diff_hunks() {
    let hunks = vec!["@@ -1,2 +1,2 @@\n line1\n line2\n".to_string()];
    let result = concatenate_diff_hunks("test.txt", &hunks);

    assert!(result.contains("--- a/test.txt"));
    assert!(result.contains("+++ b/test.txt"));
    assert!(result.contains("@@ -1,2 +1,2 @@"));
}

#[test]
fn test_concatenate_diff_hunks_empty() {
    let hunks: Vec<String> = vec![];
    let result = concatenate_diff_hunks("test.txt", &hunks);

    assert!(result.contains("--- a/test.txt"));
    assert!(result.contains("+++ b/test.txt"));
}

#[test]
fn test_create_unified_diff() {
    let old = "line1\nline2\nline3\n";
    let new = "line1\nline2_modified\nline3\n";
    let result = create_unified_diff("test.txt", old, new);

    assert!(result.contains("--- a/test.txt"));
    assert!(result.contains("+++ b/test.txt"));
    assert!(result.contains("line2_modified"));
    assert!(result.contains("-line2"));
}

#[test]
fn test_normalize_unified_diff() {
    let diff = r#"@@ -1,3 +1,3 @@
 line1
-line2
+line2_new
 line3
"#;
    let result = normalize_unified_diff("test.txt", diff);

    assert!(result.contains("--- a/test.txt"));
    assert!(result.contains("+++ b/test.txt"));
}

#[test]
fn test_compute_line_change_counts_additions() {
    let old = "line1\nline2\n";
    let new = "line1\nline2\nline3\nline4\n";
    let (adds, dels) = compute_line_change_counts(old, new);

    assert_eq!(adds, 2);
    assert_eq!(dels, 0);
}

#[test]
fn test_compute_line_change_counts_deletions() {
    let old = "line1\nline2\nline3\n";
    let new = "line1\n";
    let (adds, dels) = compute_line_change_counts(old, new);

    assert_eq!(adds, 0);
    assert_eq!(dels, 2);
}

#[test]
fn test_compute_line_change_counts_both() {
    let old = "line1\nline2\nline3\n";
    let new = "line1\nline2_modified\nline4\n";
    let (adds, dels) = compute_line_change_counts(old, new);

    assert!(adds > 0);
    assert!(dels > 0);
}

#[test]
fn test_compute_line_change_counts_no_changes() {
    let content = "line1\nline2\nline3\n";
    let (adds, dels) = compute_line_change_counts(content, content);

    assert_eq!(adds, 0);
    assert_eq!(dels, 0);
}

#[test]
fn test_compute_line_change_counts_no_newline() {
    // Test with content that doesn't end with newline
    let old = "line1\nline2";
    let new = "line1\nline2\nline3";
    let (adds, dels) = compute_line_change_counts(old, new);

    assert!(adds > 0);
}

#[test]
fn test_diff_change_kind_variants() {
    use utils::diff::DiffChangeKind;

    // Just verify the enum variants exist and can be constructed
    let _added = DiffChangeKind::Added;
    let _deleted = DiffChangeKind::Deleted;
    let _modified = DiffChangeKind::Modified;
    let _renamed = DiffChangeKind::Renamed;
    let _copied = DiffChangeKind::Copied;
    let _perm = DiffChangeKind::PermissionChange;
}
