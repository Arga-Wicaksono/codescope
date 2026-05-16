#!/usr/bin/env bash
# Install CodeScope git hooks
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "Installing CodeScope pre-commit hook..."
cp "$SCRIPT_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-commit"
echo "Done! Pre-commit hook installed at $HOOKS_DIR/pre-commit"
