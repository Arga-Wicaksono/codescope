# CodeScope Python SDK

Python interface for [CodeScope](https://github.com/Arga-Wicaksono/codescope) — Repository Intelligence Engine.

## Installation

```bash
# Requires cs binary installed (see https://github.com/Arga-Wicaksono/codescope)
pip install .
```

## Quick Start

```python
from codescope import CodeScope

cs = CodeScope("/path/to/repo")

# Search files
files = cs.search_files("config")
for f in files:
    print(f"{f.path} (score: {f.score})")

# Search content
results = cs.search_content("fn main", extensions=["rs"])

# Find symbols
symbols = cs.find_symbol("authenticate", kind="function")

# Repository stats
stats = cs.stats()
print(f"{stats.total_files} files, {stats.total_lines} lines")

# Dependency graph
graph = cs.dependency_graph()
print(graph.to_dot())  # Graphviz DOT format

# Impact analysis
impact = cs.impact("auth.rs")
print(f"Modifying auth.rs affects {impact.total_affected} files")

# Context extraction (for AI prompts)
context = cs.get_context("authentication")
packed = cs.pack_context("login flow", budget=4000)
```

## API Reference

### CodeScope(repo_path, cs_binary=None)

Main entry point.

### Methods

| Method | Description |
|--------|-------------|
| `search_files(pattern)` | Fuzzy file search |
| `search_content(pattern)` | Content search with filters |
| `find_symbol(name)` | Find symbol definitions |
| `find_references(name)` | Find symbol references |
| `find_callers(name)` | Find function callers |
| `list_symbols()` | List all symbols |
| `stats()` | Repository statistics |
| `dependency_graph()` | Build dependency graph |
| `impact(target)` | Impact analysis |
| `get_context(topic)` | Extract context |
| `pack_context(desc)` | Pack context for LLM |
| `trace(symbol)` | Trace execution flow |

## License

MIT
