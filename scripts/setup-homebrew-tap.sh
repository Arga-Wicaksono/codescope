#!/usr/bin/env bash
# ─── Create the Homebrew Tap Repository ─────────────────────────────────
# Run this ONCE to set up the tap repo at github.com/Arga-Wicaksono/homebrew-codescope
# After that, users can: brew tap arga-wicaksono/codescope && brew install codescope
#
set -euo pipefail

TAP_REPO="homebrew-codescope"
FORMULA_DIR="$(dirname "$0")/../distribution/homebrew"
FORMULA_SOURCE="${FORMULA_DIR}/codescope.rb"

echo "=== CodeScope Homebrew Tap Setup ==="
echo ""
echo "This script will:"
echo "  1. Create a new GitHub repo: Arga-Wicaksono/${TAP_REPO}"
echo "  2. Copy the formula file"
echo "  3. Push to GitHub"
echo ""
read -rp "Continue? [y/N] " -n 1 -r
echo
[[ ! $REPLY =~ ^[Yy]$ ]] && exit 0

# Create temp dir for tap repo
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT
cd "$TMPDIR"

# Initialize repo
git init
git checkout -b main

# Copy formula
mkdir -p Formula
cp "$FORMULA_SOURCE" "Formula/codescope.rb"

# Create README
cat > README.md << 'EOF'
# CodeScope Homebrew Tap

Install [CodeScope](https://github.com/Arga-Wicaksono/codescope) via Homebrew.

```bash
brew tap Arga-Wicaksono/codescope
brew install codescope
```

## Upgrade

```bash
brew upgrade codescope
```
EOF

git add -A
git commit -m "Initial tap: add codescope formula"

# Create remote and push
gh repo create "Arga-Wicaksono/${TAP_REPO}" --public --description "Homebrew tap for CodeScope (cs) — Repository Intelligence Engine" --source=. --push

echo ""
echo "=== Done! ==="
echo ""
echo "Users can now install with:"
echo "  brew tap Arga-Wicaksono/codescope"
echo "  brew install codescope"
echo ""
echo "To update the formula after a new release:"
echo "  1. Update version and sha256 in distribution/homebrew/codescope.rb"
echo "  2. cd /tmp && git clone git@github.com:Arga-Wicaksono/${TAP_REPO}.git"
echo "  3. cp distribution/homebrew/codescope.rb ${TAP_REPO}/Formula/"
echo "  4. cd ${TAP_REPO} && git add -A && git commit -m 'codescope vX.Y.Z' && git push"
