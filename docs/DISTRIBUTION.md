# Distribution Guide — CodeScope (cs)

## Overview

CodeScope is distributed through **7 channels**:

| Channel | Platform | Command |
|---------|----------|---------|
| GitHub Releases | All | Manual download |
| Install script | macOS / Linux | `curl ... \| bash` |
| Homebrew tap | macOS / Linux | `brew install codescope` |
| Scoop bucket | Windows | `scoop install codescope` |
| AUR | Arch Linux | `yay -S codescope` |
| crates.io (future) | All | `cargo install codescope` |
| npm | All | `npm install -g codescope` |

---

## Releasing a New Version

### 1. Update version in Cargo.toml

```bash
# Bump version
vim Cargo.toml  # Update version = "X.Y.Z"

# Update changelog
vim CHANGELOG.md

# Commit
git add -A && git commit -m "vX.Y.Z — description"
```

### 2. Build locally and test

```bash
cargo build --release
cargo test --lib
./target/release/cs --help
./target/release/cs file "main"
```

### 3. Tag and push

```bash
git tag vX.Y.Z
git push origin main --tags
```

### 4. GitHub Actions handles the rest

The `release.yml` workflow will:
- Build for 7 targets (Linux glibc/musl, macOS Intel/ARM, Windows)
- Generate sha256 checksums
- Generate shell completions
- Create GitHub Release with all artifacts

### 5. Update package managers

After the release is published:

#### Homebrew tap
```bash
# Clone the tap repo
git clone git@github.com:Arga-Wicaksono/homebrew-codescope.git /tmp/tap
cd /tmp/tap

# Get new sha256
curl -sL https://github.com/Arga-Wicaksono/codescope/releases/download/vX.Y.Z/cs-aarch64-macos.tar.gz | shasum -a 256
curl -sL https://github.com/Arga-Wicaksono/codescope/releases/download/vX.Y.Z/cs-x86_64-macos.tar.gz | shasum -a 256

# Update formula
vim Formula/codescope.rb  # Update version + sha256

git add -A && git commit -m "codescope vX.Y.Z" && git push
```

#### Scoop
```bash
# Update sha256 in distribution/scoop/codescope.json
# Get hash: curl -sL .../cs-x86_64-windows.zip | sha256sum
vim distribution/scoop/codescope.json
```

#### AUR
```bash
# Update pkgver and sha256 in distribution/aur/PKGBUILD
# x86_64: curl -sL .../cs-x86_64-linux.tar.gz | sha256sum
# aarch64: curl -sL .../cs-aarch64-linux.tar.gz | sha256sum
vim distribution/aur/PKGBUILD

# Build and test
cd /tmp && cp PKGBUILD . && makepkg -sf
```

#### npm
```bash
cd distribution/npm
# Update version in package.json
vim package.json
npm publish
```

---

## Pre-release Checklist

- [ ] Version bumped in `Cargo.toml`
- [ ] `CHANGELOG.md` updated
- [ ] `cargo test --lib` passes
- [ ] `cargo clippy --all-features` passes
- [ ] `cargo fmt` clean
- [ ] Binary smoke tested (`cs --help`, `cs file "test"`, `cs content "fn"`)
- [ ] README version badge updated
- [ ] Git tag created and pushed
- [ ] GitHub Release auto-created by Actions
- [ ] Homebrew formula sha256 updated
- [ ] Scoop manifest sha256 updated
- [ ] AUR PKGBUILD updated
- [ ] npm package updated (if applicable)

---

## File Structure

```
distribution/
├── homebrew/
│   └── codescope.rb          # Homebrew formula template
├── scoop/
│   └── codescope.json        # Scoop manifest template
├── aur/
│   └── PKGBUILD              # AUR package build file
└── npm/
    ├── package.json           # npm package metadata
    ├── install.js             # Post-install binary downloader
    ├── uninstall.js           # Pre-uninstall cleanup
    └── cs.js                  # Binary wrapper (bin entry point)

scripts/
├── install.sh                # One-liner installer
├── uninstall.sh              # Uninstaller
└── setup-homebrew-tap.sh     # One-time Homebrew tap setup
```
