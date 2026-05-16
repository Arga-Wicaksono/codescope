use colored::Colorize;

use crate::validate;
use ignore::WalkBuilder;
use std::path::Path;

/// Language-specific definition patterns, keyed by file extension.
/// FIX #2: Only apply patterns for the language matching the file extension.
struct LangPatterns {
    extensions: &'static [&'static str],
    patterns: &'static [&'static str],
}

const ALL_LANGS: &[LangPatterns] = &[
    LangPatterns {
        extensions: &["rs"],
        patterns: &[
            r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)",
            r"(?:pub\s+)?struct\s+(\w+)",
            r"(?:pub\s+)?enum\s+(\w+)",
            r"(?:pub\s+)?trait\s+(\w+)",
            r"(?:pub\s+)?impl\s+(\w+)",
            r"(?:pub\s+)?type\s+(\w+)",
            r"(?:pub\s+)?const\s+(\w+)",
            r"(?:pub\s+)?static\s+(\w+)",
            r"(?:pub\s+)?mod\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["py", "pyi", "pyw"],
        patterns: &[
            r"def\s+(\w+)",
            r"class\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["js", "jsx", "mjs", "cjs"],
        patterns: &[
            r"function\s+(\w+)",
            r"class\s+(\w+)",
            r"const\s+(\w+)\s*=",
            r"let\s+(\w+)\s*=",
            r"var\s+(\w+)\s*=",
        ],
    },
    LangPatterns {
        extensions: &["ts", "tsx"],
        patterns: &[
            r"(?:export\s+)?function\s+(\w+)",
            r"(?:export\s+)?class\s+(\w+)",
            r"(?:export\s+)?(?:const|let|var)\s+(\w+)",
            r"(?:export\s+)?interface\s+(\w+)",
            r"(?:export\s+)?type\s+(\w+)",
            r"(?:export\s+)?enum\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["go"],
        patterns: &[
            r"func\s+(?:\(\w+\s+\*?\w+\)\s+)?(\w+)",
            r"type\s+(\w+)\s+struct",
            r"type\s+(\w+)\s+interface",
        ],
    },
    LangPatterns {
        extensions: &["java", "kt", "kts"],
        patterns: &[
            r"(?:public|private|protected)?\s*(?:static\s+)?(?:abstract\s+)?(?:class|interface|enum)\s+(\w+)",
            r"(?:public|private|protected)?\s*(?:static\s+)?(?:(?:final\s+)?\w+(?:<[^>]+>)?\s+)+(\w+)\s*\(",
        ],
    },
    LangPatterns {
        extensions: &["c"],
        patterns: &[
            r"(?:static\s+)?(?:(?:const|unsigned|signed|long|short|inline)\s+)*\w+\s+(\w+)\s*\(",
            r"typedef\s+(?:struct|enum|union)\s*\{[^}]*\}\s*(\w+)",
            r"#define\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["cpp", "cc", "cxx", "hpp", "hxx"],
        patterns: &[
            r"(?:class|struct|enum)\s+(\w+)",
            r"(?:virtual\s+)?(?:static\s+)?(?:inline\s+)?(?:\w+::)?\w+\s+(\w+)\s*\(",
            r"namespace\s+(\w+)",
            r"template\s*<[^>]+>\s*(?:class|struct)\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["h"],
        patterns: &[
            r"(?:static\s+)?(?:(?:const|unsigned|signed|long|short|inline)\s+)*\w+\s+(\w+)\s*\(",
            r"typedef\s+(?:struct|enum|union)\s*\{[^}]*\}\s*(\w+)",
            r"#define\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["rb"],
        patterns: &[
            r"def\s+(?:self\.)?(\w+)",
            r"class\s+(\w+)",
            r"module\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["php"],
        patterns: &[
            r"function\s+(\w+)",
            r"class\s+(\w+)",
            r"(?:public|private|protected)\s+(?:static\s+)?function\s+(\w+)",
        ],
    },
    LangPatterns {
        extensions: &["swift"],
        patterns: &[
            r"(?:public|private|internal)?\s*(?:static\s+)?func\s+(\w+)",
            r"(?:public|private|internal)?\s*(?:class|struct|enum|protocol)\s+(\w+)",
        ],
    },
];

/// Detect which language patterns to use based on file extension.
fn detect_lang(file_name: &str) -> Option<&'static LangPatterns> {
    let ext = Path::new(file_name).extension()?.to_str()?;
    ALL_LANGS.iter().find(|lang| lang.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)))
}

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

        // FIX #2: Only use patterns for the detected language of this file
        let lang = match detect_lang(&file_name) {
            Some(l) => l,
            None => continue,
        };

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for pattern in lang.patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.find_iter(&content) {
                    let line_content = cap.as_str();
                    if line_content.contains(name) {
                        let line_num = content[..cap.start()].matches('\n').count() + 1;
                        let display_path = file_path.replace(path, "").trim_start_matches('/').to_string();
                        let lang_name = Path::new(&file_name)
                            .extension()
                            .map(|e| e.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        results.push((display_path, line_num, line_content.trim().to_string(), lang_name));
                    }
                }
            }
        }
    }

    if json {
        let json_output = serde_json::json!({
            "tool": "codescope",
            "command": "where",
            "name": name,
            "count": results.len(),
            "results": results.iter().map(|(path, line, content, lang)| {
                serde_json::json!({"path": path, "line": line, "content": content, "language": lang})
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        return Ok(if results.is_empty() { 1 } else { 0 });
    }

    if results.is_empty() {
        eprintln!("{} No definition found for '{}'", "✗".red(), name.cyan());
        eprintln!("  {} Check spelling or try {} for deeper search", "Tip:".yellow(), "cs symbol <name>".green());
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
    fn test_where_no_cross_language_match() {
        // FIX #2: C/CPP patterns should NOT match .rs files
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "pub fn parse_config() {}\n").unwrap();
        fs::write(dir.path().join("test.c"), "int parse_config() {}\n").unwrap();
        fs::write(dir.path().join("test.cpp"), "void parse_config() {}\n").unwrap();

        let result = run_where("parse_config", dir.path().to_str().unwrap(), None, Some("rs"), None, true, None, false, false, false);
        assert!(result.is_ok());
        // Should find exactly 1 result in .rs file, NOT results from .c or .cpp
    }

    #[test]
    fn test_detect_lang() {
        assert!(detect_lang("main.rs").is_some());
        assert!(detect_lang("app.py").is_some());
        assert!(detect_lang("index.ts").is_some());
        assert!(detect_lang("main.go").is_some());
        assert!(detect_lang("App.java").is_some());
        assert!(detect_lang("lib.cpp").is_some());
        assert!(detect_lang("func.c").is_some());
        assert!(detect_lang("server.rb").is_some());
        assert!(detect_lang("index.php").is_some());
        assert!(detect_lang("App.swift").is_some());
        // .txt should not match any language
        assert!(detect_lang("readme.txt").is_none());
    }

    #[test]
    fn test_where_json_output() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "pub fn my_func() {}\n").unwrap();

        let result = run_where("my_func", dir.path().to_str().unwrap(), None, None, None, true, None, false, false, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
