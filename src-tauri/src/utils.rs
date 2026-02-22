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
    // First, replace newlines/tabs/null bytes with a space (they corrupt FFmpeg paths)
    let name = name.replace('\n', " ").replace('\r', " ").replace('\t', " ").replace('\0', "");
    let name = name.trim().to_string();

    let re = regex::Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    let clean = re.replace_all(&name, "");
    let clean = clean.trim().to_string();

    // Collapse multiple spaces into one
    let re_space = regex::Regex::new(r" {2,}").unwrap();
    let clean = re_space.replace_all(&clean, " ").to_string();

    // Use chars() to properly handle UTF-8 instead of byte slicing
    let chars: Vec<char> = clean.chars().collect();
    if chars.len() > 80 {
        chars[..80].iter().collect()
    } else {
        clean
    }
}
