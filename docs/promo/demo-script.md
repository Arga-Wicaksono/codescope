# 🎮 Asciinema Demo Script — Untuk Semua Platform

## Setup

```bash
# Install asciinema
cargo install asciinema
# or: brew install asciinema
# or: pip install asciinema

# Record
asciinema rec demo.cast

# Play
asciinema play demo.cast

# Upload (dapatkan link untuk share)
asciinema upload demo.cast
```

---

## Demo Script (60-90 detik)

Setup: gunakan terminal dengan font monospace, ukuran jendela 120x30, dark theme.

```bash
# ─── Scene 1: Introduction (10s) ──────────────────────────────────
cs
# (Tampilkan banner + command reference)

# ─── Scene 2: File Search (8s) ────────────────────────────────────
cs file "config" -I
# (Pilih file dari interactive picker)

# ─── Scene 3: Content Search (8s) ─────────────────────────────────
cs content "fn main" -n
cs content 'TODO|FIXME' --regex --count

# ─── Scene 4: Symbol Intelligence (10s) ───────────────────────────
cs where "parse_config"
cs refs "Config"
cs callers "process_data"

# ─── Scene 5: Context Engine for AI (12s) ─────────────────────────
cs context "authentication"
cs pack "auth flow" -b 4000

# ─── Scene 6: Dependency Graph (8s) ───────────────────────────────
cs graph --type modules
cs impact "utils.rs"

# ─── Scene 7: Cross-repo Search (8s) ──────────────────────────────
cs across 'TODO' --workspace ~/projects -l 5

# ─── Scene 8: JSON Output for Scripting (6s) ──────────────────────
cs file "config" -l 3 -j | python3 -m json.tool

# ─── Scene 9: End (5s) ────────────────────────────────────────────
cs --help | head -20
echo "github.com/Arga-Wicaksono/codescope"
```

---

## Tips Asciinema

1. **Kecepatan**: Jangan terlalu cepat, viewer perlu membaca output
2. **Pause**: Tambahkan pause antar scene (gunakan `sleep 1` di antara perintah)
3. **Kesalahan**: Jika salah ketik, stop dan rekam ulang — jangan biarkan typo
4. **Output panjang**: Pipe ke `head -20` jika output terlalu panjang
5. **Highlight**: Gunakan `cs` di repo yang relatif besar supaya hasil terlihat impresif

## Script Otomatis (tanpa interaktif)

```bash
#!/bin/bash
# Demo script yang bisa direkam dengan asciinema
export PS1="\[\e[32m\]❯ \[\e[0m\]"
cd ~/projects/some-real-repo  # Gunakan repo yang besar!

echo ""; sleep 1
cs; sleep 2

echo "── File Search ──"; sleep 1
cs file "config" -l 5; sleep 2

echo "── Content Search ──"; sleep 1
cs content "fn main" -n -l 3; sleep 2

echo "── Symbol Intelligence ──"; sleep 1
cs where "main"; sleep 2
cs refs "Config" -l 3; sleep 2

echo "── Context Engine ──"; sleep 1
cs context "error" -l 5; sleep 2

echo "── Dependency Graph ──"; sleep 1
cs graph --type modules -d 2; sleep 2

echo "── JSON Output ──"; sleep 1
cs file "src" -l 3 -j; sleep 2

echo ""; sleep 1
echo "✨ github.com/Arga-Wicaksono/codescope"; sleep 3
```

Simpan sebagai `scripts/demo.sh`, lalu: `asciinema rec -c "bash scripts/demo.sh" demo.cast`
