pub fn is_valid_branch_prefix(prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }

    if prefix.contains('/') {
        return false;
    }

    git2::Branch::name_is_valid(&format!("{prefix}/x")).unwrap_or_default()
}
