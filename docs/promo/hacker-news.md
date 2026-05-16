# 📝 Hacker News — Show HN Post

## Title

```
Show HN: CodeScope – One Rust CLI to replace fd, rg, and ctags for codebase intelligence
```

## Body

```text
I built CodeScope (cs) because I was tired of chaining fd, rg, ctags, and custom scripts every time I needed to understand a codebase. It's a single ~2 MB Rust binary with 28 commands that handles file search, content search, symbol lookup, dependency graphs, impact analysis, cross-repo search, and AI-ready context extraction.

What it does:
- cs file/content: fuzzy, exact, and regex search with .gitignore awareness
- cs where/symbol/refs/callers: find definitions, references, and callers across 10 languages (Rust, Python, JS/TS, Go, Java, C/C++, Ruby, PHP, Swift)
- cs context/pack: extract ranked code context and pack it into token-budgeted prompts for LLMs
- cs graph/impact: build module dependency graphs and analyze what depends on a file
- cs across: search across multiple repositories at once
- cs serve --mcp: MCP server for AI agents (Claude, Cursor)
- cs semantic: TF-IDF semantic search with cosine similarity
- cs lsp-bridge: LSP protocol bridge for editor integration

Every command supports -j (JSON output) for scripting and -I (interactive mode) with a built-in fuzzy picker. Smart case is on by default (case-insensitive unless query has uppercase). Zero runtime dependencies.

Install:
  curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash

Or: cargo install --git https://github.com/Arga-Wicaksono/codescope.git

Repo: https://github.com/Arga-Wicaksono/codescope
License: MIT

I'd love feedback on the architecture and any benchmarks you can run against fd/rg on real codebases.
```

---

## ⚠️ Tips Kritis untuk Hacker News

1. **Format**: HN mendukung plain text minimal. JANGAN pakai Markdown berat. Gunakan indentasi untuk daftar.

2. **Waktu posting**: 
   - **Terbaik**: Selasa-Kamis, 8:00-10:00 AM US Eastern
   - **Hindari**: Weekend, hari libur US

3. **JANGAN**:
   - Pakai emoji di title
   - Over-hype atau clickbait
   - Mention "AI" terlalu banyak (komunitas HN skeptis terhadap AI hype)

4. **DO**:
   - Tunjukkan technical depth di komentar
   - Siapkan benchmark numbers (cs vs fd, cs vs rg, cs vs ctags)
   - Jawab setiap pertanyaan teknis dengan detail
   - Tunjukkan code quality (architecture, test coverage)

5. **Comment strategy**:
   - Jika ada yang tanya "why not just use ast-grep/tree-sitter?" → jawab: "cs is complementary, not a replacement. ast-grep is AST-based pattern matching; cs is fast codebase navigation with symbol lookup + context extraction. Different layer."
   - Jika ada yang tanya performance → siapkan benchmark: `time cs content "pattern"` vs `time rg "pattern"` pada repo besar
   - Jika kritik "yet another search tool" → jawab: "cs is specifically designed as repository intelligence, not just search. The dependency graph, impact analysis, context extraction, and MCP server are what make it different."

---

## Benchmark Script (siapkan sebelum posting)

```bash
# Compare cs vs rg vs fd on a large repo
echo "=== File Search ==="
time fd "test" ~/projects/large-repo > /dev/null
time cs file "test" ~/projects/large-repo > /dev/null

echo "=== Content Search ==="
time rg "function" ~/projects/large-repo > /dev/null
time cs content "function" ~/projects/large-repo > /dev/null
```

Siapkan hasil benchmark ini untuk menjawab pertanyaan performance di komentar.
