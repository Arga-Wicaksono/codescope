//! Shared data types for codescope.

use serde::{Deserialize, Serialize};

/// Result of a file search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub filename: String,
    pub path: String,
    pub score: i64,
}

/// Result of a content search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResult {
    pub file: String,
    pub path: String,
    pub line: usize,
    pub content: String,
    pub score: i64,
}

/// Result of a web search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// File type presets for filtering.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum FileType {
    Rust,
    Python,
    Js,
    Web,
    C,
    Cpp,
    Go,
    Java,
    Config,
    Doc,
    Data,
    Shell,
}

impl FileType {
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            FileType::Rust => &["rs"],
            FileType::Python => &["py", "pyi", "pyw"],
            FileType::Js => &["js", "jsx", "ts", "tsx", "mjs", "cjs"],
            FileType::Web => &["html", "htm", "css", "scss", "sass", "less", "vue", "svelte"],
            FileType::C => &["c", "h"],
            FileType::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx", "h"],
            FileType::Go => &["go"],
            FileType::Java => &["java", "kt", "kts"],
            FileType::Config => &["toml", "yaml", "yml", "json", "ini", "cfg", "conf", "env"],
            FileType::Doc => &["md", "txt", "rst", "adoc", "org"],
            FileType::Data => &["csv", "tsv", "xml", "sql"],
            FileType::Shell => &["sh", "bash", "zsh", "fish", "ps1"],
        }
    }
}

/// A single entry in the search history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub command: String,
    pub pattern: String,
    pub path: String,
    pub results: usize,
    pub elapsed_secs: f64,
}

/// Match mode for content search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchMode {
    Fuzzy,
    Exact,
    Regex,
}

/// Statistics for a single language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangStats {
    pub language: String,
    pub files: usize,
    pub lines: usize,
    pub bytes: usize,
    pub percentage: f64,
}
