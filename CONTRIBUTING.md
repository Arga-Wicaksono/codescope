# Contributing to CodeScope

Thank you for your interest in contributing! This guide covers everything you need to get started.

## Prerequisites

- **Rust 1.70+** — [Install Rust](https://rustup.rs/)
- **Git** — For cloning and branching
- A terminal (bash, zsh, fish, or powershell)

## Getting Started

### 1. Fork and clone

```bash
# Fork the repo on GitHub, then:
git clone https://github.com/YOUR_USERNAME/codescope.git
cd codescope
```

### 2. Build and test

```bash
# Build (default: web-search + interactive)
cargo build --release

# Run tests
cargo test

# Check formatting
cargo fmt -- --check

# Lint
cargo clippy -- -D warnings
```

### 3. Create a branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

## Development Workflow

### Code style

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` — zero warnings allowed
- All tests must pass: `cargo test`

### Writing tests

Add tests to the relevant module file. Tests use Rust's built-in `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        assert_eq!(my_function("input"), "expected");
    }
}
```

Run a specific test:

```bash
cargo test test_my_feature
```

### Commit messages

Use clear, descriptive commit messages:

```
feat(content_search): add multiline regex support
fix(file_search): handle symlinks correctly
docs: update README installation section
refactor: extract common matching logic
test: add edge case tests for --invert
```

## Project Structure

```
codescope/
├── src/
│   ├── main.rs          # Entry point, command dispatch
│   ├── lib.rs           # Module declarations
│   ├── cli.rs           # Argument parsing (clap)
│   ├── config.rs        # Config file handling
│   ├── file_search.rs   # cs file — fuzzy file finder
│   ├── content_search.rs # cs content — search inside files
│   ├── web_search.rs    # cs web — DuckDuckGo search
│   ├── open.rs          # cs open — find + open in editor
│   ├── recent.rs        # cs recent — recently modified files
│   ├── where_cmd.rs     # cs where — find definitions
│   ├── explain.rs       # cs explain — regex explainer
│   ├── history.rs       # cs history — search history
│   ├── across.rs        # cs across — cross-repo search
│   ├── stats.rs         # cs stats — file statistics
│   ├── interactive.rs   # Interactive fuzzy-select picker
│   ├── types.rs         # Shared types and enums
│   ├── utils.rs         # Shared utility functions
│   ├── validate.rs      # Input validation
│   └── output.rs        # Output formatting helpers
├── Cargo.toml
├── LICENSE              # MIT
├── CHANGELOG.md
├── CONTRIBUTING.md
└── README.md
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `web-search` | Yes | Enables `cs web` (reqwest + scraper) |
| `interactive` | Yes | Enables `-I` flag (dialoguer) |

Build without a feature:

```bash
cargo build --release --no-default-features
```

## Pull Request Process

1. **Update tests** if you add or change behavior
2. **Update documentation** (README, CHANGELOG, or inline docs) if needed
3. **Ensure CI passes** — `cargo test`, `cargo clippy`, `cargo fmt`
4. **Keep PRs focused** — one feature or fix per PR when possible
5. **Squash commits** for clean history

## Reporting Bugs

Use [GitHub Issues](https://github.com/Arga-Wicaksono/codescope/issues) with:

1. **OS and Rust version** (`rustc --version`)
2. **Command that failed** (full command with arguments)
3. **Expected vs actual behavior**
4. **Minimal reproduction** if possible

## Feature Requests

Open an issue with the `[Feature]` label describing:

1. **Use case** — What problem does this solve?
2. **Proposed command** — How would the API look? (`cs something ...`)
3. **Alternatives considered** — Existing tools or workarounds

## License

By contributing, you agree that your code will be licensed under the [MIT License](LICENSE).
