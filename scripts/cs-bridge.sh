#!/usr/bin/env bash
# ============================================================================
# CodeScope Bridge for Z AI Agent
# ============================================================================
# Starts the Python HTTP bridge server and provides query interface.
#
# Usage:
#   cs-bridge start [--port 4567] [--path /repo/path]
#   cs-bridge stop
#   cs-bridge status
#   cs-bridge query <endpoint> [params...]
#
# Examples:
#   cs-bridge start --path /home/user/repo
#   cs-bridge query search q=search_files path=/home/user/repo/src
#   cs-bridge query symbol name=Config path=/home/user/repo/src
#   cs-bridge query stats path=/home/user/repo/src
#   cs-bridge query health
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BRIDGE_PY="${SCRIPT_DIR}/cs-http-bridge.py"
CS_BIN="${CS_BIN:-$(which cs 2>/dev/null || echo '/usr/local/bin/cs')}"
PID_FILE="/tmp/codescope-bridge.pid"
LOG_FILE="/tmp/codescope-bridge.log"
DEFAULT_PORT=4567
DEFAULT_PATH="."

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[cs-bridge]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[cs-bridge]${NC} $*"; }
log_error() { echo -e "${RED}[cs-bridge]${NC} $*"; }

check_deps() {
    if ! command -v python3 &>/dev/null; then
        log_error "python3 is required but not found"
        exit 1
    fi
    if [[ ! -f "$BRIDGE_PY" ]]; then
        log_error "Bridge script not found at '$BRIDGE_PY'"
        exit 1
    fi
}

resolve_port() {
    local port="$DEFAULT_PORT"
    local args=("$@")
    for ((i=0; i<${#args[@]}; i++)); do
        if [[ "${args[$i]}" == "--port" ]] && ((i+1 < ${#args[@]})); then
            port="${args[$i+1]}"
            break
        fi
    done
    echo "$port"
}

resolve_path() {
    local path="$DEFAULT_PATH"
    local args=("$@")
    for ((i=0; i<${#args[@]}; i++)); do
        if [[ "${args[$i]}" == "--path" ]] && ((i+1 < ${#args[@]})); then
            path="${args[$i+1]}"
            break
        fi
    done
    echo "$path"
}

cmd_start() {
    check_deps

    if [[ -f "$PID_FILE" ]]; then
        local pid
        pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            log_warn "CodeScope bridge already running (PID: $pid)"
            return 0
        else
            rm -f "$PID_FILE"
        fi
    fi

    local port
    port=$(resolve_port "$@")
    local workdir
    workdir=$(resolve_path "$@")

    if [[ "$workdir" != "." ]]; then
        workdir=$(cd "$workdir" && pwd)
    else
        workdir=$(pwd)
    fi

    log_info "Starting CodeScope HTTP Bridge..."
    log_info "  Python:    $(python3 --version 2>&1)"
    log_info "  CS binary: $CS_BIN"
    log_info "  Port:      $port"
    log_info "  Path:      $workdir"

    CS_BIN="$CS_BIN" nohup python3 "$BRIDGE_PY" --port "$port" --path "$workdir" > "$LOG_FILE" 2>&1 &
    local pid=$!
    echo "$pid" > "$PID_FILE"

    local retries=0
    local max_retries=10
    while (( retries < max_retries )); do
        if curl -s "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
            log_info "Server ready! (PID: $pid)"
            log_info ""
            log_info "Query examples:"
            log_info "  cs-bridge query health"
            log_info "  cs-bridge query search q=pattern path=$workdir"
            log_info "  cs-bridge query symbol name=FuncName path=$workdir"
            log_info "  cs-bridge query stats path=$workdir"
            return 0
        fi
        sleep 0.5
        ((retries++))
    done

    log_error "Server failed to start. Check log:"
    cat "$LOG_FILE" 2>/dev/null
    rm -f "$PID_FILE"
    return 1
}

cmd_stop() {
    if [[ ! -f "$PID_FILE" ]]; then
        log_warn "Server not running (no PID file)"
        return 0
    fi
    local pid
    pid=$(cat "$PID_FILE")
    if kill -0 "$pid" 2>/dev/null; then
        log_info "Stopping bridge (PID: $pid)..."
        kill "$pid" 2>/dev/null
        sleep 1
        kill -9 "$pid" 2>/dev/null 2>/dev/null || true
        log_info "Stopped"
    fi
    rm -f "$PID_FILE"
}

cmd_status() {
    local port
    port=$(resolve_port "$@")
    if [[ -f "$PID_FILE" ]] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
        log_info "Running (PID: $(cat "$PID_FILE"), Port: $port)"
        curl -s "http://127.0.0.1:$port/health" 2>/dev/null | python3 -m json.tool 2>/dev/null
    else
        log_info "Not running"
    fi
}

cmd_query() {
    local port
    port=$(resolve_port "$@")

    if [[ -f "$PID_FILE" ]] && ! kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
        log_error "Server not running (stale PID)"
        rm -f "$PID_FILE"
        exit 1
    fi

    local endpoint="${1:-}"
    shift || true

    if [[ -z "$endpoint" ]]; then
        log_error "Usage: cs-bridge query <endpoint> [key=value ...]"
        exit 1
    fi

    endpoint="${endpoint#/}"

    local query_string=""
    for param in "$@"; do
        if [[ "$param" == *"="* ]]; then
            local key="${param%%=*}"
            local val="${param#*=}"
            val=$(python3 -c "import urllib.parse; print(urllib.parse.quote('$val'))" 2>/dev/null || echo "$val")
            [[ -n "$query_string" ]] && query_string+="&"
            query_string+="${key}=${val}"
        fi
    done

    local url="http://127.0.0.1:${port}/${endpoint}"
    [[ -n "$query_string" ]] && url+="?${query_string}"

    local http_code
    http_code=$(curl -s -o /tmp/cs-response.json -w "%{http_code}" "$url" 2>/dev/null)
    if [[ "$http_code" == "200" ]]; then
        python3 -m json.tool /tmp/cs-response.json 2>/dev/null || cat /tmp/cs-response.json
    else
        log_error "HTTP $http_code"
        cat /tmp/cs-response.json 2>/dev/null
    fi
    rm -f /tmp/cs-response.json
}

case "${1:-}" in
    start)   shift; cmd_start "$@" ;;
    stop)    cmd_stop ;;
    status)  shift; cmd_status "$@" ;;
    query)   shift; cmd_query "$@" ;;
    restart) shift; cmd_stop; sleep 1; cmd_start "$@" ;;
    *)
        echo ""
        echo -e "${CYAN}CodeScope Bridge — HTTP API for AI Agents${NC}"
        echo ""
        echo "Usage: $0 <command> [options]"
        echo ""
        echo "Commands:"
        echo "  start [--port N] [--path DIR]  Start HTTP server"
        echo "  stop                           Stop server"
        echo "  status                         Check status"
        echo "  query <endpoint> [key=value]   Query endpoint"
        echo "  restart                        Restart server"
        echo ""
        echo "Endpoints: health, search, files, symbol, refs, callers,"
        echo "           symbols, context, pack, trace, stats, where"
        ;;
esac
