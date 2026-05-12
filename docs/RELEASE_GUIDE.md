# Build & Upload GitHub Release — CodeScope

Panduan manual untuk build binary di berbagai platform dan upload ke GitHub Releases.

## Build dari Source

### 1. Clone dan build

```bash
git clone https://github.com/Arga-Wicaksono/codescope.git
cd codescope

# Full build (web search + interactive)
cargo build --release

# Binary ada di: target/release/cs
# Ukuran kira-kira: ~2 MB
```

### 2. Build untuk platform lain (cross-compile)

```bash
# Linux x86_64 (default)
cargo build --release --target x86_64-unknown-linux-gnu

# macOS Intel
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon (M1/M2/M3)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Windows (dari Linux, perlu cross toolchain)
cargo install cross
cross build --release --target x86_64-pc-windows-msvc
```

### 3. Build variants

```bash
# Full (default) — web search + interactive
cargo build --release

# Tanpa web search — lebih kecil
cargo build --release --no-default-features --features interactive

# Minimal — hanya file + content search
cargo build --release --no-default-features
```

## Upload ke GitHub Releases (Manual)

### Step 1: Rename binary

```bash
# Linux
cp target/release/cs cs-x86_64-linux

# macOS Intel
cp target/x86_64-apple-darwin/release/cs cs-x86_64-macos

# macOS Apple Silicon
cp target/aarch64-apple-darwin/release/cs cs-aarch64-macos

# Windows
cp target/x86_64-pc-windows-msvc/release/cs.exe cs-x86_64-windows.exe
```

### Step 2: Buat tag (jika belum ada)

```bash
git tag v1.0.0
git push origin v1.0.0
```

### Step 3: Buat Release via GitHub API

```bash
# Buat release
curl -X POST \
  -H "Authorization: token YOUR_GITHUB_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  https://api.github.com/repos/Arga-Wicaksono/codescope/releases \
  -d '{
    "tag_name": "v1.0.0",
    "name": "CodeScope v1.0.0",
    "body": "## What is CodeScope?\n\nA blazing fast CLI search tool — file, content, and web search in one binary.\n\n## Downloads\n\n| Platform | File |\n|----------|------|\n| Linux x86_64 | `cs-x86_64-linux` |\n| macOS Intel | `cs-x86_64-macos` |\n| macOS Apple Silicon | `cs-aarch64-macos` |\n| Windows | `cs-x86_64-windows.exe` |\n\n## Install\n\n```bash\n# Download and install\ncurl -sL https://github.com/Arga-Wicaksono/codescope/releases/download/v1.0.0/cs-x86_64-linux -o cs && chmod +x cs && sudo mv cs /usr/local/bin/\n\n# Or from source\ncargo install --git https://github.com/Arga-Wicaksono/codescope.git\n```\n\n## Changelog\n\nSee CHANGELOG.md for full details.",
    "draft": false,
    "prerelease": false
  }'

# Simpan release_id dari respons JSON
```

### Step 4: Upload binary ke release

```bash
# Upload setiap binary (ganti RELEASE_ID dari Step 3)
RELEASE_ID=YOUR_RELEASE_ID

curl -X POST \
  -H "Authorization: token YOUR_GITHUB_TOKEN" \
  -H "Content-Type: application/octet-stream" \
  https://uploads.github.com/repos/Arga-Wicaksono/codescope/releases/$RELEASE_ID/assets?name=cs-x86_64-linux \
  --data-binary @cs-x86_64-linux

curl -X POST \
  -H "Authorization: token YOUR_GITHUB_TOKEN" \
  -H "Content-Type: application/octet-stream" \
  https://uploads.github.com/repos/Arga-Wicaksono/codescope/releases/$RELEASE_ID/assets?name=cs-x86_64-macos \
  --data-binary @cs-x86_64-macos

curl -X POST \
  -H "Authorization: token YOUR_GITHUB_TOKEN" \
  -H "Content-Type: application/octet-stream" \
  https://uploads.github.com/repos/Arga-Wicaksono/codescope/releases/$RELEASE_ID/assets?name=cs-aarch64-macos \
  --data-binary @cs-aarch64-macos

curl -X POST \
  -H "Authorization: token YOUR_GITHUB_TOKEN" \
  -H "Content-Type: application/octet-stream" \
  https://uploads.github.com/repos/Arga-Wicaksono/codescope/releases/$RELEASE_ID/assets?name=cs-x86_64-windows.exe \
  --data-binary @cs-x86_64-windows.exe
```

### Alternatif: Upload via GitHub Web UI

1. Buka https://github.com/Arga-Wicaksono/codescope/releases
2. Klik **"Draft a new release"** atau **"Edit"** pada tag v1.0.0
3. Isi title: `CodeScope v1.0.0`
4. Isi description (copy dari atas)
5. Drag & drop file binary ke area **"Attach binaries by dropping them here"**
6. Klik **"Publish release"**

## Quick One-Liner (Upload Release via API)

```bash
TOKEN="YOUR_GITHUB_TOKEN"
REPO="Arga-Wicaksono/codescope"
TAG="v1.0.0"

# Buat release
RESPONSE=$(curl -s -X POST \
  -H "Authorization: token $TOKEN" \
  -H "Accept: application/vnd.github+json" \
  "https://api.github.com/repos/$REPO/releases" \
  -d "{\"tag_name\":\"$TAG\",\"name\":\"CodeScope $TAG\",\"body\":\"First stable release of CodeScope.\",\"draft\":false,\"prerelease\":false}")

RELEASE_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Upload semua binary
for FILE in cs-x86_64-linux cs-x86_64-macos cs-aarch64-macos cs-x86_64-windows.exe; do
  curl -s -X POST \
    -H "Authorization: token $TOKEN" \
    -H "Content-Type: application/octet-stream" \
    "https://uploads.github.com/repos/$REPO/releases/$RELEASE_ID/assets?name=$FILE" \
    --data-binary "@$FILE"
done

echo "Release uploaded! https://github.com/$REPO/releases/tag/$TAG"
```
