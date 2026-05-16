# 📝 Dev.to / Medium Article — "I Replaced 5 CLI Tools With One Rust Binary"

## Title Options

**Option A (Story-driven — recommended):**
```
I Was Tired of Chaining 5 CLI Tools to Understand Codebases. So I Built One.
```

**Option B (Technical):**
```
Building a 2MB Rust CLI That Replaces fd, rg, ctags, and fzf for Code Intelligence
```

**Option C (AI angle):**
```
How I Give LLMs Perfect Code Context Without Hallucinations — Using a Single CLI Tool
```

---

## Article (Option A — full draft)

```markdown
# I Was Tired of Chaining 5 CLI Tools to Understand Codebases. So I Built One.

Every developer knows this feeling: you join a new project, open the terminal, and spend the next hour just trying to understand where things are.

```
fd "config" | fzf                      # Find the config file
rg "parse_config" -l                   # Where is it used?
ctags -R && grep tags                  # Where is it defined?
# How do I know what depends on it?
# How do I get context for my AI assistant?
```

Four different tools. Four different interfaces. And some questions have no standard tool at all.

## The Problem With Modern Code Navigation

The Unix philosophy says "do one thing well." And tools like fd, rg, and fzf are excellent at what they do. But when you need to understand a codebase — not just search it — you end up in an integration nightmare.

Consider a real workflow:

1. **Find a file**: `fd "auth" | fzf` — works great
2. **Search inside files**: `rg "authenticate" -C 3` — works great
3. **Find where a function is defined**: `ctags -R && grep "authenticate" tags` — wait, ctags doesn't handle all languages well, and the output format is outdated
4. **Find all callers of a function**: No standard CLI tool exists for this
5. **Understand what depends on a module**: You need custom scripts or an IDE
6. **Get context for AI**: Copy-paste manually

For AI-assisted development, the situation is even worse. When Claude or ChatGPT tries to help you with code, it often gets random snippets and hallucinates. The problem isn't the AI — it's the context.

## One Binary to Rule Them All

That's why I built **CodeScope** (`cs`) — a single Rust binary that handles all of the above and more. Here's what it looks like:

```bash
# File search (fuzzy matching)
cs file "config" -I                    # Interactive picker
cs open "main" --line 42               # Find + open in $EDITOR

# Content search (3 modes: fuzzy, exact, regex)
cs content "fn main"                    # Fuzzy search
cs content "TODO|FIXME" --regex         # Regex search
cs content "config" -x                  # Exact substring
cs content "old" --replace "new" --write # Find and replace

# Symbol intelligence (10 languages)
cs where "parse_config"                 # Find definition
cs refs "MyStruct"                      # Find all references
cs callers "process_data"               # Who calls this function?
cs symbols . --symbol-type function -l 20  # List all functions

# Context engine for AI
cs context "authentication"             # Extract ranked context
cs pack "auth flow" -b 8000             # Pack into token-efficient LLM prompt
cs trace "main" --max-depth 5           # Trace execution flow

# Dependency intelligence
cs graph --type modules                 # Module dependency tree
cs graph --type calls --format dot      # Call graph (Graphviz)
cs impact "utils.rs"                    # What depends on this?

# Cross-repo search
cs across 'TODO' --workspace ~/projects # Search all repos at once

# AI integration
cs serve --mcp                          # MCP server for Claude/Cursor
cs semantic "database connection pool"   # Semantic search
cs rewrite "add error handling" --write # AI-powered code rewrite
```

## What Makes CodeScope Different

### 1. Deterministic, Not Probabilistic

Unlike AI-powered tools, CodeScope always returns the same results for the same query. No randomness, no hallucinations. This is critical for developer trust — you need to know your tools are reliable.

### 2. AI-Consumable by Design

Every command supports JSON output (`-j` flag). This makes it trivial to pipe results into AI workflows:

```bash
# Get structured context for Claude
cs context "authentication" -j | Claude "explain this code"

# Pack code for an LLM prompt
cs pack "error handling" -b 8000 -j > prompt.txt
```

### 3. Zero Runtime Dependencies

One static binary. No Python, no Node.js, no database. Install it once and it works everywhere. The binary is ~2 MB thanks to Rust's optimization and LTO + strip.

### 4. Feature Flags for Lean Builds

```bash
# Full build (web search + interactive) — default, ~2 MB
cargo build --release

# Without web search — smaller
cargo build --release --no-default-features --features interactive

# Minimal: file + content only — smallest possible
cargo build --release --no-default-features
```

## Technical Architecture

CodeScope is organized into 28 Rust modules covering 7 capability pillars:

| Pillar | Commands |
|--------|----------|
| Search & Navigation | file, content, web, across, open, where, recent |
| Symbol Intelligence | symbol, refs, callers, symbols |
| Context Engine | context, pack, trace |
| Dependency Graph | graph, impact |
| Developer Tools | stats, explain, history, config, completions, schema |
| AI & Integration | serve, semantic, rewrite, lsp-bridge |
| Caching | cache stats, clear, cleanup |

The symbol intelligence engine uses regex-based parsing that supports 10 programming languages: Rust, Python, JavaScript/TypeScript, Go, Java/Kotlin, C, C++, Ruby, PHP, and Swift.

For semantic search, CodeScope uses a pure TF-IDF implementation with cosine similarity — no external ML library needed. It tokenizes source files, strips common keywords, and ranks results by relevance.

## Real-World Example: Onboarding to a New Project

Here's how I use CodeScope when joining a new project:

```bash
# Step 1: Get an overview
cs stats                              # File count per language
cs graph --type modules                # Module structure

# Step 2: Find the authentication system
cs context "authentication"            # All relevant files + symbols
cs where "authenticate" --open         # Jump to definition

# Step 3: Understand the data flow
cs trace "handle_request" --max-depth 5  # Follow the call chain
cs impact "models/user.rs"             # What uses the user model?

# Step 4: Search for common patterns
cs across 'TODO' --workspace ~/projects  # All TODOs across all repos
cs content 'deprecated' --count -j     # Deprecated APIs with counts

# Step 5: Get AI assistance
cs pack "authentication flow" -b 8000    # Pack context for LLM
cs rewrite "add input validation" --dry-run  # AI-suggested improvements
```

What used to take 30+ minutes with 5 different tools now takes 2 minutes with one binary.

## Try It

```bash
# Quick install (macOS/Linux)
curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash

# Homebrew
brew tap Arga-Wicaksono/codescope && brew install codescope

# From source
cargo install --git https://github.com/Arga-Wicaksono/codescope.git
```

**GitHub:** https://github.com/Arga-Wicaksono/codescope

CodeScope is MIT licensed and open source. I'd love to hear your feedback, ideas, and contributions. What codebase navigation workflows would you like to see supported?

---

*If you found this useful, follow me for more posts about Rust CLI tools and developer productivity. Feel free to star the repo on GitHub!*
```

---

## Tips untuk Dev.to

- **Tags**: `#rust` `#cli` `#opensource` `#productivity` `#ai` `#devtools`
- **Cross-post ke**: Hashnode, Medium, dan personal blog
- **Follow-up articles** (ide untuk seri):
  - "How I implemented TF-IDF semantic search in pure Rust"
  - "Building an MCP server for Claude in Rust"
  - "Zero-dependency CLI tools: the Rust advantage"
  - "Benchmarking code search: fd vs rg vs cs on real codebases"
