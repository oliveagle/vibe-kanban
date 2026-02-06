use regex::Regex;
use uuid::Uuid;

pub fn git_branch_id(input: &str) -> String {
    // 1. lowercase
    let lower = input.to_lowercase();

    // 2. replace non-alphanumerics with hyphens
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    let slug = re.replace_all(&lower, "-");

    // 3. trim extra hyphens
    let trimmed = slug.trim_matches('-');

    // 4. take up to 16 chars, then trim trailing hyphens again
    let cut: String = trimmed.chars().take(16).collect();
    cut.trim_end_matches('-').to_string()
}

pub fn short_uuid(u: &Uuid) -> String {
    // to_simple() gives you a 32-char hex string with no hyphens
    let full = u.simple().to_string();
    full.chars().take(4).collect() // grab the first 4 chars
}

pub fn truncate_to_char_boundary(content: &str, max_len: usize) -> &str {
    if content.len() <= max_len {
        return content;
    }

    let cutoff = content
        .char_indices()
        .map(|(idx, _)| idx)
        .chain(std::iter::once(content.len()))
        .take_while(|&idx| idx <= max_len)
        .last()
        .unwrap_or(0);

    debug_assert!(content.is_char_boundary(cutoff));
    &content[..cutoff]
}
