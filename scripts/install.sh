#!/usr/bin/env bash
#
# CodeScope (cs) — Quick installer
# Usage: curl -sSL https://raw.githubusercontent.com/Arga-Wicaksono/codescope/main/scripts/install.sh | bash
#        or:  curl -sSL ... | bash -s -- --version 1.3.0
#        or:  curl -sSL ... | bash -s -- --prefix ~/.local
#
set -euo pipefail

# ─── Config ──────────────────────────────────────────────────────────────
REPO="Arga-Wicaksono/codescope"
BINARY="cs"
INSTALL_PREFIX="${PREFIX:-/usr/local/bin}"
VERSION="${CS_VERSION:-latest}"

# ─── Colors ─────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[info]${NC} $*"; }
ok()    { echo -e "${GREEN}[ok]${NC} $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $*"; }
err()   { echo -e "${RED}[error]${NC} $*" >&2; }

# ─── Parse arguments ─────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --version|-v) VERSION="$2"; shift 2 ;;
        --prefix|-p)  INSTALL_PREFIX="$2"; shift 2 ;;
        --help|-h)
            echo "CodeScope (cs) installer"
            echo ""
            echo "Usage: install.sh [options]"
            echo ""
            echo "Options:"
            echo "  --version, -v VERSION  Install specific version (default: latest)"
            echo "  --prefix, -p PREFIX    Install prefix (default: /usr/local/bin)"
            echo "  --help, -h             Show this help"
            exit 0
            ;;
        *) err "Unknown option: $1"; exit 1 ;;
    esac
done

# ─── Detect platform ────────────────────────────────────────────────────
detect_platform() {
    local os arch suffix
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$arch" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        *) err "Unsupported architecture: $arch"; exit 1 ;;
    esac

    case "$os" in
        linux)
            # Prefer musl for static binary (works on all distros)
            if command -v ldd &>/dev/null && ldd --version 2>&1 | grep -qi musl; then
                suffix="linux-musl"
            else
                suffix="linux"
            fi
            ;;
        darwin) suffix="macos" ;;
        *) err "Unsupported OS: $os"; exit 1 ;;
    esac

    echo "${BINARY}-${arch}-${suffix}"
}

# ─── Get latest version ─────────────────────────────────────────────────
get_version() {
    if [[ "$VERSION" == "latest" ]]; then
        info "Fetching latest version..."
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        if [[ -z "$VERSION" ]]; then
            err "Failed to fetch latest version"
            exit 1
        fi
    fi
    # Strip 'v' prefix if present
    VERSION="${VERSION#v}"
    echo "$VERSION"
}

# ─── Main ───────────────────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║     CodeScope (cs) Installer         ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
    echo ""

    # Check dependencies
    for cmd in curl tar; do
        if ! command -v "$cmd" &>/dev/null; then
            err "Required command not found: $cmd"
            exit 1
        fi
    done

    # Get version
    local ver
    ver="$(get_version)"
    info "Installing CodeScope v${ver}"

    # Detect platform
    local asset_name
    asset_name="$(detect_platform)"
    info "Detected platform: ${asset_name}"

    # Construct download URL
    local download_url="https://github.com/${REPO}/releases/download/v${ver}/${asset_name}.tar.gz"
    local checksum_url="${download_url}.sha256"

    # Create temp dir
    local tmp_dir
    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    # Download
    info "Downloading from GitHub..."
    if ! curl -fsSL -o "${tmp_dir}/${asset_name}.tar.gz" "$download_url"; then
        err "Download failed. Check if version v${ver} exists at:"
        err "  https://github.com/${REPO}/releases"
        exit 1
    fi

    # Verify checksum if available
    info "Verifying checksum..."
    if curl -fsSL -o "${tmp_dir}/${asset_name}.tar.gz.sha256" "$checksum_url" 2>/dev/null; then
        (
            cd "$tmp_dir"
            if sha256sum -c "${asset_name}.tar.gz.sha256" --quiet 2>/dev/null; then
                ok "Checksum verified"
            else
                warn "Checksum verification failed — continuing anyway"
            fi
        )
    else
        warn "Checksum file not found — skipping verification"
    fi

    # Extract
    info "Extracting..."
    tar xzf "${tmp_dir}/${asset_name}.tar.gz" -C "${tmp_dir}"

    # Install
    local install_dir
    install_dir="$(dirname "$INSTALL_PREFIX")/$(basename "$INSTALL_PREFIX")"
    
    if [[ ! -d "$install_dir" ]]; then
        info "Creating directory: ${install_dir}"
        mkdir -p "$install_dir"
    fi

    if [[ -w "$install_dir" ]]; then
        cp "${tmp_dir}/${BINARY}" "${INSTALL_PREFIX}/${BINARY}"
        chmod +x "${INSTALL_PREFIX}/${BINARY}"
    else
        warn "No write access to ${install_dir} — using sudo"
        sudo cp "${tmp_dir}/${BINARY}" "${INSTALL_PREFIX}/${BINARY}"
        sudo chmod +x "${INSTALL_PREFIX}/${BINARY}"
    fi

    # Verify
    if command -v "${INSTALL_PREFIX}/${BINARY}" &>/dev/null; then
        echo ""
        ok "CodeScope v${ver} installed successfully!"
        echo ""
        info "Binary: ${INSTALL_PREFIX}/${BINARY}"
        echo ""
        info "Quick start:"
        echo -e "  ${GREEN}cs --help${NC}           Show all commands"
        echo -e "  ${GREEN}cs file \"pattern\"${NC}  Find files"
        echo -e "  ${GREEN}cs content \"text\"${NC}   Search content"
        echo -e "  ${GREEN}cs where \"fn_name\"${NC}  Find definitions"
        echo ""
        info "Shell completions (optional):"
        echo -e "  ${CYAN}cs completions bash${NC}  > /etc/bash_completion.d/cs"
        echo -e "  ${CYAN}cs completions zsh${NC}   > ~/.zfunc/_cs"
        echo -e "  ${CYAN}cs completions fish${NC}  > ~/.config/fish/completions/cs.fish"
        echo ""
    else
        err "Installation failed — binary not found at ${INSTALL_PREFIX}/${BINARY}"
        exit 1
    fi
}

main
