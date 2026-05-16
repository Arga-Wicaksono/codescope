//! codescope (`cs`) — A blazing fast Rust CLI search tool.
//!
//! Search files by name, search content inside files with regex support,
//! and search the web directly from your terminal.
//!
//! Features:
//! - .gitignore-aware file/content search (uses the `ignore` crate, same as ripgrep)
//! - Smart case: case-insensitive unless pattern contains uppercase
//! - Stdin pipe support: `cat file | cs content "pattern"`
//! - Shell completions: `cs completions bash|zsh|fish|powershell|elvish`
//! - Interactive fuzzy-select mode for all search types
//! - File type presets: `--type rust`, `--type web`, etc.
//! - Replace mode: `--replace 'text'` (dry run) and `--write` (apply)
//! - Count mode: `--count` for per-file match counts
//! - Invert match: `--invert` to show non-matching lines
//! - Open files in editor: `cs open "main" --line 42`
//! - Recently modified files: `cs recent --since "2h ago"`
//! - Find definitions: `cs where "parse_config"`
//! - Explain regex: `cs explain "\\s+"`
//! - Search history: `cs history`

pub mod types;
pub mod utils;
pub mod validate;
pub mod output;
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
pub mod graph;
pub mod impact;
pub mod context;
pub mod serve;
pub mod symbol;
pub mod cli;
pub mod semantic;
pub mod cache;
pub mod rewrite;
pub mod lsp_bridge;

#[cfg(feature = "interactive")]
pub mod interactive;

#[cfg(feature = "web-search")]
pub mod web_search;
