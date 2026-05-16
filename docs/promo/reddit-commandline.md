# 📝 Reddit r/commandline — Launch Post

## Title Options

**Option A (Utility-first):**
```
cs — One binary to replace fd + rg + ctags + fzf for codebase navigation. 28 commands, JSON output, cross-repo search.
```

**Option B (Workflow):**
```
Showcased my CLI tool: how I replaced 5+ terminal tools with a single 2MB Rust binary for understanding any codebase.
```

---

## Body

```markdown
**TL;DR:** `cs` is a Rust CLI that combines file search, content search, symbol lookup, dependency graphs, cross-repo search, and AI context extraction into one binary. Zero runtime dependencies.

## The workflow problem

My typical codebase navigation looks like this:

```bash
# Find a file → open it
fd "config" | fzf | xargs vim

# Search for a function → find its definition
rg "parse_config" --type rust -l
ctags -R && grep "parse_config" tags

# Understand impact of changing a file
# No standard tool exists for this

# Get context for AI/LLM
# Manual copy-paste into prompt
```

Each step uses a different tool with a different interface. Some tasks (dependency graphs, impact analysis, cross-repo search) have no standard CLI tool at all.

## Enter `cs`

```bash
# File search (fuzzy, exact, glob)
cs file "config" -I                    # Interactive picker
cs open "main" --line 42               # Find + open in $EDITOR

# Content search (3 modes)
cs content "fn main"                    # Fuzzy
cs content '\w+\s*=\s*\d+' --regex     # Regex
cs content "TODO" --count               # Per-file counts

# Symbol intelligence (10 languages)
cs where "parse_config"                 # Find definition
cs refs "MyStruct"                      # Find references
cs callers "process_data"               # Find callers

# Context for AI
cs context "authentication"             # Ranked code context
cs pack "auth flow" -b 8000             # Token-budgeted for LLM
cs serve --mcp                          # MCP server for Claude

# Cross-repo search
cs across 'TODO' --workspace ~/projects

# Dependency graph
cs graph --type modules --format dot | dot -Tpng -o deps.png
cs impact "utils.rs"                    # Who depends on this?

# Every command supports JSON:
cs file "config" -j | jq .
```

## Key features for CLI enthusiasts

- **3 matching modes**: fuzzy (SkimMatcherV2), exact substring, regex
- **Smart case**: case-insensitive unless query has uppercase (like ripgrep)
- **Interactive mode**: built-in fuzzy picker (`-I`), no fzf needed
- **JSON output**: `-j` flag on every command for piping to `jq`
- **Shell completions**: bash, zsh, fish, powershell
- **Stdin pipe**: `cat file | cs content "pattern"`
- **.gitignore aware**: respects ignore files by default
- **Single binary**: ~2 MB, no runtime deps, static linking

## Install

```bash
# One-liner (macOS/Linux)
curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash

# Build variants
cargo install --git https://github.com/Arga-Wicaksono/codescope.git
# Minimal (no web/interactive): --no-default-features
# Full: default features
```

**Repo:** https://github.com/Arga-Wicaksono/codescope

Happy to hear feedback, especially on the interactive mode and matching behavior!
```

---

## Tips untuk r/commandline

- Komunitas ini sangat peduli **UX terminal** — tunjukkan output yang bersih dan readable
- Mereka suka ** piping/composition** — tunjukkan contoh `| jq`, `| xargs`
- Hindari buzzword "AI" terlalu banyak — fokus ke utilitas CLI
- Screenshot asciinema demo jika ada
