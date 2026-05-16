# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2025-06-25

### Added

#### New Commands (28 total, +10 from v1.0.0)
- `cs symbol <name>` — Find symbol definitions across the codebase with file, line, kind, and language metadata
- `cs refs <name>` — Find all references to a symbol across project files
- `cs callers <name>` — Find callers of a specific function with call-site context
- `cs symbols [path]` — List all symbols (functions, structs, traits, impls, etc.) in a file or directory
- `cs context <topic>` — Multi-source context extraction with relevance ranking for LLM prompting
- `cs pack <description>` — LLM-optimized prompt packing that gathers and ranks relevant code context
- `cs trace <symbol>` — Execution flow tracing across function call chains
- `cs graph` — Module dependency graph with three output formats: tree, flat list, and DOT
- `cs impact <target>` — Impact analysis showing what depends on a target file or module
- `cs semantic <query>` — TF-IDF semantic search with cosine similarity ranking
- `cs rewrite <instruction>` — AI-powered code rewriting via LLM with targeted file/function scoping
- `cs serve --mcp` — MCP (Model Context Protocol) server for AI tool integration
- `cs serve --http` — HTTP API server exposing all CodeScope commands as REST endpoints
- `cs cache stats|clear|cleanup` — Query result caching with TTL and LRU eviction
- `cs lsp-bridge` — LSP bridge for editor integration supporting 6 request types: initialize, completion, gotoDefinition, references, hover, documentSymbol
- `cs schema <command>` — Print the JSON output schema for any command (useful for pipeline scripting)

#### Language & Architecture
- **9 new modules**: `symbol`, `context`, `graph`, `impact`, `serve`, `semantic`, `cache`, `rewrite`, `lsp_bridge`
- **28 Rust modules** total (was 18)
- **10 language support** (was 7): Rust, Python, JS/TS, Go, Java/Kotlin, C, C++, Ruby, PHP, Swift
- Extension-based language detection system to prevent cross-language false positives
- TF-IDF semantic search engine with cosine similarity ranking
- File-system based cache with TTL expiration and LRU eviction policy
- LSP bridge architecture supporting 6 request types (initialize, completion, definition, references, hover, documentSymbol)
- MCP (Model Context Protocol) server for AI agent integration
- HTTP API server exposing all commands as REST endpoints

### Fixed
- **Inconsistent CLI flags**: Standardized `--limit`, `--json`, and `--exclude` flags across all 28 commands for uniform interface
- **Language detection false positives**: `where_cmd.rs` now uses extension-based language matching instead of iterating all language patterns on all files, preventing C/CPP patterns from incorrectly matching `.rs` files and similar cross-language issues
- **Error messages for unknown commands**: Added Levenshtein-distance-based command suggestion system — e.g., `cs flie` now suggests `cs file`
- **Impact analysis UX**: Added helpful message when no dependents are found for a file, instead of silent empty output

---

## [1.0.0] - 2025-05-12

### Added
- **File search** (`cs file`) — Fuzzy file name search with extension filtering, depth limits, hidden file support, and .gitignore awareness
- **Content search** (`cs content`) — Search inside files with 3 matching modes: fuzzy, exact substring, and regex. Supports line numbers, context lines, and parallel processing via rayon
- **Web search** (`cs web`) — Search DuckDuckGo directly from the terminal with result limiting and timeout control
- **Open command** (`cs open`) — Search files by name and open in `$EDITOR` with optional line number support
- **Recent files** (`cs recent`) — Find recently modified files with time-based filtering (`--since '2h'`, `--since '1d'`)
- **Where command** (`cs where`) — Find function, class, struct, and interface definitions across 7+ programming languages (Rust, Python, JS/TS, Go, Java/Kotlin, C/C++)
- **Explain regex** (`cs explain`) — Break down regex patterns into plain language explanations
- **Search history** (`cs history`) — Persistent search history with auto-rotation (stored in `~/.codescope_history.json`)
- **Cross-repo search** (`cs across`) — Search content across multiple repositories at once via `--repos`, `--workspace`, or `--repos-file`
- **File statistics** (`cs stats`) — Per-language file and line count statistics with JSON output support
- **Interactive mode** (`-I`) — Built-in fuzzy-select picker for file search, content search, web search, and open commands (requires `interactive` feature)
- **JSON output** (`-j`) — Structured JSON output for all search commands, enabling easy scripting and piping
- **Replace mode** (`--replace 'text'`) — Dry-run or apply (`--write`) find-and-replace across files
- **Count mode** (`--count`) — Per-file match count summary for content search
- **Invert match** (`--invert`) — Show non-matching lines instead of matching lines
- **Shell completions** (`cs completions`) — Generate completions for bash, zsh, fish, powershell, and elvish
- **Config file** (`~/.codescope.json`) — Persistent defaults for limit, depth, exclude, color, and web timeout
- **Configuration command** (`cs config`) — Display current configuration and config file path
- **Colored output** — Auto-detected TTY colors with `--no-color` override
- **Smart case** — Case-insensitive by default, case-sensitive when pattern contains uppercase characters (like ripgrep)
- **Stdin pipe support** — `cat file | cs content 'pattern'` for searching piped input
- **File type presets** — `--type rust/python/js/web/cpp/go/java/config/doc/data/shell` for common language filtering
- **Branded banner** — ASCII art banner displayed on bare `cs` invocation with quick-start command reference
- **146 unit tests** — Comprehensive test suite covering all modules
- **MIT License**
- **CI/CD workflows** — GitHub Actions for automated testing, linting (clippy), formatting checks, and cross-platform release builds

### Architecture
- 18 Rust modules: `cli`, `config`, `content_search`, `explain`, `file_search`, `history`, `interactive`, `open`, `output`, `recent`, `stats`, `types`, `utils`, `validate`, `web_search`, `where_cmd`, `across`
- Feature flags: `web-search` (default), `interactive` (default) for optional dependency management
- Release-optimized binary: ~2 MB with LTO + strip, zero runtime dependencies
