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

## Phase 2 — Structured Output (Priority: Critical)

**Timeline:** 1–2 weeks

Make every command produce stable, parseable, AI-consumable JSON.

### Goals
- **Stable JSON schemas** — Define output schemas for all commands
- **Machine-friendly first** — JSON is not an afterthought, it's the primary interface
- **Schema documentation** — Published schemas in `docs/schemas/`

### Deliverables
- [ ] Define JSON schema for `cs file` (file path, score, extension, size)
- [ ] Define JSON schema for `cs content` (file, line, column, context, match)
- [ ] Define JSON schema for `cs where` (symbol, file, line, kind, language)
- [ ] Define JSON schema for `cs stats` (language, files, lines, bytes)
- [ ] Define JSON schema for `cs across` (repo, file, line, match)
- [ ] Create `docs/schemas/` with JSON Schema files
- [ ] Add `--schema` flag to print JSON schema for any command
- [ ] Ensure zero-breaking-change contract (field names never change)

### Why This First?
AI agents need deterministic structure. If `cs content --json` returns `{"results": [...]}` today and `{"matches": [...]}` tomorrow, AI integrations break. Stable schemas = reliable AI partnerships.

---

## Phase 3 — Symbol Intelligence (Priority: High)

**Timeline:** 2–3 weeks

Repository understanding through code structure analysis.

### Approach
Use **Tree-sitter** for grammar-based symbol extraction. Supports 40+ languages out of the box.

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
- [ ] Add `tree-sitter` dependency
- [ ] Implement symbol parser for Rust, Python, JS/TS (MVP)
- [ ] `cs symbol <name>` — Find definition with metadata
- [ ] `cs refs <name>` — Find all references
- [ ] `cs callers <name>` — Find call graph (who calls this)
- [ ] `cs symbols [path]` — List all symbols in scope
- [ ] Add `--type` filter for symbol kinds (function, class, struct, etc.)
- [ ] JSON output (`-j`) for all symbol commands
- [ ] Expand to Go, Java, C/C++, Kotlin

### Why It Matters
This is what separates CodeScope from ripgrep. Symbol intelligence enables:
- **Architecture understanding** — See how modules connect
- **AI context building** — AI needs relationships, not just text matches
- **Refactoring safety** — Know every caller before changing a function

---

## Phase 4 — Context Engine (Priority: High)

**Timeline:** 2–4 weeks

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
- [ ] Context extraction engine (gathers files, symbols, dependencies)
- [ ] Ranking system (most referenced, recent, dependency weight)
- [ ] `cs context <topic>` — Multi-source context extraction
- [ ] `cs pack <description>` — LLM-optimized prompt packing
- [ ] Token counting and budget awareness
- [ ] Architecture-aware ranking (entry points rank higher)
- [ ] JSON output with ranking scores

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

## Phase 6 — TUI Mode (Priority: Medium)

**Timeline:** 3–5 weeks

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
- [ ] TUI app with ratatui
- [ ] File/symbol browser panel
- [ ] Live code preview panel
- [ ] AI context sidebar (relevance score, token estimate)
- [ ] Keyboard navigation (vim-style)
- [ ] Split view for diff/comparison

---

## Phase 7 — AI Agent Integration (Priority: High)

**Timeline:** 2 weeks

Make CodeScope consumable by any AI system.

### MCP Support (Model Context Protocol)
The #1 integration point. MCP lets Claude, Cursor, and any MCP-compatible agent use CodeScope as a tool.

```
AI Agent ←→ MCP Server ←→ CodeScope CLI
```

### Deliverables
- [ ] MCP server implementation
- [ ] Register CodeScope tools: `search`, `context`, `symbol`, `trace`, `pack`
- [ ] Claude Desktop integration tested
- [ ] Cursor integration tested
- [ ] CLI API: `cs serve` — HTTP/unix socket API
  - `GET /search?q=auth&type=rust`
  - `GET /context?q=authentication&tokens=4000`
  - `GET /symbol?name=UserService`
  - `GET /trace?symbol=login_user`
  - `GET /pack?description=auth+bug&budget=8000`
- [ ] `--llm` flag — Output optimized for token efficiency

### Why This Matters
Once MCP support lands, any AI agent can use CodeScope for repository understanding. This is the distribution flywheel — CodeScope becomes infrastructure that AI tools depend on.

---

## Phase 8 — Performance Excellence (Priority: Ongoing)

**Timeline:** Ongoing

Live up to the "blazing fast" promise with benchmarks.

### Deliverables
- [ ] Benchmark suite: `cs` vs `rg`, `grep`, `fzf`, `fd`
- [ ] Large repo testing (Linux kernel, Kubernetes, React, rustc)
- [ ] Incremental indexing for symbol data
- [ ] Memory profiling and optimization
- [ ] Parallel walk optimization
- [ ] Published benchmarks in README

---

## Phase 9 — Open Source Ecosystem (Priority: Low)

**Timeline:** Future

### Deliverables
- [ ] Plugin architecture (post-v2.0)
- [ ] Rust SDK library (`codescope-sdk`)
- [ ] Python SDK (`pip install codescope`)
- [ ] VS Code extension (via LSP bridge)
- [ ] Neovim plugin

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
Phase 1  Repositioning          ✅ Done
Phase 2  Structured Output      ← NEXT (1–2 weeks)
Phase 3  Symbol Intelligence     (2–3 weeks)
Phase 7  AI Agent Integration    (2 weeks) ↑ Elevated priority
Phase 4  Context Engine          (2–4 weeks)
Phase 5  Dependency Graph        (2–3 weeks)
Phase 6  TUI Mode                (3–5 weeks)
Phase 8  Performance             (ongoing)
Phase 9  Ecosystem               (future)
```

**Rationale for elevated Phase 7:** MCP support creates the distribution flywheel. Once Claude, Cursor, and other agents can use CodeScope natively, adoption grows through their user bases — not just our direct installs.
