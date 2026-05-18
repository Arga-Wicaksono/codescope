#!/usr/bin/env python3
"""Generate CodeScope asciinema demo recording — v2 (no f-string issues)."""

import json
import time
import sys

output_path = sys.argv[1] if len(sys.argv) > 1 else "/home/z/my-project/codescope/demo.cast"

frames = []
width = 80
height = 32

CY = "\033[36m"   # cyan
GR = "\033[33m"   # yellow
RD = "\033[31m"   # red
GN = "\033[32m"   # green
BL = "\033[34m"   # blue
MG = "\033[35m"   # magenta
BD = "\033[1m"    # bold
DM = "\033[2m"    # dim
RS = "\033[0m"    # reset

def esc(code):
    return "\033[" + code + "m"

def out(text, delay_ms):
    t = frames[-1][0] if frames else 0
    frames.append([t + delay_ms / 1000.0, "o", text])

def inp(text, delay_ms):
    t = frames[-1][0] if frames else 0
    frames.append([t + delay_ms / 1000.0, "o", text + "\r\n"])

def wait(ms):
    t = frames[-1][0] if frames else 0
    frames.append([t + ms / 1000.0, "o", ""])

# ═══ Scene 1: Banner ═══
wait(300)
banner = (
    CY + "\n"
    "     ____                 __\n"
    "    / __/___  ____  _____/ /_\n"
    "   / /_/ __ \\/ __ \\/ ___/ __/\n"
    "  / __/ /_/ / / / / /__/ /_\n"
    " /_/  \\____/_/ /_/\\___/\\__/\n"
    "\n"
    "  cs v1.3.0 — Repository Intelligence Engine\n"
    "  28 commands. 10 languages. One binary.\n"
    "\n"
    "  Quick start:\n"
    "    cs file \"pattern\"        Find files\n"
    "    cs content \"text\"        Search content\n"
    "    cs where \"fn_name\"       Find definitions\n"
    "    cs context \"topic\"       Extract AI context\n"
    "    cs graph --type modules  Dependency graph\n"
    + RS + "\n"
)
out(banner, 2000)

# ═══ Scene 2: File Search ═══
wait(400)
inp("cs file \"config\"", 600)
r = ("  " + GR + "src/config.rs" + RS + "          " + DM + "(98%)" + RS + "\n"
     "  " + GR + "src/cli.rs" + RS + "            " + DM + "(85%)" + RS + "\n"
     "  " + GR + "src/lib.rs" + RS + "            " + DM + "(72%)" + RS + "\n"
     "  " + GR + "tests/config_test.rs" + RS + "  " + DM + "(68%)" + RS + "\n"
     "  " + GR + "docs/config.md" + RS + "        " + DM + "(65%)" + RS + "\n"
     "\n" + DM + "  5 results in 12ms" + RS + "\n")
out(r, 1800)

# ═══ Scene 3: Content Search ═══
wait(400)
inp("cs content \"fn main\" -n", 600)
r = (MG + "src/main.rs" + RS + ":\n"
     "  " + GN + "15" + RS + "  fn main() " + DM + "{" + RS + "\n"
     "  " + GN + "16" + RS + "      let args = Cli::parse();\n"
     "  " + GN + "17" + RS + "      match args.command " + DM + "{" + RS + "\n"
     "\n"
     + MG + "src/bin/cs.rs" + RS + ":\n"
     "  " + GN + " 3" + RS + "  fn main() -> Result<()> " + DM + "{" + RS + "\n"
     "  " + GN + " 4" + RS + "      codescope::run()\n"
     "\n" + DM + "  2 files, 2 matches in 8ms" + RS + "\n")
out(r, 1800)

# ═══ Scene 4: Symbol Intelligence ═══
wait(400)
inp("cs where \"parse_config\"", 600)
r = ("  " + GR + BD + "src/config.rs:42" + RS + "  " + DM + "function" + RS
     + "  parse_config(path: &Path) -> Result<Config>\n"
     "  " + GR + BD + "src/config.rs:78" + RS + "  " + DM + "function" + RS
     + "  parse_config_file(content: &str) -> Result<Value>\n"
     "\n" + DM + "  2 definitions in 5ms" + RS + "\n")
out(r, 1800)

# ═══ Scene 5: References ═══
wait(400)
inp("cs refs \"Config\"", 600)
r = (MG + "src/main.rs" + RS + ":\n"
     "  " + GN + "16" + RS + "      let " + BD + "Config" + RS + " = load_config()?;\n"
     "  " + GN + "45" + RS + "      app." + BD + "Config" + RS + "(&cfg);\n"
     "\n"
     + MG + "src/config.rs" + RS + ":\n"
     "  " + GN + "12" + RS + "  struct " + BD + "Config" + RS + " " + DM + "{" + RS + "\n"
     "  " + GN + "18" + RS + "  impl " + BD + "Config" + RS + " " + DM + "{" + RS + "\n"
     "\n" + DM + "  4 references in 6ms" + RS + "\n")
out(r, 1800)

# ═══ Scene 6: Context Engine ═══
wait(400)
inp("cs context \"authentication\"", 600)
r = ("  " + GR + BD + "[FILE] " + RS + "src/auth/handler.rs        "
     + DM + "score: 95" + RS + "  " + DM + "tokens: 1,247" + RS + "\n"
     "  " + GR + BD + "[FILE] " + RS + "src/auth/middleware.rs      "
     + DM + "score: 87" + RS + "  " + DM + "tokens: 892" + RS + "\n"
     "  " + GR + BD + "[FILE] " + RS + "src/auth/token.rs           "
     + DM + "score: 82" + RS + "  " + DM + "tokens: 654" + RS + "\n"
     "  " + GR + BD + "[SYMB] " + RS + "src/auth/handler.rs:23     "
     + DM + "score: 91" + RS + "  fn authenticate()\n"
     "  " + GR + BD + "[CONT] " + RS + "src/api/routes.rs:45       "
     + DM + "score: 74" + RS + "  auth guard middleware\n"
     "\n" + DM + "  5 results in 14ms" + RS + "\n")
out(r, 1800)

# ═══ Scene 7: Dependency Graph ═══
wait(400)
inp("cs graph --type modules", 600)
r = (CY + "cs" + RS + " (root)\n"
     + CY + "├── cli\n"
     + CY + "│   └── config\n"
     + CY + "├── file_search\n"
     + CY + "│   ├── utils\n"
     + CY + "│   └── output\n"
     + CY + "├── content_search\n"
     + CY + "│   ├── utils\n"
     + CY + "│   └── output\n"
     + CY + "├── symbol\n"
     + CY + "│   ├── where_cmd\n"
     + CY + "│   └── types\n"
     + CY + "├── graph\n"
     + CY + "│   └── impact\n"
     + CY + "└── serve\n"
     + CY + "    ├── semantic\n"
     + CY + "    └── cache\n"
     + RS + "\n" + DM + "  12 modules in 9ms" + RS + "\n")
out(r, 1800)

# ═══ Scene 8: JSON Output ═══
wait(400)
inp("cs file \"config\" -l 2 -j", 600)
r = ("{\n"
     "  " + BL + "\"tool\"" + RS + ": \"codescope\",\n"
     "  " + BL + "\"command\"" + RS + ": \"file\",\n"
     "  " + BL + "\"count\"" + RS + ": 2,\n"
     "  " + BL + "\"results\"" + RS + ": [\n"
     "    {\n"
     "      " + BL + "\"path\"" + RS + ": \"src/config.rs\",\n"
     "      " + BL + "\"score\"" + RS + ": 0.98\n"
     "    },\n"
     "    {\n"
     "      " + BL + "\"path\"" + RS + ": \"src/cli.rs\",\n"
     "      " + BL + "\"score\"" + RS + ": 0.85\n"
     "    }\n"
     "  ]\n"
     "}\n")
out(r, 1800)

# ═══ Scene 9: End ═══
wait(400)
r = ("\n" + CY + "═══════════════════════════════════════════════════════════════" + RS + "\n"
     + BD + "  CodeScope — Repository Intelligence Engine for AI & Developers" + RS + "\n"
     + DM + "  28 commands  ·  10 languages  ·  ~2 MB  ·  Zero dependencies" + RS + "\n"
     + CY + "═══════════════════════════════════════════════════════════════" + RS + "\n"
     "\n" + DM + "  github.com/Arga-Wicaksono/codescope" + RS + "\n\n")
out(r, 2500)

# ═══ Write file ═══
header = {"version": 2, "width": width, "height": height, "timestamp": int(time.time())}
with open(output_path, "w") as f:
    f.write(json.dumps(header) + "\n")
    for frame in frames:
        f.write(json.dumps(frame) + "\n")

total = frames[-1][0] if frames else 0
print(f"Written {len(frames)} frames, {total:.1f}s total -> {output_path}")
