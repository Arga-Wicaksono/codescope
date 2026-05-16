/// Command-line argument parsing for CodeScope.
use clap::{Parser, Subcommand, ValueEnum};
use crate::types::FileType;

/// CodeScope — Repository Intelligence Engine for AI & Developers.
#[derive(Parser, Debug)]
#[command(
    name = "cs",
    version,
    about = "Repository Intelligence Engine — fast code retrieval, symbol lookup, and AI-ready context extraction",
    long_about = "CodeScope makes repositories understandable instantly — for humans and AI systems.\n\n\
                  Fast file search, content search, definition finding, cross-repo search,\n\
                  and structured JSON output for AI agents and scripting.\n\n\
                  Core principles: deterministic, blazing fast, scriptable,\n\
                  AI-consumable, local-first, zero runtime dependencies.\n\n\
                  Respects .gitignore by default (use --no-ignore to disable).\n\
                  Uses smart case: case-insensitive unless pattern has uppercase.\n\n\
                  Config file: ~/.codescope.json (or $CS_CONFIG env var)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for files by name with fuzzy matching
    File {
        /// Search pattern (supports fuzzy matching)
        pattern: String,

        /// Directory to search (default: current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Exclude directories (comma separated, e.g. "target,node_modules")
        #[arg(long)]
        exclude: Option<String>,

        /// Filter by file extension (e.g. "rs", "py", "toml")
        #[arg(short = 'e', long)]
        extension: Option<String>,

        /// Filter by file type preset (rust, python, js, web, c, cpp, go, java, config, doc, data, shell)
        #[arg(long, conflicts_with = "extension")]
        file_type: Option<FileType>,

        /// Include hidden files (dotfiles)
        #[arg(long)]
        hidden: bool,

        /// Case insensitive search (overrides smart case)
        #[arg(short = 'i', long, conflicts_with = "case_sensitive")]
        case_insensitive: bool,

        /// Force case-sensitive search (overrides smart case)
        #[arg(long, conflicts_with = "case_insensitive")]
        case_sensitive: bool,

        /// Don't respect .gitignore, .ignore, and other ignore files
        #[arg(long)]
        no_ignore: bool,

        /// Maximum recursive depth (default: unlimited)
        #[arg(long)]
        depth: Option<usize>,

        /// Maximum results to show (default: 20)
        #[arg(short = 'l', long)]
        limit: Option<usize>,

        /// Output as JSON
        #[arg(short = 'j', long)]
        json: bool,

        /// Interactive mode: fuzzy-select from results
        #[arg(short = 'I', long)]
        interactive: bool,
    },

    /// Search for content inside files (parallel processing)
    Content {
        /// Search pattern (text, regex, or exact string)
        pattern: String,

        /// Directory to search (default: current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Filter by file extension (e.g. "rs", "py")
        #[arg(short = 'e', long)]
        extension: Option<String>,

        /// Filter by file type preset (rust, python, js, web, c, cpp, go, java, config, doc, data, shell)
        #[arg(long, conflicts_with = "extension")]
        file_type: Option<FileType>,

        /// Use regex instead of fuzzy matching
        #[arg(short, long, conflicts_with = "exact")]
        regex: bool,

        /// Use exact string match (no fuzzy, no regex)
        #[arg(short = 'x', long, conflicts_with = "regex")]
        exact: bool,

        /// Exclude directories (comma separated)
        #[arg(long)]
        exclude: Option<String>,

        /// Case insensitive search (overrides smart case)
        #[arg(short = 'i', long, conflicts_with = "case_sensitive")]
        case_insensitive: bool,

        /// Force case-sensitive search (overrides smart case)
        #[arg(long, conflicts_with = "case_insensitive")]
        case_sensitive: bool,

        /// Don't respect .gitignore, .ignore, and other ignore files
        #[arg(long)]
        no_ignore: bool,

        /// Show line numbers in output
        #[arg(short = 'n', long)]
        line_number: bool,

        /// Number of context lines around each match (default: 0)
        #[arg(short = 'C', long, default_value_t = 0)]
        context: usize,

        /// Maximum recursive depth (default: unlimited)
        #[arg(long)]
        depth: Option<usize>,

        /// Maximum results to show (default: 20)
        #[arg(short = 'l', long)]
        limit: Option<usize>,

        /// Output as JSON
        #[arg(short = 'j', long)]
        json: bool,

        /// Replace matches with the given text (dry run by default)
        #[arg(long)]
        replace: Option<String>,

        /// Actually write replacement changes to files
        #[arg(long, requires = "replace")]
        write: bool,

        /// Show per-file match counts instead of matched lines
        #[arg(long, conflicts_with = "interactive")]
        count: bool,

        /// Show lines that do NOT match the pattern
        #[arg(long, conflicts_with = "interactive")]
        invert: bool,

        /// Interactive mode: fuzzy-select from results
        #[arg(short = 'I', long)]
        interactive: bool,
    },

    /// Search the web (requires `web-search` feature)
    #[cfg(feature = "web-search")]
    Web {
        /// Search query
        query: String,

        /// Maximum results to show (default: 10)
        #[arg(short = 'l', long)]
        limit: Option<usize>,

        /// Request timeout in seconds (default: 10)
        #[arg(long)]
        timeout: Option<u64>,

        /// Output as JSON
        #[arg(short = 'j', long)]
        json: bool,

        /// Interactive mode: fuzzy-select from results
        #[arg(short = 'I', long)]
        interactive: bool,
    },

    /// Generate shell completions for bash, zsh, fish, powershell, or elvish
    Completions {
        /// Shell to generate completions for
        shell: ShellName,
    },

    /// Search for files and open in editor
    Open {
        /// Search pattern (fuzzy match file names)
        pattern: String,

        #[arg(short, long)]
        path: Option<String>,

        #[arg(long)]
        exclude: Option<String>,

        #[arg(short = 'e', long)]
        extension: Option<String>,

        #[arg(long, conflicts_with = "extension")]
        file_type: Option<FileType>,

        #[arg(long)]
        hidden: bool,

        #[arg(short = 'i', long, conflicts_with = "case_sensitive")]
        case_insensitive: bool,

        #[arg(long, conflicts_with = "case_insensitive")]
        case_sensitive: bool,

        #[arg(long)]
        no_ignore: bool,

        #[arg(long)]
        depth: Option<usize>,

        #[arg(long)]
        line: Option<usize>,

        #[arg(short = 'I', long)]
        interactive: bool,

        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Show recently modified files
    Recent {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(long)]
        exclude: Option<String>,

        #[arg(long, conflicts_with = "extension")]
        file_type: Option<FileType>,

        #[arg(short = 'e', long)]
        extension: Option<String>,

        #[arg(long)]
        hidden: bool,

        #[arg(long)]
        no_ignore: bool,

        #[arg(long)]
        since: Option<String>,

        #[arg(short = 'l', long)]
        limit: Option<usize>,

        #[arg(short = 'I', long)]
        interactive: bool,

        #[arg(long, conflicts_with = "interactive")]
        open: bool,

        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Find where functions, classes, structs, etc. are defined
    Where {
        /// Definition name to search for
        name: String,

        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(long)]
        exclude: Option<String>,

        #[arg(long, conflicts_with = "extension")]
        file_type: Option<FileType>,

        #[arg(short = 'e', long)]
        extension: Option<String>,

        #[arg(long)]
        no_ignore: bool,

        #[arg(long)]
        depth: Option<usize>,

        #[arg(short = 'I', long)]
        interactive: bool,

        #[arg(long, conflicts_with = "interactive")]
        open: bool,

        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Explain a regex pattern in plain language
    Explain {
        /// Regex pattern to explain
        pattern: String,

        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Show search history
    History {
        /// Number of entries to show (default: 20)
        #[arg(short = 'l', long)]
        limit: Option<usize>,

        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Search content across multiple repositories at once
    Across {
        /// Search pattern (text, regex, or exact string)
        pattern: String,

        /// Comma-separated list of repository paths
        #[arg(long)]
        repos: Option<String>,

        /// Workspace directory to auto-discover git repos
        #[arg(long, conflicts_with = "repos")]
        workspace: Option<String>,

        /// File containing repository paths (one per line)
        #[arg(long, conflicts_with_all = ["repos", "workspace"])]
        repos_file: Option<String>,

        #[arg(long, conflicts_with = "extension")]
        file_type: Option<FileType>,

        #[arg(short = 'e', long)]
        extension: Option<String>,

        #[arg(short, long, conflicts_with = "exact")]
        regex: bool,

        #[arg(short = 'x', long, conflicts_with = "regex")]
        exact: bool,

        #[arg(short = 'l', long)]
        limit: Option<usize>,

        #[arg(short = 'j', long)]
        json: bool,

        #[arg(short = 'I', long)]
        interactive: bool,
    },

    /// Show file statistics for a project
    Stats {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(long, conflicts_with = "extension")]
        file_type: Option<FileType>,

        #[arg(short = 'e', long)]
        extension: Option<String>,

        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Show configuration file path and current config
    Config,

    /// Print JSON output schema for a command (for AI integration and documentation)
    Schema {
        /// Command name to show schema for (file, content, web, where, stats, recent, across, open, explain, history)
        #[arg(value_name = "COMMAND")]
        command: Option<String>,
    },
}

/// Supported shell names for completion generation.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellName {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl std::fmt::Display for ShellName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellName::Bash => write!(f, "bash"),
            ShellName::Zsh => write!(f, "zsh"),
            ShellName::Fish => write!(f, "fish"),
            ShellName::PowerShell => write!(f, "powershell"),
            ShellName::Elvish => write!(f, "elvish"),
        }
    }
}
