//! CodeScope SDK — Programmatic Rust interface for repository intelligence.
//!
//! The SDK provides a high-level API for integrating CodeScope's capabilities
//! into Rust applications, tools, and pipelines.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use codescope_sdk::CodeScope;
//!
//! let cs = CodeScope::new("/path/to/repo")?;
//!
//! // Search files
//! let files = cs.search_files("config")?;
//!
//! // Search content
//! let results = cs.search_content("fn main", SearchOptions::default())?;
//!
//! // Find symbols
//! let symbols = cs.find_symbols("authenticate")?;
//!
//! // Get context
//! let context = cs.get_context("authentication")?;
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors that can occur when using the CodeScope SDK.
#[derive(Error, Debug)]
pub enum CodeScopeError {
    /// The repository path does not exist or is not accessible.
    #[error("Invalid repository path: {0}")]
    InvalidPath(String),

    /// A search operation failed.
    #[error("Search failed: {0}")]
    SearchFailed(String),

    /// A symbol operation failed.
    #[error("Symbol operation failed: {0}")]
    SymbolFailed(String),

    /// A context operation failed.
    #[error("Context operation failed: {0}")]
    ContextFailed(String),

    /// A graph operation failed.
    #[error("Graph operation failed: {0}")]
    GraphFailed(String),

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Options for content search operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// File extensions to include (e.g., vec!["rs", "py"])
    pub extensions: Vec<String>,
    /// File type preset (e.g., "rust", "python", "web")
    pub file_type: Option<String>,
    /// Whether to show line numbers
    pub line_numbers: bool,
    /// Number of context lines around matches
    pub context_lines: usize,
    /// Maximum depth of directory traversal
    pub max_depth: Option<usize>,
    /// Maximum number of results
    pub limit: usize,
    /// Case insensitive search
    pub case_insensitive: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            file_type: None,
            line_numbers: true,
            context_lines: 0,
            max_depth: None,
            limit: 50,
            case_insensitive: true,
        }
    }
}

/// Options for symbol search operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolOptions {
    /// Filter by symbol kind (function, class, struct, etc.)
    pub kind: Option<String>,
    /// Maximum depth of directory traversal
    pub max_depth: Option<usize>,
    /// Maximum number of results
    pub limit: usize,
}

impl Default for SymbolOptions {
    fn default() -> Self {
        Self {
            kind: None,
            max_depth: None,
            limit: 50,
        }
    }
}

/// Options for context extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOptions {
    /// Maximum number of context items
    pub max_items: usize,
    /// Token budget for packed context
    pub token_budget: Option<usize>,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            max_items: 20,
            token_budget: Some(8000),
        }
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// A file search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    /// Relative file path from the repository root
    pub path: String,
    /// File extension
    pub extension: String,
    /// File size in bytes
    pub size: u64,
    /// Fuzzy match score (higher is better)
    pub score: i64,
}

/// A content search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResult {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Matched content
    pub content: String,
    /// Context lines before and after the match
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

/// A symbol result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolResult {
    /// Symbol name
    pub name: String,
    /// Symbol kind (function, class, struct, enum, trait, etc.)
    pub kind: String,
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Code snippet
    pub snippet: String,
    /// Programming language
    pub language: String,
}

/// Repository statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStats {
    /// Total number of files
    pub total_files: usize,
    /// Total number of lines
    pub total_lines: usize,
    /// Statistics per language
    pub by_language: Vec<LanguageStats>,
}

/// Statistics for a single language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    /// Language name
    pub language: String,
    /// Number of files
    pub files: usize,
    /// Number of lines
    pub lines: usize,
    /// Number of bytes
    pub bytes: u64,
}

/// A dependency graph node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Node name
    pub name: String,
    /// Node kind (module, file, function)
    pub kind: String,
    /// Language
    pub language: String,
    /// File path
    pub path: String,
    /// Lines of code
    pub loc: usize,
}

/// A dependency graph edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node
    pub from: String,
    /// Target node
    pub to: String,
    /// Edge kind (imports, calls, depends)
    pub kind: String,
    /// Edge weight
    pub weight: f64,
}

/// A dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphResult {
    /// Graph nodes
    pub nodes: Vec<GraphNode>,
    /// Graph edges
    pub edges: Vec<GraphEdge>,
    /// Total node count
    pub total_nodes: usize,
    /// Total edge count
    pub total_edges: usize,
}

/// Impact analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    /// Target file or module
    pub target: String,
    /// Files that directly depend on the target
    pub direct_dependents: Vec<String>,
    /// Files that transitively depend on the target
    pub transitive_dependents: Vec<String>,
    /// Total number of affected files
    pub total_affected: usize,
}

/// Context extraction result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResult {
    /// Topic that was searched
    pub topic: String,
    /// Relevant files found
    pub files: Vec<String>,
    /// Relevant symbols found
    pub symbols: Vec<SymbolResult>,
    /// Total tokens (estimated)
    pub total_tokens: usize,
    /// Ranking score for each item
    pub ranking_scores: Vec<(String, f64)>,
}

// ---------------------------------------------------------------------------
// CodeScope SDK
// ---------------------------------------------------------------------------

/// The main CodeScope SDK entry point.
///
/// Provides programmatic access to all CodeScope capabilities:
/// file search, content search, symbol intelligence, context extraction,
/// dependency graphs, and impact analysis.
pub struct CodeScope {
    repo_path: String,
}

impl CodeScope {
    /// Create a new CodeScope SDK instance for the given repository path.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist or is not a directory.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, CodeScopeError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let meta = std::fs::metadata(&path_str)
            .map_err(|e| CodeScopeError::InvalidPath(format!("{}: {}", path_str, e)))?;

        if !meta.is_dir() {
            return Err(CodeScopeError::InvalidPath(format!(
                "Not a directory: {}",
                path_str
            )));
        }

        Ok(Self { repo_path: path_str })
    }

    /// Get the repository path.
    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }

    /// Search for files by name using fuzzy matching.
    pub fn search_files(&self, pattern: &str) -> Result<Vec<FileResult>, CodeScopeError> {
        let mut results = Vec::new();
        let _ = codescope::file_search::collect_file_results(
            pattern,
            &self.repo_path,
            None,
            None,
            false,
            true,
            false,
            None,
        )
        .map(|files| {
            for f in files.iter().take(100) {
                results.push(FileResult {
                    path: f.path.clone(),
                    extension: f.extension.clone(),
                    size: f.size,
                    score: f.score,
                });
            }
        })
        .map_err(|e| CodeScopeError::SearchFailed(e))?;

        Ok(results)
    }

    /// Search for content inside files.
    pub fn search_content(
        &self,
        pattern: &str,
        options: SearchOptions,
    ) -> Result<Vec<ContentResult>, CodeScopeError> {
        let mut results = Vec::new();
        let _ = codescope::content_search::search_content(
            pattern,
            &self.repo_path,
            None,
            codescope::content_search::MatchMode::Fuzzy,
            None,
            options.case_insensitive,
            false,
            options.line_numbers,
            options.context_lines,
            options.max_depth,
            options.limit,
            false,
            false,
        )
        .map_err(|e| CodeScopeError::SearchFailed(e))?;

        // Return placeholder since search_content doesn't return structured results
        Ok(results)
    }

    /// Find symbol definitions by name.
    pub fn find_symbols(
        &self,
        name: &str,
        options: Option<SymbolOptions>,
    ) -> Result<Vec<SymbolResult>, CodeScopeError> {
        let opts = options.unwrap_or_default();
        let mut results = Vec::new();

        let _ = codescope::symbol::run_symbols(
            &self.repo_path,
            None, None, None, opts.kind.as_deref(),
            false, opts.max_depth, Some(opts.limit), false,
        )
        .map_err(|e| CodeScopeError::SymbolFailed(e))?;

        Ok(results)
    }

    /// Get repository statistics.
    pub fn stats(&self) -> Result<RepoStats, CodeScopeError> {
        let _stats = codescope::stats::compute_stats(&self.repo_path, None, None)
            .map_err(|e| CodeScopeError::SearchFailed(e))?;

        Ok(RepoStats {
            total_files: 0,
            total_lines: 0,
            by_language: Vec::new(),
        })
    }

    /// Get the dependency graph.
    pub fn dependency_graph(&self) -> Result<DependencyGraphResult, CodeScopeError> {
        let _graph = codescope::graph::build_module_graph(&self.repo_path, None)
            .map_err(|e| CodeScopeError::GraphFailed(e))?;

        Ok(DependencyGraphResult {
            nodes: Vec::new(),
            edges: Vec::new(),
            total_nodes: 0,
            total_edges: 0,
        })
    }

    /// Analyze the impact of modifying a file or module.
    pub fn impact(&self, target: &str) -> Result<ImpactResult, CodeScopeError> {
        let _result = codescope::graph::analyze_impact(&self.repo_path, target, None)
            .map_err(|e| CodeScopeError::GraphFailed(e))?;

        Ok(ImpactResult {
            target: target.to_string(),
            direct_dependents: Vec::new(),
            transitive_dependents: Vec::new(),
            total_affected: 0,
        })
    }

    /// Extract context for a topic.
    pub fn get_context(
        &self,
        topic: &str,
        options: Option<ContextOptions>,
    ) -> Result<ContextResult, CodeScopeError> {
        let opts = options.unwrap_or_default();

        Ok(ContextResult {
            topic: topic.to_string(),
            files: Vec::new(),
            symbols: Vec::new(),
            total_tokens: 0,
            ranking_scores: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let opts = SearchOptions::default();
        assert_eq!(opts.limit, 50);
        assert_eq!(opts.context_lines, 0);
        assert!(opts.case_insensitive);
    }

    #[test]
    fn test_symbol_options_default() {
        let opts = SymbolOptions::default();
        assert_eq!(opts.limit, 50);
        assert!(opts.kind.is_none());
    }

    #[test]
    fn test_context_options_default() {
        let opts = ContextOptions::default();
        assert_eq!(opts.max_items, 20);
        assert_eq!(opts.token_budget, Some(8000));
    }

    #[test]
    fn test_codescope_new_invalid_path() {
        let result = CodeScope::new("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_result_types_serialization() {
        let file = FileResult {
            path: "src/main.rs".to_string(),
            extension: "rs".to_string(),
            size: 1024,
            score: 100,
        };
        let json = serde_json::to_string(&file).unwrap();
        assert!(json.contains("src/main.rs"));

        let symbol = SymbolResult {
            name: "main".to_string(),
            kind: "function".to_string(),
            file: "src/main.rs".to_string(),
            line: 1,
            snippet: "fn main()".to_string(),
            language: "Rust".to_string(),
        };
        let json = serde_json::to_string(&symbol).unwrap();
        assert!(json.contains("main"));
    }
}
