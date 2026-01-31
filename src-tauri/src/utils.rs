use std::path::PathBuf;

/// Helper function to expand ~ to home directory
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Sanitize filename - handle UTF-8 properly
pub fn sanitize_filename(name: &str) -> String {
    let re = regex::Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    let clean = re.replace_all(name, "");
    let clean = clean.trim();

    // Use chars() to properly handle UTF-8 instead of byte slicing
    let chars: Vec<char> = clean.chars().collect();
    if chars.len() > 50 {
        chars[..50].iter().collect()
    } else {
        clean.to_string()
    }
}
