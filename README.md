<p align="center">
  <img src="assets/logo.png" alt="cs logo" width="120" height="120">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/version-1.0.0-blue?style=flat-square" alt="Version">
  <img src="https://img.shields.io/badge/license-MIT-yellow?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/tests-146%20passed-brightgreen?style=flat-square" alt="Tests">
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20Windows-lightgrey?style=flat-square" alt="Platform">
</p>

<h1 align="center">cs — Code Scope</h1>

<p align="center">
  <strong>Scope your codebase — file + content + web search in one binary.</strong><br>
  No pipe setup. No runtime dependencies. Just <code>cs</code>.
</p>

<p align="center">
  <img src="assets/demo.gif" alt="cs demo" width="640">
</p>

<p align="center">
  <a href="#installation">Install</a> ·
  <a href="#quick-start">Quick Start</a> ·
  <a href="#commands">Commands</a> ·
  <a href="#matching-modes">Matching Modes</a> ·
  <a href="#configuration">Config</a> ·
  <a href="#comparison">Comparison</a>
</p>

---

## Why `cs`?

Tools like `rg` and `fzf` are great, but real workflows often look like this:

```bash
# Find a file, then open it
fd pattern | fzf

# Search content, pipe to interactive picker
rg "TODO" | fzf

# Search the web? Open a browser.
```

`cs` is a single tool that handles all three — **file search**, **content search**, and **web search** — with built-in interactive mode, three matching strategies, JSON output for scripting, and a persistent config file. One ~2 MB binary, zero runtime dependencies.

## Features

| Feature | Details |
|---------|---------|
| **Unified search** | File names, file contents, and web — all in one tool |
| **3 matching modes** | Fuzzy (default), exact (`-x`), and regex (`--regex`) |
| **Interactive mode** | Built-in fuzzy-select with `-I` — no `fzf` pipe needed |
| **JSON output** | Zero-config `-j` flag for scripting and piping |
| **Parallel processing** | Content search parallelized with rayon |
| **Config file** | Persistent defaults in `~/.codescope.json` |
| **Colored output** | Auto-detected TTY colors, disable with `--no-color` |
| **Cross-platform** | Prebuilt binaries for Linux/macOS/Windows |
| **Compact binary** | ~2 MB with LTO + strip, no runtime dependencies |
| **.gitignore aware** | Respects `.gitignore`, `.ignore`, global gitignore by default |
| **Smart case** | Case-insensitive unless pattern has uppercase (like ripgrep) |
| **Stdin pipe** | `cat file \| cs content 'pattern'` — search piped input |
| **Shell completions** | `cs completions bash\|zsh\|fish\|powershell\|elvish` |
| **File type presets** | `--type rust/python/js/web/cpp/go/java/config/doc/data/shell` |
| **Replace mode** | `--replace 'text'` dry run, `--write` to apply changes |
| **Count mode** | `--count` shows per-file match counts |
| **Invert match** | `--invert` shows non-matching lines |
| **cs open** | Search files + open in `$EDITOR` with line support |
| **cs recent** | Find recently modified files with relative times |
| **cs where** | Find function/class definitions across languages |
| **cs explain** | Explain regex patterns in plain language |
| **cs history** | Search history with auto-rotation |
| **cs across** | Cross-repository search (`--repos`, `--workspace`) |
| **cs stats** | File statistics (per-language line counts) |
| **Feature flags** | Build only what you need (`web-search`, `interactive`) |

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
# Find files by name
cs file "Cargo"

# Search content with fuzzy matching
cs content "fn main"

# Search content with line numbers and context
cs content "fn main" -n -C 2

# Exact match (no fuzzy false positives)
cs content "config" -x -i

# Regex match
cs content 'TODO|FIXME|HACK' --regex

# Interactive selection from results
cs file "config" -I

# Web search
cs web "rust tutorial" -l 5

# Replace mode
cs content 'old_api' --replace 'new_api' -x
cs content 'old_api' --replace 'new_api' -x --write

# Open file in editor
cs open 'main' --line 42
cs open 'TODO' -I

# Recently modified files
cs recent --type rust --since '2h'

# Find function/class definitions
cs where 'parse_config'
cs where 'MyStruct' --open

# Explain regex
cs explain '\s+\w+'

# Cross-repository search
cs across 'TODO' --workspace ~/projects

# File statistics
cs stats --type rust
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
cs file "config" -j       # JSON output
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
```

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

### `cs where <name>` — Find Definitions

```bash
cs where 'parse_config'   # Search all source files
cs where 'MyStruct' --type rust
cs where 'handler' --open  # Find + open at definition line
```

Supports: Rust, Python, JS/TS, Go, Java/Kotlin, C/C++.

### `cs explain <pattern>` — Explain Regex

```bash
cs explain '\s+\w+'         # Explains each token
cs explain '[A-Z][a-z]+'
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
cs stats --json          # JSON output
```

---

## Matching Modes

| Mode | Flag | Best For |
|------|------|----------|
| **Fuzzy** | *(default)* | Approximate matching, tolerates typos |
| **Exact** | `-x` / `--exact` | Precise substring, zero false positives |
| **Regex** | `--regex` | Full pattern power |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Results found |
| `1` | No results found |
| `2` | Error |

---

## Comparison with Other Tools

| Feature | `cs` | `fd` | `rg` | `fzf` |
|---------|------|------|------|-------|
| Fuzzy file search | Yes | Yes | No | Yes |
| Content search | Fuzzy + exact + regex | No | Regex only | No |
| Web search | Yes (DuckDuckGo) | No | No | No |
| Matching modes | 3 (fuzzy/exact/regex) | 1 (glob) | 1 (regex) | 1 (fuzzy) |
| Interactive mode | Built-in (`-I`) | Via pipe | Via pipe | Yes |
| JSON output | Yes (`-j`) | No | No | No |
| Config file | Yes | No | Yes | No |
| Parallel processing | Yes (rayon) | Yes | Yes | No |
| Cross-repo search | Yes | No | No | No |
| Written in | Rust | Rust | Rust | Go |

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
