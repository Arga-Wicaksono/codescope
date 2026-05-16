use colored::Colorize;

use crate::utils::Timer;
use crate::validate;
use ignore::WalkBuilder;

const DEFINITION_PATTERNS: &[(&str, &str)] = &[
    ("rust", r"(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|trait|impl|type|const|static|mod)\s+(\w+)"),
    ("python", r"(?:def|class)\s+(\w+)"),
    ("javascript", r"(?:function|class|const|let|var)\s+(\w+)|(\w+)\s*=\s*(?:function|class|\()"),
    ("go", r"func\s+(?:\(\w+\s+\*?\w+\)\s+)?(\w+)"),
    ("java", r"(?:public|private|protected|static)?\s*(?:class|interface|enum|abstract\s+class)\s+(\w+)"),
    ("c", r"(?:static\s+)?(?:\w+\s+)+(\w+)\s*\("),
    ("cpp", r"(?:(?:class|struct|enum|namespace|template)\s*<?\s*(?:\w+\s*::\s*)*|(?:(?:virtual\s+)?(?:static\s+)?(?:inline\s+)?(?:\w+::)?\w+\s+(\w+))\s*\()"),
];

pub fn run_where(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    interactive: bool,
    open: bool,
    json: bool,
) -> Result<i32, String> {
    validate::validate_pattern(name)?;

    let timer = Timer::new();
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

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

    let mut results: Vec<(String, usize, String, String)> = Vec::new();

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_path = entry.path().to_string_lossy().to_string();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if let Some(exts) = &extensions {
            let matches = exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)));
            if !matches { continue; }
        }

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (lang, pattern) in DEFINITION_PATTERNS {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.find_iter(&content) {
                    let line_content = cap.as_str();
                    if line_content.contains(name) {
                        let line_num = content[..cap.start()].matches('\n').count() + 1;
                        let display_path = file_path.replace(path, "").trim_start_matches('/').to_string();
                        results.push((display_path, line_num, line_content.trim().to_string(), lang.to_string()));
                    }
                }
            }
        }
    }

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = results
            .iter()
            .map(|(path, line, content, lang)| {
                let kind = infer_kind(content);
                serde_json::to_value(crate::output_schema::WhereResultItem {
                    path: path.clone(),
                    line: *line,
                    content: content.clone(),
                    language: lang.clone(),
                    kind,
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "where", name, "filesystem", results.len(), elapsed,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
        return Ok(if results.is_empty() { 1 } else { 0 });
    }

    if results.is_empty() {
        eprintln!("{} No definition found for '{}'", "✗".red(), name.cyan());
        return Ok(1);
    }

    let separator = "─".repeat(50);
    eprintln!("{} Definitions of '{}'", ">>".cyan(), name.cyan());
    eprintln!("{}", separator.dimmed());

    for (path, line, content, lang) in &results {
        eprintln!("  {}:{}  {}", path.cyan(), line.to_string().yellow(), content.green());
        eprintln!("  {}", format!("[{}]", lang).dimmed());
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} definition(s) found", "✓".green(), results.len());

    if open && !results.is_empty() {
        let (path, line, _, _) = &results[0];
        let full_path = format!("{}/{}", path.trim_start_matches('/'), "");
        let editor = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")).unwrap_or_else(|_| "vim".to_string());
        let _ = std::process::Command::new(&editor).arg(format!("+{}", line)).arg(&full_path).status();
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_where_rust_definition() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "pub fn search_files() {}\nfn helper() {}\n").unwrap();

        let result = run_where("search_files", dir.path().to_str().unwrap(), None, None, None, true, None, false, false, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_where_not_found() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {}\n").unwrap();

        let result = run_where("nonexistent", dir.path().to_str().unwrap(), None, None, None, true, None, false, false, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_infer_kind() {
        assert_eq!(infer_kind("pub fn hello() {}"), "function");
        assert_eq!(infer_kind("fn world() {}"), "function");
        assert_eq!(infer_kind("def foo(): pass"), "function");
        assert_eq!(infer_kind("func Bar() {}"), "function");
        assert_eq!(infer_kind("struct Config {}"), "struct");
        assert_eq!(infer_kind("class MyClass {}"), "class");
        assert_eq!(infer_kind("enum Color {}"), "enum");
        assert_eq!(infer_kind("trait Printable {}"), "trait");
        assert_eq!(infer_kind("impl Foo {}"), "impl");
        assert_eq!(infer_kind("let x = 5"), "unknown");
    }
}

/// Infer the kind of a code definition from its content.
fn infer_kind(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub async fn ")
        || trimmed.starts_with("async fn ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("func ")
        || trimmed.starts_with("function ")
    {
        "function".to_string()
    } else if trimmed.contains("struct ") {
        "struct".to_string()
    } else if trimmed.contains("class ") {
        "class".to_string()
    } else if trimmed.contains("enum ") {
        "enum".to_string()
    } else if trimmed.contains("trait ") {
        "trait".to_string()
    } else if trimmed.contains("impl ") {
        "impl".to_string()
    } else {
        "unknown".to_string()
    }
}
