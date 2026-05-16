#!/bin/bash
# CodeScope (cs) — Demo Script for asciinema recording
# Usage: asciinema rec -c "bash scripts/demo.sh" demo.cast
#
set -euo pipefail

# ═══ Configuration ═════════════════════════════════════════════════════
export PS1="\[\e[32m\]❯ \[\e[0m\]"
SLEEP=1.5

# Try to cd to a real project; fall back to codescope itself
if [[ -d "${1:-}" ]]; then
    cd "$1"
elif [[ -d "$HOME/projects" ]] && ls "$HOME/projects"/*/Cargo.toml 2>/dev/null | head -1 | grep -q .; then
    REPO=$(ls -d "$HOME/projects"/*/Cargo.toml 2>/dev/null | head -1 | xargs dirname)
    cd "$REPO"
fi

echo ""
echo "Recording CodeScope demo in: $(pwd)"
echo "Press Ctrl+C to stop recording"
sleep 2

# ═══ Scene 1: Banner ════════════════════════════════════════════════
cs
sleep "$SLEEP"

# ═══ Scene 2: File Search ═══════════════════════════════════════════
echo ""; sleep 1
echo "─── File Search ───"; sleep 1
cs file "config" -l 5
sleep "$SLEEP"

# ═══ Scene 3: Content Search ════════════════════════════════════════
echo ""; sleep 1
echo "─── Content Search ───"; sleep 1
cs content "fn " -l 5 -n
sleep "$SLEEP"

echo ""; sleep 1
cs content "TODO|FIXME" --regex --count 2>/dev/null || echo "(no matches)"
sleep "$SLEEP"

# ═══ Scene 4: Symbol Intelligence ═══════════════════════════════════
echo ""; sleep 1
echo "─── Symbol Intelligence ───"; sleep 1
cs where "main" -l 3 2>/dev/null || cs where "parse" -l 3 2>/dev/null || echo "(no symbols found)"
sleep "$SLEEP"

echo ""; sleep 1
cs symbols . --symbol-type function -l 5 2>/dev/null || echo "(listing symbols...)"
sleep "$SLEEP"

# ═══ Scene 5: Context Engine ════════════════════════════════════════
echo ""; sleep 1
echo "─── Context Engine ───"; sleep 1
cs context "config" -l 5 2>/dev/null || echo "(extracting context...)"
sleep "$SLEEP"

# ═══ Scene 6: Dependency Graph ══════════════════════════════════════
echo ""; sleep 1
echo "─── Dependency Graph ───"; sleep 1
cs graph --type modules -d 2 2>/dev/null || cs graph -d 2 2>/dev/null || echo "(building graph...)"
sleep "$SLEEP"

# ═══ Scene 7: Stats ═════════════════════════════════════════════════
echo ""; sleep 1
echo "─── Repository Stats ───"; sleep 1
cs stats
sleep "$SLEEP"

# ═══ Scene 8: JSON Output ═══════════════════════════════════════════
echo ""; sleep 1
echo "─── JSON Output (AI-ready) ───"; sleep 1
cs file "src" -l 3 -j 2>/dev/null | head -20 || echo "(JSON output...)"
sleep "$SLEEP"

# ═══ Scene 9: End ══════════════════════════════════════════════════
echo ""; sleep 1
echo "══════════════════════════════════════"
echo "  CodeScope — Repository Intelligence Engine"
echo "  github.com/Arga-Wicaksono/codescope"
echo "══════════════════════════════════════"
sleep 3
