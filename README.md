<p align="center">
  <img src="assets/logo.png" alt="cs logo" width="120" height="120">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/version-1.3.0-blue?style=flat-square" alt="Version">
  <img src="https://img.shields.io/badge/commands-28-green?style=flat-square" alt="Commands">
  <img src="https://img.shields.io/badge/modules-28-purple?style=flat-square" alt="Modules">
  <img src="https://img.shields.io/badge/languages-10-cyan?style=flat-square" alt="Languages">
  <img src="https://img.shields.io/badge/license-MIT-yellow?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20Windows-lightgrey?style=flat-square" alt="Platform">
</p>

<h1 align="center">cs — CodeScope</h1>

<p align="center">
  <strong>Repository Intelligence Engine for AI &amp; Developers.</strong><br>
  28 commands. 28 modules. 10 languages. One binary. Zero runtime dependencies.
</p>

<p align="center">
  <img src="assets/demo.gif" alt="cs demo" width="640">
</p>

<p align="center">
  <a href="#why-codescope">Why cs?</a> ·
  <a href="#installation">Install</a> ·
  <a href="#quick-start">Quick Start</a> ·
  <a href="#search--navigation">Commands</a> ·
  <a href="#matching-modes">Matching Modes</a> ·
  <a href="#json-output">JSON</a> ·
  <a href="#ai--integration">AI</a> ·
  <a href="#configuration">Config</a> ·
  <a href="#comparison">Comparison</a>
</p>

---

## Why CodeScope?

Real development workflows look like this:

```bash
# Find a file, then pipe to interactive picker
fd pattern | fzf

# Search content, pipe to interactive picker
rg "TODO" | fzf

# Search symbols? Install tree-sitter, ctags, ast-grep...
# Search the web? Open a browser.
# Get context for AI? Manually copy-paste code.
# Analyze dependencies? No standard tool.
```

**CodeScope replaces all of that with one tool.** A single ~2 MB Rust binary that handles file search, content search, symbol intelligence, context extraction, dependency graphs, semantic search, AI rewrite, and more — with built-in interactive mode, JSON output for scripting, and zero runtime dependencies.

| What you need | Before cs | With cs |
|---|---|---|
| Find files | `fd` + `fzf` | `cs file "pattern" -I` |
| Search content | `rg` + `fzf` | `cs content "pattern" -I` |
| Find definitions | `ctags` / tree-sitter | `cs where "fn_name"` |
| Find references | Language server | `cs refs "symbol"` |
| Find callers | Language server | `cs callers "fn_name"` |
| Get AI context | Manual copy-paste | `cs context "topic"` |
| Pack for LLM | Manual concatenation | `cs pack "description" -b 8000` |
| Dependency graph | No standard tool | `cs graph --type modules` |
| Impact analysis | No standard tool | `cs impact "utils.rs"` |
| Semantic search | Embeddings model | `cs semantic "database connection"` |
| AI rewrite | Copy-paste + manual | `cs rewrite "add error handling" --write` |
| Cross-repo search | Manual loops | `cs across 'TODO' --workspace ~/projects` |
| Search the web | Open browser | `cs web "rust tutorial" -l 5` |

---

## Core Principles

| Principle | Description |
|---|---|
| **Deterministic** | Same inputs always produce the same outputs — no randomness, no ML hallucination |
| **Blazing fast** | Rust + rayon parallelism + smart case + .gitignore-aware walking |
| **Scriptable** | Every command supports `-j` (JSON output) for piping and AI integration |
| **AI-consumable** | `context`, `pack`, `trace` extract structured, token-budgeted code for LLM prompts |
| **Local-first** | All analysis runs locally — no data leaves your machine (except `web` and `rewrite`) |
| **Zero dependencies** | Single static binary, no runtime deps, no `node_modules`, no Python venv |
| **Progressive** | v1.0 search → v1.1 symbols → v1.2 context → v1.3 AI — each release adds capability pillars |

---

## Commands at a Glance

### 1. Search & Navigation (7 commands)

| Command | Description | Example |
|---|---|---|
| `file` | Find files by name (fuzzy) | `cs file "Cargo"` |
| `content` | Search inside files (3 modes) | `cs content "fn main" --regex` |
| `web` | Search the web (DuckDuckGo) | `cs web "rust tutorial" -l 5` |
| `across` | Cross-repository search | `cs across 'TODO' --workspace ~/projects` |
| `open` | Find + open in `$EDITOR` | `cs open 'main' --line 42` |
| `where` | Find definitions across languages | `cs where 'parse_config'` |
| `recent` | Recently modified files | `cs recent --type rust --since '2h'` |

### 2. Symbol Intelligence (4 commands)

| Command | Description | Example |
|---|---|---|
| `symbol` | Find definitions with metadata | `cs symbol "MyStruct" --symbol-type struct` |
| `refs` | Find all references (excl. defs) | `cs refs "parse_config"` |
| `callers` | Find functions that call a function | `cs callers "process_data"` |
| `symbols` | List all symbols in file/dir | `cs symbols . --symbol-type function` |

### 3. Context Engine (3 commands)

| Command | Description | Example |
|---|---|---|
| `context` | Extract context for a topic (files + symbols + snippets) | `cs context "authentication"` |
| `pack` | Pack context into token-efficient LLM prompt | `cs pack "auth flow" -b 8000` |
| `trace` | Trace execution flow through call graph | `cs trace "main" --max-depth 5` |

### 4. Dependency Graph (2 commands)

| Command | Description | Example |
|---|---|---|
| `graph` | Build module or call dependency graph | `cs graph --type modules --format dot` |
| `impact` | Analyze what depends on a file/module | `cs impact "utils.rs"` |

### 5. Developer Tools (6 commands)

| Command | Description | Example |
|---|---|---|
| `stats` | File statistics per language | `cs stats --type rust --json` |
| `explain` | Explain regex patterns in plain language | `cs explain '\s+\w+'` |
| `history` | Show search history | `cs history -l 20 --json` |
| `config` | Show configuration path and values | `cs config` |
| `completions` | Generate shell completions | `cs completions bash` |
| `schema` | Print JSON output schema for AI | `cs schema content` |

### 6. AI & Integration (4 commands)

| Command | Description | Example |
|---|---|---|
| `serve` | MCP server (JSON-RPC) or HTTP API | `cs serve --mcp` / `cs serve --http -p 4567` |
| `semantic` | TF-IDF semantic search with cosine similarity | `cs semantic "database connection pool"` |
| `rewrite` | AI-powered code rewrite (Ollama / OpenAI) | `cs rewrite "add error handling" --write` |
| `lsp-bridge` | LSP bridge for editor integration | `cs lsp-bridge --port 8765` |

### 7. Caching (1 command, 3 sub-commands)

| Command | Description | Example |
|---|---|---|
| `cache stats` | Show cache entries, size, hit rate | `cs cache stats` |
| `cache clear` | Clear all cached entries | `cs cache clear` |
| `cache cleanup` | Remove expired entries | `cs cache cleanup` |

---

## Installation

### One-liner install (macOS / Linux)

```bash
curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash
```

Custom version or prefix:

```bash
# Install specific version
curl -sSL ... | bash -s -- --version 1.3.0

# Custom install prefix
curl -sSL ... | bash -s -- --prefix ~/.local/bin
```

### Package managers

| Platform | Command |
|----------|---------|
| **macOS (Homebrew)** | `brew tap Arga-Wicaksono/codescope && brew install codescope` |
| **Windows (Scoop)** | `scoop bucket add codescope https://github.com/Arga-Wicaksono/codescope && scoop install codescope` |
| **Arch Linux (AUR)** | `yay -S codescope` |
| **Rust (cargo)** | `cargo install --git https://github.com/Arga-Wicaksono/codescope.git` |
| **npm** | `npm install -g codescope` |

### Download prebuilt binary

Download from [GitHub Releases](https://github.com/Arga-Wicaksono/codescope/releases/latest):

| Platform | File |
|----------|------|
| Linux x86_64 (glibc) | `cs-x86_64-linux.tar.gz` |
| Linux x86_64 (musl/static) | `cs-x86_64-linux-musl.tar.gz` |
| Linux aarch64 (glibc) | `cs-aarch64-linux.tar.gz` |
| Linux aarch64 (musl/static) | `cs-aarch64-linux-musl.tar.gz` |
| macOS Intel | `cs-x86_64-macos.tar.gz` |
| macOS Apple Silicon | `cs-aarch64-macos.tar.gz` |
| Windows x86_64 | `cs-x86_64-windows.zip` |

```bash
# Download and install manually
curl -sL https://github.com/Arga-Wicaksono/codescope/releases/latest/download/cs-x86_64-linux-musl.tar.gz | tar xz
sudo mv cs /usr/local/bin/
```

### Build from source

```bash
git clone https://github.com/Arga-Wicaksono/codescope.git
cd codescope
cargo install --path .
```

### Build variants

```bash
# Full build (web search + interactive) — default
cargo build --release

# Without web search (smaller binary)
cargo build --release --no-default-features --features interactive

# Minimal: file + content only (smallest binary)
cargo build --release --no-default-features
```

### Shell completions

```bash
# Bash
cs completions bash | sudo tee /usr/share/bash-completion/completions/cs > /dev/null

# Zsh
cs completions zsh > ~/.zfunc/_cs

# Fish
cs completions fish > ~/.config/fish/completions/cs.fish

# PowerShell
cs completions powershell | Out-File -Encoding utf8 $PROFILE
```

### Uninstall

```bash
# Using the uninstall script
curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/uninstall.sh | bash

# Or manually
rm $(which cs)
```

---

## Quick Start

```bash
# ── Search & Navigation ──────────────────────────────────────────
cs file "Cargo"                       # Find files by name
cs content "fn main"                   # Fuzzy content search
cs content 'TODO|FIXME' --regex        # Regex content search
cs content "config" -x                 # Exact substring match
cs web "rust tutorial" -l 5            # Web search
cs across 'TODO' --workspace ~/projects  # Cross-repo search
cs open 'main' --line 42               # Find + open in editor
cs open 'TODO' -I                      # Interactive select, then open
cs where 'parse_config'                # Find definitions
cs where 'MyStruct' --open             # Find + open at definition
cs recent --type rust --since '2h'     # Recently modified files

# ── Symbol Intelligence ─────────────────────────────────────────
cs symbol "MyStruct"                   # Find symbol with metadata
cs symbol "handler" --symbol-type function  # Filter by kind
cs refs "parse_config"                 # Find all references
cs callers "process_data"              # Find callers of a function
cs symbols . --symbol-type function -l 20   # List all functions

# ── Context Engine ──────────────────────────────────────────────
cs context "authentication"            # Extract context for AI
cs pack "auth flow" -b 8000            # Pack for LLM prompt
cs trace "main" --max-depth 5          # Trace execution flow

# ── Dependency Graph ────────────────────────────────────────────
cs graph                              # Module dependency graph (tree)
cs graph --type calls --format dot     # Call graph in Graphviz DOT
cs graph -f json -j                    # Graph as JSON
cs impact "utils.rs"                   # What depends on this?

# ── Developer Tools ─────────────────────────────────────────────
cs stats --type rust                   # Rust file statistics
cs explain '\s+\w+'                    # Explain regex
cs history -l 20                       # Search history
cs config                              # Show configuration
cs completions bash | sudo tee /usr/share/bash-completion/completions/cs

# ── AI & Integration ────────────────────────────────────────────
cs serve --mcp                         # MCP server for AI agents
cs serve --http -p 4567                # HTTP API server
cs semantic "database connection pool"  # TF-IDF semantic search
cs rewrite "add error handling" --write # AI-powered rewrite
cs lsp-bridge --port 8765              # LSP bridge for editors

# ── Caching ─────────────────────────────────────────────────────
cs cache stats                         # Cache statistics
cs cache clear                         # Clear cache
cs cache cleanup                       # Remove expired entries
```

---

## Detailed Command Documentation

### `cs file <pattern>` — File Search

Find files by name using fuzzy matching (SkimMatcherV2).

```bash
cs file "Cargo"           # Basic fuzzy search
cs file "main" -e rs      # Filter by extension
cs file "test" --depth 2  # Limit recursion depth
cs file "config" -I       # Interactive selection
cs file "config" -j       # JSON output
cs file "lib" --type rust # Use file type preset
cs file "env" --hidden    # Include hidden files
```

### `cs content <pattern>` — Content Search

Search text inside files with three matching modes. Uses **rayon** for parallel processing.

```bash
cs content "function"           # Fuzzy search (default)
cs content "fn\s+\w+" --regex  # Regex search
cs content "config" -x          # Exact substring match
cs content "TODO" -n            # Show line numbers
cs content "error" -C 3         # 3 context lines
cs content "TODO" --count       # Per-file match counts
cs content "fn" --invert        # Non-matching lines
cs content "old" --replace "new" -x --write  # Replace in files
cat file | cs content "pattern" # Search piped stdin
```

### `cs web <query>` — Web Search *(requires `web-search` feature)*

```bash
cs web "rust tutorial" -l 5        # Top 5 results
cs web "async python" --timeout 15 # Custom timeout
cs web "web assembly" -I           # Interactive select, then open
```

### `cs across <pattern>` — Cross-Repository Search

```bash
cs across 'TODO' --workspace ~/projects        # Auto-discover git repos
cs across 'error' --repos /a/repo1,/b/repo2    # Explicit repos
cs across 'bug' --repos_file ~/repos.txt       # Read repos from file
cs across 'API' --regex --type rust            # Regex + type filter
```

### `cs open <pattern>` — Search + Open in Editor

```bash
cs open 'main'             # Find and open best match
cs open 'config' --line 42 # Open at specific line
cs open 'TODO' -I          # Interactive select, then open
```

### `cs where <name>` — Find Definitions

```bash
cs where 'parse_config'       # Search all source files
cs where 'MyStruct' --type rust
cs where 'handler' --open      # Find + open at definition line
```

Supports: Rust, Python, JS/TS, Go, Java/Kotlin, C/C++, Ruby, PHP, Swift.

### `cs recent [options]` — Recently Modified Files

```bash
cs recent                  # 20 most recent files
cs recent --type rust      # Rust files only
cs recent --since '2h'     # Modified in last 2 hours
cs recent --open           # Open most recent in editor
cs recent -I               # Interactive selection
```

---

### `cs symbol <name>` — Find Symbol Definitions

Find symbol definitions with rich metadata: kind, language, file path, line number.

```bash
cs symbol "search_files"                    # Find any symbol
cs symbol "Config" --symbol-type struct     # Filter by kind
cs symbol "handler" --type rust -j          # JSON output
```

Supported kinds: `function`, `method`, `struct`, `class`, `enum`, `trait`, `interface`, `constant`, `type`, `module`. Aliases: `fn`→`function`, `mod`→`module`, `const`→`constant`, etc.

### `cs refs <name>` — Find References

Find all references to a symbol, **excluding definitions and comments**.

```bash
cs refs "Config"           # All usages of Config
cs refs "helper" --type py # Python files only
```

### `cs callers <name>` — Find Callers

Find all functions that call a specific function. Reports the enclosing function name.

```bash
cs callers "process_data"    # Who calls process_data?
cs callers "validate" -j     # JSON output with caller names
```

### `cs symbols [path]` — List All Symbols

List all symbol definitions in a file or directory.

```bash
cs symbols . --type rust                  # All Rust symbols
cs symbols . --symbol-type function -l 20 # Top 20 functions
cs symbols src/lib.rs -j                 # Single file, JSON
```

---

### `cs context <topic>` — Extract Context

Multi-source context extraction with ranking: file name matches > content density > symbol definitions.

```bash
cs context "authentication"            # Files + lines + symbols about auth
cs context "Config" --type rust        # Rust files only
cs context "handler" -l 30 --json      # JSON with scores
```

Output includes `type` (file/content/symbol), `path`, `line`, `content`, `score`, and `tokens`.

### `cs pack <description>` — Pack for LLM

Token-budgeted context packing optimized for LLM prompts.

```bash
cs pack "authentication flow" -b 8000    # Pack up to 8000 tokens
cs pack "error handling" --type rust -j  # JSON output
```

 packing order: exact filename matches → symbol definitions → content snippets.

### `cs trace <name>` — Trace Execution Flow

Trace function call chains recursively up to a configurable depth.

```bash
cs trace "main" --max-depth 5    # Trace from main
cs trace "process" --depth 3     # Limit directory depth + trace depth
cs trace "main" -j              # JSON tree output
```

Builds a call tree using a global symbol table and brace-matching body extraction.

---

### `cs graph` — Dependency Graph

Build module import or function-call dependency graphs. Supports Rust, Python, JS/TS, Go, Java/Kotlin.

```bash
cs graph                              # Module graph (tree format)
cs graph --type calls                 # Function call graph
cs graph --format dot                 # Graphviz DOT output
cs graph --format flat                # Flat edge list
cs graph -f json -j                   # JSON output
cs graph --type modules --depth 2     # Limit graph depth
```

Pipe DOT output to Graphviz:
```bash
cs graph --type modules --format dot | dot -Tpng -o deps.png
```

### `cs impact <target>` — Impact Analysis

Find all files that depend on a given file or module (directly and transitively).

```bash
cs impact "utils.rs"       # Who depends on utils.rs?
cs impact "utils" -j       # JSON output
cs impact "config.py"      # Python modules too
```

Reports direct dependents (1st degree) and indirect dependents (2nd degree+) separately.

---

### `cs serve` — MCP + HTTP Server

Expose CodeScope capabilities as a server for AI agents and external tools.

**MCP Server** (JSON-RPC 2.0 over stdin/stdout):
```bash
cs serve --mcp
```

Tools exposed: `search_files`, `search_content`, `find_symbol`, `find_references`, `find_callers`, `list_symbols`, `get_context`, `pack_context`, `trace_symbol`, `repo_stats`.

**HTTP API Server**:
```bash
cs serve --http -p 4567
```

Zero-dependency TCP-based HTTP server. Query: `GET /api/search?pattern=main&limit=10`.

### `cs semantic <query>` — Semantic Search

TF-IDF-based semantic search with cosine similarity — no external ML library needed.

```bash
cs semantic "database connection pool"    # Natural language query
cs semantic "error handling" --type rust # Filter by language
cs semantic "auth" -l 10 -j              # JSON with similarity scores
```

**How it works:**
1. Tokenize all files (strip punctuation, filter stop words including common keywords like `fn`, `def`, `return`)
2. Compute TF-IDF vectors per document and line
3. Rank by cosine similarity between query and document vectors
4. Show color-coded scores: green (≥50%), yellow (≥20%), dimmed (<20%)

### `cs rewrite <instruction>` — AI-Powered Code Rewrite

Combines context extraction with LLM API calls to rewrite code.

```bash
cs rewrite "add error handling" --type rust --write     # Apply changes
cs rewrite "refactor this function" --dry-run            # Preview only
cs rewrite "optimize" --symbol "process_data" -j         # Target a symbol
cs rewrite "add docs" -m codellama --budget 50           # Custom model + budget
```

**Environment variables:**
| Variable | Default | Description |
|---|---|---|
| `CODESCOPE_LLM_MODEL` | `llama3` | LLM model name |
| `CODESCOPE_LLM_API` | `http://localhost:11434` | API base URL |
| `CODESCOPE_LLM_PROVIDER` | `ollama` | `ollama` or `openai` |

Uses Ollama's `/api/generate` endpoint or OpenAI-compatible `/chat/completions`.

### `cs lsp-bridge` — LSP Bridge

Lightweight LSP protocol server for editor integration (Neovim, VS Code, etc.).

```bash
cs lsp-bridge --port 8765
```

**Supported LSP requests:**
| Request | Maps to |
|---|---|
| `initialize` | Server capabilities |
| `textDocument/completion` | `cs content` (symbol completions) |
| `textDocument/definition` | `cs where` (go-to-definition) |
| `textDocument/references` | `cs content` (find-references) |
| `textDocument/hover` | `cs where` + `cs content` (hover info) |
| `textDocument/documentSymbol` | `cs where` (file symbols) |
| `shutdown` | Clean shutdown |

---

### `cs stats` — File Statistics

```bash
cs stats                   # Current directory stats
cs stats --type rust       # Rust files only
cs stats --json            # JSON output
```

### `cs explain <pattern>` — Explain Regex

```bash
cs explain '\s+\w+'         # Explains each token
cs explain '[A-Z][a-z]+'
```

### `cs history` — Search History

```bash
cs history           # 20 most recent searches
cs history -l 50     # Last 50 searches
cs history -j        # JSON output
```

### `cs config` — Configuration

```bash
cs config    # Show current config and file path
```

### `cs completions <shell>` — Shell Completions

```bash
cs completions bash    # Bash completions
cs completions zsh     # Zsh completions
cs completions fish    # Fish completions
cs completions powershell  # PowerShell completions
cs completions elvish  # Elvish completions
```

### `cs schema [command]` — JSON Output Schema

```bash
cs schema             # List all commands with schemas
cs schema content     # Show schema for content command
```

### `cs cache <action>` — Cache Management

```bash
cs cache stats     # Show entries, size, hit rate
cs cache clear     # Clear all cached entries
cs cache cleanup   # Remove expired entries
```

Cache is stored in `~/.codescope/cache/` as individual JSON files with TTL support.

---

## Matching Modes

| Mode | Flag | Best For |
|------|------|----------|
| **Fuzzy** | *(default)* | Approximate matching, tolerates typos |
| **Exact** | `-x` / `--exact` | Precise substring, zero false positives |
| **Regex** | `--regex` | Full pattern power |

Smart case is enabled by default: case-insensitive unless pattern contains uppercase (like ripgrep).

---

## JSON Output

Every command supports `-j` / `--json` for structured output:

```bash
cs file "config" -j
cs content "TODO" -n -j
cs symbol "Config" -j
cs refs "handler" -j
cs context "auth" -j
cs graph -j
cs impact "utils.rs" -j
cs semantic "database" -j
cs stats -j
```

All JSON output includes:
- `"tool": "codescope"`
- `"command": "<command name>"`
- `"count": <result count>`
- `"results": [...]`

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Results found |
| `1` | No results found |
| `2` | Error |

---

## Supported Languages (Symbol Intelligence)

| Language | Extensions | Symbol Types |
|---|---|---|
| **Rust** | `.rs` | fn, struct, enum, trait, const, type, mod |
| **Python** | `.py`, `.pyi`, `.pyw` | def, class |
| **JavaScript/TypeScript** | `.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs` | function, class, const, interface, type, enum |
| **Go** | `.go` | func, method, struct, interface, type, const |
| **Java/Kotlin** | `.java`, `.kt`, `.kts` | class, function, interface, enum, object |
| **C++** | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`, `.h` | class, struct, enum, namespace, method, function, constant, type |
| **C** | `.c` | struct, enum, function, constant, type |
| **Ruby** | `.rb` | method, class, module, constant |
| **PHP** | `.php` | function, class, interface, trait, constant |
| **Swift** | `.swift` | function, class, struct, protocol, enum, constant, type |

---

## Comparison with Other Tools

| Feature | `cs` | `fd` | `rg` | `fzf` | `ast-grep` | `ctags` |
|---------|------|------|------|-------|-------------|---------|
| Fuzzy file search | Yes | Yes | No | Yes | No | No |
| Content search | Fuzzy + exact + regex | No | Regex only | No | AST-based | No |
| Web search | Yes (DuckDuckGo) | No | No | No | No | No |
| Matching modes | 3 (fuzzy/exact/regex) | 1 (glob) | 1 (regex) | 1 (fuzzy) | 1 (AST) | N/A |
| Interactive mode | Built-in (`-I`) | Via pipe | Via pipe | Yes | No | No |
| JSON output | Yes (`-j`) | No | No | No | Yes | No |
| Config file | Yes | No | Yes | No | No | No |
| Symbol intelligence | 10 languages | No | No | No | Yes | Yes |
| Find references | Yes | No | No | No | Yes | No |
| Call graph | Yes | No | No | No | No | No |
| Context extraction | Yes | No | No | No | No | No |
| Dependency graph | Yes | No | No | No | No | No |
| Impact analysis | Yes | No | No | No | No | No |
| Semantic search | Yes (TF-IDF) | No | No | No | No | No |
| AI rewrite | Yes (Ollama/OpenAI) | No | No | No | No | No |
| MCP server | Yes | No | No | No | No | No |
| LSP bridge | Yes | No | No | No | No | No |
| Cross-repo search | Yes | No | No | No | No | No |
| Parallel processing | Yes (rayon) | Yes | Yes | No | No | No |
| Written in | Rust | Rust | Rust | Go | Rust | C |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        cs CLI (clap)                           │
│  file  content  web  where  open  recent  across  ...          │
├─────────────────────────────────────────────────────────────────┤
│                     Core Engine (28 modules)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ File Search   │  │ Symbol Intel │  │ Context Engine        │  │
│  │ Content Search│  │ • symbol     │  │ • context (3-source)  │  │
│  │ Web Search    │  │ • refs       │  │ • pack (token budget) │  │
│  │ Where (defs)  │  │ • callers    │  │ • trace (call tree)   │  │
│  │ Across (x-repo│  │ • symbols    │  │                        │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Graph/Impact  │  │ AI & LSP     │  │ Caching               │  │
│  │ • modules     │  │ • MCP server │  │ • FS-backed JSON      │  │
│  │ • calls       │  │ • HTTP API   │  │ • TTL + size limits   │  │
│  │ • tree/dot/   │  │ • LSP bridge │  │ • stats/clear/cleanup │  │
│  │   flat/json   │  │ • semantic   │  │                        │  │
│  │ • impact BFS  │  │ • rewrite    │  │                        │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Shared: ignore (walk), rayon, regex, fuzzy-matcher, serde_json │
├─────────────────────────────────────────────────────────────────┤
│  Feature flags: web-search (reqwest+scraper)                   │
│                  interactive (dialoguer)                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Configuration

Create `~/.codescope.json` for persistent defaults:

```json
{
  "default_limit": 50,
  "default_depth": 5,
  "default_exclude": ["target", "node_modules", "dist"],
  "default_extension": "rs",
  "color": true,
  "web_timeout": 15,
  "interactive": false
}
```

Set config path via environment variable:
```bash
export CS_CONFIG=/path/to/custom/codescope.json
```

Show current config:
```bash
cs config
```

---

## File Type Presets

Use `--type <preset>` with any search command:

| Preset | Extensions |
|---|---|
| `rust` | `.rs` |
| `python` | `.py`, `.pyi`, `.pyw` |
| `js` | `.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs` |
| `web` | `.html`, `.htm`, `.css`, `.scss`, `.sass`, `.less`, `.vue`, `.svelte` |
| `c` | `.c`, `.h` |
| `cpp` | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx`, `.h` |
| `go` | `.go` |
| `java` | `.java`, `.kt`, `.kts` |
| `config` | `.toml`, `.yaml`, `.yml`, `.json`, `.ini`, `.cfg`, `.conf`, `.env` |
| `doc` | `.md`, `.txt`, `.rst`, `.adoc`, `.org` |
| `data` | `.csv`, `.tsv`, `.xml`, `.sql` |
| `shell` | `.sh`, `.bash`, `.zsh`, `.fish`, `.ps1` |

---

## Running Tests

```bash
cargo test
```

---

## Roadmap

| Phase | Status | Features |
|---|---|---|
| **v1.0** | Completed | File search, content search, web search, interactive mode, replace, completions |
| **v1.1** | Completed | `where` (definitions), `across` (cross-repo), `open`, `recent`, `explain`, `history`, `stats`, `config` |
| **v1.2** | Completed | `symbol`, `refs`, `callers`, `symbols` (Symbol Intelligence), `context`, `pack`, `trace` (Context Engine), `graph`, `impact` (Dependency Graph) |
| **v1.3** | Completed | `serve` (MCP + HTTP), `semantic` (TF-IDF), `rewrite` (AI-powered), `lsp-bridge`, `cache`, `schema` |
| **v1.4** | Future | Incremental file watching, git blame integration, smarter caching |

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>Built with Rust by Arga Wicaksono</strong> ·
  <a href="https://github.com/Arga-Wicaksono/codescope">GitHub</a> ·
  <a href="https://github.com/Arga-Wicaksono/codescope/releases">Releases</a>
</p>
