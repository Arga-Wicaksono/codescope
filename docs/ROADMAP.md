# CodeScope Roadmap

> **Vision:** Make repositories understandable instantly for humans and AI systems.
>
> **Positioning:** Repository Intelligence Infrastructure for the AI Coding Era.

---

## Core Principles

- **Deterministic** — Same query, same results. Always.
- **Blazing fast** — Rust-native, parallelized, built for large repos.
- **Scriptable** — Structured JSON output for every command.
- **AI-consumable** — Stable schemas, ranked data, context extraction.
- **Local-first** — No cloud, no API keys, no network required.
- **Zero dependencies** — One static binary, zero runtime dependencies.

---

## Phase 1 — Repositioning ✅

**Status:** Complete

Redefine CodeScope from "fast search CLI" to "Repository Intelligence Engine."

- [x] Rewrite README with new positioning
- [x] Update CLI branding (about, banner, help text)
- [x] Update Cargo.toml metadata and keywords
- [x] Add use cases for developers and AI agents
- [x] Add architecture diagram
- [x] Publish ROADMAP.md

---

## Phase 2 — Structured Output (Priority: Critical) ✅

**Status:** Complete

Make every command produce stable, parseable, AI-consumable JSON.

### Deliverables
- [x] Define JSON schema for `cs file` (file path, score, extension, size)
- [x] Define JSON schema for `cs content` (file, line, column, context, match)
- [x] Define JSON schema for `cs where` (symbol, file, line, kind, language)
- [x] Define JSON schema for `cs stats` (language, files, lines, bytes)
- [x] Define JSON schema for `cs across` (repo, file, line, match)
- [x] Create `docs/schemas/` with JSON Schema files
- [x] Add `--schema` flag to print JSON schema for any command
- [x] Ensure zero-breaking-change contract (field names never change)

---

## Phase 3 — Symbol Intelligence (Priority: High) ✅

**Status:** Complete

Repository understanding through code structure analysis.

### Approach
Use **enhanced regex patterns** for grammar-based symbol extraction across 12+ languages. Tree-sitter integration planned for future AST-level accuracy.

### New Commands
```bash
# Symbol lookup — find where something is defined
cs symbol UserService

# Find all references to a symbol
cs refs login_user

# Find all callers of a function
cs callers validate_token

# List all symbols in a file/module
cs symbols src/auth/mod.rs
```

### Deliverables
- [x] Add `tree-sitter` dependency
- [x] Implement symbol parser for Rust, Python, JS/TS (MVP)
- [x] `cs symbol <name>` — Find definition with metadata
- [x] `cs refs <name>` — Find all references
- [x] `cs callers <name>` — Find call graph (who calls this)
- [x] `cs symbols [path]` — List all symbols in scope
- [x] Add `--type` filter for symbol kinds (function, class, struct, etc.)
- [x] JSON output (`-j`) for all symbol commands

### Why It Matters
This is what separates CodeScope from ripgrep. Symbol intelligence enables:
- **Architecture understanding** — See how modules connect
- **AI context building** — AI needs relationships, not just text matches
- **Refactoring safety** — Know every caller before changing a function

---

## Phase 4 — Context Engine (Priority: High) ✅

**Status:** Complete

The killer feature. Intelligent context extraction for humans and AI.

### New Commands
```bash
# Extract relevant context for a topic
cs context auth
# → files, symbols, dependencies, summaries

# Pack context for LLM prompts (token-efficient)
cs pack "authentication bug"
# → ranked code snippets, architecture summary, compressed context

# Explain a code path
cs trace login_user
# → execution flow through functions
```

### Deliverables
- [x] Context extraction engine (gathers files, symbols, dependencies)
- [x] Ranking system (most referenced, recent, dependency weight)
- [x] `cs context <topic>` — Multi-source context extraction
- [x] `cs pack <description>` — LLM-optimized prompt packing
- [x] Token counting and budget awareness (~4 chars/token)
- [x] Architecture-aware ranking (entry points rank higher)
- [x] JSON output with ranking scores

### Why It Matters
The #1 bottleneck in AI coding is **context selection**. Give an AI too much context = confused. Too little = wrong. CodeScope's context engine solves this with deterministic, ranked extraction.

---

## Phase 5 — Dependency Graph (Priority: Medium)

**Timeline:** 2–3 weeks

Repository graph intelligence for understanding relationships.

### New Commands
```bash
# Module dependency graph
cs graph

# Trace a function call chain
cs trace login_user

# Impact analysis — what would change if I modify this file?
cs impact auth.rs
```

### Deliverables
- [ ] Module dependency extraction
- [ ] Function call graph construction
- [ ] `cs graph` — ASCII/JSON dependency graph
- [ ] `cs trace <symbol>` — Call chain visualization
- [ ] `cs impact <file>` — Impact analysis (who depends on this)
- [ ] DOT format export (for Graphviz)
- [ ] JSON output for all graph commands

---

## Phase 6 — TUI Mode (Priority: Medium) ✅

**Status:** Complete

Modern terminal UX with live preview and AI context display.

### Stack
- `ratatui` — Rust TUI framework
- `crossterm` — Terminal backend

### Layout
```
┌─────────────────────────────────┐
│ Search: auth handler           │
├──────────────┬──────────────────┤
│ Symbols/Files│   Preview         │
│  > auth.rs   │   pub fn auth()  │
│    user.rs   │     .token       │
│    handler.rs│     .validate()  │
├──────────────┤                  │
│ AI Context   │                  │
│ Score: 0.92  │                  │
│ Tokens: 1.2k │                  │
└──────────────┴──────────────────┘
```

### Deliverables
- [x] TUI app with ratatui
- [x] File/symbol browser panel
- [x] Live code preview panel
- [x] AI context sidebar (relevance score, token estimate)
- [x] Keyboard navigation (vim-style)
- [x] Split view for diff/comparison

---

## Phase 7 — AI Agent Integration (Priority: High) ✅

**Status:** Complete

Make CodeScope consumable by any AI system.

### MCP Support (Model Context Protocol)
The #1 integration point. MCP lets Claude, Cursor, and any MCP-compatible agent use CodeScope as a tool.

```
AI Agent - MCP Server - CodeScope CLI
AI Agent - HTTP API   - CodeScope CLI
```

### Deliverables
- [x] MCP server implementation
- [x] Register CodeScope tools: `search`, `context`, `symbol`, `trace`, `pack`
- [x] Claude Desktop integration tested
- [x] Cursor integration tested
- [x] CLI API: `cs serve` — HTTP/unix socket API
  - `GET /search?q=auth&type=rust`
  - `GET /context?q=authentication&tokens=4000`
  - `GET /symbol?name=UserService`
  - `GET /trace?symbol=login_user`
  - `GET /pack?description=auth+bug&budget=8000`
- [x] `--llm` flag — Output optimized for token efficiency

### Why This Matters
Once MCP support lands, any AI agent can use CodeScope for repository understanding. This is the distribution flywheel — CodeScope becomes infrastructure that AI tools depend on.

---

## Phase 8 — Performance Excellence (Priority: Ongoing) ✅

**Status:** Complete

Live up to the "blazing fast" promise with benchmarks.

### Deliverables
- [x] Benchmark suite: `cs` vs `rg`, `grep`, `fzf`, `fd` (criterion)
- [x] File search benchmarks (fuzzy, extension filter, collect)
- [x] Content search benchmarks (fuzzy, exact, regex, context, invert)
- [x] Symbol search benchmarks (find, refs, callers, list all)
- [x] Context engine benchmarks (extract, pack, trace)
- [x] Graph benchmarks (module graph, call graph, impact analysis)
- [x] Stats benchmark

---

## Phase 9 — Open Source Ecosystem (Priority: Low) ✅

**Status:** Complete

### Deliverables
- [x] Plugin architecture (trait-based, hook points, plugin manager)
- [x] Built-in plugins: RecencyBoost, MarkdownFormatter, ExtraLanguages
- [x] Rust SDK library (`codescope-sdk`)
- [x] Python SDK (`pip install .`)
- [ ] VS Code extension (via LSP bridge) — future
- [ ] Neovim plugin — future

---

## What We Will NOT Build

These are explicitly out of scope to maintain focus:

| ❌ Don't Build | Why |
|---------------|-----|
| AI chatbot | Not our strength. Claude, GPT, etc. do this better. |
| Code generation | Cursor, Copilot own this space. |
| Copilot clone | We're infrastructure, not an assistant. |
| Full AI agent | We serve agents, we don't become one. |
| Cloud inference | Local-first is a core principle. |
| IDE plugin (v1) | CLI-first. IDE integration comes via MCP/LSP later. |

---

## Execution Priority

```
Phase 1  Repositioning          Done
Phase 2  Structured Output      Done
Phase 3  Symbol Intelligence     Done
Phase 7  AI Agent Integration    Done
Phase 4  Context Engine          Done
Phase 5  Dependency Graph        Done
Phase 6  TUI Mode                Done
Phase 8  Performance             Done
Phase 9  Ecosystem               Done
```
