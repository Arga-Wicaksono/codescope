use colored::Colorize;

use crate::utils::Timer;
use rayon::prelude::*;

pub fn run_across(
    pattern: &str,
    repos: Option<&str>,
    workspace: Option<&str>,
    repos_file: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    regex: bool,
    exact: bool,
    limit: Option<usize>,
    json: bool,
    interactive: bool,
) -> Result<i32, String> {
    crate::validate::validate_pattern(pattern)?;

    let timer = Timer::new();
    let repo_paths = resolve_repos(repos, workspace, repos_file)?;

    if repo_paths.is_empty() {
        eprintln!("{}", "No repositories found.".yellow());
        return Ok(1);
    }

    let max_per_repo = limit.unwrap_or(20);
    let mode = crate::content_search::resolve_match_mode(regex, exact);
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    eprintln!("{} Searching '{}' across {} repositories", ">>".cyan(), pattern.cyan(), repo_paths.len().to_string().green());

    let mut all_results: Vec<(String, String, usize, String, String)> = repo_paths
        .par_iter()
        .flat_map(|repo_path| {
            let repo_name = std::path::Path::new(repo_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| repo_path.clone());

            let results = crate::content_search::collect_content_results(
                pattern,
                repo_path,
                extensions.as_deref(),
                mode,
                None,
                false,
                false,
                0,
                None,
                false,
            ).unwrap_or_default();

            results.into_iter().map(move |(file, _path, line, content, _score)| {
                (repo_name.clone(), file, line, content, repo_path.clone())
            }).collect::<Vec<_>>()
        })
        .collect();

    all_results.truncate(max_per_repo * repo_paths.len());

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = all_results
            .iter()
            .map(|(repo, file, line, content, repo_path)| {
                serde_json::to_value(crate::output_schema::AcrossResultItem {
                    repo: repo.clone(),
                    file: file.clone(),
                    path: repo_path.clone(),
                    line: *line,
                    content: content.clone(),
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "across", pattern, "filesystem", all_results.len(), elapsed,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
        return Ok(if all_results.is_empty() { 1 } else { 0 });
    }

    let separator = "─".repeat(50);
    let mut current_repo = String::new();

    for (repo, file, line, content, _) in &all_results {
        if repo != &current_repo {
            current_repo = repo.clone();
            eprintln!("{}", separator.dimmed());
            eprintln!("  {} {}", "repo:".dimmed(), repo.cyan());
        }
        eprintln!("    {}:{}  {}", file.green(), line.to_string().yellow(), content);
    }

    if !all_results.is_empty() {
        eprintln!("{}", separator.dimmed());
        eprintln!("{} {} matches across {} repos", "✓".green(), all_results.len(), repo_paths.len());
    } else {
        eprintln!("{}", "No matches found across repositories.".yellow());
    }

    Ok(if all_results.is_empty() { 1 } else { 0 })
}

fn resolve_repos(repos: Option<&str>, workspace: Option<&str>, repos_file: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(repos_str) = repos {
        let paths: Vec<String> = repos_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if paths.is_empty() {
            return Err("No repository paths provided".to_string());
        }
        return Ok(paths);
    }

    if let Some(ws) = workspace {
        let mut paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir(ws) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join(".git").exists() {
                    if let Some(p) = path.to_str() {
                        paths.push(p.to_string());
                    }
                }
            }
        }
        paths.truncate(50);
        return Ok(paths);
    }

    if let Some(file) = repos_file {
        let content = std::fs::read_to_string(file).map_err(|e| format!("Cannot read repos file: {}", e))?;
        let paths: Vec<String> = content.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .collect();
        return Ok(paths);
    }

    Err("No repository source specified. Use --repos, --workspace, or --repos-file".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resolve_repos_from_list() {
        let result = resolve_repos(Some("/a,/b,/c"), None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_resolve_repos_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("repo1")).unwrap();
        std::fs::create_dir(dir.path().join("repo1/.git")).unwrap();
        std::fs::create_dir(dir.path().join("repo2")).unwrap();
        std::fs::create_dir(dir.path().join("repo2/.git")).unwrap();

        let result = resolve_repos(None, Some(dir.path().to_str().unwrap()), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
