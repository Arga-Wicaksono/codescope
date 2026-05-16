//! Utility functions for codescope.

use std::time::Instant;

/// A simple timer for measuring elapsed time.
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }
}

/// Resolve case sensitivity based on pattern content and flags.
/// Returns true if search should be case-insensitive.
pub fn resolve_case_insensitive(pattern: &str, force_insensitive: bool, force_sensitive: bool) -> bool {
    if force_sensitive {
        return false;
    }
    if force_insensitive {
        return true;
    }
    // Smart case: insensitive unless pattern has uppercase
    !pattern.chars().any(|c| c.is_uppercase())
}

/// Truncate a string to a maximum length, adding "..." if truncated.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Get the home directory path.
pub fn home_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_case_insensitive_smart() {
        assert!(resolve_case_insensitive("hello", false, false));
        assert!(!resolve_case_insensitive("Hello", false, false));
        assert!(resolve_case_insensitive("hello", true, false));
        assert!(!resolve_case_insensitive("hello", false, true));
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
        assert_eq!(truncate_str("hi", 5), "hi");
    }
}
