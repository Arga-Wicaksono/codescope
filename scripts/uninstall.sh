#!/usr/bin/env bash
# CodeScope (cs) — Uninstaller
# Usage: curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/uninstall.sh | bash
set -euo pipefail

BINARY="cs"
PREFIX="${PREFIX:-/usr/local/bin}"

echo ""
echo -e "\033[0;36m[info]\033[0m Uninstalling CodeScope..."

# Remove binary
if [[ -f "${PREFIX}/${BINARY}" ]]; then
    if [[ -w "${PREFIX}" ]]; then
        rm "${PREFIX}/${BINARY}"
    else
        sudo rm "${PREFIX}/${BINARY}"
    fi
    echo -e "\033[0;32m[ok]\033[0m Removed ${PREFIX}/${BINARY}"
else
    echo -e "\033[1;33m[warn]\033[0m Binary not found at ${PREFIX}/${BINARY}"
fi

# Remove config
if [[ -d ~/.codescope ]]; then
    read -rp "Remove config directory ~/.codescope? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf ~/.codescope
        echo -e "\033[0;32m[ok]\033[0m Removed ~/.codescope"
    fi
fi

# Remove history
if [[ -f ~/.codescope_history.json ]]; then
    rm ~/.codescope_history.json
    echo -e "\033[0;32m[ok]\033[0m Removed history file"
fi

# Remove cache
if [[ -d ~/.codescope/cache ]]; then
    rm -rf ~/.codescope/cache
    echo -e "\033[0;32m[ok]\033[0m Removed cache directory"
fi

echo ""
echo -e "\033[0;32m[ok]\033[0m CodeScope uninstalled successfully"
echo ""
