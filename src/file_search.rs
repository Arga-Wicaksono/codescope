//! File name search with fuzzy matching for codescope.
use colored::Colorize;

use crate::types::FileResult;
use crate::utils::Timer;
use crate::validate;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;

/// Search for files by name using fuzzy matching.
pub fn search_files(
    pattern: &str,
    path: &str,
    exclude: Option<&str>,
    extensions: Option<&[&str]>,
    hidden: bool,
    case_insensitive: bool,
    no_ignore: bool,
    depth: Option<usize>,
    limit: usize,
    json: bool,
) -> Result<bool, String> {
    validate::validate_pattern(pattern)?;

    let timer = Timer::new();
    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(String, String, i64)> = Vec::new();

    let mut builder = WalkBuilder::new(path);
    builder.hidden(!hidden);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);

    if let Some(d) = depth {
        builder.max_depth(Some(d));
    }

    if let Some(exclude_dirs) = exclude {
        for dir in exclude_dirs.split(',') {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                builder.add_custom_ignore_filename(trimmed);
            }
        }
    }

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        // Extension filter
        if let Some(exts) = extensions {
            let matches_ext = exts.iter().any(|ext| {
                file_name.ends_with(&format!(".{}", ext))
            });
            if !matches_ext {
                continue;
            }
        }

        let search_name = if case_insensitive {
            file_name.to_lowercase()
        } else {
            file_name.clone()
        };

        let search_pattern = if case_insensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        if let Some(score) = matcher.fuzzy_match(&search_name, &search_pattern) {
            let full_path = entry.path().to_string_lossy().to_string();
            results.push((file_name, full_path, score));
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.2.cmp(&a.2));
    results.truncate(limit);

    let elapsed = timer.elapsed_secs();

    if json {
        use std::path::Path;
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|(filename, full_path, score)| {
                let ext = Path::new(full_path)
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();
                let size = std::fs::metadata(full_path).map(|m| m.len()).unwrap_or(0);
                serde_json::to_value(crate::output_schema::FileResultItem {
                    filename: filename.clone(),
                    path: full_path.clone(),
                    score: *score,
                    extension: ext,
                    size_bytes: size,
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "file", pattern, "filesystem", results.len(), elapsed,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
    } else {
        crate::output::print_file_results(&results, pattern, path, elapsed);
    }

    Ok(!results.is_empty())
}

/// Collect file results for interactive mode.
pub fn collect_file_results(
    pattern: &str,
    path: &str,
    exclude: Option<&str>,
    extensions: Option<&[&str]>,
    hidden: bool,
    case_insensitive: bool,
    no_ignore: bool,
    depth: Option<usize>,
) -> Result<Vec<(String, String, i64)>, String> {
    validate::validate_pattern(pattern)?;

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(String, String, i64)> = Vec::new();

    let mut builder = WalkBuilder::new(path);
    builder.hidden(!hidden);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);

    if let Some(d) = depth {
        builder.max_depth(Some(d));
    }

    if let Some(exclude_dirs) = exclude {
        for dir in exclude_dirs.split(',') {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                builder.add_custom_ignore_filename(trimmed);
            }
        }
    }

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        if let Some(exts) = extensions {
            let matches_ext = exts.iter().any(|ext| {
                file_name.ends_with(&format!(".{}", ext))
            });
            if !matches_ext {
                continue;
            }
        }

        let search_name = if case_insensitive {
            file_name.to_lowercase()
        } else {
            file_name.clone()
        };

        let search_pattern = if case_insensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        if let Some(score) = matcher.fuzzy_match(&search_name, &search_pattern) {
            let full_path = entry.path().to_string_lossy().to_string();
            results.push((file_name, full_path, score));
        }
    }

    results.sort_by(|a, b| b.2.cmp(&a.2));
    Ok(results)
}

/// Collect file results for MCP/HTTP API (raw, no validation needed).
pub fn collect_file_results_raw(
    pattern: &str,
    path: &str,
    _exclude: Option<&str>,
    extension: Option<&str>,
    _hidden: bool,
    case_insensitive: bool,
    no_ignore: bool,
    _depth: Option<usize>,
    results: &mut Vec<(String, String, i64)>,
) -> Result<(), String> {
    let matcher = SkimMatcherV2::default();
    let mut builder = WalkBuilder::new(path);
    builder.hidden(false);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);

    for entry in builder.build() {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().map_or(false, |ft| ft.is_file()) { continue; }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some(ext) = extension {
            if !file_name.ends_with(&format!(".{}", ext)) { continue; }
        }
        let search_name = if case_insensitive { file_name.to_lowercase() } else { file_name.clone() };
        let search_pattern = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };
        if let Some(score) = matcher.fuzzy_match(&search_name, &search_pattern) {
            let full_path = entry.path().to_string_lossy().to_string();
            results.push((file_name, full_path, score));
        }
    }
    results.sort_by(|a, b| b.2.cmp(&a.2));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_search_files_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("config.rs"), "struct Config {}").unwrap();
        fs::write(dir.path().join("README.md"), "# hello").unwrap();

        let result = search_files("main", dir.path().to_str().unwrap(), None, None, false, false, true, None, 10, false);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_search_files_with_extension() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("main.py"), "def main(): pass").unwrap();

        let result = search_files("main", dir.path().to_str().unwrap(), None, Some(&["rs"]), false, false, true, None, 10, false);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_search_files_no_results() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

        let result = search_files("zzzzzz", dir.path().to_str().unwrap(), None, None, false, false, true, None, 10, false);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_search_files_empty_pattern() {
        let dir = tempfile::tempdir().unwrap();
        let result = search_files("", dir.path().to_str().unwrap(), None, None, false, false, true, None, 10, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_files_json_output() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

        let result = search_files("main", dir.path().to_str().unwrap(), None, None, false, false, true, None, 10, true);
        assert!(result.is_ok());
    }
}
