//! Plugin system for CodeScope.
//!
//! CodeScope's plugin architecture allows extending functionality without
//! modifying the core binary. Plugins can:
//!
//! - Register custom search providers
//! - Add new output formatters
//! - Hook into the search pipeline (pre/post processing)
//! - Provide custom symbol extractors for new languages
//!
//! # Plugin Discovery
//!
//! Plugins are discovered from:
//! 1. `~/.codescope/plugins/` directory
//! 2. `.codescope/plugins/` in the project directory
//! 3. Plugins specified in `~/.codescope.json` under the `"plugins"` key
//!
//! # Example Plugin (WASM-based)
//!
//! ```rust,ignore
//! use codescope::plugin::{Plugin, PluginContext, SearchResult};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "my-plugin" }
//!     fn version(&self) -> &str { "0.1.0" }
//!
//!     fn on_search_result(&self, ctx: &PluginContext, result: &mut SearchResult) {
//!         // Modify search results
//!         result.score *= 1.5;
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Metadata about a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name (unique identifier)
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Author name/email
    pub author: String,
    /// Supported CodeScope version range (e.g., ">=1.2.0")
    pub min_codescope_version: Option<String>,
    /// Hooks this plugin subscribes to
    pub hooks: Vec<HookPoint>,
    /// Supported languages for symbol extraction
    pub languages: Vec<String>,
}

/// Hook points where plugins can intercept execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookPoint {
    /// Before file search begins (can filter/modify query)
    PreFileSearch,
    /// After file search completes (can filter/sort results)
    PostFileSearch,
    /// Before content search begins
    PreContentSearch,
    /// After content search completes
    PostContentSearch,
    /// Before symbol extraction
    PreSymbolExtract,
    /// After symbol extraction
    PostSymbolExtract,
    /// Before context packing
    PreContextPack,
    /// After context packing
    PostContextPack,
    /// On output formatting (can add custom formatters)
    OnOutputFormat,
}

/// A search result that plugins can modify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchResult {
    /// File path
    pub file: String,
    /// Line number (0 if not applicable)
    pub line: usize,
    /// Column number (0 if not applicable)
    pub column: usize,
    /// Matched content
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    pub score: f64,
    /// Extra metadata added by plugins
    pub metadata: HashMap<String, String>,
}

/// Context provided to plugin hooks.
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// The repository root path
    pub repo_path: PathBuf,
    /// The current query/pattern
    pub query: String,
    /// Search mode (fuzzy, exact, regex)
    pub search_mode: String,
    /// File extensions filter
    pub extensions: Vec<String>,
    /// Plugin-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Result of plugin initialization.
pub enum PluginResult {
    /// Plugin loaded successfully
    Ok(Box<dyn Plugin>),
    /// Plugin failed to load
    Err(String),
}

/// A custom symbol extractor for a specific language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolExtractorDef {
    /// Language identifier (e.g., "kotlin", "swift")
    pub language: String,
    /// File extensions this extractor handles
    pub extensions: Vec<String>,
    /// Regex patterns for different symbol kinds
    pub patterns: Vec<SymbolPattern>,
}

/// A regex pattern for extracting symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolPattern {
    /// Symbol kind (function, class, struct, enum, trait, interface, etc.)
    pub kind: String,
    /// Named capture group for the symbol name (must have a "name" group)
    pub pattern: String,
    /// Description of what this pattern matches
    pub description: String,
}

// ---------------------------------------------------------------------------
// Plugin Trait
// ---------------------------------------------------------------------------

/// The core trait that all CodeScope plugins must implement.
///
/// Plugins receive context about the current operation and can modify
/// results, add metadata, or provide custom functionality.
pub trait Plugin: Send + Sync {
    /// Return plugin metadata.
    fn metadata(&self) -> PluginMetadata;

    /// Called when the plugin is first loaded.
    /// Use this to validate configuration and initialize resources.
    fn on_load(&mut self, _config: &HashMap<String, serde_json::Value>) -> Result<(), String> {
        Ok(())
    }

    /// Called when the plugin is being unloaded.
    fn on_unload(&mut self) {}

    // ── Search hooks ─────────────────────────────────────────────────────

    /// Hook called before file search. Can modify the query or filter.
    fn on_pre_file_search(
        &self,
        _ctx: &PluginContext,
        query: &mut String,
    ) -> Result<(), String> {
        let _ = query;
        Ok(())
    }

    /// Hook called after file search. Can filter, sort, or annotate results.
    fn on_post_file_search(
        &self,
        _ctx: &PluginContext,
        results: &mut Vec<PluginSearchResult>,
    ) -> Result<(), String> {
        let _ = results;
        Ok(())
    }

    /// Hook called before content search.
    fn on_pre_content_search(
        &self,
        _ctx: &PluginContext,
        query: &mut String,
    ) -> Result<(), String> {
        let _ = query;
        Ok(())
    }

    /// Hook called after content search.
    fn on_post_content_search(
        &self,
        _ctx: &PluginContext,
        results: &mut Vec<PluginSearchResult>,
    ) -> Result<(), String> {
        let _ = results;
        Ok(())
    }

    // ── Symbol hooks ─────────────────────────────────────────────────────

    /// Hook called before symbol extraction.
    fn on_pre_symbol_extract(
        &self,
        _ctx: &PluginContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Hook called after symbol extraction.
    fn on_post_symbol_extract(
        &self,
        _ctx: &PluginContext,
        results: &mut Vec<PluginSearchResult>,
    ) -> Result<(), String> {
        let _ = results;
        Ok(())
    }

    // ── Context hooks ────────────────────────────────────────────────────

    /// Hook called before context packing.
    fn on_pre_context_pack(
        &self,
        _ctx: &PluginContext,
        description: &mut String,
        budget: &mut usize,
    ) -> Result<(), String> {
        let _ = (description, budget);
        Ok(())
    }

    /// Hook called after context packing.
    fn on_post_context_pack(
        &self,
        _ctx: &PluginContext,
        packed: &mut String,
    ) -> Result<(), String> {
        let _ = packed;
        Ok(())
    }

    // ── Custom formatters ────────────────────────────────────────────────

    /// Return a list of custom output format names this plugin provides.
    fn custom_formats(&self) -> Vec<String> {
        Vec::new()
    }

    /// Format output using a custom format.
    fn format_output(
        &self,
        _format_name: &str,
        _results: &[PluginSearchResult],
    ) -> Result<String, String> {
        Err(format!("Unknown format"))
    }

    // ── Custom symbol extractors ─────────────────────────────────────────

    /// Return custom symbol extractor definitions for additional languages.
    fn symbol_extractors(&self) -> Vec<SymbolExtractorDef> {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Plugin Manager
// ---------------------------------------------------------------------------

/// Manages plugin discovery, loading, and execution.
pub struct PluginManager {
    /// Loaded plugins
    plugins: Vec<Box<dyn Plugin>>,
    /// Plugin configuration
    config: HashMap<String, HashMap<String, serde_json::Value>>,
    /// Whether plugin system is enabled
    enabled: bool,
}

impl PluginManager {
    /// Create a new plugin manager (disabled by default).
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            config: HashMap::new(),
            enabled: false,
        }
    }

    /// Enable the plugin system.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Check if the plugin system is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Load a plugin and return its metadata.
    pub fn load_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<PluginMetadata, String> {
        let meta = plugin.metadata();
        let plugin_config = self.config.remove(&meta.name).unwrap_or_default();

        plugin.on_load(&plugin_config)?;

        if self.plugins.iter().any(|p| p.metadata().name == meta.name) {
            return Err(format!("Plugin '{}' is already loaded", meta.name));
        }

        let name = meta.name.clone();
        self.plugins.push(plugin);
        Ok(self.plugins.iter().find(|p| p.metadata().name == name).unwrap().metadata())
    }

    /// Unload a plugin by name.
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), String> {
        let idx = self
            .plugins
            .iter()
            .position(|p| p.metadata().name == name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?;
        self.plugins[idx].on_unload();
        self.plugins.remove(idx);
        Ok(())
    }

    /// List all loaded plugins.
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.iter().map(|p| p.metadata()).collect()
    }

    /// Get the number of loaded plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Set configuration for a plugin.
    pub fn set_config(&mut self, plugin_name: &str, config: HashMap<String, serde_json::Value>) {
        self.config.insert(plugin_name.to_string(), config);
    }

    /// Discover plugins from the filesystem.
    ///
    /// Searches in order:
    /// 1. `~/.codescope/plugins/`
    /// 2. `.codescope/plugins/` in the current directory
    pub fn discover_plugins(&self) -> Vec<PathBuf> {
        let mut plugin_dirs = Vec::new();

        // Home directory plugins
        if let Some(home) = dirs::home_dir() {
            let global_plugins = home.join(".codescope").join("plugins");
            if global_plugins.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&global_plugins) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_dir() || path.extension().map_or(false, |e| e == "so" || e == "dylib" || e == "dll") {
                            plugin_dirs.push(path);
                        }
                    }
                }
            }
        }

        // Project-local plugins
        let local_plugins = PathBuf::from(".codescope").join("plugins");
        if local_plugins.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&local_plugins) {
                for entry in entries.filter_map(|e| e.ok()) {
                    plugin_dirs.push(entry.path());
                }
            }
        }

        plugin_dirs
    }

    /// Collect all custom symbol extractors from loaded plugins.
    pub fn collect_symbol_extractors(&self) -> Vec<SymbolExtractorDef> {
        let mut extractors = Vec::new();
        for plugin in &self.plugins {
            extractors.extend(plugin.symbol_extractors());
        }
        extractors
    }

    /// Collect all custom format names from loaded plugins.
    pub fn collect_custom_formats(&self) -> Vec<String> {
        let mut formats = Vec::new();
        for plugin in &self.plugins {
            formats.extend(plugin.custom_formats());
        }
        formats.sort();
        formats.dedup();
        formats
    }

    /// Get the plugin configuration as JSON.
    pub fn to_json(&self) -> serde_json::Value {
        let plugins: Vec<serde_json::Value> = self
            .plugins
            .iter()
            .map(|p| {
                let m = p.metadata();
                serde_json::json!({
                    "name": m.name,
                    "version": m.version,
                    "description": m.description,
                    "author": m.author,
                    "hooks": m.hooks.iter().map(|h| format!("{:?}", h)).collect::<Vec<_>>(),
                    "languages": m.languages,
                })
            })
            .collect();

        serde_json::json!({
            "enabled": self.enabled,
            "plugin_count": self.plugins.len(),
            "plugins": plugins,
        })
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Built-in example plugins
// ---------------------------------------------------------------------------

/// Example plugin that boosts the score of files that have been recently modified.
pub struct RecencyBoostPlugin {
    boost_factor: f64,
    max_age_hours: u64,
}

impl RecencyBoostPlugin {
    pub fn new(boost_factor: f64, max_age_hours: u64) -> Self {
        Self { boost_factor, max_age_hours }
    }
}

impl Plugin for RecencyBoostPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "recency-boost".to_string(),
            version: "1.0.0".to_string(),
            description: "Boost search ranking for recently modified files".to_string(),
            author: "CodeScope".to_string(),
            min_codescope_version: Some("1.2.0".to_string()),
            hooks: vec![HookPoint::PostFileSearch, HookPoint::PostContentSearch],
            languages: vec![],
        }
    }

    fn on_post_file_search(
        &self,
        ctx: &PluginContext,
        results: &mut Vec<PluginSearchResult>,
    ) -> Result<(), String> {
        let now = std::time::SystemTime::now();
        let max_age = std::time::Duration::from_secs(self.max_age_hours * 3600);

        for result in results.iter_mut() {
            let file_path = ctx.repo_path.join(&result.file);
            if let Ok(metadata) = std::fs::metadata(&file_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age < max_age {
                            let recency_ratio = 1.0 - (age.as_secs_f64() / max_age.as_secs_f64());
                            result.score *= 1.0 + (self.boost_factor * recency_ratio);
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(())
    }
}

/// Example plugin that adds Markdown documentation formatting.
pub struct MarkdownFormatterPlugin;

impl Plugin for MarkdownFormatterPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "markdown-formatter".to_string(),
            version: "1.0.0".to_string(),
            description: "Format search results as Markdown tables and code blocks".to_string(),
            author: "CodeScope".to_string(),
            min_codescope_version: Some("1.2.0".to_string()),
            hooks: vec![HookPoint::OnOutputFormat],
            languages: vec![],
        }
    }

    fn custom_formats(&self) -> Vec<String> {
        vec!["markdown".to_string()]
    }

    fn format_output(
        &self,
        format_name: &str,
        results: &[PluginSearchResult],
    ) -> Result<String, String> {
        if format_name != "markdown" {
            return Err(format!("Unknown format: {}", format_name));
        }

        let mut md = String::new();
        md.push_str("| File | Line | Content | Score |\n");
        md.push_str("|------|------|---------|-------|\n");

        for result in results {
            md.push_str(&format!(
                "| `{}` | {} | `{}` | {:.2} |\n",
                result.file, result.line, result.content, result.score
            ));
        }

        Ok(md)
    }
}

/// Example plugin that provides additional language symbol extractors.
pub struct ExtraLanguagesPlugin;

impl Plugin for ExtraLanguagesPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "extra-languages".to_string(),
            version: "1.0.0".to_string(),
            description: "Add symbol extraction support for Kotlin, Swift, Ruby, and PHP".to_string(),
            author: "CodeScope".to_string(),
            min_codescope_version: Some("1.2.0".to_string()),
            hooks: vec![HookPoint::PostSymbolExtract],
            languages: vec!["kotlin".to_string(), "swift".to_string(), "ruby".to_string(), "php".to_string()],
        }
    }

    fn symbol_extractors(&self) -> Vec<SymbolExtractorDef> {
        vec![
            SymbolExtractorDef {
                language: "kotlin".to_string(),
                extensions: vec!["kt".to_string(), "kts".to_string()],
                patterns: vec![
                    SymbolPattern {
                        kind: "function".to_string(),
                        pattern: r"(?:fun\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Kotlin function definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "class".to_string(),
                        pattern: r"(?:class\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Kotlin class definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "interface".to_string(),
                        pattern: r"(?:interface\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Kotlin interface definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "object".to_string(),
                        pattern: r"(?:object\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Kotlin object declaration".to_string(),
                    },
                ],
            },
            SymbolExtractorDef {
                language: "swift".to_string(),
                extensions: vec!["swift".to_string()],
                patterns: vec![
                    SymbolPattern {
                        kind: "function".to_string(),
                        pattern: r"(?:func\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Swift function definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "class".to_string(),
                        pattern: r"(?:class\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Swift class definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "struct".to_string(),
                        pattern: r"(?:struct\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Swift struct definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "protocol".to_string(),
                        pattern: r"(?:protocol\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Swift protocol definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "enum".to_string(),
                        pattern: r"(?:enum\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Swift enum definition".to_string(),
                    },
                ],
            },
            SymbolExtractorDef {
                language: "ruby".to_string(),
                extensions: vec!["rb".to_string()],
                patterns: vec![
                    SymbolPattern {
                        kind: "function".to_string(),
                        pattern: r"(?:def\s+)([A-Za-z_]\w*[!?]?)".to_string(),
                        description: "Ruby method definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "class".to_string(),
                        pattern: r"(?:class\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Ruby class definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "module".to_string(),
                        pattern: r"(?:module\s+)([A-Za-z_]\w*)".to_string(),
                        description: "Ruby module definition".to_string(),
                    },
                ],
            },
            SymbolExtractorDef {
                language: "php".to_string(),
                extensions: vec!["php".to_string()],
                patterns: vec![
                    SymbolPattern {
                        kind: "function".to_string(),
                        pattern: r"(?:function\s+)([A-Za-z_]\w*)".to_string(),
                        description: "PHP function definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "class".to_string(),
                        pattern: r"(?:class\s+)([A-Za-z_]\w*)".to_string(),
                        description: "PHP class definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "interface".to_string(),
                        pattern: r"(?:interface\s+)([A-Za-z_]\w*)".to_string(),
                        description: "PHP interface definition".to_string(),
                    },
                    SymbolPattern {
                        kind: "trait".to_string(),
                        pattern: r"(?:trait\s+)([A-Za-z_]\w*)".to_string(),
                        description: "PHP trait definition".to_string(),
                    },
                ],
            },
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata_serialization() {
        let meta = PluginMetadata {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test".to_string(),
            min_codescope_version: Some("1.0.0".to_string()),
            hooks: vec![HookPoint::PostFileSearch],
            languages: vec!["rust".to_string()],
        };

        let json = serde_json::to_string(&meta).unwrap();
        let parsed: PluginMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.version, "1.0.0");
        assert_eq!(parsed.hooks.len(), 1);
    }

    #[test]
    fn test_plugin_manager_load_unload() {
        let mut mgr = PluginManager::new();
        mgr.enable();

        let plugin = RecencyBoostPlugin::new(1.5, 24);
        let meta = mgr.load_plugin(Box::new(plugin)).unwrap();
        assert_eq!(meta.name, "recency-boost");
        assert_eq!(mgr.plugin_count(), 1);

        mgr.unload_plugin("recency-boost").unwrap();
        assert_eq!(mgr.plugin_count(), 0);
    }

    #[test]
    fn test_plugin_manager_duplicate() {
        let mut mgr = PluginManager::new();
        mgr.enable();

        mgr.load_plugin(Box::new(RecencyBoostPlugin::new(1.0, 24))).unwrap();
        let result = mgr.load_plugin(Box::new(RecencyBoostPlugin::new(1.0, 24)));
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_manager_list() {
        let mut mgr = PluginManager::new();
        mgr.enable();

        mgr.load_plugin(Box::new(RecencyBoostPlugin::new(1.0, 24))).unwrap();
        mgr.load_plugin(Box::new(MarkdownFormatterPlugin)).unwrap();

        let plugins = mgr.list_plugins();
        assert_eq!(plugins.len(), 2);
    }

    #[test]
    fn test_plugin_manager_json() {
        let mut mgr = PluginManager::new();
        mgr.enable();

        mgr.load_plugin(Box::new(RecencyBoostPlugin::new(1.5, 24))).unwrap();
        mgr.load_plugin(Box::new(ExtraLanguagesPlugin)).unwrap();

        let json = mgr.to_json();
        assert_eq!(json["plugin_count"], 2);
        assert!(json["plugins"].is_array());
    }

    #[test]
    fn test_markdown_formatter() {
        let plugin = MarkdownFormatterPlugin;
        let results = vec![
            PluginSearchResult {
                file: "src/main.rs".to_string(),
                line: 10,
                column: 0,
                content: "fn main()".to_string(),
                score: 0.95,
                metadata: HashMap::new(),
            },
            PluginSearchResult {
                file: "src/lib.rs".to_string(),
                line: 5,
                column: 4,
                content: "pub fn hello()".to_string(),
                score: 0.87,
                metadata: HashMap::new(),
            },
        ];

        let md = plugin.format_output("markdown", &results).unwrap();
        assert!(md.contains("| `src/main.rs` |"));
        assert!(md.contains("fn main()"));
    }

    #[test]
    fn test_extra_languages_extractors() {
        let plugin = ExtraLanguagesPlugin;
        let extractors = plugin.symbol_extractors();

        assert_eq!(extractors.len(), 4);

        let kotlin = extractors.iter().find(|e| e.language == "kotlin").unwrap();
        assert!(kotlin.extensions.contains(&"kt".to_string()));
        assert!(kotlin.patterns.iter().any(|p| p.kind == "function"));
        assert!(kotlin.patterns.iter().any(|p| p.kind == "class"));

        let swift = extractors.iter().find(|e| e.language == "swift").unwrap();
        assert!(swift.patterns.iter().any(|p| p.kind == "protocol"));
    }

    #[test]
    fn test_custom_formats_collection() {
        let mut mgr = PluginManager::new();
        mgr.enable();

        mgr.load_plugin(Box::new(MarkdownFormatterPlugin)).unwrap();
        let formats = mgr.collect_custom_formats();
        assert!(formats.contains(&"markdown".to_string()));
    }

    #[test]
    fn test_hook_point_serialization() {
        let hook = HookPoint::PostFileSearch;
        let json = serde_json::to_string(&hook).unwrap();
        let parsed: HookPoint = serde_json::from_str(&json).unwrap();
        assert_eq!(hook, parsed);
    }

    #[test]
    fn test_symbol_extractor_def_serialization() {
        let def = SymbolExtractorDef {
            language: "kotlin".to_string(),
            extensions: vec!["kt".to_string()],
            patterns: vec![
                SymbolPattern {
                    kind: "function".to_string(),
                    pattern: r"fun\s+(\w+)".to_string(),
                    description: "Kotlin function".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&def).unwrap();
        let parsed: SymbolExtractorDef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.language, "kotlin");
        assert_eq!(parsed.patterns.len(), 1);
    }

    #[test]
    fn test_plugin_context() {
        let ctx = PluginContext {
            repo_path: PathBuf::from("/tmp/repo"),
            query: "test".to_string(),
            search_mode: "fuzzy".to_string(),
            extensions: vec!["rs".to_string()],
            config: HashMap::new(),
        };

        assert_eq!(ctx.query, "test");
        assert_eq!(ctx.extensions.len(), 1);
    }

    #[test]
    fn test_plugin_disabled_by_default() {
        let mgr = PluginManager::new();
        assert!(!mgr.is_enabled());
    }
}
