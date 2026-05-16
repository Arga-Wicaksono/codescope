//! Input validation for codescope with helpful error messages.

/// Available command names for suggestion matching.
const COMMAND_NAMES: &[&str] = &[
    "file", "content", "web", "open", "recent", "where", "explain",
    "history", "across", "stats", "completions", "config", "symbol",
    "refs", "callers", "symbols", "context", "pack", "trace",
    "graph", "impact", "serve", "semantic", "cache", "rewrite",
    "lsp-bridge", "tui", "schema",
];

/// Validate a search pattern is not empty.
pub fn validate_pattern(pattern: &str) -> Result<(), String> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Err("Search pattern cannot be empty. Example: cs file \"config\"".to_string());
    }
    if trimmed.len() > 10_000 {
        return Err(format!(
            "Search pattern is too long ({} chars, max 10,000). Try a shorter, more specific query.",
            trimmed.len()
        ));
    }
    Ok(())
}

/// Validate a directory path exists.
pub fn validate_path(path: &str) -> Result<(), String> {
    let p = std::path::Path::new(path);
    if !p.exists() {
        // FIX #3: Suggest similar paths
        let suggestion = suggest_path(path);
        match suggestion {
            Some(sug) => return Err(format!(
                "Path does not exist: {}\n  {} Did you mean: {}?",
                path, "Tip:".yellow().to_string(), sug
            )),
            None => return Err(format!(
                "Path does not exist: {}\n  {} Check the path and try again.",
                path, "Tip:".yellow().to_string()
            )),
        }
    }
    if !p.is_dir() {
        return Err(format!(
            "Path is not a directory: {}\n  {} Use 'cs file \"{}\"' to search for this file.",
            path, "Tip:".yellow().to_string(), p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
        ));
    }
    Ok(())
}

/// Suggest a command name similar to the given input.
pub fn suggest_command(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let mut best_match: Option<String> = None;
    let mut best_score = 0;

    for &cmd in COMMAND_NAMES {
        let score = levenshtein_similarity(&input_lower, cmd);
        if score > best_score && score >= 0.4 {
            best_score = score;
            best_match = Some(cmd.to_string());
        }
    }

    best_match
}

/// Provide a helpful error message for unknown commands.
pub fn unknown_command_help(input: &str) -> String {
    let suggestion = suggest_command(input);
    match suggestion {
        Some(cmd) => format!(
            "Unknown command: '{}'\n  {} Did you mean: {}?\n  Run {} to see all commands.",
            input,
            "Hint:".to_string(),
            cmd.green().to_string(),
            "cs help".green().to_string()
        ),
        None => format!(
            "Unknown command: '{}'\n  Run {} to see available commands.",
            input,
            "cs help".green().to_string()
        ),
    }
}

/// Suggest a similar path that exists.
fn suggest_path(path: &str) -> Option<String> {
    let p = std::path::Path::new(path);
    let parent = p.parent()?;

    if !parent.exists() {
        return None;
    }

    let target_name = p.file_name()?.to_str()?;
    let mut best_match: Option<String> = None;
    let mut best_score = 0.0;

    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                let score = levenshtein_similarity(target_name, name);
                if score > best_score && score >= 0.5 {
                    best_score = score;
                    best_match = Some(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }

    best_match
}

/// Compute similarity between two strings using normalized Levenshtein distance.
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let a_len = a.chars().count();
    let b_len = b.chars().count();
    let max_len = a_len.max(b_len);

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    let distance = matrix[a_len][b_len];
    1.0 - (distance as f64 / max_len as f64)
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

    #[test]
    fn test_validate_pattern_too_long() {
        let long_pattern = "a".repeat(10_001);
        let result = validate_pattern(&long_pattern);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too long"));
    }

    #[test]
    fn test_suggest_command() {
        assert_eq!(suggest_command("flie"), Some("file".to_string()));
        assert_eq!(suggest_command("simbol"), Some("symbol".to_string()));
        assert_eq!(suggest_command("content"), Some("content".to_string()));
        assert_eq!(suggest_command("xyz"), None);
    }

    #[test]
    fn test_unknown_command_help() {
        let help = unknown_command_help("flie");
        assert!(help.contains("Did you mean"));
        assert!(help.contains("file"));

        let help2 = unknown_command_help("xyz");
        assert!(help2.contains("cs help"));
        assert!(!help2.contains("Did you mean"));
    }

    #[test]
    fn test_levenshtein_similarity() {
        assert!((levenshtein_similarity("file", "file") - 1.0).abs() < 0.001);
        assert!((levenshtein_similarity("file", "flie") - 0.75).abs() < 0.01);
        assert!(levenshtein_similarity("", "file") < 0.01);
        assert!(levenshtein_similarity("xyz", "abc") < 0.01);
    }

    #[test]
    fn test_validate_path_not_exists() {
        let result = validate_path("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }
}
