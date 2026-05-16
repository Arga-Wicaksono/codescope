# CodeScope — Development Roadmap

> **Current Version:** v1.3.0 · **Status:** All phases complete

---

## Phase 1 — Core Search Engine ✅

**Goal:** Fast file and content search as a unified CLI tool.

- [x] Fuzzy file search (`cs file`) with extension filtering and depth limits
- [x] Content search (`cs content`) with 3 matching modes (fuzzy, exact, regex)
- [x] Parallel processing via rayon for large codebases
- [x] .gitignore-aware traversal (respects `.gitignore`, `.ignore`, global gitignore)
- [x] Smart case sensitivity (case-insensitive unless pattern has uppercase)
- [x] Hidden file support (`--hidden`)
- [x] Line numbers (`-n`) and context lines (`-C`)
- [x] JSON output (`-j`) for scripting and piping
- [x] Replace mode (`--replace` / `--write`) for batch edits
- [x] Count mode (`--count`) and invert match (`--invert`)
- [x] Stdin pipe support (`cat file | cs content 'pattern'`)
- [x] File type presets (`--type rust/python/js/web/cpp/go/java/config/doc/data/shell`)

---

## Phase 2 — Interactive Experience ✅

**Goal:** Built-in interactive mode eliminating external tool dependencies.

- [x] Interactive fuzzy-select picker (`-I`) for file, content, web, and open commands
- [x] Color output with auto TTY detection and `--no-color` override
- [x] Shell completions (bash, zsh, fish, powershell, elvish)
- [x] Config file (`~/.codescope.json`) for persistent defaults
- [x] Configuration command (`cs config`) to display current config and path
- [x] Branded ASCII art banner with quick-start command reference
- [x] Levenshtein-distance command suggestions for typos (e.g., `cs flie` → `cs file`)

---

## Phase 3 — Symbol Intelligence ✅

**Goal:** Navigate codebases by symbol, not by file.

- [x] `cs where <name>` — Find function/class/struct definitions across languages
- [x] `cs symbol <name>` — Find symbol definitions with file, line, kind, and language metadata
- [x] `cs refs <name>` — Find all references to a symbol across project files
- [x] `cs callers <name>` — Find callers of a specific function with call-site context
- [x] `cs symbols [path]` — List all symbols in a file or directory
- [x] **Extension-based language detection** supporting 10 languages (Rust, Python, JS/TS, Go, Java/Kotlin, C, C++, Ruby, PHP, Swift) — prevents cross-language false positives

> **v1.3.0 —** Fixed cross-language false positives: `where_cmd.rs` now uses extension-based language matching instead of iterating all language patterns on all files, preventing C/CPP patterns from incorrectly matching `.rs` files and similar issues.

---

## Phase 4 — Context Engine ✅

**Goal:** Gather and rank relevant code context for LLM prompting.

- [x] `cs context <topic>` — Multi-source context extraction with relevance ranking
- [x] `cs pack <description>` — LLM-optimized prompt packing with ranked context
- [x] `cs trace <symbol>` — Execution flow tracing across function call chains
- [x] `cs schema <command>` — Print JSON output schema for any command
- [x] **TF-IDF ranking** for semantic relevance scoring of context results
- [x] **Token budget awareness** for LLM-optimized context packing

> **v1.3.0 —** Added TF-IDF-based relevance ranking to the context engine for smarter context extraction. Token budget awareness ensures context packs fit within LLM window limits. Execution flow tracing (`cs trace`) follows call chains across files.

---

## Phase 5 — Dependency Graph ✅

**Goal:** Visualize and analyze project dependency structure.

- [x] `cs graph` — Module dependency graph with three output formats (tree, flat list, DOT)
- [x] `cs impact <target>` — Impact analysis showing what depends on a target file or module
- [x] **DOT format support** for generating Graphviz-compatible dependency visualizations
- [x] **Helpful impact analysis messages** — informative output when no dependents are found, instead of silent empty output

> **v1.3.0 —** Dependency graph now supports DOT format output for Graphviz rendering. Impact analysis provides helpful messages when no dependents are found.

---

## Phase 6 — Developer Workflows ✅

**Goal:** Polish CLI experience for daily developer use.

- [x] `cs open <pattern>` — Search files and open in `$EDITOR` with line support
- [x] `cs recent [options]` — Find recently modified files with relative times (`--since`)
- [x] `cs explain <pattern>` — Explain regex patterns in plain language
- [x] `cs history` — Persistent search history with auto-rotation
- [x] `cs across <pattern>` — Cross-repository search (`--repos`, `--workspace`, `--repos-file`)
- [x] `cs stats [options]` — Per-language file and line count statistics
- [x] Standardized `--limit`, `--json`, and `--exclude` flags across all 28 commands

---

## Phase 7 — AI Agent Integration ✅

**Goal:** Expose CodeScope capabilities to AI agents and editor workflows.

- [x] `cs serve --mcp` — MCP (Model Context Protocol) server for AI tool integration
- [x] `cs lsp-bridge` — LSP bridge for editor integration (6 request types: initialize, completion, gotoDefinition, references, hover, documentSymbol)
- [x] **HTTP API server** (`cs serve --http`) exposing all CodeScope commands as REST endpoints
- [x] **LSP bridge** supporting initialize, completion, definition, references, hover, and documentSymbol requests

> **v1.3.0 —** HTTP API server allows any HTTP client to invoke CodeScope commands as REST endpoints. LSP bridge enables editor integration via the Language Server Protocol with 6 request types.

---

## Phase 8 — Performance Excellence ✅

**Goal:** Optimize for speed and scalability on large codebases.

- [x] Parallel content search via rayon
- [x] .gitignore-aware directory traversal (skip irrelevant paths)
- [x] Extension-based filtering to avoid unnecessary file processing
- [x] Release-optimized binary (~2 MB with LTO + strip)
- [x] Zero runtime dependencies
- [x] **TF-IDF semantic search** as an alternative search method with cosine similarity ranking

> **v1.3.0 —** `cs semantic <query>` provides TF-IDF-based semantic search with cosine similarity ranking as an alternative to pattern-based search methods.

---

## Phase 9 — Open Source Ecosystem ✅

**Goal:** Build community-ready open source infrastructure.

- [x] MIT License
- [x] Comprehensive README with installation, examples, and comparison table
- [x] CONTRIBUTING.md with contribution guidelines
- [x] CHANGELOG.md following Keep a Changelog format
- [x] CI/CD via GitHub Actions (testing, clippy, formatting, cross-platform releases)
- [x] Cross-platform releases (Linux, macOS Intel, macOS ARM, Windows)
- [x] Feature flags for optional dependency management (`web-search`, `interactive`)
- [x] **Cache system** (`cs cache stats|clear|cleanup`) with TTL expiration and LRU eviction
- [x] **AI rewrite feature** (`cs rewrite <instruction>`) for LLM-powered code refactoring with targeted file/function scoping

> **v1.3.0 —** File-system based cache with TTL expiration and LRU eviction policy for query result caching. AI rewrite command enables LLM-powered code refactoring with scoped file/function targeting.

---

## What's Next (Post-v1.3.0)

| Feature | Description | Priority |
|---------|-------------|----------|
| TUI Mode | Interactive terminal UI with ratatui | Medium |
| Plugin Architecture | Trait-based plugin system with hook points | Medium |
| VS Code Extension | Via LSP bridge | Low |
| Neovim Plugin | Native Lua plugin via LSP bridge | Low |
| Tree-sitter Integration | AST-level symbol accuracy | Medium |
| Incremental Indexing | Git-aware change detection for cache | Low |
| Community Plugin Registry | Share and discover plugins | Low |
