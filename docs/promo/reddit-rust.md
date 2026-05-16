# 📝 Reddit r/rust — Launch Post

## Title Options (pilih satu — A/B test jika bisa)

**Option A (Problem-first — recommended):**
```
I was tired of chaining fd, rg, and ctags just to understand a codebase. So I built one Rust binary that does it all.
```

**Option B (Tech showcase):**
```
CodeScope: A ~2MB Rust CLI that replaces fd + rg + ctags + fzf for codebase intelligence. 28 commands, 10 languages, zero runtime deps.
```

**Option C (AI angle — hot topic):**
```
I built a Rust CLI that extracts structured code context for LLMs. No hallucinations, no API keys — just deterministic code intelligence.
```

---

## Body (Option A — copy-paste ready)

```markdown
**TL;DR:** One binary (`cs`) that handles file search, content search, symbol lookup, dependency graphs, cross-repo search, and AI-ready context extraction. ~2 MB, zero runtime deps, written in Rust.

## The problem

Every time I join a new project or try to understand a large codebase, I end up chaining tools:

```bash
fd pattern | fzf                     # find files
rg "pattern" | fzf                   # search content
ctags -R && grep tags                # find definitions
# No standard tool for: dependency graphs, impact analysis, AI context extraction
```

Each tool has its own interface, its own flags, and they don't compose well. For AI-assisted development, the situation is worse — LLMs get random code snippets and hallucinate.

## The solution

I built **CodeScope** (`cs`) — a single Rust binary that replaces all of the above:

```bash
cs file "config"           # Find files (fuzzy)
cs content "fn main"       # Search content (fuzzy/exact/regex)
cs where "parse_config"    # Find definitions (10 languages)
cs symbol "MyStruct"       # Find symbols with metadata
cs refs "handler"          # Find all references
cs callers "process_data"  # Find callers of a function
cs context "auth"          # Extract context for AI/LLM
cs pack "auth flow" -b 8000 # Pack context into token-efficient LLM prompt
cs graph --type modules    # Dependency graph (tree/dot/json)
cs impact "utils.rs"       # What depends on this file?
cs across 'TODO' --workspace ~/projects  # Cross-repo search
cs semantic "database connection pool"     # TF-IDF semantic search
cs serve --mcp             # MCP server for Claude/Cursor
```

## What makes it different

- **Deterministic** — same query always returns same results, no randomness
- **AI-consumable** — every command supports `-j` (JSON output) for scripting and LLM integration
- **Zero dependencies** — single static binary, no runtime deps, works offline
- **Blazing fast** — Rust + rayon parallelism + .gitignore-aware walking
- **Cross-platform** — Linux (glibc/musl), macOS (Intel/ARM), Windows

## Architecture

28 Rust modules, 10 language support, 3 matching modes (fuzzy/exact/regex), feature flags for optional dependencies (`web-search`, `interactive`, `embeddings`).

```
cs file "main" -j     → structured JSON for AI agents
cs context "auth"     → ranked context with relevance scores
cs pack "feature" -b 8000 → token-budgeted prompt for LLM
```

## Try it

```bash
# Quick install
curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash

# Or from source
cargo install --git https://github.com/Arga-Wicaksono/codescope.git

# Or via Homebrew
brew tap Arga-Wicaksono/codescope && brew install codescope
```

**Repo:** https://github.com/Arga-Wicaksono/codescope
**License:** MIT

Feedback, issues, and contributions are very welcome! I'm particularly interested in:
- Ideas for improving the symbol intelligence (currently regex-based, planning tree-sitter)
- MCP protocol feedback from Claude/Cursor users
- Performance benchmarks against fd/rg for real-world codebases
```

---

## Timing

- **Post**: Senin atau Selasa pagi (waktu US Eastern, ~9-11 AM EST)
- **Mengapa**: Reddit traffic tinggi di hari kerja pagi, post naik ke top faster
- **Hindari**: Weekend (traffic rendah)

## Pasca-post

- Balas SETIAP komentar dalam 30 menit pertama (kritis!)
- Jika ada bug report, responsif dan positif
- Jika ada kritik arsitektur, jawab dengan detail teknis
- Update post jika ada fix/release baru
