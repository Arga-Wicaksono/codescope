#!/usr/bin/env bash
# ============================================================================
# CodeScope MCP Client — Call CodeScope tools via MCP protocol (JSON-RPC 2.0)
# ============================================================================
# Pipes JSON-RPC messages to `cs serve --mcp` and returns the response.
# Ideal for AI agent integration — one-shot tool calls.
#
# Usage:
#   cs-mcp <tool_name> [param1=value1] [param2=value2]
#
# Examples:
#   cs-mcp search_files pattern="config" path="/repo/src" limit=10
#   cs-mcp find_symbol name="Config" path="/repo/src"
#   cs-mcp search_content pattern="fn main" path="/repo/src" exact=true
#   cs-mcp find_references name="search_files" path="/repo/src"
#   cs-mcp find_callers name="search_files" path="/repo/src"
#   cs-mcp list_symbols path="/repo/src" limit=20
#   cs-mcp get_context topic="auth" path="/repo/src" max_items=10
#   cs-mcp pack_context description="authentication flow" path="/repo/src" budget=4000
#   cs-mcp trace_symbol name="main" path="/repo/src" max_depth=3
#   cs-mcp repo_stats path="/repo/src"
#   cs-mcp tools_list
# ============================================================================

set -euo pipefail

CS_BIN="${CS_BIN:-$(which cs 2>/dev/null || echo '/usr/local/bin/cs')}"
TOOL_NAME="${1:-}"

if [[ -z "$TOOL_NAME" ]]; then
    echo "Usage: cs-mcp <tool_name> [param1=value1] [param2=value2]" >&2
    echo "" >&2
    echo "Tools:" >&2
    echo "  tools_list                          List available MCP tools" >&2
    echo "  search_files  pattern=  path=  ...  Search files by name" >&2
    echo "  search_content pattern=  path=  ... Search content in files" >&2
    echo "  find_symbol   name=     path=  ... Find symbol definitions" >&2
    echo "  find_references name=  path=  ... Find references" >&2
    echo "  find_callers name=     path=  ... Find function callers" >&2
    echo "  list_symbols  path=     ...       List symbols in dir" >&2
    echo "  get_context   topic=    path=  ... Get context for topic" >&2
    echo "  pack_context  description= path= ... Pack for LLM" >&2
    echo "  trace_symbol  name=     path=  ... Trace execution" >&2
    echo "  repo_stats    path=              Repo statistics" >&2
    exit 1
fi

# Parse key=value arguments into JSON object
shift || true
ARGS_JSON="{}"

for param in "$@"; do
    if [[ "$param" == *"="* ]]; then
        local_key="${param%%=*}"
        local_value="${param#*=}"

        # Auto-detect value type
        if [[ "$local_value" == "true" || "$local_value" == "false" ]]; then
            type_hint="bool"
        elif [[ "$local_value" =~ ^[0-9]+$ ]]; then
            type_hint="int"
        else
            type_hint="string"
        fi

        # Build JSON using python
        ARGS_JSON=$(python3 -c "
import json, sys
args = json.loads('''$ARGS_JSON''')
key = '''$local_key'''
value = '''$local_value'''
if '''$type_hint''' == 'bool':
    args[key] = value.lower() == 'true'
elif '''$type_hint''' == 'int':
    args[key] = int(value)
else:
    args[key] = value
print(json.dumps(args))
" 2>/dev/null || echo "$ARGS_JSON")
    fi
done

# Build JSON-RPC messages
INIT_REQ='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"cs-mcp","version":"1.0"}}}'
INIT_NOTIFY='{"jsonrpc":"2.0","method":"notifications/initialized"}'

if [[ "$TOOL_NAME" == "tools_list" ]]; then
    TOOL_CALL="{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}"
else
    TOOL_CALL="{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"$TOOL_NAME\",\"arguments\":$ARGS_JSON}}"
fi

# Pipe to cs serve --mcp and capture output
RESULT=$(printf '%s\n%s\n%s\n' "$INIT_REQ" "$INIT_NOTIFY" "$TOOL_CALL" | \
    timeout 30 "$CS_BIN" serve --mcp --path "." 2>/dev/null || echo "TIMEOUT")

# Extract the tools/call response (last JSON line with id:2)
echo "$RESULT" | grep -o '{[^}]*"id":2[^}]*}[^}]*}' | tail -1 | \
    python3 -c "
import sys, json
line = sys.stdin.read().strip()
if line:
    data = json.loads(line)
    if 'result' in data:
        r = data['result']
        if isinstance(r, dict):
            print(json.dumps(r, indent=2))
        else:
            print(json.dumps(r, indent=2))
    elif 'error' in data:
        print(json.dumps({'error': data['error']}, indent=2), file=sys.stderr)
        sys.exit(1)
" 2>/dev/null || echo "$RESULT" | tail -1
