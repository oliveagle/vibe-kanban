use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use globwalk::GlobWalkerBuilder;
use services::services::container::ContainerError;

/// Normalize pattern for cross-platform glob matching (convert backslashes to forward slashes)
pub fn normalize_pattern(pattern: &str) -> String {
    pattern.replace('\\', "/")
}

/// Copy project files from source to target directory based on glob patterns.
/// Skips files that already exist at target with same size.
pub fn copy_project_files_impl(
    source_dir: &Path,
    target_dir: &Path,
    copy_files: &str,
) -> Result<(), ContainerError> {
    let patterns: Vec<&str> = copy_files
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Track files to avoid duplicates
    let mut seen = HashSet::new();

    for pattern in patterns {
        let pattern = normalize_pattern(pattern);
        let pattern_path = source_dir.join(&pattern);

        if pattern_path.is_file() {
            if let Err(e) = copy_single_file(&pattern_path, source_dir, target_dir, &mut seen) {
                tracing::warn!(
                    "Failed to copy file {} (from {}): {}",
                    pattern,
                    pattern_path.display(),
                    e
                );
            }
            continue;
        }

        let glob_pattern = if pattern_path.is_dir() {
            // For directories, append /** to match all contents recursively
            format!("{pattern}/**")
        } else {
            pattern.clone()
        };

        let walker = match GlobWalkerBuilder::from_patterns(source_dir, &[&glob_pattern])
            .file_type(globwalk::FileType::FILE)
            .build()
        {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!("Invalid glob pattern '{glob_pattern}': {e}");
                continue;
            }
        };

        for entry in walker.flatten() {
            if let Err(e) = copy_single_file(entry.path(), source_dir, target_dir, &mut seen) {
                tracing::warn!("Failed to copy file {:?}: {e}", entry.path());
            }
        }
    }

    Ok(())
}

fn copy_single_file(
    source_file: &Path,
    source_root: &Path,
    target_root: &Path,
    seen: &mut HashSet<PathBuf>,
) -> Result<bool, ContainerError> {
    let canonical_source = source_root.canonicalize()?;
    let canonical_file = source_file.canonicalize()?;
    // Validate path is within source_dir
    if !canonical_file.starts_with(canonical_source) {
        return Err(ContainerError::Other(anyhow!(
            "File {source_file:?} is outside project directory"
        )));
    }

    if !seen.insert(canonical_file.clone()) {
        return Ok(false);
    }

    let relative_path = source_file.strip_prefix(source_root).map_err(|e| {
        ContainerError::Other(anyhow!(
            "Failed to get relative path for {source_file:?}: {e}"
        ))
    })?;

    let target_file = target_root.join(relative_path);

    if target_file.exists() {
        return Ok(false);
    }

    if let Some(parent) = target_file.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source_file, &target_file)?;

    Ok(true)
}
