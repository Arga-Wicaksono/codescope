//! CodeScope (`cs`) — Repository Intelligence Engine for AI & Developers.
//!
//! Makes repositories understandable instantly — fast code retrieval,
//! symbol lookup, and AI-ready context extraction.
//!
//! Features:
//! - .gitignore-aware file/content search (uses the `ignore` crate, same as ripgrep)
//! - Symbol intelligence: find function/class/struct definitions across 7+ languages
//! - Cross-repository search with workspace auto-discovery
//! - Structured JSON output (`-j`) for AI agents and scripting pipelines
//! - Smart case: case-insensitive unless pattern contains uppercase
//! - Stdin pipe support: `cat file | cs content "pattern"`
//! - Shell completions: `cs completions bash|zsh|fish|powershell|elvish`
//! - Interactive fuzzy-select mode for all search types
//! - File type presets: `--type rust`, `--type web`, etc.
//! - Replace mode: `--replace 'text'` (dry run) and `--write` (apply)
//! - Open files in editor: `cs open "main" --line 42`
//! - Recently modified files: `cs recent --since "2h ago"`
//! - Repository statistics: `cs stats --type rust`

pub mod types;
pub mod utils;
pub mod validate;
pub mod output;
pub mod output_schema;
pub mod config;
pub mod file_search;
pub mod content_search;
pub mod open;
pub mod recent;
pub mod where_cmd;
pub mod explain;
pub mod history;
pub mod across;
pub mod stats;
pub mod cli;

#[cfg(feature = "interactive")]
pub mod interactive;

#[cfg(feature = "web-search")]
pub mod web_search;
