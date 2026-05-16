<p align="center">
  <img src="assets/logo.png" alt="CodeScope logo" width="120" height="120">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/version-1.0.0-blue?style=flat-square" alt="Version">
  <img src="https://img.shields.io/badge/license-MIT-yellow?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/tests-146%20passed-brightgreen?style=flat-square" alt="Tests">
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20Windows-lightgrey?style=flat-square" alt="Platform">
</p>

<h1 align="center">CodeScope</h1>

<p align="center">
  <strong>Repository Intelligence Engine for AI & Developers</strong><br>
  Make repositories understandable instantly — fast, deterministic, and AI-ready.
</p>

<p align="center">
  <img src="assets/demo.gif" alt="cs demo" width="640">
</p>

<p align="center">
  <a href="#why-codescope">Why</a> ·
  <a href="#use-cases">Use Cases</a> ·
  <a href="#installation">Install</a> ·
  <a href="#quick-start">Quick Start</a> ·
  <a href="#commands">Commands</a> ·
  <a href="#architecture">Architecture</a> ·
  <a href="#roadmap">Roadmap</a>
</p>

---

## Why CodeScope?

Large codebases are hard to understand — for humans navigating unfamiliar code, and for AI agents that need precise context to be effective. Existing tools solve pieces of the problem:

| Tool | Does one thing well | But misses... |
|------|---------------------|---------------|
| `ripgrep` | Fast content search | No symbol intelligence, no context |
| `fd` | File finding | No content, no definitions |
| `ctags` | Symbol indexing | Stale, no ranking, not AI-ready |
| `LSP` | Editor navigation | Not scriptable, editor-bound |
| `Sourcegraph` | Code intelligence | Cloud-only, not local-first |

**CodeScope bridges all of these gaps** in a single ~2 MB binary. It provides fast file and content search today, with a clear path toward symbol intelligence, dependency tracing, and AI-consumable structured output. One tool, local-first, zero runtime dependencies, deterministic results every time.

### Core Principles

| Principle | What it means |
|-----------|---------------|
| **Deterministic** | Same query, same results — always. No randomness, no approximation. |
| **Blazing fast** | Rust-native with rayon parallelism. Built for large repos (100k+ files). |
| **Scriptable** | Every command outputs structured JSON (`-j`), perfect for pipes and automation. |
| **AI-consumable** | Stable schemas, ranked results, context extraction built for LLM prompt packing. |
| **Local-first** | No cloud, no API keys, no network. Everything runs on your machine. |
| **Zero dependencies** | One static binary. No runtime, no JVM, no Node. Just `cs`. |

---

## Use Cases

### For Developers

```bash
# Debugging — find where an error originates
cs content "connection refused" --type rust -n -C 3
cs where "handle_connection"          # Jump to the function definition

# Navigation — understand unfamiliar code
cs content "UserService" -n           # Find all references
cs where "authenticate"               # Find where auth logic lives
cs stats --type rust                  # Understand project composition

# Architecture understanding — see the big picture
cs content "pub fn\|pub async fn" --type rust --count   # All public APIs
cs file "mod.rs" -j                   # Module structure as JSON

# Refactoring — find what needs to change
cs across "deprecated" --workspace ~/projects  # Find across all repos
cs content "TODO\|FIXME\|HACK" --regex --count # Tech debt inventory
```

### For AI Agents

```bash
# Context retrieval — give AI the right files
cs content "auth middleware" --type rust -n -C 5 -j
# → Structured JSON with file paths, line numbers, and context

# Prompt packing — gather ranked context for LLM
cs file "config" -e rs -j                  # Config files as JSON
cs content "pub struct" --type rust -j      # All types for context

# Repository mapping — let AI understand structure
cs stats -j                                # Project composition
cs where "trait Handler" -j                # Interface definitions

# Symbol lookup — precise code navigation
cs where "handle_request" -j               # Definition location
cs content "handle_request" -n -C 2 -j     # Usage with context
```

> **Why JSON matters for AI:** Structured output (`-j`) turns freeform search results into deterministic, parseable data. AI agents and CI/CD pipelines can consume `cs` output directly without parsing human-readable text.

---

## Installation

### Download prebuilt binary

Download from [GitHub Releases](https://github.com/Arga-Wicaksono/codescope/releases/latest):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `cs-x86_64-linux` |
| macOS Intel | `cs-x86_64-macos` |
| macOS Apple Silicon | `cs-aarch64-macos` |
| Windows | `cs-x86_64-windows.exe` |

```bash
# Linux / macOS
curl -sL https://github.com/Arga-Wicaksono/codescope/releases/latest/download/cs-x86_64-linux -o cs && chmod +x cs && sudo mv cs /usr/local/bin/

# Or install from source
cargo install --git https://github.com/Arga-Wicaksono/codescope.git
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

---

## Quick Start

```bash
# Understand a project instantly
cs stats                            # Project composition by language
cs file "config" -I                 # Fuzzy-find config files interactively
cs content "fn main" -n -C 2        # Search with context

# Navigate code like a pro
cs where "parse_config"             # Jump to definition
cs open "main" --line 42            # Open file at exact line
cs recent --type rust --since '2h'  # What changed recently?

# Get AI-ready structured output
cs content "auth" --type rust -j    # JSON output for scripts/AI
cs where "Handler" -j               # Symbol locations as JSON

# Search across repositories
cs across "TODO" --workspace ~/projects   # Find across all repos
cs explain '\s+\w+'                      # Understand regex patterns

# Web search from terminal
cs web "rust async tutorial" -l 5
```

---

## Commands

### `cs file <pattern>` — File Search

Find files by name using fuzzy matching (SkimMatcherV2).

```bash
cs file "Cargo"           # Basic fuzzy search
cs file "main" -e rs      # Filter by extension
cs file "test" --depth 2  # Limit recursion depth
cs file "config" -I       # Interactive selection
cs file "config" -j       # JSON output (AI-ready)
```

### `cs content <pattern>` — Content Search

Search text inside files with three matching modes. Uses **rayon** for parallel processing.

```bash
cs content "function"     # Fuzzy search (default)
cs content "fn\s+\w+" --regex  # Regex search
cs content "config" -x     # Exact substring match
cs content "TODO" -n      # Show line numbers
cs content "error" -C 3   # 3 context lines
cs content "TODO" --count  # Per-file match counts
cs content "fn" --invert   # Non-matching lines
cs content "auth" -j       # Structured JSON output
```

### `cs where <name>` — Find Definitions

```bash
cs where 'parse_config'   # Search all source files
cs where 'MyStruct' --type rust
cs where 'handler' --open  # Find + open at definition line
cs where 'Handler' -j      # JSON output for AI consumption
```

Supports: Rust, Python, JS/TS, Go, Java/Kotlin, C/C++.

### `cs open <pattern>` — Search + Open in Editor

```bash
cs open 'main'             # Find and open best match
cs open 'config' --line 42  # Open at specific line
cs open 'TODO' -I          # Interactive select, then open
```

### `cs recent [options]` — Recently Modified Files

```bash
cs recent                  # 20 most recent files
cs recent --type rust      # Rust files only
cs recent --since '2h'     # Modified in last 2 hours
cs recent --open           # Open most recent in editor
```

### `cs across <pattern>` — Cross-Repository Search

```bash
cs across 'TODO' --workspace ~/projects    # Auto-discover git repos
cs across 'error' --repos /a/repo1,/b/repo2
```

### `cs stats [options]` — File Statistics

```bash
cs stats                   # Current directory stats
cs stats --type rust       # Rust files only
cs stats -j                # JSON output (AI-ready)
```

### `cs explain <pattern>` — Explain Regex

```bash
cs explain '\s+\w+'         # Explains each token
cs explain '[A-Z][a-z]+'
```

### `cs web <query>` — Web Search

```bash
cs web "rust tutorial" -l 5   # Search DuckDuckGo from terminal
cs web "async await" -j        # JSON output
```

---

## Matching Modes

| Mode | Flag | Best For |
|------|------|----------|
| **Fuzzy** | *(default)* | Approximate matching, tolerates typos |
| **Exact** | `-x` / `--exact` | Precise substring, zero false positives |
| **Regex** | `--regex` | Full pattern power |

---

## JSON Output (`-j`)

Every command supports structured JSON output via the `-j` flag. This is the foundation for AI integration and scripting:

```bash
# Human-readable
$ cs content "auth" --type rust -n
src/auth/mod.rs:15:pub fn authenticate(token: &str) -> Result<User>
src/auth/handler.rs:8:async fn auth_handler(req: Request) -> Response

# Same query, AI-ready JSON
$ cs content "auth" --type rust -n -j
{"results":[{"file":"src/auth/mod.rs","line":15,"content":"pub fn authenticate(token: &str) -> Result<User>"},{"file":"src/auth/handler.rs","line":8,"content":"async fn auth_handler(req: Request) -> Response"}],"count":2,"query":"auth"}
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Results found |
| `1` | No results found |
| `2` | Error |

---

## Architecture

```
┌──────────────────────────────────────────────┐
│                 Repository                    │
│  (files, symbols, dependencies, git history) │
└──────────────────┬───────────────────────────┘
                   │
          ┌────────▼────────┐
          │    CodeScope     │
          │    (cs binary)   │
          │                  │
          │  ┌────────────┐  │
          │  │ File Search │  │
          │  │ Content S.  │  │
          │  │ Symbol Intel│  │
          │  │ Context Eng.│  │
          │  │ Dep Graph   │  │
          │  └────────────┘  │
          └────────┬────────┘
                   │
       ┌───────────┼───────────┐
       │                       │
  ┌────▼─────┐          ┌──────▼──────┐
  │ Developer │          │  AI Agent   │
  │  (CLI)   │          │ (JSON/CLI)  │
  │ Terminal │          │ MCP / Pipe  │
  └──────────┘          └─────────────┘
```

### Design Philosophy

CodeScope is a **pipeline tool**, not a platform. It takes a repository as input and produces structured context as output. This makes it composable — you can pipe `cs` output into any AI agent, CI/CD pipeline, or custom script.

**Today:** File search, content search, definition finding, cross-repo search, statistics.
**Next:** Symbol indexing (Tree-sitter), context engine, dependency graph, MCP protocol.
**Never:** AI chatbot, code generation, cloud platform. CodeScope is infrastructure.

---

## Comparison

| Feature | `cs` | `fd` | `rg` | `fzf` | `ctags` | LSP |
|---------|------|------|------|-------|---------|-----|
| Fuzzy file search | Yes | Yes | No | Yes | No | No |
| Content search | Fuzzy + exact + regex | No | Regex only | No | No | Yes |
| Symbol intelligence | Basic → Full (roadmap) | No | No | No | Stale | Yes |
| Cross-repo search | Yes | No | No | No | No | No |
| JSON output | Yes (`-j`) | No | No | No | No | No |
| AI-ready context | Yes (roadmap) | No | No | No | No | No |
| Interactive mode | Built-in (`-I`) | Via pipe | Via pipe | Yes | No | Editor |
| Local-first | Yes | Yes | Yes | Yes | Yes | Yes |
| Zero runtime deps | Yes | Yes | Yes | Yes | Yes | No |
| Written in | Rust | Rust | Rust | Go | C | Various |

---

## Configuration

Create `~/.codescope.json` for persistent defaults:

```json
{
  "default_limit": 50,
  "default_depth": 5,
  "color": true,
  "web_timeout": 15
}
```

---

## Running Tests

```bash
cargo test
```

---

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for the full development roadmap.

**Next milestones:**
- **Phase 2** — Stable JSON schemas, machine-friendly output for all commands
- **Phase 3** — Symbol intelligence with Tree-sitter (`cs symbol`, `cs refs`, `cs callers`)
- **Phase 7** — MCP protocol support (Claude, Cursor, AI agents can use `cs` directly)

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>Repository Intelligence Infrastructure for the AI Coding Era</strong><br>
  Built with Rust by <a href="https://github.com/Arga-Wicaksono">Arga Wicaksono</a> ·
  <a href="https://github.com/Arga-Wicaksono/codescope">GitHub</a> ·
  <a href="https://github.com/Arga-Wicaksono/codescope/releases">Releases</a> ·
  <a href="docs/ROADMAP.md">Roadmap</a>
</p>
