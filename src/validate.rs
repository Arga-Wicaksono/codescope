//! Input validation for codescope.

/// Validate a search pattern is not empty.
pub fn validate_pattern(pattern: &str) -> Result<(), String> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Err("Search pattern cannot be empty".to_string());
    }
    if trimmed.len() > 10_000 {
        return Err("Search pattern is too long (max 10,000 characters)".to_string());
    }
    Ok(())
}

/// Validate a directory path exists.
pub fn validate_path(path: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);
    if !p.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    if !p.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pattern() {
        assert!(validate_pattern("hello").is_ok());
        assert!(validate_pattern("  hello  ").is_ok());
        assert!(validate_pattern("").is_err());
        assert!(validate_pattern("   ").is_err());
    }
}
