//! File content search for codescope.
use colored::Colorize;

use crate::types::MatchMode;
use crate::utils::Timer;
use crate::validate;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::io::{self, BufRead, BufReader, Read};

/// Resolve the match mode from flags.
pub fn resolve_match_mode(regex: bool, exact: bool) -> MatchMode {
    if regex {
        MatchMode::Regex
    } else if exact {
        MatchMode::Exact
    } else {
        MatchMode::Fuzzy
    }
}

/// Search content inside files.
pub fn search_content(
    pattern: &str,
    path: &str,
    extensions: Option<&[&str]>,
    mode: MatchMode,
    exclude: Option<&str>,
    case_insensitive: bool,
    no_ignore: bool,
    line_number: bool,
    context: usize,
    depth: Option<usize>,
    limit: usize,
    json: bool,
    invert: bool,
) -> Result<bool, String> {
    validate::validate_pattern(pattern)?;

    let timer = Timer::new();
    let matcher = SkimMatcherV2::default();
    let regex_pattern = if case_insensitive {
        regex::RegexBuilder::new(pattern).case_insensitive(true).build().ok()
    } else {
        regex::Regex::new(pattern).ok()
    };

    // Collect files first
    let files: Vec<String> = collect_files(path, extensions, exclude, no_ignore, depth)?;

    // Search in parallel
    let mut results: Vec<(String, String, usize, String, i64)> = files
        .par_iter()
        .flat_map(|file_path| {
            search_single_file(file_path, pattern, mode, case_insensitive, &matcher, &regex_pattern, invert)
        })
        .collect();

    // Sort by score descending
    results.sort_by(|a, b| b.4.cmp(&a.4));
    results.truncate(limit);

    let elapsed = timer.elapsed_secs();

    if json {
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|(file, path, line, content, score)| {
                serde_json::to_value(crate::output_schema::ContentResultItem {
                    file: file.clone(),
                    path: path.clone(),
                    line: *line,
                    content: content.clone(),
                    score: *score,
                    language: None,
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "content", pattern, "filesystem", results.len(), elapsed,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
    } else {
        crate::output::print_content_results(&results, pattern, path, line_number, context, elapsed);
    }

    Ok(!results.is_empty())
}

/// Search content with replace mode.
pub fn search_content_replace(
    pattern: &str,
    path: &str,
    extensions: Option<&[&str]>,
    mode: MatchMode,
    exclude: Option<&str>,
    case_insensitive: bool,
    no_ignore: bool,
    depth: Option<usize>,
    replacement: &str,
    write: bool,
    json: bool,
) -> Result<bool, String> {
    validate::validate_pattern(pattern)?;

    let timer = Timer::new();
    let files = collect_files(path, extensions, exclude, no_ignore, depth)?;
    let matcher = SkimMatcherV2::default();
    let regex_pattern = if case_insensitive {
        regex::RegexBuilder::new(pattern).case_insensitive(true).build().ok()
    } else {
        regex::Regex::new(pattern).ok()
    };

    let mut all_changes: Vec<(String, usize, String, String)> = Vec::new();

    for file_path in &files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let matches = match mode {
                MatchMode::Fuzzy => matcher.fuzzy_match(line, pattern).map_or(false, |s| s > 50),
                MatchMode::Exact => {
                    let search_line = if case_insensitive { line.to_lowercase() } else { line.to_string() };
                    let search_pat = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };
                    search_line.contains(&search_pat)
                }
                MatchMode::Regex => regex_pattern.as_ref().map_or(false, |re| re.is_match(line)),
            };

            if matches {
                let new_line = match &regex_pattern {
                    Some(re) => re.replace(line, replacement).to_string(),
                    None => {
                        let search_line = if case_insensitive { line.to_lowercase() } else { line.to_string() };
                        let search_pat = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };
                        if search_line.contains(&search_pat) {
                            line.replace(&search_pat, replacement)
                        } else {
                            line.to_string()
                        }
                    }
                };
                all_changes.push((file_path.clone(), i + 1, line.to_string(), new_line.clone()));
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_string());
            }
        }

        if write && !all_changes.is_empty() {
            let _ = fs::write(file_path, new_lines.join("\n"));
        }
    }

    let elapsed = timer.elapsed_secs();

    if json {
        let results_json: Vec<serde_json::Value> = all_changes
            .iter()
            .map(|(file, line, old, new)| {
                let file_name = std::path::Path::new(file)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| file.clone());
                serde_json::to_value(crate::output_schema::ReplaceResultItem {
                    file: file_name,
                    path: file.clone(),
                    line: *line,
                    old: old.clone(),
                    new_val: new.clone(),
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope_with_extra(
            "content", pattern, "filesystem", all_changes.len(), elapsed,
            serde_json::json!(results_json),
            serde_json::json!({"dry_run": !write}),
        );
        crate::output_schema::print_json(&output);
    } else {
        crate::output::print_replace_results(&all_changes);
    }

    Ok(!all_changes.is_empty())
}

/// Search content with count mode.
pub fn search_content_count(
    pattern: &str,
    path: &str,
    extensions: Option<&[&str]>,
    mode: MatchMode,
    exclude: Option<&str>,
    case_insensitive: bool,
    no_ignore: bool,
    depth: Option<usize>,
    json: bool,
    invert: bool,
) -> Result<bool, String> {
    validate::validate_pattern(pattern)?;

    let timer = Timer::new();
    let files = collect_files(path, extensions, exclude, no_ignore, depth)?;
    let matcher = SkimMatcherV2::default();
    let regex_pattern = if case_insensitive {
        regex::RegexBuilder::new(pattern).case_insensitive(true).build().ok()
    } else {
        regex::Regex::new(pattern).ok()
    };

    let mut counts: Vec<(String, usize)> = files
        .par_iter()
        .filter_map(|file_path| {
            let content = fs::read_to_string(file_path).ok()?;
            let mut count = 0usize;

            for line in content.lines() {
                let matches = match mode {
                    MatchMode::Fuzzy => matcher.fuzzy_match(line, pattern).map_or(false, |s| s > 50),
                    MatchMode::Exact => {
                        let search_line = if case_insensitive { line.to_lowercase() } else { line.to_string() };
                        let search_pat = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };
                        search_line.contains(&search_pat)
                    }
                    MatchMode::Regex => regex_pattern.as_ref().map_or(false, |re| re.is_match(line)),
                };
                if invert { if !matches { count += 1; } } else { if matches { count += 1; } }
            }

            if count > 0 {
                let display_name = file_path.replace(path, "").trim_start_matches('/').trim_start_matches('\\').to_string();
                Some((display_name, count))
            } else {
                None
            }
        })
        .collect();

    counts.sort_by(|a, b| b.1.cmp(&a.1));

    let elapsed = timer.elapsed_secs();

    if json {
        let results_json: Vec<serde_json::Value> = counts
            .iter()
            .map(|(file, count)| {
                serde_json::to_value(crate::output_schema::CountResultItem {
                    file: file.clone(),
                    path: file.clone(),
                    count: *count,
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "content", pattern, "filesystem", counts.len(), elapsed,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
    } else {
        crate::output::print_content_count(&counts);
    }

    Ok(!counts.is_empty())
}

/// Search piped stdin content.
pub fn search_content_stdin(
    pattern: &str,
    mode: MatchMode,
    case_insensitive: bool,
    line_number: bool,
    context: usize,
    limit: usize,
    json: bool,
    invert: bool,
    count_only: bool,
) -> Result<bool, String> {
    validate::validate_pattern(pattern)?;

    let matcher = SkimMatcherV2::default();
    let regex_pattern = if case_insensitive {
        regex::RegexBuilder::new(pattern).case_insensitive(true).build().ok()
    } else {
        regex::Regex::new(pattern).ok()
    };

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut results: Vec<(String, String, usize, String, i64)> = Vec::new();

    for (i, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        let matches = match mode {
            MatchMode::Fuzzy => matcher.fuzzy_match(&line, pattern).map_or(false, |s| s > 50),
            MatchMode::Exact => {
                let search_line = if case_insensitive { line.to_lowercase() } else { line.to_string() };
                let search_pat = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };
                search_line.contains(&search_pat)
            }
            MatchMode::Regex => regex_pattern.as_ref().map_or(false, |re| re.is_match(&line)),
        };

        if invert { if !matches { continue; } } else { if !matches { continue; } }

        results.push(("<stdin>".to_string(), "<stdin>".to_string(), i + 1, line, 100));
    }

    results.truncate(limit);

    if json {
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|(file, path, line, content, score)| {
                serde_json::to_value(crate::output_schema::ContentResultItem {
                    file: file.clone(),
                    path: path.clone(),
                    line: *line,
                    content: content.clone(),
                    score: *score,
                    language: None,
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "content", pattern, "stdin", results.len(), 0.0,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
    } else {
        for (_file, _path, line, content, _score) in &results {
            if line_number {
                eprintln!("{:>4}: {}", line, content);
            } else {
                eprintln!("{}", content);
            }
        }
    }

    Ok(!results.is_empty())
}

/// Check if stdin has data (for pipe detection).
pub fn stdin_has_data() -> bool {
    use is_terminal::IsTerminal;
    !std::io::stdin().is_terminal()
}

/// Collect files matching extension filter.
fn collect_files(
    path: &str,
    extensions: Option<&[&str]>,
    exclude: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
) -> Result<Vec<String>, String> {
    let mut builder = WalkBuilder::new(path);
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

    let files: Vec<String> = builder
        .build()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.file_type()?.is_file() {
                return None;
            }
            let path_str = entry.path().to_string_lossy().to_string();

            if let Some(exts) = extensions {
                let file_name = entry.file_name().to_string_lossy();
                let matches = exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)));
                if !matches {
                    return None;
                }
            }

            Some(path_str)
        })
        .collect();

    Ok(files)
}

/// Search a single file for content matches.
fn search_single_file(
    file_path: &str,
    pattern: &str,
    mode: MatchMode,
    case_insensitive: bool,
    matcher: &SkimMatcherV2,
    regex_pattern: &Option<regex::Regex>,
    invert: bool,
) -> Vec<(String, String, usize, String, i64)> {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let file_name = std::path::Path::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    let mut results = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let matches = match mode {
            MatchMode::Fuzzy => matcher.fuzzy_match(line, pattern).map_or(false, |s| s > 50),
            MatchMode::Exact => {
                let search_line = if case_insensitive { line.to_lowercase() } else { line.to_string() };
                let search_pat = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };
                search_line.contains(&search_pat)
            }
            MatchMode::Regex => regex_pattern.as_ref().map_or(false, |re| re.is_match(line)),
        };

        if invert {
            if matches { continue; }
        } else {
            if !matches { continue; }
        }

        let score = match mode {
            MatchMode::Fuzzy => matcher.fuzzy_match(line, pattern).unwrap_or(0),
            _ => 100,
        };

        results.push((file_name.clone(), file_path.to_string(), i + 1, line.to_string(), score));
    }

    results
}

/// Collect content results for interactive mode.
pub fn collect_content_results(
    pattern: &str,
    path: &str,
    extensions: Option<&[&str]>,
    mode: MatchMode,
    exclude: Option<&str>,
    case_insensitive: bool,
    no_ignore: bool,
    context: usize,
    depth: Option<usize>,
    invert: bool,
) -> Result<Vec<(String, String, usize, String, i64)>, String> {
    validate::validate_pattern(pattern)?;

    let matcher = SkimMatcherV2::default();
    let regex_pattern = if case_insensitive {
        regex::RegexBuilder::new(pattern).case_insensitive(true).build().ok()
    } else {
        regex::Regex::new(pattern).ok()
    };

    let files = collect_files(path, extensions, exclude, no_ignore, depth)?;

    let mut results: Vec<(String, String, usize, String, i64)> = files
        .par_iter()
        .flat_map(|file_path| {
            search_single_file(file_path, pattern, mode, case_insensitive, &matcher, &regex_pattern, invert)
        })
        .collect();

    results.sort_by(|a, b| b.4.cmp(&a.4));
    Ok(results)
}

/// Collect content results for MCP/HTTP API (raw, no validation needed).
pub fn collect_content_results_raw(
    pattern: &str,
    path: &str,
    extension: Option<&str>,
    mode: MatchMode,
    _exclude: Option<&str>,
    case_insensitive: bool,
    no_ignore: bool,
    context: usize,
    depth: Option<usize>,
    invert: bool,
    results: &mut Vec<(String, String, usize, String, i64)>,
) -> Result<(), String> {
    let matcher = SkimMatcherV2::default();
    let regex_pattern = if case_insensitive {
        regex::RegexBuilder::new(pattern).case_insensitive(true).build().ok()
    } else {
        regex::Regex::new(pattern).ok()
    };
    let extensions: Option<Vec<&str>> = extension.map(|e| vec![e]);
    let files = collect_files(path, extensions.as_deref(), None, no_ignore, depth)?;
    let mut raw: Vec<(String, String, usize, String, i64)> = files
        .par_iter()
        .flat_map(|file_path| {
            search_single_file(file_path, pattern, mode, case_insensitive, &matcher, &regex_pattern, invert)
        })
        .collect();
    raw.sort_by(|a, b| b.4.cmp(&a.4));
    results.extend(raw);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_search_content_fuzzy() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.rs");
        fs::write(&file, "fn main() {}\nfn helper() {}\n").unwrap();

        let result = search_content("main", dir.path().to_str().unwrap(), None, MatchMode::Fuzzy, None, false, true, false, 0, None, 10, false, false);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_search_content_exact() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.rs");
        fs::write(&file, "fn main() {}\nfn helper() {}\n").unwrap();

        let result = search_content("fn main", dir.path().to_str().unwrap(), None, MatchMode::Exact, None, false, true, false, 0, None, 10, false, false);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_search_content_regex() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.rs");
        fs::write(&file, "fn main() {}\nfn helper() {}\n").unwrap();

        let result = search_content("fn \\w+", dir.path().to_str().unwrap(), None, MatchMode::Regex, None, false, true, false, 0, None, 10, false, false);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_search_content_no_results() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.rs");
        fs::write(&file, "fn main() {}\n").unwrap();

        let result = search_content("zzzzz", dir.path().to_str().unwrap(), None, MatchMode::Fuzzy, None, false, true, false, 0, None, 10, false, false);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_search_content_with_extension() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("test.py"), "def main(): pass\n").unwrap();

        let result = search_content("main", dir.path().to_str().unwrap(), Some(&["py"]), MatchMode::Exact, None, false, true, false, 0, None, 10, false, false);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_resolve_match_mode() {
        assert_eq!(resolve_match_mode(false, false), MatchMode::Fuzzy);
        assert_eq!(resolve_match_mode(false, true), MatchMode::Exact);
        assert_eq!(resolve_match_mode(true, false), MatchMode::Regex);
    }

    #[test]
    fn test_search_content_count() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}\nfn helper() {}\nfn other() {}\n").unwrap();

        let result = search_content_count("fn", dir.path().to_str().unwrap(), None, MatchMode::Exact, None, false, true, None, false, false);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_search_content_invert() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}\nhello world\n").unwrap();

        let result = search_content("fn", dir.path().to_str().unwrap(), None, MatchMode::Exact, None, false, true, false, 0, None, 10, false, true);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_search_content_empty_pattern() {
        let dir = tempfile::tempdir().unwrap();
        let result = search_content("", dir.path().to_str().unwrap(), None, MatchMode::Fuzzy, None, false, true, false, 0, None, 10, false, false);
        assert!(result.is_err());
    }
}
