#!/bin/bash
set -euo pipefail

clear

# ═══ Scene 1: Banner ═════════════════════════════════════════════════
printf '\033[36m'
cat << 'BANNER'

     ____                 __
    / __/___  ____  _____/ /_
   / /_/ __ \/ __ \/ ___/ __/
  / __/ /_/ / / / / /__/ /_
 /_/  \____/_/ /_/\___/\__/

  cs v1.3.0 — Repository Intelligence Engine
  28 commands. 10 languages. One binary.

  Quick start:
    cs file "pattern"        Find files
    cs content "text"        Search content
    cs where "fn_name"       Find definitions
    cs context "topic"       Extract AI context
    cs graph --type modules  Dependency graph

BANNER
printf '\033[0m'
sleep 2

# ═══ Scene 2: File Search ════════════════════════════════════════════
echo ""
printf '\033[1m$ cs file "config"\033[0m'
sleep 0.8
echo ""
printf '  \033[33msrc/config.rs\033[0m          \033[2m(98%%)\033[0m\n'
printf '  \033[33msrc/cli.rs\033[0m            \033[2m(85%%)\033[0m\n'
printf '  \033[33msrc/lib.rs\033[0m            \033[2m(72%%)\033[0m\n'
printf '  \033[33mtests/config_test.rs\033[0m  \033[2m(68%%)\033[0m\n'
printf '  \033[33mdocs/config.md\033[0m        \033[2m(65%%)\033[0m\n'
echo ""
printf '\033[90m  5 results in 12ms\033[0m'
sleep 2

# ═══ Scene 3: Content Search ═════════════════════════════════════════
echo ""
printf '\033[1m$ cs content "fn main" -n\033[0m'
sleep 0.8
echo ""
printf '\033[35msrc/main.rs\033[0m:\n'
printf '  \033[32m15\033[0m | fn main() {\n'
printf '  \033[32m16\033[0m |     let args = Cli::parse();\n'
printf '  \033[32m17\033[0m |     match args.command {\n'
echo ""
printf '\033[35msrc/bin/cs.rs\033[0m:\n'
printf '  \033[32m 3\033[0m | fn main() -> Result<()> {\n'
printf '  \033[32m 4\033[0m |     codescope::run()\n'
echo ""
printf '\033[90m  2 files, 2 matches in 8ms\033[0m'
sleep 2

# ═══ Scene 4: Symbol Intelligence ════════════════════════════════════
echo ""
printf '\033[1m$ cs where "parse_config"\033[0m'
sleep 0.8
echo ""
printf '  \033[33m\033[1msrc/config.rs:42\033[0m  \033[2mfunction\033[0m  parse_config(path: &Path) -> Result<Config>\n'
printf '  \033[33m\033[1msrc/config.rs:78\033[0m  \033[2mfunction\033[0m  parse_config_file(content: &str) -> Result<Value>\n'
echo ""
printf '\033[90m  2 definitions in 5ms\033[0m'
sleep 2

# ═══ Scene 5: References ═════════════════════════════════════════════
echo ""
printf '\033[1m$ cs refs "Config"\033[0m'
sleep 0.8
echo ""
printf '\033[35msrc/main.rs\033[0m:\n'
printf '  \033[32m16\033[0m |     let \033[1mConfig\033[0m = load_config()?;\n'
printf '  \033[32m45\033[0m |     app.\033[1mConfig\033[0m(&cfg);\n'
echo ""
printf '\033[35msrc/config.rs\033[0m:\n'
printf '  \033[32m12\033[0m | struct \033[1mConfig\033[0m {\n'
printf '  \033[32m18\033[0m | impl \033[1mConfig\033[0m {\n'
echo ""
printf '\033[90m  4 references in 6ms\033[0m'
sleep 2

# ═══ Scene 6: Context Engine ═════════════════════════════════════════
echo ""
printf '\033[1m$ cs context "authentication"\033[0m'
sleep 0.8
echo ""
printf '  \033[33m\033[1m[FILE] \033[0msrc/auth/handler.rs        \033[2mscore: 95\033[0m  \033[2mtokens: 1,247\033[0m\n'
printf '  \033[33m\033[1m[FILE] \033[0msrc/auth/middleware.rs      \033[2mscore: 87\033[0m  \033[2mtokens: 892\033[0m\n'
printf '  \033[33m\033[1m[FILE] \033[0msrc/auth/token.rs           \033[2mscore: 82\033[0m  \033[2mtokens: 654\033[0m\n'
printf '  \033[33m\033[1m[SYMB] \033[0msrc/auth/handler.rs:23     \033[2mscore: 91\033[0m  fn authenticate()\n'
printf '  \033[33m\033[1m[CONT] \033[0msrc/api/routes.rs:45       \033[2mscore: 74\033[0m  auth guard middleware\n'
echo ""
printf '\033[90m  5 results in 14ms\033[0m'
sleep 2

# ═══ Scene 7: Dependency Graph ═══════════════════════════════════════
echo ""
printf '\033[1m$ cs graph --type modules\033[0m'
sleep 0.8
echo ""
printf '\033[36mcs\033[0m (root)\n'
printf '  ├── \033[36mcli\033[0m\n'
printf '  │   └── \033[36mconfig\033[0m\n'
printf '  ├── \033[36mfile_search\033[0m\n'
printf '  │   ├── \033[36mutils\033[0m\n'
printf '  │   └── \033[36moutput\033[0m\n'
printf '  ├── \033[36mcontent_search\033[0m\n'
printf '  │   ├── \033[36mutils\033[0m\n'
printf '  │   └── \033[36moutput\033[0m\n'
printf '  ├── \033[36msymbol\033[0m\n'
printf '  │   ├── \033[36mwhere_cmd\033[0m\n'
printf '  │   └── \033[36mtypes\033[0m\n'
printf '  ├── \033[36mgraph\033[0m\n'
printf '  │   └── \033[36mimpact\033[0m\n'
printf '  └── \033[36mserve\033[0m\n'
printf '      ├── \033[36msemantic\033[0m\n'
printf '      └── \033[36mcache\033[0m\n'
echo ""
printf '\033[90m  12 modules in 9ms\033[0m'
sleep 2

# ═══ Scene 8: JSON Output ════════════════════════════════════════════
echo ""
printf '\033[1m$ cs file "config" -l 2 -j\033[0m'
sleep 0.8
echo ""
printf '{\n'
printf '  \033[34m"tool"\033[0m: "codescope",\n'
printf '  \033[34m"command"\033[0m: "file",\n'
printf '  \033[34m"count"\033[0m: 2,\n'
printf '  \033[34m"results"\033[0m: [\n'
printf '    {\n'
printf '      \033[34m"path"\033[0m: "src/config.rs",\n'
printf '      \033[34m"score"\033[0m: 0.98\n'
printf '    },\n'
printf '    {\n'
printf '      \033[34m"path"\033[0m: "src/cli.rs",\n'
printf '      \033[34m"score"\033[0m: 0.85\n'
printf '    }\n'
printf '  ]\n'
printf '}\n'
sleep 2

# ═══ Scene 9: End ═══════════════════════════════════════════════════
echo ""
printf '\033[36m═══════════════════════════════════════════════════════════════\033[0m'
printf '\033[1m  CodeScope — Repository Intelligence Engine for AI & Developers\033[0m\n'
printf '\033[90m  28 commands  ·  10 languages  ·  ~2 MB  ·  Zero dependencies\033[0m\n'
printf '\033[36m═══════════════════════════════════════════════════════════════\033[0m'
echo ""
printf '\033[90m  github.com/Arga-Wicaksono/codescope\033[0m'
echo ""
sleep 3
