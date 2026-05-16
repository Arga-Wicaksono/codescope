# 📣 CodeScope Launch Playbook

> Strategi end-to-end untuk memperkenalkan CodeScope ke publik.
> Dari nol pengguna ke komunitas yang aktif.

---

## 🎯 Target Audience

| Segmen | Siapa mereka | Kenapa butuh cs |
|--------|-------------|-----------------|
| **AI/AI Agent Developers** | Developer yang memakai Claude, Cursor, Copilot | `cs pack`, `cs context` memberikan konteks kode terbaik untuk LLM |
| **Rustaceans** | Rust developers, CLI tool enthusiasts | Rust-native, cepat, zero deps — impressive technical showcase |
| **Open Source Maintainers** | Maintainer repo besar, contributor | `cs impact`, `cs graph`, `cs trace` memahami arsitektur cepat |
| **Terminal Power Users** | Pengguna Vim/Neovim, tmux, CLI-first workflow | Menggantikan fd+rg+fzf+ctags dengan satu binary |
| **Backend/Fullstack Devs** | Developer yang handle repo besar | `cs across`, `cs symbol`, `cs semantic` navigasi kode super cepat |

---

## 📅 Timeline Launch (2 Minggu)

### Minggu 1: Persiapan

- [ ] Pastikan CI hijau (semua test pass)
- [ ] Buat GitHub Release v1.3.0 (pertama di releases page)
- [ ] Buat asciinema demo (lebih baik dari GIF untuk terminal)
- [ ] Persiapkan semua draft konten (template di bawah)
- [ ] Buat Twitter/X account jika belum ada
- [ ] Setup GitHub Discussions

### Minggu 2: Launch

| Hari | Aksi | Platform |
|------|------|----------|
| **Senin** | Post ke r/rust + r/commandline | Reddit |
| **Selasa** | Post "Show HN" | Hacker News |
| **Rabu** | Launch | Product Hunt |
| **Kamis** | Publish article | Dev.to / Medium |
| **Jumat** | Thread viral | Twitter/X |
| **Sabtu** | Submit ke directories | Rust newsletter, awesome-lists |
| **Minggu** | Engage: balas komentar, fix issues | Semua |

---

## 📝 Konten Siap Pakai

Lihat file terpisah untuk masing-masing platform:
- `docs/promo/reddit-rust.md` — Posting ke r/rust
- `docs/promo/reddit-commandline.md` — Posting ke r/commandline
- `docs/promo/hacker-news.md` — Show HN post
- `docs/promo/product-hunt.md` — Product Hunt listing
- `docs/promo/devto-article.md` — Dev.to / Medium article
- `docs/promo/twitter-thread.md` — Twitter/X thread
- `docs/promo/newsletter.md` — Rust newsletter submission
- `docs/promo/awesome-lists.md` — Awesome list PR templates

---

## 💡 Tips Kunci

### 1. JANGAN jual fitur — jual masalah yang terselesaikan
❌ "CodeScope has 28 commands and 10 language support"
✅ "I was tired of chaining fd + rg + ctags to understand a repo. So I built one tool."

### 2. Demo > Deskripsi
- Asciinema recording lebih baik dari screenshot
- Tunjukkan real workflow, bukan help text
- Before/after: "Sebelum: 5 tools. Sesudah: 1 binary."

### 3. Jawab setiap komentar
- Di Reddit, jawab dalam 30 menit pertama (kritis untuk visibility)
- Di HN, siapkan argumen teknis (mereka akan tanya architecture)
- Di PH, balas review dengan detail

### 4. "Built in public" narrative
- Developer Indonesia yang bikin tool global → angle unik
- Journey: dari masalah pribadi → open source → komunitas
- Tunjukkan progress (changelog, metrics, contributions)

### 5. Ikut komunitas dulu, baru promosi
- Don't spam — participate in discussions first
- Help others with CLI/Rust questions
- Then naturally mention CodeScope when relevant

---

## 📊 Metrics untuk Track

| Metric | Target (1 bulan) | Target (3 bulan) |
|--------|-------------------|-------------------|
| GitHub Stars | 100 | 500 |
| GitHub Releases downloads | 500 | 2,000 |
| npm installs/week | 50 | 200 |
| Discord/members | 20 | 100 |
| Contributors | 3 | 10 |

Track dengan: `gh api repos/Arga-Wicaksono/codescope` (stars, forks, watchers)
