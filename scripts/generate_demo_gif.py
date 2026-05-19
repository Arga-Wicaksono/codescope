#!/usr/bin/env python3
"""
CodeScope Demo GIF Generator — all 32 cargo tests.
Renders a scrolling terminal with Tokyo Night theme using Pillow (fast, no ffmpeg needed).
"""
import time
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

# ── Config ──────────────────────────────────────────────────────────────────
OUTPUT = Path(__file__).parent.parent / "assets" / "demo.gif"
WIDTH, HEADER_H = 1100, 38
PAD_X, PAD_Y = 18, 14
LINE_H = 19
BODY_VIEW_H = 720
GIF_H = HEADER_H + BODY_VIEW_H

# ── Colors (Tokyo Night) ───────────────────────────────────────────────────
BG       = (26, 27, 38)
HDR_BG   = (22, 22, 30)
DOT_R    = (247, 118, 142)
DOT_Y    = (224, 175, 104)
DOT_G    = (158, 206, 106)
TITLE_C  = (122, 162, 247)
PROMPT_C = (158, 206, 106)
CMD_C    = (192, 202, 245)
DIM_C    = (86, 95, 137)
OK_C     = (158, 206, 106)
W_C      = (192, 202, 245)

# ── Fonts ───────────────────────────────────────────────────────────────────
fn   = ImageFont.truetype("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf", 14)
fn_b = ImageFont.truetype("/usr/share/fonts/truetype/liberation/LiberationMono-Bold.ttf", 15)
fn_s = ImageFont.truetype("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf", 12)

# ── All 32 Tests ────────────────────────────────────────────────────────────
TESTS = [
    ("file_search", "test_search_files_basic"),
    ("file_search", "test_search_files_with_extension"),
    ("file_search", "test_search_files_no_results"),
    ("file_search", "test_search_files_empty_pattern"),
    ("file_search", "test_search_files_json_output"),
    ("history", "test_chrono_timestamp"),
    ("where_cmd", "test_where_rust_definition"),
    ("where_cmd", "test_where_not_found"),
    ("content_search", "test_search_content_fuzzy"),
    ("content_search", "test_search_content_exact"),
    ("content_search", "test_search_content_regex"),
    ("content_search", "test_search_content_no_results"),
    ("content_search", "test_search_content_with_extension"),
    ("content_search", "test_resolve_match_mode"),
    ("content_search", "test_search_content_count"),
    ("content_search", "test_search_content_invert"),
    ("content_search", "test_search_content_empty_pattern"),
    ("config", "test_default_config"),
    ("config", "test_config_roundtrip"),
    ("explain", "test_explain_basic"),
    ("explain", "test_explain_empty"),
    ("explain", "test_explain_character_class"),
    ("across", "test_resolve_repos_from_list"),
    ("across", "test_resolve_repos_workspace"),
    ("stats", "test_stats_basic"),
    ("stats", "test_detect_language"),
    ("web_search", "test_web_search_invalid_query"),
    ("recent", "test_recent_basic"),
    ("recent", "test_parse_relative_time"),
    ("validate", "test_validate_pattern"),
    ("utils", "test_resolve_case_insensitive_smart"),
    ("utils", "test_truncate_str"),
]
N = len(TESTS)


def main():
    t0 = time.time()

    # ── Build content lines ─────────────────────────────────────────────────
    content = [
        ("codescope $ cargo test", CMD_C, fn_b),
        ("", W_C, fn),
        ("   Compiling codescope v1.3.0", DIM_C, fn),
        ("    Finished test [unoptimized + debuginfo] target(s) in 0.42s", DIM_C, fn),
        ("     Running unittests src/lib.rs (target/debug/deps/codescope-38a7f2a5)", DIM_C, fn),
        ("", W_C, fn),
    ]
    for mod, test_name in TESTS:
        content.append((f"     test {mod}::{test_name} ... ok", OK_C, fn))
    content += [
        ("", W_C, fn),
        (f"     test result: ok. {N} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out", OK_C, fn_b),
        ("", W_C, fn),
        ("   Doc-tests codescope", DIM_C, fn),
        ("", W_C, fn),
        ("     test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out", OK_C, fn_b),
        ("", W_C, fn),
        ("codescope $ ", PROMPT_C, fn_b),
    ]

    # ── Render body-only tall image ─────────────────────────────────────────
    body_h = PAD_Y * 2 + len(content) * LINE_H + LINE_H
    max_scroll = max(0, body_h - BODY_VIEW_H)

    body = Image.new("RGB", (WIDTH, body_h), BG)
    d = ImageDraw.Draw(body)
    for i, (txt, col, f) in enumerate(content):
        d.text((PAD_X, PAD_Y + i * LINE_H), txt, fill=col, font=f)

    # ── Pre-render header strip ─────────────────────────────────────────────
    header = Image.new("RGB", (WIDTH, HEADER_H), HDR_BG)
    hd = ImageDraw.Draw(header)
    for i, c in enumerate([DOT_R, DOT_Y, DOT_G]):
        cx, cy = 16 + i * 20, HEADER_H // 2
        hd.ellipse([cx - 6, cy - 6, cx + 6, cy + 6], fill=c)
    hd.text((82, HEADER_H // 2 - 8), "codescope ~ cargo test", fill=TITLE_C, font=fn_s)

    def make_frame(scroll_y):
        """Composite header + cropped body into a single frame."""
        frame = Image.new("RGB", (WIDTH, GIF_H), BG)
        frame.paste(header, (0, 0))
        crop_h = min(BODY_VIEW_H, body_h - scroll_y)
        frame.paste(body.crop((0, scroll_y, WIDTH, scroll_y + crop_h)), (0, HEADER_H))
        return frame

    # ── Build frame sequence: hold → scroll → hold ─────────────────────────
    scrolls = sorted(set(range(0, max_scroll + 1, 12)))

    frames = []
    durations = []

    # Hold at top (show command + compile + first tests) — 2.1s
    for _ in range(6):
        frames.append(make_frame(0))
        durations.append(350)

    # Progressive scroll through all tests — ~3s
    for s in scrolls[1:]:
        frames.append(make_frame(s))
        durations.append(180)

    # Hold at bottom (show test result summary) — 4.9s
    for _ in range(14):
        frames.append(make_frame(max_scroll))
        durations.append(350)

    # ── Save animated GIF ───────────────────────────────────────────────────
    if OUTPUT.exists():
        OUTPUT.unlink()

    frames[0].save(
        OUTPUT,
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        palette=Image.ADAPTIVE,
        colors=256,
    )

    kb = OUTPUT.stat().st_size / 1024
    total_dur = sum(durations) / 1000
    print(f"Generated: {OUTPUT}")
    print(f"  Resolution: {WIDTH}x{GIF_H}")
    print(f"  Frames: {len(frames)}, Duration: ~{total_dur:.1f}s")
    print(f"  File size: {kb:.0f} KB")
    print(f"  Time: {time.time() - t0:.1f}s")


if __name__ == "__main__":
    main()
