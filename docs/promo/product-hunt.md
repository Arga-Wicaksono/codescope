# 📝 Product Hunt — Launch Content

## 🎯 Listing Details

### Product Name
**CodeScope**

### Tagline (60 chars)
```
One CLI to understand any codebase. Search, symbols, AI context.
```

### One-liner Description (260 chars)
```
CodeScope (cs) replaces fd + rg + ctags + fzf with one 2MB Rust binary. 28 commands for file search, symbol lookup, dependency graphs, cross-repo search, and AI-ready context extraction. Zero runtime deps. Open source.
```

### Full Description
```markdown
## The problem
Developers chain 5+ tools to navigate codebases: fd for files, rg for content, ctags for symbols, fzf for interactive picking. AI agents get random code context and hallucinate. Neither approach scales.

## The solution
CodeScope (`cs`) is a single ~2 MB Rust binary that handles everything:

🔍 **Search & Navigation**
- File search (fuzzy), content search (fuzzy/exact/regex), web search, cross-repo search
- Interactive mode with built-in fuzzy picker — no fzf needed

🧠 **Symbol Intelligence**
- Find definitions across 10 languages (Rust, Python, JS/TS, Go, Java, C/C++, Ruby, PHP, Swift)
- Find all references and callers of any function
- List all symbols in a file or directory

📦 **Context Engine**
- Extract ranked code context for any topic
- Pack context into token-efficient LLM prompts (budget-aware)
- Trace execution flow through call chains

🔗 **Dependency Intelligence**
- Module and call dependency graphs (tree, DOT, JSON)
- Impact analysis — what depends on this file?

🤖 **AI Integration**
- MCP server for Claude, Cursor, and other AI agents
- LSP bridge for editor integration
- Semantic search (TF-IDF)
- AI-powered code rewrite (Ollama/OpenAI)

## Why CodeScope?
- **Deterministic**: Same query → same results, no randomness
- **Zero dependencies**: Single static binary, no runtime, no cloud
- **AI-consumable**: Every command outputs JSON for LLM integration
- **Blazing fast**: Rust + rayon parallelism
- **Cross-platform**: Linux, macOS, Windows
```

### Topics/Tags
```
Developer Tools, Open Source, Productivity, CLI, Rust, AI Tools, Code Intelligence, Search, Terminal, DevOps
```

### First Comment (Maker Comment) — UPDATED
```markdown
Hey Product Hunt! 👋

I'm Arga, the creator of CodeScope.

**The backstory:** Every time I joined a new project, I spent hours chaining fd, rg, ctags, and custom scripts just to understand the codebase. For AI-assisted development, it was worse — LLMs got random context and hallucinated.

So I built CodeScope: one Rust binary that makes any codebase instantly navigable and AI-ready.

**What's new in v1.3:**
- 🧠 Symbol intelligence: find definitions, references, callers across 10 languages
- 📦 Context engine: extract and pack ranked code context for LLM prompts
- 🔗 Dependency graphs: module graphs, impact analysis, call tracing
- 🤖 MCP server: Claude and Cursor can use CodeScope natively via Model Context Protocol
- 🔍 Semantic search: TF-IDF with cosine similarity — no ML library needed
- ✍️ AI rewrite: rewrite code using Ollama or OpenAI
- 🖥️ LSP bridge: editor integration for Neovim, VS Code, etc.

**One install, zero dependencies:**
```bash
curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash
```

**Or:**
- `brew tap Arga-Wicaksono/codescope && brew install codescope` (macOS)
- `cargo install --git https://github.com/Arga-Wicaksono/codescope.git` (any platform)
- `scoop install codescope` (Windows)

Open source (MIT). I'd love to hear your feedback — especially from AI/LLM power users!

🌐 https://github.com/Arga-Wicaksono/codescope
```

---

## 📸 Gallery Assets

| # | Asset | File | Notes |
|---|-------|------|-------|
| 1 | Logo/icon | `assets/logo.png` | 1024x1024 |
| 2 | Demo GIF | `assets/demo.gif` | Quick command showcase |
| 3 | Screenshot: banner | Bare `cs` | ASCII art + command reference |
| 4 | Screenshot: search | `cs file "config" -I` | Interactive picker |
| 5 | Screenshot: JSON | `cs where "main" -j` | AI-consumable output |
| 6 | Screenshot: graph | `cs graph --type modules` | Dependency tree |

**Tip**: Screenshot harus high-res (1440px width). Gunakan dark terminal theme.

---

## 🗓️ Timing

- **Launch day**: Rabu (traffic PH tertinggi di pertengahan minggu)
- **Jam**: 12:01 AM PST (supaya muncul di "Today" page sepagi mungkin)
- **Duration**: Aktif 24-48 jam (upvote awal sangat penting)

## ⚡ Upvote Strategy

1. Share ke komunitas yang sudah Anda ikuti (Discord, Slack, Twitter)
2. Minta teman developer untuk upvote (JANGAN bot!)
3. Post di Twitter/X dengan link PH
4. Share di Reddit dengan mention "we just launched on PH"
