use colored::Colorize;

use crate::utils::Timer;
use std::collections::HashMap;
use std::fs;

pub fn run_stats(path: &str, file_type: Option<crate::types::FileType>, extension: Option<&str>, json: bool) -> Result<(), String> {
    let timer = Timer::new();
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    let mut lang_stats: HashMap<String, (usize, usize, usize)> = HashMap::new(); // (files, lines, bytes)

    let mut builder = ignore::WalkBuilder::new(path);
    builder.git_ignore(true);
    builder.git_global(true);

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        if let Some(exts) = &extensions {
            let matches = exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)));
            if !matches { continue; }
        }

        let file_path = entry.path();
        let lang = detect_language(&file_name);

        if let Ok(meta) = entry.metadata() {
            let size = meta.len() as usize;
            let lines = if let Ok(content) = fs::read_to_string(file_path) {
                content.lines().count()
            } else {
                0
            };

            let entry = lang_stats.entry(lang).or_insert((0, 0, 0));
            entry.0 += 1;
            entry.1 += lines;
            entry.2 += size;
        }
    }

    let total_lines: usize = lang_stats.values().map(|(_, l, _)| l).sum();
    let total_files: usize = lang_stats.values().map(|(f, _, _)| f).sum();

    let mut results: Vec<crate::types::LangStats> = lang_stats
        .into_iter()
        .map(|(language, (files, lines, bytes))| {
            let percentage = if total_lines > 0 { (lines as f64 / total_lines as f64) * 100.0 } else { 0.0 };
            crate::types::LangStats { language, files, lines, bytes, percentage }
        })
        .collect();

    results.sort_by(|a, b| b.lines.cmp(&a.lines));

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|stat| {
                serde_json::to_value(crate::output_schema::StatsResultItem {
                    language: stat.language.clone(),
                    files: stat.files,
                    lines: stat.lines,
                    bytes: stat.bytes as u64,
                    percentage: stat.percentage,
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope_with_extra(
            "stats", ".", "filesystem", results.len(), elapsed,
            serde_json::json!(results_json),
            serde_json::json!({"total_files": total_files, "total_lines": total_lines}),
        );
        crate::output_schema::print_json(&output);
        return Ok(());
    }

    let separator = "─".repeat(50);
    eprintln!("{} File Statistics: {}", ">>".cyan(), path);
    eprintln!("{}", separator.dimmed());
    eprintln!("  {:<15} {:>8} {:>10} {:>8}", "Language", "Files", "Lines", "%");
    eprintln!("{}", separator.dimmed());

    for stat in &results {
        let bar_len = (stat.percentage / 5.0) as usize;
        let bar = "█".repeat(bar_len.max(1));
        eprintln!("  {:<15} {:>8} {:>10} {:>6.1}% {}", stat.language.green(), stat.files, stat.lines, stat.percentage, bar.dimmed());
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("  {:<15} {:>8} {:>10}", "Total".bold(), total_files, total_lines);

    Ok(())
}

/// Collect stats for MCP/HTTP API.
pub fn collect_stats(
    path: &str,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
) -> Result<Vec<serde_json::Value>, String> {
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };
    let mut lang_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();
    let mut builder = ignore::WalkBuilder::new(path);
    builder.git_ignore(true);
    builder.git_global(true);
    for entry in builder.build() {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().map_or(false, |ft| ft.is_file()) { continue; }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some(exts) = &extensions {
            let matches = exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)));
            if !matches { continue; }
        }
        let file_path = entry.path();
        let lang = detect_language(&file_name);
        if let Ok(meta) = entry.metadata() {
            let size = meta.len() as usize;
            let lines = if let Ok(content) = fs::read_to_string(file_path) { content.lines().count() } else { 0 };
            let entry = lang_stats.entry(lang).or_insert((0, 0, 0));
            entry.0 += 1;
            entry.1 += lines;
            entry.2 += size;
        }
    }
    let total_lines: usize = lang_stats.values().map(|(_, l, _)| l).sum();
    let mut results: Vec<serde_json::Value> = lang_stats.into_iter().map(|(language, (files, lines, bytes))| {
        let percentage = if total_lines > 0 { (lines as f64 / total_lines as f64) * 100.0 } else { 0.0 };
        serde_json::json!({ "language": language, "files": files, "lines": lines, "bytes": bytes, "percentage": percentage })
    }).collect();
    results.sort_by(|a, b| b["lines"].as_u64().unwrap_or(0).cmp(&a["lines"].as_u64().unwrap_or(0)));
    Ok(results)
}

fn detect_language(filename: &str) -> String {
    let lower = filename.to_lowercase();
    if lower.ends_with(".rs") { return "Rust".to_string(); }
    if lower.ends_with(".py") || lower.ends_with(".pyi") { return "Python".to_string(); }
    if lower.ends_with(".js") || lower.ends_with(".jsx") { return "JavaScript".to_string(); }
    if lower.ends_with(".ts") || lower.ends_with(".tsx") { return "TypeScript".to_string(); }
    if lower.ends_with(".go") { return "Go".to_string(); }
    if lower.ends_with(".java") || lower.ends_with(".kt") { return "Java/Kotlin".to_string(); }
    if lower.ends_with(".c") && !lower.ends_with(".cpp") { return "C".to_string(); }
    if lower.ends_with(".cpp") || lower.ends_with(".cc") || lower.ends_with(".cxx") || lower.ends_with(".hpp") { return "C++".to_string(); }
    if lower.ends_with(".rb") { return "Ruby".to_string(); }
    if lower.ends_with(".php") { return "PHP".to_string(); }
    if lower.ends_with(".swift") { return "Swift".to_string(); }
    if lower.ends_with(".sh") || lower.ends_with(".bash") || lower.ends_with(".zsh") { return "Shell".to_string(); }
    if lower.ends_with(".html") || lower.ends_with(".htm") { return "HTML".to_string(); }
    if lower.ends_with(".css") || lower.ends_with(".scss") { return "CSS".to_string(); }
    if lower.ends_with(".toml") || lower.ends_with(".yaml") || lower.ends_with(".yml") || lower.ends_with(".json") { return "Config".to_string(); }
    if lower.ends_with(".md") || lower.ends_with(".txt") || lower.ends_with(".rst") { return "Docs".to_string(); }
    if lower.ends_with(".sql") { return "SQL".to_string(); }
    "Other".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_stats_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();

        let result = run_stats(dir.path().to_str().unwrap(), None, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), "Rust");
        assert_eq!(detect_language("app.py"), "Python");
        assert_eq!(detect_language("index.ts"), "TypeScript");
        assert_eq!(detect_language("main.go"), "Go");
    }
}
