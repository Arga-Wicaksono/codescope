# 🐦 Twitter/X Thread — CodeScope Launch

## Thread (9 tweets — copy-paste ready)

---

**Tweet 1/9 — Hook**

```
I was tired of chaining fd + rg + ctags + fzf to understand codebases.

So I built one Rust binary that does it all.

28 commands. 10 languages. 2MB. Zero runtime deps.

Meet CodeScope (cs) 👇

#rust #cli #opensource
```

---

**Tweet 2/9 — The Problem**

```
The problem with modern code navigation:

fd → find files
rg → search content
ctags → find definitions
fzf → interactive picking
??? → dependency graphs
??? → impact analysis
??? → AI context extraction

5 tools, 5 interfaces, and some tasks have no solution at all.
```

---

**Tweet 3/9 — The Solution**

```
One binary replaces them all:

cs file "config"          → find files (fuzzy)
cs content "fn main"      → search content
cs where "parse_config"   → find definitions
cs graph --type modules   → dependency graph
cs context "auth"         → extract AI context
cs across 'TODO' -w ~/projects → cross-repo search

Every command supports JSON output (-j) for scripting.
```

---

**Tweet 4/9 — Symbol Intelligence**

```
The symbol engine supports 10 languages:

Rust, Python, JS/TS, Go, Java, C, C++, Ruby, PHP, Swift

cs where "authenticate"     → go to definition
cs refs "Config"            → find all references
cs callers "process_data"   → who calls this?
cs symbols . --type function → list all functions

No tree-sitter needed. Pure regex-based. Fast.
```

---

**Tweet 5/9 — AI Integration**

```
The best part: AI-ready context extraction.

cs context "auth"           → ranked code context
cs pack "auth flow" -b 8000 → token-budgeted LLM prompt
cs serve --mcp              → MCP server for Claude/Cursor
cs semantic "database pool" → TF-IDF semantic search
cs rewrite "add validation" → AI-powered rewrite (Ollama/OpenAI)

No hallucinations. Deterministic. Structured.
```

---

**Tweet 6/9 — Dependency Intelligence**

```
Unique features you won't find in fd/rg:

cs graph --type modules --format dot | dot -Tpng → visual dependency graph

cs impact "utils.rs" → "these 5 files depend on utils.rs"

cs trace "main" --max-depth 5 → follow the entire call chain

Understanding codebases just got a lot easier.
```

---

**Tweet 7/9 — Technical Stats**

```
Under the hood:

• 28 Rust modules, 7 capability pillars
• ~2 MB binary (LTO + strip)
• Rayon parallelism for search
• .gitignore-aware file walking
• 3 matching modes: fuzzy/exact/regex
• Smart case (like ripgrep)
• Feature flags for optional deps
• 146+ tests
```

---

**Tweet 8/9 — Install**

```
Install in 10 seconds:

curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash

Or:
• brew tap Arga-Wicaksono/codescope && brew install codescope
• cargo install --git https://github.com/Arga-Wicaksono/codescope.git

macOS, Linux, Windows. MIT license.
```

---

**Tweet 9/9 — CTA**

```
Repo: github.com/Arga-Wicaksono/codescope

Feedback, issues, and contributions welcome!

RT if you think developer tools should be fast, simple, and deterministic.

#rustlang #devtools #opensource #productivity #buildinpublic
```

---

## 📅 Posting Strategy

1. **Timing**: Post thread Jumat pagi (before weekend browsing)
2. **Quote tweet**: Quote each tweet with screenshots/GIFs
3. **Engage**: Reply to every reply within 1 hour
4. **Amplify**: Quote-retweet anyone who tries it and shares results
5. **Follow-up**: Post benchmark results as a follow-up thread

## 🔗 Accounts to Tag/Engage

- @rustlang
- @clijockey (CLI tools community)
- Developers who tweet about Rust, fd, rg, fzf
- Anyone who recently asked about code navigation tools

## 📸 Suggested Media

- Asciinema recording of real workflow (link in tweet 3)
- Screenshot of `cs graph --type modules` output
- Screenshot of `cs context "auth" -j` showing JSON output
- Before/after comparison: 5 commands vs 1 command
