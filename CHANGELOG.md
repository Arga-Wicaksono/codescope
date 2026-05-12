# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
