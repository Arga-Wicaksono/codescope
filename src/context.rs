//! Context Engine — intelligent context extraction for humans and AI.
//!
//! Commands:
//! - `cs context <topic>` — Multi-source context extraction with ranking
//! - `cs pack <description>` — LLM-optimized prompt packing with token budgets
//! - `cs trace <symbol>` — Execution flow tracing through functions
//!
//! The context engine solves the #1 bottleneck in AI coding: context selection.
//! It gathers files, symbols, dependencies, and content matches, then ranks
//! them by relevance to produce a deterministic, structured context package.

use colored::Colorize;
use ignore::WalkBuilder;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

use crate::output_schema;
use crate::symbol;
use crate::utils::Timer;
use crate::validate;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Approximate characters per token for English/code text.
/// This is a rough heuristic: most LLM tokenizers average ~4 chars/token.
const CHARS_PER_TOKEN: usize = 4;

/// Default token budget for `cs pack`.
const DEFAULT_TOKEN_BUDGET: usize = 8000;

/// Maximum snippet length (in characters) for context items.
const MAX_SNIPPET_LEN: usize = 2000;

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

/// A ranked context item with relevance score.
#[derive(Debug, Clone)]
pub struct ContextItem {
    /// The source type (file, symbol, content, dependency).
    pub source_type: ContextSourceType,
    /// File path (relative to search root).
    pub file: String,
    /// Line number (if applicable).
    pub line: Option<usize>,
    /// Symbol name (if applicable).
    pub symbol_name: Option<String>,
    /// Kind of symbol (function, class, etc.).
    pub symbol_kind: Option<String>,
    /// Language of the file.
    pub language: String,
    /// Relevance score (0.0 - 1.0).
    pub relevance: f64,
    /// Content snippet.
    pub snippet: String,
    /// Why this item is relevant.
    pub reason: String,
}

/// Trace step for execution flow.
#[derive(Debug, Clone)]
pub struct TraceStep {
    pub step: usize,
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub signature: String,
    pub depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContextSourceType {
    File,
    Symbol,
    Content,
    Dependency,
}

impl std::fmt::Display for ContextSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextSourceType::File => write!(f, "file"),
            ContextSourceType::Symbol => write!(f, "symbol"),
            ContextSourceType::Content => write!(f, "content"),
            ContextSourceType::Dependency => write!(f, "dependency"),
        }
    }
}

/// A context package ready for LLM consumption.
#[derive(Debug, Clone)]
pub struct ContextPack {
    topic: String,
    items: Vec<ContextItem>,
    total_tokens: usize,
    budget: usize,
    truncated: bool,
}

// ---------------------------------------------------------------------------
// Token estimation
// ---------------------------------------------------------------------------

/// Estimate the number of tokens in a string.
/// Uses a simple heuristic of ~4 characters per token.
fn estimate_tokens(text: &str) -> usize {
    (text.len() / CHARS_PER_TOKEN).max(1)
}

// ---------------------------------------------------------------------------
// Ranking system
// ---------------------------------------------------------------------------

/// Score a context item based on multiple factors.
fn score_item(
    topic: &str,
    source_type: ContextSourceType,
    name_match: Option<&str>,
    content: &str,
    is_entry_point: bool,
    reference_count: usize,
) -> f64 {
    let topic_lower = topic.to_lowercase();
    let mut score = 0.0;

    // Factor 1: Direct name match (highest weight)
    if let Some(name) = name_match {
        if name.eq_ignore_ascii_case(topic) {
            score += 0.35; // Exact match
        } else if name.to_lowercase().contains(&topic_lower) {
            score += 0.25; // Partial match
        }
    }

    // Factor 2: Content relevance
    let content_lower = content.to_lowercase();
    if content_lower.contains(&topic_lower) {
        score += 0.20;
    }

    // Factor 3: Source type weight
    score += match source_type {
        ContextSourceType::Symbol => 0.15,
        ContextSourceType::Content => 0.10,
        ContextSourceType::Dependency => 0.08,
        ContextSourceType::File => 0.05,
    };

    // Factor 4: Entry point bonus
    if is_entry_point {
        score += 0.10;
    }

    // Factor 5: Reference count bonus (more referenced = more important)
    score += (reference_count as f64 * 0.02).min(0.10);

    // Cap at 1.0
    score.min(1.0)
}

// ---------------------------------------------------------------------------
// Entry point detection
// ---------------------------------------------------------------------------

/// Check if a file is likely an entry point for the project.
fn is_entry_point(file_name: &str, path: &str) -> bool {
    let lower = file_name.to_lowercase();
    // Common entry point files
    let entry_points = [
        "main.rs", "main.py", "main.go", "main.js", "main.ts",
        "index.js", "index.ts", "index.tsx", "index.jsx",
        "app.rs", "app.py", "app.go", "app.js", "app.ts",
        "lib.rs", "mod.rs",
        "main.c", "main.cpp", "main.java",
        "cli.rs", "server.rs",
        "Cargo.toml", "package.json", "go.mod", "setup.py",
        "Makefile", "Dockerfile",
    ];
    if entry_points.iter().any(|ep| lower.contains(ep)) {
        return true;
    }

    // Check if file is in the root of the project (likely entry point)
    let path_obj = Path::new(path);
    if path_obj.parent().map_or(false, |p| {
        // If parent is the search root, it's a top-level file
        p.as_os_str().is_empty() || p == Path::new(".")
    }) {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Context extraction
// ---------------------------------------------------------------------------

/// Extract context for a topic from a codebase.
fn extract_context(
    topic: &str,
    path: &str,
    extensions: Option<&[&str]>,
    no_ignore: bool,
    depth: Option<usize>,
    max_items: usize,
) -> Result<Vec<ContextItem>, String> {
    validate::validate_path(path)?;

    let mut items: Vec<ContextItem> = Vec::new();

    // --- Source 1: Symbol definitions matching the topic ---
    let all_symbols = symbol::walk_and_extract_symbols(path, extensions, no_ignore, depth)
        .unwrap_or_default();

    // Build a reference count map (how many times each symbol name appears)
    let mut ref_counts: HashMap<String, usize> = HashMap::new();
    for sym in &all_symbols {
        *ref_counts.entry(sym.name.clone()).or_insert(0) += 1;
    }

    // Add matching symbols as context
    let topic_lower = topic.to_lowercase();
    let mut topic_words: Vec<&str> = topic_lower.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();

    for sym in &all_symbols {
        let sym_name_lower = sym.name.to_lowercase();

        // Check if symbol matches topic directly or any topic word
        let matches_topic = sym_name_lower.contains(&topic_lower)
            || topic_words.iter().any(|w| sym_name_lower.contains(w));

        if matches_topic {
            let snippet = sym.signature.clone();
            let relevance = score_item(
                topic,
                ContextSourceType::Symbol,
                Some(&sym.name),
                &snippet,
                is_entry_point(&sym.file, path),
                *ref_counts.get(&sym.name).unwrap_or(&0),
            );

            let rel_path = shorten_path(&sym.file, path);
            items.push(ContextItem {
                source_type: ContextSourceType::Symbol,
                file: rel_path,
                line: Some(sym.line),
                symbol_name: Some(sym.name.clone()),
                symbol_kind: Some(sym.kind.to_string()),
                language: sym.language.clone(),
                relevance,
                snippet: truncate_str(&snippet, MAX_SNIPPET_LEN),
                reason: format!("Symbol '{}' matches topic", sym.name),
            });
        }
    }

    // --- Source 2: Content search for the topic in file contents ---
    let topic_pattern = Regex::new(&regex::escape(&topic_lower))
        .map_err(|e| format!("Invalid topic: {}", e))?;

    let mut builder = WalkBuilder::new(path);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);
    if let Some(d) = depth {
        builder.max_depth(Some(d));
    }

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_path = entry.path();
        let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();

        // Filter by extension if specified
        if let Some(exts) = extensions {
            let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
            if !exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
                continue;
            }
        }

        let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        let lang = symbol::language_for_ext(&ext).map(|l| l.to_string()).unwrap_or_else(|| "unknown".to_string());

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_path = shorten_path(&file_path.to_string_lossy(), path);
        let entry = is_entry_point(file_name.as_ref(), &rel_path);

        // Count matches in this file
        let match_count = topic_pattern.find_iter(&content.to_lowercase()).count();
        if match_count == 0 {
            continue;
        }

        // Extract relevant lines around matches
        let mut context_lines = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        for (line_idx, line) in lines.iter().enumerate() {
            if topic_pattern.is_match(&line.to_lowercase()) {
                let start = line_idx.saturating_sub(3);
                let end = (line_idx + 4).min(lines.len());
                for i in start..end {
                    context_lines.push(format!("{:>4}: {}", i + 1, lines[i]));
                }
                context_lines.push("...".to_string());
                if context_lines.len() > 30 {
                    break; // Limit context lines
                }
            }
        }

        let snippet = context_lines.join("\n");

        // Score for content match
        let relevance = score_item(
            topic,
            ContextSourceType::Content,
            None,
            &content,
            entry,
            match_count,
        );

        items.push(ContextItem {
            source_type: ContextSourceType::Content,
            file: rel_path,
            line: None,
            symbol_name: None,
            symbol_kind: None,
            language: lang,
            relevance,
            snippet: truncate_str(&snippet, MAX_SNIPPET_LEN),
            reason: format!("{} matches in file content", match_count),
        });
    }

    // --- Source 3: Files whose names match topic words ---
    for sym in &all_symbols {
        let file_name = Path::new(&sym.file)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        let file_name_lower = file_name.to_lowercase();
        let name_matches = topic_words.iter().any(|w| file_name_lower.contains(w));

        if name_matches {
            // Check if we already have this file from content search
            let rel_path = shorten_path(&sym.file, path);
            let already_has = items.iter().any(|i| i.file == rel_path);

            if !already_has {
                let relevance = score_item(
                    topic,
                    ContextSourceType::File,
                    Some(&file_name),
                    "",
                    is_entry_point(file_name.as_ref(), &rel_path),
                    0,
                );

                items.push(ContextItem {
                    source_type: ContextSourceType::File,
                    file: rel_path,
                    line: None,
                    symbol_name: None,
                    symbol_kind: None,
                    language: sym.language.clone(),
                    relevance,
                    snippet: String::new(),
                    reason: "File name matches topic".to_string(),
                });
            }
        }
    }

    // Sort by relevance (descending)
    items.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));

    // Truncate to max items
    items.truncate(max_items);

    Ok(items)
}

// ---------------------------------------------------------------------------
// Trace: execution flow through functions
// ---------------------------------------------------------------------------

/// Trace the execution flow starting from a symbol.
fn trace_symbol(
    name: &str,
    path: &str,
    extensions: Option<&[&str]>,
    no_ignore: bool,
    depth: Option<usize>,
    max_depth: usize,
) -> Result<Vec<TraceStep>, String> {
    validate::validate_pattern(name)?;

    let all_symbols = symbol::walk_and_extract_symbols(path, extensions, no_ignore, depth)
        .unwrap_or_default();

    // Find the starting symbol
    let start_sym = all_symbols
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(name) && (s.kind == symbol::SymbolKind::Function || s.kind == symbol::SymbolKind::Method));

    let start_sym = match start_sym {
        Some(s) => s.clone(),
        None => return Ok(Vec::new()),
    };

    let mut trace = Vec::new();
    let mut visited = std::collections::HashSet::new();
    visited.insert(name.to_lowercase());

    trace_step(&start_sym, path, &all_symbols, &mut trace, &mut visited, 0, max_depth, extensions, no_ignore);

    Ok(trace)
}

fn trace_step(
    sym: &symbol::Symbol,
    path: &str,
    all_symbols: &[symbol::Symbol],
    trace: &mut Vec<TraceStep>,
    visited: &mut std::collections::HashSet<String>,
    current_depth: usize,
    max_depth: usize,
    extensions: Option<&[&str]>,
    no_ignore: bool,
) {
    if current_depth >= max_depth {
        return;
    }

    // Add current step
    let rel_path = shorten_path(&sym.file, path);
    trace.push(TraceStep {
        step: trace.len() + 1,
        name: sym.name.clone(),
        kind: sym.kind.to_string(),
        file: rel_path,
        line: sym.line,
        signature: sym.signature.clone(),
        depth: current_depth,
    });

    // Read the file and find function calls within this symbol's body
    let content = match std::fs::read_to_string(&sym.file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lines: Vec<&str> = content.lines().collect();
    let start_line = sym.line.saturating_sub(1);
    let end_line = std::cmp::min(start_line + 200, lines.len());

    // Find called function names
    let call_pattern = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();

    for line in &lines[start_line..end_line] {
        for cap in call_pattern.captures_iter(line) {
            if let Some(call_match) = cap.get(1) {
                let called_name = call_match.as_str();

                // Skip language keywords and common names
                if is_keyword(called_name) {
                    continue;
                }

                // Skip self-recursion
                if called_name.eq_ignore_ascii_case(&sym.name) {
                    continue;
                }

                // Skip already visited
                if visited.contains(&called_name.to_lowercase()) {
                    continue;
                }

                // Find the symbol definition
                if let Some(called_sym) = all_symbols
                    .iter()
                    .find(|s| {
                        s.name.eq_ignore_ascii_case(called_name)
                            && (s.kind == symbol::SymbolKind::Function || s.kind == symbol::SymbolKind::Method)
                    })
                {
                    visited.insert(called_name.to_lowercase());
                    trace_step(called_sym, path, all_symbols, trace, visited, current_depth + 1, max_depth, extensions, no_ignore);
                }
            }
        }
    }
}

fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "if" | "else" | "for" | "while" | "match" | "return" | "println"
            | "print" | "format" | "assert" | "expect" | "unwrap" | "clone"
            | "to_string" | "to_owned" | "into" | "from" | "new" | "default"
            | "panic" | "Error" | "Option" | "Result" | "Some" | "None"
            | "Ok" | "Err" | "true" | "false" | "self" | "super" | "Self"
            | "vec" | "String" | "Box" | "Rc" | "Arc" | "HashMap" | "Vec"
            | "push" | "pop" | "len" | "is_empty" | "iter" | "collect"
            | "map" | "filter" | "fold" | "reduce" | "spawn" | "join"
            | "send" | "recv" | "lock" | "await" | "async" | "spawn"
            | "log" | "info" | "warn" | "error" | "debug" | "trace"
            | "require" | "include" | "use" | "import" | "export"
            | "typeof" | "instanceof" | "in" | "of" | "delete" | "void"
    )
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn shorten_path(full_path: &str, base: &str) -> String {
    if let Some(stripped) = full_path.strip_prefix(base) {
        stripped.trim_start_matches('/').to_string()
    } else {
        full_path.to_string()
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut end = max_len;
        // Try to break at a newline
        if let Some(pos) = s[..max_len].rfind('\n') {
            end = pos + 1;
        }
        format!("{}...\n[truncated]", &s[..end])
    }
}

/// Pack context items into a token-budgeted string for LLM consumption.
fn pack_context(items: &[ContextItem], budget: usize) -> String {
    let mut result = String::new();
    let mut used_tokens = 0;

    // Header
    let header = format!("<context>\n");
    used_tokens += estimate_tokens(&header);
    result.push_str(&header);

    for item in items {
        let section = format!(
            "--- [{}] {} ({}) score={:.2} ---\n{}\n",
            item.source_type,
            item.file,
            item.symbol_name.as_deref().unwrap_or("-"),
            item.relevance,
            item.snippet,
        );

        let section_tokens = estimate_tokens(&section);
        if used_tokens + section_tokens > budget {
            result.push_str(&format!("\n[... context truncated: budget of {} tokens exceeded]\n", budget));
            return result;
        }

        result.push_str(&section);
        used_tokens += section_tokens;
    }

    result.push_str("</context>\n");
    result
}

// ---------------------------------------------------------------------------
// Public collection functions (used by serve.rs for MCP/HTTP)
// ---------------------------------------------------------------------------

/// Collect context items for MCP/HTTP API.
pub fn collect_context_items(
    topic: &str,
    path: &str,
    _exclude: Option<&str>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    max_items: usize,
) -> Result<Vec<ContextItem>, String> {
    let extensions: Option<Vec<&str>> = extension.map(|e| vec![e]);
    extract_context(topic, path, extensions.as_deref(), no_ignore, depth, max_items)
}

/// Collect packed context for MCP/HTTP API.
pub fn collect_packed_context(
    description: &str,
    path: &str,
    _exclude: Option<&str>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    budget: Option<usize>,
) -> Result<serde_json::Value, String> {
    let extensions: Option<Vec<&str>> = extension.map(|e| vec![e]);
    let effective_budget = budget.unwrap_or(DEFAULT_TOKEN_BUDGET);
    let items = extract_context(description, path, extensions.as_deref(), no_ignore, depth, 50)?;
    let packed = pack_context(&items, effective_budget);
    let actual_tokens = estimate_tokens(&packed);
    let truncated = actual_tokens > effective_budget;
    Ok(serde_json::json!({
        "packed_context": packed,
        "token_budget": effective_budget,
        "estimated_tokens": actual_tokens,
        "truncated": truncated,
        "item_count": items.len(),
    }))
}

/// Collect trace steps for MCP/HTTP API.
pub fn collect_trace_steps(
    name: &str,
    path: &str,
    _exclude: Option<&str>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    max_depth: Option<usize>,
) -> Result<Vec<TraceStep>, String> {
    let extensions: Option<Vec<&str>> = extension.map(|e| vec![e]);
    let effective_max_depth = max_depth.unwrap_or(5);
    trace_symbol(name, path, extensions.as_deref(), no_ignore, depth, effective_max_depth)
}

// ---------------------------------------------------------------------------
// Public commands
// ---------------------------------------------------------------------------

/// `cs context <topic>` — Extract relevant context for a topic.
pub fn run_context(
    topic: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    max_items: Option<usize>,
    json: bool,
) -> Result<i32, String> {
    validate::validate_pattern(topic)?;

    let timer = Timer::new();
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    let effective_max_items = max_items.unwrap_or(20);
    let items = extract_context(topic, path, extensions.as_deref(), no_ignore, depth, effective_max_items)?;

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = items
            .iter()
            .map(|item| serde_json::to_value(output_schema::ContextResultItem {
                source_type: item.source_type.to_string(),
                file: item.file.clone(),
                line: item.line,
                symbol_name: item.symbol_name.clone(),
                symbol_kind: item.symbol_kind.clone(),
                language: item.language.clone(),
                relevance: item.relevance,
                snippet: item.snippet.clone(),
                reason: item.reason.clone(),
            }).unwrap())
            .collect();
        let output = output_schema::envelope(
            "context", topic, "filesystem", items.len(), elapsed,
            serde_json::json!(results_json),
        );
        output_schema::print_json(&output);
        return Ok(if items.is_empty() { 1 } else { 0 });
    }

    if items.is_empty() {
        eprintln!("{} No context found for '{}'", "✗".red(), topic.cyan());
        return Ok(1);
    }

    let separator = "─".repeat(60);
    eprintln!("{} Context for '{}'", ">>".cyan(), topic.cyan());
    eprintln!("{}", separator.dimmed());

    for item in &items {
        let source_tag = format!("[{}]", item.source_type).cyan().dimmed();
        let score_tag = format!("score={:.2}", item.relevance).yellow().dimmed();
        eprintln!("  {} {} {} {}", source_tag, item.file.green(), score_tag, item.reason.dimmed());
        if let Some(sym) = &item.symbol_name {
            eprintln!("    {} ({})", sym.green().bold(), item.symbol_kind.as_deref().unwrap_or("-").dimmed());
        }
        if !item.snippet.is_empty() {
            for line in item.snippet.lines().take(5) {
                eprintln!("    {}", line.dimmed());
            }
            if item.snippet.lines().count() > 5 {
                eprintln!("    {}", "...".dimmed());
            }
        }
        eprintln!();
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} context item(s) found in {:.3}s", "✓".green(), items.len(), timer.elapsed_secs());

    Ok(0)
}

/// `cs pack <description>` — Pack context into LLM-optimized format.
pub fn run_pack(
    description: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    budget: Option<usize>,
    json: bool,
) -> Result<i32, String> {
    validate::validate_pattern(description)?;

    let timer = Timer::new();
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    let effective_budget = budget.unwrap_or(DEFAULT_TOKEN_BUDGET);
    let items = extract_context(description, path, extensions.as_deref(), no_ignore, depth, 50)?;

    // Build the packed context
    let packed = pack_context(&items, effective_budget);
    let actual_tokens = estimate_tokens(&packed);
    let truncated = actual_tokens > effective_budget;

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = items
            .iter()
            .map(|item| serde_json::to_value(output_schema::ContextResultItem {
                source_type: item.source_type.to_string(),
                file: item.file.clone(),
                line: item.line,
                symbol_name: item.symbol_name.clone(),
                symbol_kind: item.symbol_kind.clone(),
                language: item.language.clone(),
                relevance: item.relevance,
                snippet: item.snippet.clone(),
                reason: item.reason.clone(),
            }).unwrap())
            .collect();
        let output = output_schema::envelope_with_extra(
            "pack", description, "filesystem", items.len(), elapsed,
            serde_json::json!(results_json),
            serde_json::json!({
                "packed_context": packed,
                "token_budget": effective_budget,
                "estimated_tokens": actual_tokens,
                "truncated": truncated,
            }),
        );
        output_schema::print_json(&output);
        return Ok(if items.is_empty() { 1 } else { 0 });
    }

    if items.is_empty() {
        eprintln!("{} No context found for '{}'", "✗".red(), description.cyan());
        return Ok(1);
    }

    // Output the packed context directly to stdout (for piping to LLM)
    println!("{}", packed);

    eprintln!("\n{} Packed {} items, ~{} tokens (budget: {}){}",
        ">>".cyan(),
        items.len().to_string().green(),
        actual_tokens.to_string().yellow(),
        effective_budget,
        if truncated { " — truncated".red().to_string() } else { "".to_string() },
    );

    Ok(0)
}

/// `cs trace <symbol>` — Trace execution flow through functions.
pub fn run_trace(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    max_depth: Option<usize>,
    json: bool,
) -> Result<i32, String> {
    let timer = Timer::new();
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    let effective_max_depth = max_depth.unwrap_or(5);
    let trace = trace_symbol(name, path, extensions.as_deref(), no_ignore, depth, effective_max_depth)?;

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = trace
            .iter()
            .map(|step| serde_json::to_value(output_schema::TraceResultItem {
                step: step.step,
                name: step.name.clone(),
                kind: step.kind.clone(),
                file: step.file.clone(),
                line: step.line,
                signature: step.signature.clone(),
                depth: step.depth,
            }).unwrap())
            .collect();
        let output = output_schema::envelope(
            "trace", name, "filesystem", trace.len(), elapsed,
            serde_json::json!(results_json),
        );
        output_schema::print_json(&output);
        return Ok(if trace.is_empty() { 1 } else { 0 });
    }

    if trace.is_empty() {
        eprintln!("{} Could not trace '{}' — function not found", "✗".red(), name.cyan());
        return Ok(1);
    }

    let separator = "─".repeat(60);
    eprintln!("{} Execution trace for '{}'", ">>".cyan(), name.cyan());
    eprintln!("{}", separator.dimmed());

    for step in &trace {
        let indent = "  ".repeat(step.depth + 1);
        let arrow = if step.depth == 0 { "▶".cyan() } else { "→".dimmed() };
        let kind_tag = format!("[{}]", step.kind).dimmed();
        eprintln!("{} {} {} {}:{} {}", indent, arrow, step.name.green().bold(), step.file.cyan(), step.line.to_string().yellow(), kind_tag);
        eprintln!("{}   {}", indent, step.signature.dimmed());
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} step(s) traced in {:.3}s", "✓".green(), trace.len(), timer.elapsed_secs());

    Ok(0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("hello world") > 0);
        assert!(estimate_tokens("") >= 1);
        // 16 chars ≈ 4 tokens
        assert_eq!(estimate_tokens("aabbccddeeffgghh"), 4);
    }

    #[test]
    fn test_score_item_exact_match() {
        let score = score_item("UserService", ContextSourceType::Symbol, Some("UserService"), "", false, 0);
        assert!(score > 0.3);
    }

    #[test]
    fn test_score_item_partial_match() {
        let score = score_item("User", ContextSourceType::Symbol, Some("UserService"), "", false, 0);
        assert!(score > 0.2);
    }

    #[test]
    fn test_is_entry_point() {
        assert!(is_entry_point("main.rs", "main.rs"));
        assert!(is_entry_point("lib.rs", "lib.rs"));
        assert!(!is_entry_point("helper.rs", "src/helper.rs"));
    }

    #[test]
    fn test_context_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("auth.rs"),
            "pub struct AuthService { token: String }\npub fn login(user: &str) -> bool { true }\n",
        )
        .unwrap();

        let result = run_context("AuthService", dir.path().to_str().unwrap(), None, None, None, true, None, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pack_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "fn main() {\n    println!(\"hello\");\n}\n",
        )
        .unwrap();

        let result = run_pack("main", dir.path().to_str().unwrap(), None, None, None, true, None, Some(1000), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trace_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "fn helper() { println!(\"help\"); }\nfn main() { helper(); }\n",
        )
        .unwrap();

        let result = run_trace("main", dir.path().to_str().unwrap(), None, None, None, true, None, Some(5), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pack_context_respects_budget() {
        let items = vec![ContextItem {
            source_type: ContextSourceType::Content,
            file: "test.rs".to_string(),
            line: Some(1),
            symbol_name: None,
            symbol_kind: None,
            language: "rust".to_string(),
            relevance: 0.8,
            snippet: "x".repeat(10000),
            reason: "test".to_string(),
        }];

        let packed = pack_context(&items, 100);
        assert!(estimate_tokens(&packed) <= 200); // Some overhead from XML tags
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("if"));
        assert!(is_keyword("println"));
        assert!(is_keyword("unwrap"));
        assert!(!is_keyword("my_function"));
        assert!(!is_keyword("UserService"));
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 100), "short");
        assert!(truncate_str(&"x".repeat(2000), 100).contains("truncated"));
    }
}
