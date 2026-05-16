//! AI Rewrite feature — combines context extraction with LLM API calls.
//!
//! Feature Request #4: "AI Rewrite via CLI - combining cs pack with LLM API calls"
//!
//! Collects context from the repository, packs it into a prompt with the rewrite
//! instruction, calls an LLM API (Ollama or OpenAI-compatible), and displays or
//! applies the suggested changes.

use colored::Colorize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;

use crate::utils::Timer;
use crate::validate;

/// A parsed change from the LLM response.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub file: String,
    pub old_content: String,
    pub new_content: String,
}

/// Configuration for the LLM provider, resolved from env vars and CLI flags.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub model: String,
    pub api_base: String,
    pub provider: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: std::env::var("CODESCOPE_LLM_MODEL").unwrap_or_else(|_| "llama3".to_string()),
            api_base: std::env::var("CODESCOPE_LLM_API")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            provider: std::env::var("CODESCOPE_LLM_PROVIDER")
                .unwrap_or_else(|_| "ollama".to_string()),
        }
    }
}

/// Main entry point for the `cs rewrite` command.
///
/// Collects context from the repository, calls an LLM, and optionally applies changes.
///
/// # Arguments
/// * `instruction` - The rewrite instruction (e.g., "add error handling to all functions")
/// * `path` - Directory to search (default: ".")
/// * `symbol` - Optional symbol to narrow context
/// * `file_type` - Optional file type filter
/// * `extension` - Optional extension filter
/// * `no_ignore` - Whether to respect .gitignore
/// * `depth` - Maximum directory depth
/// * `model` - Override LLM model name
/// * `budget` - Maximum context size (number of files)
/// * `dry_run` - Show changes without applying
/// * `json` - Output as JSON
pub fn run_rewrite(
    instruction: &str,
    path: &str,
    symbol: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    model: Option<&str>,
    budget: Option<usize>,
    dry_run: bool,
    write: bool,
    json: bool,
) -> Result<i32, String> {
    validate::validate_pattern(instruction)?;
    validate::validate_path(path)?;

    let timer = Timer::new();
    let mut config = LlmConfig::default();
    if let Some(m) = model {
        config.model = m.to_string();
    }

    let max_files = budget.unwrap_or(50);

    // ── 1. Collect context ──────────────────────────────────────────────
    let context_files = collect_context(path, symbol, file_type, extension, no_ignore, depth, max_files)?;

    if context_files.is_empty() {
        if json {
            println!("{}", serde_json::to_string_pretty(&json!({
                "tool": "codescope",
                "command": "rewrite",
                "instruction": instruction,
                "model": &config.model,
                "context_files": [],
                "response": null,
                "changes": [],
                "error": "No files found matching criteria"
            })).unwrap());
        } else {
            eprintln!("{} No files found matching the given criteria in '{}'", "✗".red(), path);
        }
        return Ok(1);
    }

    // ── 2. Build the prompt ────────────────────────────────────────────
    let prompt = build_prompt(instruction, &context_files);

    // ── 3. Call the LLM API ────────────────────────────────────────────
    let response = call_llm_api(&prompt, &config)?;

    // ── 4. Parse changes from the response ─────────────────────────────
    let changes = parse_changes(&response, &context_files);

    let elapsed = timer.elapsed_secs();

    // ── 5. Output ──────────────────────────────────────────────────────
    if json {
        let json_output = json!({
            "tool": "codescope",
            "command": "rewrite",
            "instruction": instruction,
            "model": &config.model,
            "provider": &config.provider,
            "context_files": context_files.keys().collect::<Vec<_>>(),
            "response": response,
            "changes": changes.iter().map(|c| json!({
                "file": &c.file,
                "old": &c.old_content,
                "new": &c.new_content,
            })).collect::<Vec<_>>(),
            "elapsed_secs": elapsed,
            "dry_run": dry_run,
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
    } else {
        print_human_output(&config, &context_files, &prompt, &response, &changes, dry_run, elapsed);
    }

    // ── 6. Apply changes (unless dry_run, requires --write) ────────────
    if !dry_run && write && !changes.is_empty() {
        apply_changes(&changes)?;
        if !json {
            eprintln!("{} {} change(s) applied to files", "✓".green(), changes.len());
        }
    } else if (dry_run || !write) && !changes.is_empty() && !json {
        eprintln!(
            "{} {} change(s) would be applied (use --write to apply, remove --dry-run to confirm)",
            "✓".yellow(),
            changes.len()
        );
    }

    Ok(if changes.is_empty() { 1 } else { 0 })
}

// ────────────────────────────────────────────────────────────────────────────
// Context collection
// ────────────────────────────────────────────────────────────────────────────

/// Collect relevant files from the repository, returning a map of path → content.
fn collect_context(
    path: &str,
    symbol: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    budget: usize,
) -> Result<BTreeMap<String, String>, String> {
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    let mut builder = ignore::WalkBuilder::new(path);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);
    if let Some(d) = depth {
        builder.max_depth(Some(d));
    }

    // If we have a symbol filter, use it to further narrow results.
    let symbol_filter = symbol.map(|s| s.to_lowercase());

    let mut files: BTreeMap<String, String> = BTreeMap::new();

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

        // Extension filter
        if let Some(exts) = &extensions {
            let matches = exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)));
            if !matches {
                continue;
            }
        }

        // Skip binary files by extension heuristic
        let skip_exts = [
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg",
            "zip", "tar", "gz", "bz2", "xz", "7z",
            "exe", "dll", "so", "dylib", "a", "o",
            "wasm", "lock", "pdb",
        ];
        if skip_exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext))) {
            continue;
        }

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Symbol filter: only include files that mention the symbol
        if let Some(ref sym) = symbol_filter {
            if !content.to_lowercase().contains(sym) {
                continue;
            }
        }

        if files.len() >= budget {
            break;
        }

        files.insert(file_path, content);
    }

    Ok(files)
}

// ────────────────────────────────────────────────────────────────────────────
// Prompt building
// ────────────────────────────────────────────────────────────────────────────

/// Build a prompt that includes the rewrite instruction and collected context.
fn build_prompt(instruction: &str, context_files: &BTreeMap<String, String>) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an expert code editor. The user wants to rewrite code according to the instruction below.\n\n");
    prompt.push_str(&format!("## Rewrite Instruction\n{}\n\n", instruction));

    prompt.push_str("## Current Code Files\n\n");
    for (file_path, content) in context_files {
        prompt.push_str(&format!("### File: {}\n```\n{}\n```\n\n", file_path, content));
    }

    prompt.push_str(
        "## Output Format\n\
         For each file that needs to change, output a block like this:\n\
         ```\n\
         <<<< FILE: path/to/file.ext\n\
         (full new content of the file)\n\
         >>>>\n\
         ```\n\
         If a file doesn't need changes, don't include it. Only output the blocks, no extra text.\n"
    );

    prompt
}

// ────────────────────────────────────────────────────────────────────────────
// LLM API calls via curl subprocess
// ────────────────────────────────────────────────────────────────────────────

/// Escape a string for inclusion in a JSON value.
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Call the configured LLM API and return the generated text.
fn call_llm_api(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    match config.provider.as_str() {
        "openai" => call_openai_api(prompt, config),
        _ => call_ollama_api(prompt, config),
    }
}

/// Call the Ollama generate API: POST {base}/api/generate
fn call_ollama_api(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    let url = format!("{}/api/generate", config.api_base);
    let body = format!(
        r#"{{"model":"{}","prompt":"{}","stream":false,"options":{{"temperature":0.2}}}}"#,
        config.model,
        escape_json(prompt)
    );

    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg(&url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(&body)
        .output()
        .map_err(|e| format!("Failed to invoke curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "LLM API call failed (exit code {:?}): {}",
            output.status.code(),
            stderr
        ));
    }

    let response = String::from_utf8_lossy(&output.stdout);
    extract_ollama_response(&response)
}

/// Call an OpenAI-compatible chat completions API: POST {base}/chat/completions
fn call_openai_api(prompt: &str, config: &LlmConfig) -> Result<String, String> {
    let url = format!("{}/chat/completions", config.api_base);
    let system_msg = "You are an expert code editor. Follow the output format precisely. Output only the change blocks, nothing else.";
    let body = format!(
        r#"{{"model":"{}","messages":[{{"role":"system","content":"{}"}},{{"role":"user","content":"{}"}}],"temperature":0.2,"stream":false}}"#,
        config.model,
        escape_json(system_msg),
        escape_json(prompt)
    );

    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg("POST")
        .arg(&url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(&body)
        .output()
        .map_err(|e| format!("Failed to invoke curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "LLM API call failed (exit code {:?}): {}",
            output.status.code(),
            stderr
        ));
    }

    let response = String::from_utf8_lossy(&output.stdout);
    extract_openai_response(&response)
}

/// Extract the `response` field from an Ollama JSON reply.
fn extract_ollama_response(raw: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| format!("Failed to parse Ollama response as JSON: {}", e))?;

    if let Some(err) = v.get("error") {
        return Err(format!("Ollama error: {}", err));
    }

    v.get("response")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No 'response' field in Ollama reply".to_string())
}

/// Extract the assistant message content from an OpenAI-compatible JSON reply.
fn extract_openai_response(raw: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| format!("Failed to parse OpenAI response as JSON: {}", e))?;

    if let Some(err) = v.get("error") {
        let msg = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or_else(|| "unknown error");
        return Err(format!("OpenAI error: {}", msg));
    }

    v.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No assistant content in OpenAI reply".to_string())
}

// ────────────────────────────────────────────────────────────────────────────
// Response parsing
// ────────────────────────────────────────────────────────────────────────────

/// Parse the LLM response into a list of `FileChange`s.
///
/// Looks for blocks delimited by `<<<< FILE: <path>` and `>>>>`.
fn parse_changes(response: &str, context_files: &BTreeMap<String, String>) -> Vec<FileChange> {
    let mut changes = Vec::new();

    // Pattern: <<<< FILE: path/to/file.ext
    let marker_re = regex::Regex::new(r"(?m)^<<<<\s+FILE:\s*(.+)$").unwrap();
    let end_marker_re = regex::Regex::new(r"(?m)^>>>>\s*$").unwrap();

    let mut markers: Vec<(usize, &str)> = Vec::new();
    for cap in marker_re.find_iter(response) {
        let file_path = marker_re
            .captures(cap.as_str())
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim())
            .unwrap_or("");
        markers.push((cap.start(), file_path));
    }

    let mut end_positions: Vec<usize> = Vec::new();
    for cap in end_marker_re.find_iter(response) {
        end_positions.push(cap.end());
    }

    for (i, (start, file_path)) in markers.iter().enumerate() {
        // Extract the header line length
        let header_end = response[*start..]
            .find('\n')
            .map(|n| *start + n + 1)
            .unwrap_or(*start);

        // Find the closing >>>>
        let end = end_positions.get(i).copied().unwrap_or_else(|| response.len());

        if end <= header_end {
            continue;
        }

        let new_content = response[header_end..end].trim_end().to_string();

        // Resolve the file path relative to our context
        let resolved_path = resolve_file_path(file_path, context_files);

        // Get old content from context if available
        let old_content = context_files
            .get(&resolved_path)
            .cloned()
            .unwrap_or_default();

        changes.push(FileChange {
            file: resolved_path,
            old_content,
            new_content,
        });
    }

    changes
}

/// Try to resolve a file path from the LLM response to a key in our context map.
fn resolve_file_path<'a>(path: &str, context_files: &'a BTreeMap<String, String>) -> String {
    // Exact match
    if context_files.contains_key(path) {
        return path.to_string();
    }

    // Try matching just the filename portion
    let filename = std::path::Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    for key in context_files.keys() {
        let key_filename = std::path::Path::new(key)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        if key_filename == filename {
            return key.clone();
        }
    }

    // Fall back to the path as-is
    path.to_string()
}

// ────────────────────────────────────────────────────────────────────────────
// Change application
// ────────────────────────────────────────────────────────────────────────────

/// Apply the parsed changes to the filesystem.
fn apply_changes(changes: &[FileChange]) -> Result<(), String> {
    for change in changes {
        // Check file exists or can be created
        let parent = std::path::Path::new(&change.file)
            .parent()
            .map(|p| p.to_path_buf());

        if let Some(parent) = parent {
            if !parent.exists() {
                fs::create_dir_all(&parent)
                    .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))?;
            }
        }

        fs::write(&change.file, &change.new_content)
            .map_err(|e| format!("Failed to write {}: {}", change.file, e))?;
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Human-readable output
// ────────────────────────────────────────────────────────────────────────────

fn print_human_output(
    config: &LlmConfig,
    context_files: &BTreeMap<String, String>,
    _prompt: &str,
    response: &str,
    changes: &[FileChange],
    dry_run: bool,
    elapsed: f64,
) {
    let separator = "─".repeat(60);

    eprintln!("{} AI Rewrite", ">>".cyan());
    eprintln!("  {} model: {}", "Provider:".dimmed(), config.provider);
    eprintln!("  {} {}", "Model:".dimmed(), config.model);
    eprintln!("  {} {}", "API:".dimmed(), config.api_base);
    eprintln!("{}", separator.dimmed());

    eprintln!("{} Context files collected:", ">>".cyan());
    for path in context_files.keys() {
        eprintln!("  {}", path.green());
    }
    eprintln!("{}", separator.dimmed());

    eprintln!("{} LLM Response:", ">>".cyan());
    eprintln!("{}", response);
    eprintln!("{}", separator.dimmed());

    if !changes.is_empty() {
        eprintln!("{} Suggested Changes ({} file(s)):", ">>".yellow(), changes.len());
        eprintln!("{}", separator.dimmed());
        for change in changes {
            eprintln!("  {}", format!("📄 {}", change.file).cyan().bold());

            // Show a simple diff-style preview
            let old_lines: Vec<&str> = change.old_content.lines().collect();
            let new_lines: Vec<&str> = change.new_content.lines().collect();

            let max_preview = 10;
            let mut shown = 0;

            for diff_line in simple_diff(&old_lines, &new_lines) {
                if shown >= max_preview {
                    eprintln!("  {} ...", "...".dimmed());
                    break;
                }
                match diff_line {
                    DiffLine::Removed(s) => eprintln!("  {}", format!("- {}", s).red()),
                    DiffLine::Added(s) => eprintln!("  {}", format!("+ {}", s).green()),
                    DiffLine::Context(s) => eprintln!("  {}", format!("  {}", s).dimmed()),
                }
                shown += 1;
            }

            if dry_run {
                eprintln!(
                    "  {}",
                    "(dry run — changes not applied)".yellow().dimmed()
                );
            }
            eprintln!();
        }
    } else {
        eprintln!(
            "{} No parseable changes found in LLM response",
            "✗".yellow()
        );
    }

    eprintln!("{}", separator.dimmed());
    eprintln!(
        "{} Completed in {:.3}s",
        "✓".green(),
        elapsed
    );
}

/// Minimal diff representation for human output.
enum DiffLine {
    Removed(String),
    Added(String),
    Context(String),
}

/// Very simple line-based diff — not optimal but sufficient for a preview.
fn simple_diff(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    let mut result = Vec::new();
    let old_set: std::collections::HashSet<_> = old.iter().collect();
    let new_set: std::collections::HashSet<_> = new.iter().collect();

    // Lines only in old → removed
    for line in old {
        if !new_set.contains(line) {
            result.push(DiffLine::Removed((*line).to_string()));
        }
    }
    // Lines only in new → added
    for line in new {
        if !old_set.contains(line) {
            result.push(DiffLine::Added((*line).to_string()));
        }
    }
    // Lines in both → context (limit to a few)
    let mut context_count = 0;
    for line in new {
        if old_set.contains(line) && context_count < 3 {
            result.push(DiffLine::Context((*line).to_string()));
            context_count += 1;
        }
    }

    result
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json("tab\there"), "tab\\there");
    }

    #[test]
    fn test_extract_ollama_response() {
        let raw = r#"{"model":"llama3","response":"Hello, world!","done":true}"#;
        let result = extract_ollama_response(raw).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_extract_ollama_error() {
        let raw = r#"{"error":"model not found"}"#;
        let result = extract_ollama_response(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("model not found"));
    }

    #[test]
    fn test_extract_openai_response() {
        let raw = r#"{"choices":[{"message":{"content":"Here are the changes."}}]}"#;
        let result = extract_openai_response(raw).unwrap();
        assert_eq!(result, "Here are the changes.");
    }

    #[test]
    fn test_extract_openai_error() {
        let raw = r#"{"error":{"message":"Invalid API key","type":"invalid_request_error"}}"#;
        let result = extract_openai_response(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid API key"));
    }

    #[test]
    fn test_parse_changes() {
        let response = "<<<< FILE: src/main.rs\nfn new_main() {\n    println!(\"new\");\n}\n>>>>\n";
        let mut context = BTreeMap::new();
        context.insert("src/main.rs".to_string(), "fn main() {\n    println!(\"old\");\n}\n".to_string());

        let changes = parse_changes(response, &context);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].file, "src/main.rs");
        assert!(changes[0].new_content.contains("new_main"));
    }

    #[test]
    fn test_parse_changes_empty() {
        let response = "No changes needed.";
        let context = BTreeMap::new();
        let changes = parse_changes(response, &context);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_resolve_file_path_exact() {
        let mut ctx = BTreeMap::new();
        ctx.insert("src/lib.rs".to_string(), "".to_string());
        assert_eq!(resolve_file_path("src/lib.rs", &ctx), "src/lib.rs");
    }

    #[test]
    fn test_resolve_file_path_by_filename() {
        let mut ctx = BTreeMap::new();
        ctx.insert("src/deeply/nested/lib.rs".to_string(), "".to_string());
        assert_eq!(resolve_file_path("lib.rs", &ctx), "src/deeply/nested/lib.rs");
    }

    #[test]
    fn test_collect_context_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("lib.rs"), "fn helper() {}").unwrap();

        let result = collect_context(
            dir.path().to_str().unwrap(),
            None,
            None,
            None,
            true,
            None,
            10,
        );
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_collect_context_with_symbol() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("helper.rs"), "fn helper() {}").unwrap();

        let result = collect_context(
            dir.path().to_str().unwrap(),
            Some("main"),
            None,
            None,
            true,
            None,
            10,
        );
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.keys().next().unwrap().contains("main.rs"));
    }

    #[test]
    fn test_build_prompt() {
        let mut ctx = BTreeMap::new();
        ctx.insert("file.rs".to_string(), "fn foo() {}".to_string());

        let prompt = build_prompt("add docs", &ctx);
        assert!(prompt.contains("Rewrite Instruction"));
        assert!(prompt.contains("add docs"));
        assert!(prompt.contains("file.rs"));
        assert!(prompt.contains("fn foo()"));
        assert!(prompt.contains("<<<< FILE:"));
        assert!(prompt.contains(">>>>"));
    }

    #[test]
    fn test_simple_diff() {
        let old = vec!["line1", "line2", "line3"];
        let new = vec!["line1", "line2_modified", "line3", "line4"];

        let diff = simple_diff(&old, &new);
        assert!(diff.iter().any(|d| matches!(d, DiffLine::Removed(s) if s == "line2")));
        assert!(diff.iter().any(|d| matches!(d, DiffLine::Added(s) if s == "line2_modified")));
        assert!(diff.iter().any(|d| matches!(d, DiffLine::Added(s) if s == "line4")));
        assert!(diff.iter().any(|d| matches!(d, DiffLine::Context(s) if s == "line1")));
    }
}
