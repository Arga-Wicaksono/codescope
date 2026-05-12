use colored::Colorize;

use crate::utils::Timer;
use crate::validate;
use std::process::Command;

pub fn run_open(
    pattern: &str,
    path: &str,
    exclude: Option<&str>,
    extensions: Option<&[&str]>,
    hidden: bool,
    case_insensitive: bool,
    no_ignore: bool,
    depth: Option<usize>,
    interactive: bool,
    line: Option<usize>,
    json: bool,
) -> Result<i32, String> {
    validate::validate_pattern(pattern)?;

    let results = crate::file_search::collect_file_results(pattern, path, exclude, extensions, hidden, case_insensitive, no_ignore, depth)?;

    if results.is_empty() {
        eprintln!("{}", "No files found.".yellow());
        return Ok(1);
    }

    let (filename, full_path, score) = &results[0];

    if json {
        let json_output = serde_json::json!({
            "tool": "codescope",
            "action": "open",
            "file": filename,
            "path": full_path,
            "score": score,
            "line": line,
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        return Ok(0);
    }

    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vim".to_string());

    if let Some(ln) = line {
        let _ = Command::new(&editor)
            .arg(format!("+{}", ln))
            .arg(full_path)
            .status();
    } else {
        let _ = Command::new(&editor)
            .arg(full_path)
            .status();
    }

    eprintln!("{} Opened {} in {}", "✓".green(), full_path.green(), editor.cyan());
    Ok(0)
}
