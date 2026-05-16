//! Symbol intelligence — find definitions, references, callers, and list symbols.
//!
//! Provides grammar-aware symbol extraction across 10+ programming languages
//! using enhanced regex patterns. Tree-sitter integration is planned for a
//! future release for AST-level accuracy.
//!
//! Commands:
//! - `cs symbol <name>` — Find where a symbol is defined
//! - `cs refs <name>` — Find all references to a symbol
//! - `cs callers <name>` — Find all callers of a function
//! - `cs symbols [path]` — List all symbols in a file or directory

use colored::Colorize;
use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::output_schema;
use crate::utils::Timer;
use crate::validate;

// ---------------------------------------------------------------------------
// Language definition patterns
// ---------------------------------------------------------------------------

/// Language-specific patterns for extracting symbol definitions.
/// Each tuple: (extensions, (kind_pattern, name_capture_group_index))
struct LangDef {
    extensions: &'static [&'static str],
    patterns: &'static [&'static str],
    kind: &'static str,
}

const LANG_DEFS: &[LangDef] = &[
    // Rust
    LangDef {
        extensions: &["rs"],
        patterns: &[
            r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)",
            r"(?:pub\s+)?struct\s+(\w+)",
            r"(?:pub\s+)?enum\s+(\w+)",
            r"(?:pub\s+)?trait\s+(\w+)",
            r"(?:pub\s+)?mod\s+(\w+)",
            r"(?:pub\s+)?type\s+(\w+)",
            r"(?:pub\s+)?const\s+(\w+)",
            r"(?:pub\s+)?static\s+(\w+)",
            r"macro_rules!\s+(\w+)",
        ],
        kind: "rust",
    },
    // Python
    LangDef {
        extensions: &["py", "pyi", "pyw"],
        patterns: &[
            r"(?:async\s+)?def\s+(\w+)",
            r"class\s+(\w+)",
        ],
        kind: "python",
    },
    // JavaScript / JSX
    LangDef {
        extensions: &["js", "jsx", "mjs", "cjs"],
        patterns: &[
            r"(?:export\s+)?(?:default\s+)?function\s+(\w+)",
            r"(?:export\s+)?(?:default\s+)?class\s+(\w+)",
            r"(?:export\s+)?const\s+(\w+)\s*=",
            r"(?:export\s+)?let\s+(\w+)\s*=",
            r"(?:export\s+)?var\s+(\w+)\s*=",
            r"(\w+)\s*=\s*(?:async\s+)?\([^)]*\)\s*=>",
            r"(\w+)\s*=\s*(?:async\s+)?function",
        ],
        kind: "javascript",
    },
    // TypeScript / TSX
    LangDef {
        extensions: &["ts", "tsx"],
        patterns: &[
            r"(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+(\w+)",
            r"(?:export\s+)?(?:default\s+)?class\s+(\w+)",
            r"(?:export\s+)?(?:const|let|var)\s+(\w+)\s*(?::\s*[^=]+)?\s*=",
            r"(?:export\s+)?interface\s+(\w+)",
            r"(?:export\s+)?type\s+(\w+)\s*=",
            r"(?:export\s+)?enum\s+(\w+)",
            r"(?:export\s+)?namespace\s+(\w+)",
        ],
        kind: "typescript",
    },
    // Go
    LangDef {
        extensions: &["go"],
        patterns: &[
            r"func\s+(?:\([^)]+\)\s+)?(\w+)",
            r"type\s+(\w+)\s+struct",
            r"type\s+(\w+)\s+interface",
            r"type\s+(\w+)\s+(?:func|map|slice)",
            r"var\s+(\w+)",
            r"const\s+(\w+)",
        ],
        kind: "go",
    },
    // Java / Kotlin
    LangDef {
        extensions: &["java", "kt", "kts"],
        patterns: &[
            r"(?:public|private|protected|static|\s)*\s*(?:class|interface|enum)\s+(\w+)",
            r"(?:public|private|protected|static|\s)*\s*(?:abstract\s+)?class\s+(\w+)",
            r"(?:public|private|protected|static|\s)*\s*\w[\w<>\[\],\s]*\s+(\w+)\s*\(",
            r"fun\s+(\w+)",
            r"object\s+(\w+)",
            r"val\s+(\w+)",
        ],
        kind: "java",
    },
    // C
    LangDef {
        extensions: &["c", "h"],
        patterns: &[
            r"(?:static\s+)?(?:inline\s+)?(?:\w+\s+)+(\w+)\s*\(",
            r"(?:typedef\s+)?struct\s+(\w+)",
            r"(?:typedef\s+)?enum\s+(\w+)",
            r"#define\s+(\w+)",
            r"typedef\s+(?:struct\s+)?(?:\w+\s+)*(\w+)\s*;",
        ],
        kind: "c",
    },
    // C++
    LangDef {
        extensions: &["cpp", "cc", "cxx", "hpp", "hxx"],
        patterns: &[
            r"(?:virtual\s+)?(?:static\s+)?(?:inline\s+)?(?:\w+::)?(?:\w+\s+)+(\w+)\s*\(",
            r"(?:class|struct|enum)\s+(\w+)",
            r"(?:class|struct)\s+(\w+)\s*:",
            r"namespace\s+(\w+)",
            r"template\s*<[^>]+>\s*(?:class|struct)\s+(\w+)",
        ],
        kind: "cpp",
    },
    // Ruby
    LangDef {
        extensions: &["rb"],
        patterns: &[
            r"def\s+(?:self\.)?(\w+)",
            r"class\s+(\w+)",
            r"module\s+(\w+)",
        ],
        kind: "ruby",
    },
    // PHP
    LangDef {
        extensions: &["php"],
        patterns: &[
            r"function\s+(\w+)",
            r"class\s+(\w+)",
            r"interface\s+(\w+)",
            r"trait\s+(\w+)",
            r"namespace\s+(\w+)",
        ],
        kind: "php",
    },
    // Swift
    LangDef {
        extensions: &["swift"],
        patterns: &[
            r"(?:public|private|internal|open)?\s*(?:static\s+)?func\s+(\w+)",
            r"(?:public|private|internal|open)?\s*class\s+(\w+)",
            r"(?:public|private|internal|open)?\s*struct\s+(\w+)",
            r"(?:public|private|internal|open)?\s*enum\s+(\w+)",
            r"(?:public|private|internal|open)?\s*protocol\s+(\w+)",
        ],
        kind: "swift",
    },
    // Shell / Bash
    LangDef {
        extensions: &["sh", "bash", "zsh"],
        patterns: &[
            r"(?:function\s+)?(\w+)\s*\(\)",
            r"(?:function\s+)?(\w+)\s*\(\s*\)",
        ],
        kind: "shell",
    },
];

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// A parsed symbol with its metadata.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub language: String,
    pub signature: String,
}

/// The kind of a symbol (function, class, struct, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Module,
    Namespace,
    Constant,
    Variable,
    Type,
    Macro,
    Unknown,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::Method => write!(f, "method"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Struct => write!(f, "struct"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::Trait => write!(f, "trait"),
            SymbolKind::Interface => write!(f, "interface"),
            SymbolKind::Module => write!(f, "module"),
            SymbolKind::Namespace => write!(f, "namespace"),
            SymbolKind::Constant => write!(f, "constant"),
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Type => write!(f, "type"),
            SymbolKind::Macro => write!(f, "macro"),
            SymbolKind::Unknown => write!(f, "unknown"),
        }
    }
}

/// Infer the symbol kind from the matched line content and language.
fn infer_symbol_kind(line: &str, lang: &str) -> SymbolKind {
    let t = line.trim();
    if lang == "rust" {
        if t.contains("fn ") { return SymbolKind::Function; }
        if t.contains("struct ") { return SymbolKind::Struct; }
        if t.contains("enum ") { return SymbolKind::Enum; }
        if t.contains("trait ") { return SymbolKind::Trait; }
        if t.contains("mod ") { return SymbolKind::Module; }
        if t.contains("type ") { return SymbolKind::Type; }
        if t.contains("const ") || t.contains("static ") { return SymbolKind::Constant; }
        if t.contains("macro_rules!") { return SymbolKind::Macro; }
    } else if lang == "python" {
        if t.starts_with("def ") || t.starts_with("async def ") { return SymbolKind::Function; }
        if t.starts_with("class ") { return SymbolKind::Class; }
    } else if lang == "go" {
        if t.starts_with("func ") { return SymbolKind::Function; }
        if t.contains("struct") { return SymbolKind::Struct; }
        if t.contains("interface") { return SymbolKind::Interface; }
        if t.starts_with("type ") { return SymbolKind::Type; }
        if t.starts_with("var ") { return SymbolKind::Variable; }
        if t.starts_with("const ") { return SymbolKind::Constant; }
    } else if lang == "javascript" || lang == "typescript" {
        if t.contains("function ") { return SymbolKind::Function; }
        if t.contains("class ") { return SymbolKind::Class; }
        if t.contains("interface ") { return SymbolKind::Interface; }
        if t.contains("enum ") { return SymbolKind::Enum; }
        if t.contains("namespace ") { return SymbolKind::Namespace; }
        if t.contains("type ") { return SymbolKind::Type; }
        if t.contains("const ") || t.contains("let ") || t.contains("var ") { return SymbolKind::Variable; }
    } else if lang == "java" {
        if t.contains("class ") { return SymbolKind::Class; }
        if t.contains("interface ") { return SymbolKind::Interface; }
        if t.contains("enum ") { return SymbolKind::Enum; }
        if t.contains("fun ") { return SymbolKind::Function; }
        if t.contains("object ") { return SymbolKind::Class; }
    } else if lang == "c" || lang == "cpp" {
        if t.contains("struct ") { return SymbolKind::Struct; }
        if t.contains("class ") { return SymbolKind::Class; }
        if t.contains("enum ") { return SymbolKind::Enum; }
        if t.contains("namespace ") { return SymbolKind::Namespace; }
        if t.contains("#define") { return SymbolKind::Macro; }
        if t.contains("(") { return SymbolKind::Function; }
        if t.contains("typedef") { return SymbolKind::Type; }
    } else if lang == "ruby" {
        if t.starts_with("def ") { return SymbolKind::Function; }
        if t.starts_with("class ") { return SymbolKind::Class; }
        if t.starts_with("module ") { return SymbolKind::Module; }
    } else if lang == "php" {
        if t.starts_with("function ") { return SymbolKind::Function; }
        if t.starts_with("class ") { return SymbolKind::Class; }
        if t.starts_with("interface ") { return SymbolKind::Interface; }
        if t.starts_with("trait ") { return SymbolKind::Trait; }
        if t.starts_with("namespace ") { return SymbolKind::Namespace; }
    } else if lang == "swift" {
        if t.contains("func ") { return SymbolKind::Function; }
        if t.contains("class ") { return SymbolKind::Class; }
        if t.contains("struct ") { return SymbolKind::Struct; }
        if t.contains("enum ") { return SymbolKind::Enum; }
        if t.contains("protocol ") { return SymbolKind::Interface; }
    } else if lang == "shell" {
        if t.contains("()") { return SymbolKind::Function; }
    }
    SymbolKind::Unknown
}

// //-----------------------------------------------------------------------------
// Public collection functions (used by serve.rs for MCP/HTTP)
//-----------------------------------------------------------------------------

/// Collect symbol definitions for MCP/HTTP API.
pub fn collect_symbol_results(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    symbol_type: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
) -> Result<Vec<Symbol>, String> {
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };
    let all_symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;
    let name_lower = name.to_lowercase();
    let mut matches: Vec<Symbol> = all_symbols
        .into_iter()
        .filter(|s| s.name.to_lowercase().contains(&name_lower))
        .collect();
    if let Some(kind_str) = symbol_type {
        let kind_lower = kind_str.to_lowercase();
        matches.retain(|s| s.kind.to_string().contains(&kind_lower));
    }
    matches.sort_by(|a, b| {
        let a_exact = if a.name.eq_ignore_ascii_case(name) { 0 } else { 1 };
        let b_exact = if b.name.eq_ignore_ascii_case(name) { 0 } else { 1 };
        a_exact.cmp(&b_exact).then_with(|| a.file.cmp(&b.file).then_with(|| a.line.cmp(&b.line)))
    });
    Ok(matches)
}

/// Collect reference results for MCP/HTTP API.
pub fn collect_ref_results(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
) -> Result<Vec<(String, usize, String, String)>, String> {
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };
    let mut builder = WalkBuilder::new(path);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);
    if let Some(d) = depth { builder.max_depth(Some(d)); }
    if let Some(exclude_dirs) = exclude {
        for dir in exclude_dirs.split(',') {
            let trimmed = dir.trim();
            if !trimmed.is_empty() { builder.add_custom_ignore_filename(trimmed); }
        }
    }
    let name_pattern = Regex::new(&format!(r"\b{}\b", regex::escape(name)))
        .map_err(|e| format!("Invalid symbol name: {}", e))?;
    let all_symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;
    let def_locations: Vec<(&str, usize)> = all_symbols
        .iter()
        .filter(|s| s.name == name)
        .map(|s| (s.file.as_str(), s.line))
        .collect();
    let mut refs: Vec<(String, usize, String, String)> = Vec::new();
    for entry in builder.build() {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().map_or(false, |ft| ft.is_file()) { continue; }
        let file_path = entry.path();
        let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
        if let Some(exts) = &extensions {
            let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
            if !exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) { continue; }
        }
        let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        let lang = language_for_ext(&ext).unwrap_or("unknown");
        let content = match std::fs::read_to_string(file_path) { Ok(c) => c, Err(_) => continue };
        for (line_idx, line) in content.lines().enumerate() {
            if name_pattern.is_match(line) {
                let line_num = line_idx + 1;
                let file_str = file_path.to_string_lossy();
                let is_def = def_locations.iter().any(|(f, l)| *f == file_str && *l == line_num);
                if is_def { continue; }
                let trimmed = line.trim();
                if trimmed.starts_with("use ") || trimmed.starts_with("import ") || trimmed.starts_with("from ") || trimmed.starts_with("require ") || trimmed.starts_with("#include") || trimmed.starts_with("mod ") { continue; }
                let rel_path = file_str.replacen(path, "", 1).trim_start_matches('/').to_string();
                refs.push((rel_path, line_num, line.trim().to_string(), lang.to_string()));
            }
        }
    }
    Ok(refs)
}

/// Collect caller results for MCP/HTTP API.
pub fn collect_caller_results(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
) -> Result<Vec<(String, String, usize, usize, String)>, String> {
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };
    let all_symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;
    let call_pattern = Regex::new(&format!(r"\b{}\s*\(", regex::escape(name)))
        .map_err(|e| format!("Invalid symbol name: {}", e))?;
    let mut symbols_by_file: HashMap<String, Vec<&Symbol>> = HashMap::new();
    for sym in &all_symbols {
        symbols_by_file.entry(sym.file.clone()).or_default().push(sym);
    }
    for syms in symbols_by_file.values_mut() {
        syms.sort_by_key(|s| s.line);
    }
    let mut callers: Vec<(String, String, usize, usize, String)> = Vec::new();
    let mut builder = WalkBuilder::new(path);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);
    if let Some(d) = depth { builder.max_depth(Some(d)); }
    if let Some(exclude_dirs) = exclude {
        for dir in exclude_dirs.split(',') {
            let trimmed = dir.trim();
            if !trimmed.is_empty() { builder.add_custom_ignore_filename(trimmed); }
        }
    }
    for entry in builder.build() {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().map_or(false, |ft| ft.is_file()) { continue; }
        let file_path = entry.path();
        let file_str = file_path.to_string_lossy().to_string();
        if let Some(exts) = &extensions {
            let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
            if !exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) { continue; }
        }
        let content = match std::fs::read_to_string(file_path) { Ok(c) => c, Err(_) => continue };
        for (line_idx, line) in content.lines().enumerate() {
            if call_pattern.is_match(line) {
                let line_num = line_idx + 1;
                let file_syms = symbols_by_file.get(&file_str);
                let caller = file_syms.and_then(|syms| {
                    syms.iter().rev()
                        .find(|s| s.kind == SymbolKind::Function || s.kind == SymbolKind::Method || s.kind == SymbolKind::Class)
                        .filter(|s| s.line <= line_num)
                });
                if let Some(caller_sym) = caller {
                    if caller_sym.name == name { continue; }
                    let rel_caller_path = shorten_path(&caller_sym.file, path);
                    callers.push((
                        caller_sym.name.clone(),
                        rel_caller_path,
                        caller_sym.line,
                        line_num,
                        line.trim().to_string(),
                    ));
                }
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    callers.retain(|c| { let key = format!("{}:{}", c.0, c.1); seen.insert(key) });
    Ok(callers)
}

/// Collect all symbols for MCP/HTTP API.
pub fn collect_all_symbols(
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    symbol_type: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    limit: Option<usize>,
) -> Result<Vec<Symbol>, String> {
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };
    let mut symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;
    if let Some(kind_str) = symbol_type {
        let kind_lower = kind_str.to_lowercase();
        symbols.retain(|s| s.kind.to_string().contains(&kind_lower));
    }
    symbols.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.line.cmp(&b.line)));
    let effective_limit = limit.unwrap_or(100);
    symbols.truncate(effective_limit);
    Ok(symbols)
}

// ---------------------------------------------------------------------------
// Extraction helpers
// ---------------------------------------------------------------------------

/// Get the language label for a file extension.
pub fn language_for_ext(ext: &str) -> Option<&'static str> {
    let lower = ext.to_lowercase();
    for ld in LANG_DEFS {
        if ld.extensions.iter().any(|e| e.eq_ignore_ascii_case(&lower)) {
            return Some(ld.kind);
        }
    }
    None
}

/// Extract all symbols from a single file content.
fn extract_symbols_from_content(content: &str, file_path: &str, ext: &str) -> Vec<Symbol> {
    let lang = match language_for_ext(ext) {
        Some(l) => l,
        None => return Vec::new(),
    };

    let lang_def = LANG_DEFS.iter().find(|ld| ld.kind == lang);
    let lang_def = match lang_def {
        Some(ld) => ld,
        None => return Vec::new(),
    };

    let mut symbols = Vec::new();

    for pattern in lang_def.patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.find_iter(content) {
                let match_str = cap.as_str();
                let line_num = content[..cap.start()].matches('\n').count() + 1;
                let line_start = content[..cap.start()].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let col = cap.start() - line_start + 1;

                // Extract the symbol name from the capture group
                if let Some(caps) = re.captures(match_str) {
                    if let Some(name_match) = caps.get(1) {
                        let name = name_match.as_str().to_string();

                        // Skip common non-symbol identifiers
                        if name == "if" || name == "for" || name == "while" || name == "match"
                            || name == "return" || name == "else" || name == "impl"
                            || name == "let" || name == "mut" || name == "pub"
                            || name == "use" || name == "mod" || name == "crate"
                            || name == "super" || name == "self" || name == "Self"
                            || name == "where" || name == "async" || name == "await"
                            || name == "true" || name == "false" || name == "import"
                            || name == "from" || name == "as" || name == "in"
                        {
                            continue;
                        }

                        let kind = infer_symbol_kind(match_str, lang);
                        // Get the full line as signature
                        let line_content = content.lines().nth(line_num - 1).unwrap_or("");
                        let signature = line_content.trim().to_string();

                        symbols.push(Symbol {
                            name,
                            kind,
                            file: file_path.to_string(),
                            line: line_num,
                            column: col,
                            language: lang.to_string(),
                            signature,
                        });
                    }
                }
            }
        }
    }

    symbols
}

/// Walk a directory and extract all symbols, optionally filtering by extension.
pub fn walk_and_extract_symbols(
    path: &str,
    extensions: Option<&[&str]>,
    no_ignore: bool,
    depth: Option<usize>,
) -> Result<Vec<Symbol>, String> {
    validate::validate_path(path)?;

    let mut builder = WalkBuilder::new(path);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);
    if let Some(d) = depth {
        builder.max_depth(Some(d));
    }

    let mut symbols = Vec::new();

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

        // Determine extension
        let ext = match file_path.extension() {
            Some(e) => e.to_string_lossy().to_string(),
            None => continue,
        };

        // Filter by extension if specified
        if let Some(exts) = extensions {
            if !exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
                continue;
            }
        }

        // Only process known languages
        if language_for_ext(&ext).is_none() {
            continue;
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_syms = extract_symbols_from_content(&content, &file_path.to_string_lossy(), &ext);
        symbols.extend(file_syms);
    }

    Ok(symbols)
}

// ---------------------------------------------------------------------------
// Public commands
// ---------------------------------------------------------------------------

/// `cs symbol <name>` — Find symbol definitions.
pub fn run_symbol(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    symbol_type: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    json: bool,
) -> Result<i32, String> {
    validate::validate_pattern(name)?;

    let timer = Timer::new();
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    let all_symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;

    // Filter by name
    let name_lower = name.to_lowercase();
    let mut matches: Vec<&Symbol> = all_symbols
        .iter()
        .filter(|s| s.name.to_lowercase().contains(&name_lower))
        .collect();

    // Filter by symbol type if specified
    if let Some(kind_str) = symbol_type {
        let kind_lower = kind_str.to_lowercase();
        matches.retain(|s| s.kind.to_string().contains(&kind_lower));
    }

    // Sort: exact matches first, then by line number
    matches.sort_by(|a, b| {
        let a_exact = if a.name.eq_ignore_ascii_case(name) { 0 } else { 1 };
        let b_exact = if b.name.eq_ignore_ascii_case(name) { 0 } else { 1 };
        a_exact.cmp(&b_exact).then_with(|| a.file.cmp(&b.file).then_with(|| a.line.cmp(&b.line)))
    });

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = matches
            .iter()
            .map(|s| serde_json::to_value(output_schema::SymbolResultItem {
                name: s.name.clone(),
                kind: s.kind.to_string(),
                file: s.file.clone(),
                line: s.line,
                column: s.column,
                language: s.language.clone(),
                signature: s.signature.clone(),
            }).unwrap())
            .collect();
        let output = output_schema::envelope(
            "symbol", name, "filesystem", matches.len(), elapsed,
            serde_json::json!(results_json),
        );
        output_schema::print_json(&output);
        return Ok(if matches.is_empty() { 1 } else { 0 });
    }

    if matches.is_empty() {
        eprintln!("{} No symbol found for '{}'", "✗".red(), name.cyan());
        return Ok(1);
    }

    let separator = "─".repeat(60);
    eprintln!("{} Symbol definitions for '{}'", ">>".cyan(), name.cyan());
    eprintln!("{}", separator.dimmed());

    for sym in &matches {
        let kind_tag = format!("[{}]", sym.kind).dimmed();
        let lang_tag = format!("<{}>", sym.language).cyan().dimmed();
        let rel_path = shorten_path(&sym.file, path);
        eprintln!("  {} {} {}:{} {}", sym.name.green().bold(), kind_tag, rel_path.cyan(), sym.line.to_string().yellow(), lang_tag);
        if !sym.signature.is_empty() {
            eprintln!("    {}", sym.signature.dimmed());
        }
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} definition(s) found in {:.3}s", "✓".green(), matches.len(), timer.elapsed_secs());

    Ok(0)
}

/// `cs refs <name>` — Find all references to a symbol.
pub fn run_refs(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
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

    let name_pattern = Regex::new(&format!(r"\b{}\b", regex::escape(name)))
        .map_err(|e| format!("Invalid symbol name: {}", e))?;

    // First, find definitions to exclude them from references
    let all_symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;
    let def_locations: Vec<(&str, usize)> = all_symbols
        .iter()
        .filter(|s| s.name == name)
        .map(|s| (s.file.as_str(), s.line))
        .collect();

    let mut refs: Vec<(String, usize, String, String)> = Vec::new();

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

        if let Some(exts) = &extensions {
            let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
            if !exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
                continue;
            }
        }

        let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        let lang = language_for_ext(&ext).unwrap_or("unknown");

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_idx, line) in content.lines().enumerate() {
            if name_pattern.is_match(line) {
                let line_num = line_idx + 1;
                let file_str = file_path.to_string_lossy();

                // Skip if this line is a definition
                let is_def = def_locations.iter().any(|(f, l)| *f == file_str && *l == line_num);
                if is_def {
                    continue;
                }

                // Skip import/use lines
                let trimmed = line.trim();
                if trimmed.starts_with("use ") || trimmed.starts_with("import ")
                    || trimmed.starts_with("from ") || trimmed.starts_with("require ")
                    || trimmed.starts_with("#include") || trimmed.starts_with("mod ")
                {
                    continue;
                }

                let rel_path = file_str.replacen(path, "", 1).trim_start_matches('/').to_string();
                refs.push((rel_path, line_num, line.trim().to_string(), lang.to_string()));
            }
        }
    }

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = refs
            .iter()
            .map(|(path, line, content, lang)| serde_json::to_value(
                output_schema::RefsResultItem {
                    file: path.clone(),
                    line: *line,
                    content: content.clone(),
                    language: lang.clone(),
                }
            ).unwrap())
            .collect();
        let output = output_schema::envelope(
            "refs", name, "filesystem", refs.len(), elapsed,
            serde_json::json!(results_json),
        );
        output_schema::print_json(&output);
        return Ok(if refs.is_empty() { 1 } else { 0 });
    }

    if refs.is_empty() {
        eprintln!("{} No references found for '{}'", "✗".red(), name.cyan());
        return Ok(1);
    }

    let separator = "─".repeat(60);
    eprintln!("{} References to '{}'", ">>".cyan(), name.cyan());
    eprintln!("{}", separator.dimmed());

    for (path, line, content, lang) in &refs {
        eprintln!("  {}:{}", path.cyan(), line.to_string().yellow());
        eprintln!("    {} {}", content.dimmed(), format!("[{}]", lang).dimmed());
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} reference(s) found in {:.3}s", "✓".green(), refs.len(), timer.elapsed_secs());

    Ok(0)
}

/// `cs callers <name>` — Find all functions that call a specific function.
pub fn run_callers(
    name: &str,
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
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

    // Build call pattern: look for `name(` or `name (` with word boundary
    let call_pattern = Regex::new(&format!(r"\b{}\s*\(", regex::escape(name)))
        .map_err(|e| format!("Invalid symbol name: {}", e))?;

    // Get all symbols for context
    let all_symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;

    // Group symbols by file for quick lookup
    let mut symbols_by_file: HashMap<String, Vec<&Symbol>> = HashMap::new();
    for sym in &all_symbols {
        symbols_by_file.entry(sym.file.clone()).or_default().push(sym);
    }

    // Sort symbols by line number within each file
    for syms in symbols_by_file.values_mut() {
        syms.sort_by_key(|s| s.line);
    }

    let mut callers: Vec<CallerInfo> = Vec::new();

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_path = entry.path();
        let file_str = file_path.to_string_lossy().to_string();

        if let Some(exts) = &extensions {
            let ext = file_path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
            if !exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
                continue;
            }
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_idx, line) in content.lines().enumerate() {
            if call_pattern.is_match(line) {
                let line_num = line_idx + 1;
                let file_syms = symbols_by_file.get(&file_str);

                // Find which function contains this call
                let caller = file_syms.and_then(|syms| {
                    syms.iter()
                        .rev()  // Most recent definition before this line
                        .find(|s| {
                            s.kind == SymbolKind::Function || s.kind == SymbolKind::Method
                                || s.kind == SymbolKind::Class
                        })
                        .filter(|s| s.line <= line_num)
                });

                if let Some(caller_sym) = caller {
                    // Don't report self-calls (function calling itself)
                    if caller_sym.name == name {
                        continue;
                    }

                    let rel_caller_path = shorten_path(&caller_sym.file, path);
                    let rel_call_site_path = file_str.replacen(path, "", 1).trim_start_matches('/').to_string();
                    callers.push(CallerInfo {
                        caller_name: caller_sym.name.clone(),
                        caller_file: rel_caller_path,
                        caller_line: caller_sym.line,
                        caller_kind: caller_sym.kind,
                        call_site_file: rel_call_site_path,
                        call_site_line: line_num,
                        call_context: line.trim().to_string(),
                    });
                }
            }
        }
    }

    // Deduplicate callers (same caller calling same function multiple times)
    let mut seen = std::collections::HashSet::new();
    callers.retain(|c| {
        let key = format!("{}:{}", c.caller_name, c.caller_file);
        seen.insert(key)
    });

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = callers
            .iter()
            .map(|c| serde_json::to_value(output_schema::CallersResultItem {
                caller_name: c.caller_name.clone(),
                caller_file: c.caller_file.clone(),
                caller_line: c.caller_line,
                caller_kind: c.caller_kind.to_string(),
                call_site_line: c.call_site_line,
                call_context: c.call_context.clone(),
            }).unwrap())
            .collect();
        let output = output_schema::envelope(
            "callers", name, "filesystem", callers.len(), elapsed,
            serde_json::json!(results_json),
        );
        output_schema::print_json(&output);
        return Ok(if callers.is_empty() { 1 } else { 0 });
    }

    if callers.is_empty() {
        eprintln!("{} No callers found for '{}'", "✗".red(), name.cyan());
        return Ok(1);
    }

    let separator = "─".repeat(60);
    eprintln!("{} Callers of '{}'", ">>".cyan(), name.cyan());
    eprintln!("{}", separator.dimmed());

    for c in &callers {
        let kind_tag = format!("[{}]", c.caller_kind).dimmed();
        eprintln!("  {} {} {}:{} {}", c.caller_name.green().bold(), kind_tag, c.caller_file.cyan(), c.caller_line.to_string().yellow(), "→ calls".dimmed());
        eprintln!("    {} at line {}", c.call_context.dimmed(), c.call_site_line.to_string().yellow());
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} caller(s) found in {:.3}s", "✓".green(), callers.len(), timer.elapsed_secs());

    Ok(0)
}

/// `cs symbols [path]` — List all symbols in a file or directory.
pub fn run_symbols(
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    symbol_type: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    limit: Option<usize>,
    json: bool,
) -> Result<i32, String> {
    let timer = Timer::new();
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    let mut symbols = walk_and_extract_symbols(path, extensions.as_deref(), no_ignore, depth)?;

    // Filter by symbol type if specified
    if let Some(kind_str) = symbol_type {
        let kind_lower = kind_str.to_lowercase();
        symbols.retain(|s| s.kind.to_string().contains(&kind_lower));
    }

    // Sort by file, then by line number
    symbols.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.line.cmp(&b.line)));

    let effective_limit = limit.unwrap_or(100);
    symbols.truncate(effective_limit);

    if json {
        let elapsed = timer.elapsed_secs();
        let results_json: Vec<serde_json::Value> = symbols
            .iter()
            .map(|s| serde_json::to_value(output_schema::SymbolsResultItem {
                name: s.name.clone(),
                kind: s.kind.to_string(),
                file: shorten_path(&s.file, path),
                line: s.line,
                column: s.column,
                language: s.language.clone(),
                signature: s.signature.clone(),
            }).unwrap())
            .collect();
        let output = output_schema::envelope_with_extra(
            "symbols", path, "filesystem", symbols.len(), elapsed,
            serde_json::json!(results_json),
            serde_json::json!({"total_symbols": symbols.len(), "languages": unique_languages(&symbols)}),
        );
        output_schema::print_json(&output);
        return Ok(if symbols.is_empty() { 1 } else { 0 });
    }

    if symbols.is_empty() {
        eprintln!("{} No symbols found in '{}'", "✗".red(), path.cyan());
        return Ok(1);
    }

    let separator = "─".repeat(60);
    eprintln!("{} Symbols in '{}'", ">>".cyan(), path.cyan());
    eprintln!("{}", separator.dimmed());

    let mut current_file = String::new();
    for sym in &symbols {
        let display_file = shorten_path(&sym.file, path);
        if display_file != current_file {
            current_file = display_file.clone();
            eprintln!("  {}", display_file.cyan().bold());
        }
        let kind_tag = format!("[{}]", sym.kind).dimmed();
        eprintln!("    {:>4}: {} {} {}", sym.line.to_string().yellow(), sym.name.green(), kind_tag, sym.signature.dimmed());
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} symbol(s) in {:.3}s (showing up to {})", "✓".green(), symbols.len(), timer.elapsed_secs(), effective_limit);

    Ok(0)
}

// ---------------------------------------------------------------------------
// Helper types and functions
// ---------------------------------------------------------------------------

struct CallerInfo {
    caller_name: String,
    caller_file: String,
    caller_line: usize,
    caller_kind: SymbolKind,
    call_site_file: String,
    call_site_line: usize,
    call_context: String,
}

fn shorten_path(full_path: &str, base: &str) -> String {
    if let Some(stripped) = full_path.strip_prefix(base) {
        stripped.trim_start_matches('/').to_string()
    } else {
        full_path.to_string()
    }
}

fn unique_languages(symbols: &[Symbol]) -> Vec<String> {
    let mut langs: Vec<String> = symbols.iter().map(|s| s.language.clone()).collect();
    langs.sort();
    langs.dedup();
    langs
}

// ---------------------------------------------------------------------------
// Helper types and functions
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_symbol_rust() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("lib.rs"),
            "pub fn search_files() {}\nstruct Config { name: String }\nenum Color { Red, Blue }\n",
        )
        .unwrap();

        let result = run_symbol("search_files", dir.path().to_str().unwrap(), None, None, None, None, true, None, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_symbol_python() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("app.py"),
            "class UserService:\n    def get_user(self, id):\n        pass\n",
        )
        .unwrap();

        let result = run_symbol("UserService", dir.path().to_str().unwrap(), None, None, None, None, true, None, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_symbol_not_found() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

        let result = run_symbol("nonexistent", dir.path().to_str().unwrap(), None, None, None, None, true, None, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_symbols_list() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("calc.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }\nfn subtract(a: i32, b: i32) -> i32 { a - b }\n").unwrap();

        let result = run_symbols(dir.path().to_str().unwrap(), None, None, None, None, true, None, None, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_refs_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "fn helper() {}\nfn main() {\n    helper();\n}\n",
        )
        .unwrap();

        let result = run_refs("helper", dir.path().to_str().unwrap(), None, None, None, true, None, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_callers_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "fn helper() {}\nfn caller_one() { helper(); }\nfn caller_two() { helper(); }\n",
        )
        .unwrap();

        let result = run_callers("helper", dir.path().to_str().unwrap(), None, None, None, true, None, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_infer_symbol_kind() {
        assert_eq!(infer_symbol_kind("pub fn hello() {}", "rust"), SymbolKind::Function);
        assert_eq!(infer_symbol_kind("struct Config {}", "rust"), SymbolKind::Struct);
        assert_eq!(infer_symbol_kind("def foo(): pass", "python"), SymbolKind::Function);
        assert_eq!(infer_symbol_kind("class MyClass {}", "python"), SymbolKind::Class);
        assert_eq!(infer_symbol_kind("func Bar() {}", "go"), SymbolKind::Function);
    }

    #[test]
    fn test_language_for_ext() {
        assert_eq!(language_for_ext("rs"), Some("rust"));
        assert_eq!(language_for_ext("py"), Some("python"));
        assert_eq!(language_for_ext("go"), Some("go"));
        assert_eq!(language_for_ext("xyz"), None);
    }

    #[test]
    fn test_extract_symbols_from_content() {
        let content = "pub fn add(a: i32, b: i32) -> i32 { a + b }\nstruct Config { name: String }\n";
        let symbols = extract_symbols_from_content(content, "/test/main.rs", "rs");
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "add");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[1].name, "Config");
        assert_eq!(symbols[1].kind, SymbolKind::Struct);
    }

    #[test]
    fn test_symbol_type_filter() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("lib.rs"),
            "pub fn search() {}\nstruct Config {}\n",
        )
        .unwrap();

        // Filter for functions only
        let result = run_symbols(dir.path().to_str().unwrap(), None, None, None, Some("function"), true, None, None, false);
        assert!(result.is_ok());
    }
}
