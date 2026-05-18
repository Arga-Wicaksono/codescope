#!/usr/bin/env python3
"""Generate CodeScope asciinema cast file with per-line frames for clear animation."""

import json, time, sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "/home/z/my-project/codescope/demo.cast"
W, H = 80, 32
t = [0.0]  # mutable time tracker

def ts():
    return t[0]

def advance(ms):
    t[0] += ms / 1000.0

def wait(ms):
    return [ts() + ms/1000.0, "o", ""]

def out(text, delay_after_ms=0):
    """Output text at current time, then advance."""
    frames.append([ts(), "o", text])
    if delay_after_ms:
        advance(delay_after_ms)

def prompt(cmd):
    """Show command prompt then type command."""
    out("$ " + cmd + "\r\n", 600)

frames = []

# ═══ ANSI helpers ═══
CY = "\033[36m"
GR = "\033[33m"
GN = "\033[32m"
BL = "\033[34m"
MG = "\033[35m"
BD = "\033[1m"
DM = "\033[2m"
RS = "\033[0m"

# ═══ Scene 1: Banner ═══
advance(300)
banner_lines = [
    CY + "     ____                 __",
    "    / __/___  ____  _____/ /_",
    "   / /_/ __ \\/ __ \\/ ___/ __/",
    "  / __/ /_/ / / / / /__/ /_",
    " /_/  \\____/_/ /_/\\___/\\__/",
    "",
    "  cs v1.3.0 — Repository Intelligence Engine",
    "  28 commands. 10 languages. One binary.",
    "",
    "  Quick start:",
    "    cs file \"pattern\"        Find files",
    "    cs content \"text\"        Search content",
    "    cs where \"fn_name\"       Find definitions",
    "    cs context \"topic\"       Extract AI context",
    "    cs graph --type modules  Dependency graph",
    RS,
]
for i, line in enumerate(banner_lines):
    out(line + "\n", 80 if i < len(banner_lines) - 1 else 800)

advance(400)

# ═══ Scene 2: File Search ═══
prompt('cs file "config"')
advance(500)
lines = [
    "  " + GR + "src/config.rs" + RS + "          " + DM + "(98%)" + RS,
    "  " + GR + "src/cli.rs" + RS + "            " + DM + "(85%)" + RS,
    "  " + GR + "src/lib.rs" + RS + "            " + DM + "(72%)" + RS,
    "  " + GR + "tests/config_test.rs" + RS + "  " + DM + "(68%)" + RS,
    "  " + GR + "docs/config.md" + RS + "        " + DM + "(65%)" + RS,
    "",
    DM + "  5 results in 12ms" + RS,
]
for i, line in enumerate(lines):
    out(line + "\n", 200)
advance(500)

# ═══ Scene 3: Content Search ═══
prompt('cs content "fn main" -n')
advance(500)
lines = [
    MG + "src/main.rs" + RS + ":",
    "  " + GN + "15" + RS + "  fn main() {",
    "  " + GN + "16" + RS + "      let args = Cli::parse();",
    "  " + GN + "17" + RS + "      match args.command {",
    "",
    MG + "src/bin/cs.rs" + RS + ":",
    "  " + GN + " 3" + RS + "  fn main() -> Result<()> {",
    "  " + GN + " 4" + RS + "      codescope::run()",
    "",
    DM + "  2 files, 2 matches in 8ms" + RS,
]
for line in lines:
    out(line + "\n", 180)
advance(500)

# ═══ Scene 4: Symbol Intelligence ═══
prompt('cs where "parse_config"')
advance(500)
lines = [
    "  " + GR + BD + "src/config.rs:42" + RS + "  " + DM + "function" + RS + "  parse_config(path: &Path) -> Result<Config>",
    "  " + GR + BD + "src/config.rs:78" + RS + "  " + DM + "function" + RS + "  parse_config_file(content: &str) -> Result<Value>",
    "",
    DM + "  2 definitions in 5ms" + RS,
]
for line in lines:
    out(line + "\n", 250)
advance(500)

# ═══ Scene 5: References ═══
prompt('cs refs "Config"')
advance(500)
lines = [
    MG + "src/main.rs" + RS + ":",
    "  " + GN + "16" + RS + "      let " + BD + "Config" + RS + " = load_config()?;",
    "  " + GN + "45" + RS + "      app." + BD + "Config" + RS + "(&cfg);",
    "",
    MG + "src/config.rs" + RS + ":",
    "  " + GN + "12" + RS + "  struct " + BD + "Config" + RS + " {",
    "  " + GN + "18" + RS + "  impl " + BD + "Config" + RS + " {",
    "",
    DM + "  4 references in 6ms" + RS,
]
for line in lines:
    out(line + "\n", 200)
advance(500)

# ═══ Scene 6: Context Engine ═══
prompt('cs context "authentication"')
advance(500)
lines = [
    "  " + GR + BD + "[FILE] " + RS + "src/auth/handler.rs        " + DM + "score: 95" + RS + "  " + DM + "tokens: 1,247" + RS,
    "  " + GR + BD + "[FILE] " + RS + "src/auth/middleware.rs      " + DM + "score: 87" + RS + "  " + DM + "tokens: 892" + RS,
    "  " + GR + BD + "[FILE] " + RS + "src/auth/token.rs           " + DM + "score: 82" + RS + "  " + DM + "tokens: 654" + RS,
    "  " + GR + BD + "[SYMB] " + RS + "src/auth/handler.rs:23     " + DM + "score: 91" + RS + "  fn authenticate()",
    "  " + GR + BD + "[CONT] " + RS + "src/api/routes.rs:45       " + DM + "score: 74" + RS + "  auth guard middleware",
    "",
    DM + "  5 results in 14ms" + RS,
]
for line in lines:
    out(line + "\n", 220)
advance(500)

# ═══ Scene 7: Dependency Graph ═══
prompt('cs graph --type modules')
advance(500)
lines = [
    CY + "cs" + RS + " (root)",
    CY + "+-- cli",
    CY + "|   +-- config",
    CY + "+-- file_search",
    CY + "|   +-- utils",
    CY + "|   +-- output",
    CY + "+-- content_search",
    CY + "|   +-- utils",
    CY + "|   +-- output",
    CY + "+-- symbol",
    CY + "|   +-- where_cmd",
    CY + "|   +-- types",
    CY + "+-- graph",
    CY + "|   +-- impact",
    CY + "+-- serve",
    CY + "    +-- semantic",
    CY + "    +-- cache",
    RS,
    "",
    DM + "  12 modules in 9ms" + RS,
]
for line in lines:
    out(line + "\n", 120)
advance(500)

# ═══ Scene 8: JSON Output ═══
prompt('cs file "config" -l 2 -j')
advance(500)
lines = [
    "{",
    "  " + BL + '"tool"' + RS + ': "codescope",',
    "  " + BL + '"command"' + RS + ': "file",',
    "  " + BL + '"count"' + RS + ': 2,',
    "  " + BL + '"results"' + RS + ': [',
    "    {",
    '      ' + BL + '"path"' + RS + ': "src/config.rs",',
    '      ' + BL + '"score"' + RS + ': 0.98',
    "    },",
    "    {",
    '      ' + BL + '"path"' + RS + ': "src/cli.rs",',
    '      ' + BL + '"score"' + RS + ': 0.85',
    "    }",
    "  ]",
    "}",
]
for line in lines:
    out(line + "\n", 150)
advance(500)

# ═══ Scene 9: End ═══
lines = [
    "",
    CY + "═══════════════════════════════════════════════════════════════" + RS,
    BD + "  CodeScope — Repository Intelligence Engine for AI & Developers" + RS,
    DM + "  28 commands · 10 languages · ~2 MB · Zero dependencies" + RS,
    CY + "═══════════════════════════════════════════════════════════════" + RS,
    "",
    DM + "  github.com/Arga-Wicaksono/codescope" + RS,
]
for line in lines:
    out(line + "\n", 100)
advance(1200)

# ═══ Write file ═══
header = {"version": 2, "width": W, "height": H, "timestamp": int(time.time())}
with open(OUT, "w") as f:
    f.write(json.dumps(header) + "\n")
    for frame in frames:
        f.write(json.dumps(frame) + "\n")

total = frames[-1][0] if frames else 0
print(f"Written {len(frames)} events, {total:.1f}s total -> {OUT}")
