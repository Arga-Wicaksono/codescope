# Product Hunt Launch Guide — CodeScope

## Tagline
**CodeScope — Repository Intelligence Engine for AI & Developers**

## Short Description (60 chars max)
```
One CLI to understand any repo. Fast search, symbol intel, AI-ready context. Built in Rust.
```

## Topics/Tags
Developer Tools, Open Source, Productivity, CLI, Rust, AI Tools, Code Intelligence, Search, Terminal

## First Comment (Maker Comment)

```
Hey Product Hunt! 👋

I built CodeScope because understanding large codebases is the biggest bottleneck in software development — for both humans and AI agents.

The problem: developers chain fd, rg, ctags, and custom scripts. AI agents get random context and hallucinate. Neither approach is reliable.

CodeScope solves this differently — it's not an AI assistant or a chatbot. It's repository infrastructure. A single ~2 MB binary that makes any codebase instantly understandable through fast search, symbol intelligence, and AI-ready structured output.

What makes it different:
• Deterministic and blazing fast — Rust-native, same query always returns the same results
• AI-consumable — every command outputs structured JSON, perfect for LLM prompt packing
• Symbol intelligence — find function/class definitions across 7+ languages
• Cross-repo search — search across your entire workspace at once
• Zero dependencies — one binary, no runtime, no cloud, no API keys

Roadmap highlights:
• Tree-sitter symbol indexing (cs symbol, cs refs, cs callers)
• Context engine — intelligent code extraction for AI prompts
• MCP protocol support — Claude and Cursor can use CodeScope natively

Written in Rust. MIT license. Open source.

Try it: cargo install --git https://github.com/Arga-Wicaksono/codescope.git
Repo: https://github.com/Arga-Wicaksono/codescope
```

## Gallery
1. `assets/logo.png` — Logo/icon (1024x1024)
2. `assets/demo.gif` — Demo showing commands
3. Screenshots: bare `cs` (banner), `cs file`, `cs content -j`, `cs where`, `cs stats`

## Pricing
Free and Open Source (MIT)
