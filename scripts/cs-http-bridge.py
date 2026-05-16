#!/usr/bin/env python3
"""
CodeScope HTTP Bridge for AI Agents.

Lightweight HTTP server that wraps CodeScope CLI commands.
Start it and query via REST endpoints.

Usage:
    python3 cs-http-bridge.py [--port 4567] [--path /repo/path]

Endpoints:
    GET /health                         Server health check
    GET /search?q=<pattern>&path=<dir>  Search content in files
    GET /files?q=<pattern>&path=<dir>   Search files by name
    GET /symbol?name=<name>&path=<dir>  Find symbol definitions
    GET /refs?name=<name>&path=<dir>    Find references
    GET /callers?name=<name>&path=<dir> Find function callers
    GET /symbols?path=<dir>&limit=50    List all symbols
    GET /context?q=<topic>&path=<dir>   Extract context for topic
    GET /pack?description=<desc>&budget=<n> Pack for LLM
    GET /trace?symbol=<name>&path=<dir> Trace function calls
    GET /stats?path=<dir>               Repository statistics
    GET /where?name=<name>&path=<dir>   Find definitions (faster)
"""

import argparse
import json
import os
import subprocess
import sys
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

# ---------------------------------------------------------------------------
# CodeScope CLI wrapper
# ---------------------------------------------------------------------------

CS_BIN = os.environ.get("CS_BIN", "cs")


def run_cs(args: list[str], timeout: int = 30) -> dict:
    """Run a cs command and return parsed JSON output."""
    try:
        result = subprocess.run(
            [CS_BIN] + args,
            capture_output=True,
            text=True,
            timeout=timeout,
            env={**os.environ, "NO_COLOR": "1"},
        )
        stdout = result.stdout.strip()
        if result.returncode == 0 and stdout:
            # Try to parse as JSON
            try:
                return json.loads(stdout)
            except json.JSONDecodeError:
                return {"raw": stdout, "exit_code": 0}
        return {"error": result.stderr.strip() or "command failed", "exit_code": result.returncode}
    except subprocess.TimeoutExpired:
        return {"error": f"command timed out after {timeout}s", "exit_code": -1}
    except FileNotFoundError:
        return {"error": f"cs binary not found at '{CS_BIN}'", "exit_code": -1}
    except Exception as e:
        return {"error": str(e), "exit_code": -1}


def run_cs_json(args: list[str], timeout: int = 30) -> dict:
    """Run cs with --json flag and return parsed output."""
    return run_cs(args + ["--json"], timeout)


# ---------------------------------------------------------------------------
# HTTP handler
# ---------------------------------------------------------------------------

class CodeScopeHandler(BaseHTTPRequestHandler):
    """HTTP request handler for CodeScope API."""

    working_dir = "."

    def log_message(self, format, *args):
        # Suppress default logging for cleaner output
        pass

    def _send_json(self, data: dict, status: int = 200):
        body = json.dumps(data, indent=2, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def _send_error(self, message: str, status: int = 400):
        self._send_json({"error": message, "status": status}, status)

    def _get_param(self, params: dict, key: str, default: str = None) -> str:
        values = params.get(key, [])
        return values[0] if values else default

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path.rstrip("/")
        params = parse_qs(parsed.query)

        if path in ("/", "/health"):
            self._send_json({
                "tool": "codescope",
                "version": "1.1.0",
                "status": "ok",
                "working_dir": self.working_dir,
                "endpoints": [
                    "/health", "/search", "/files", "/symbol", "/refs",
                    "/callers", "/symbols", "/context", "/pack", "/trace",
                    "/stats", "/where",
                ],
            })
            return

        if path == "/search":
            q = self._get_param(params, "q") or self._get_param(params, "query")
            if not q:
                return self._send_error("Missing 'q' parameter")
            search_path = self._get_param(params, "path", self.working_dir)
            limit = self._get_param(params, "limit", "20")
            exact = self._get_param(params, "exact", "false")
            regex = self._get_param(params, "regex", "false")
            extension = self._get_param(params, "extension") or self._get_param(params, "type")

            args = ["content", q, "--path", search_path, "--limit", limit]
            if exact == "true":
                args.append("--exact")
            if regex == "true":
                args.append("--regex")
            if extension:
                args.extend(["--extension", extension])
            args.extend(["--no-ignore"])  # Ensure search works without .gitignore issues

            result = run_cs(args)
            self._send_json(result)

        elif path == "/files":
            q = self._get_param(params, "q") or self._get_param(params, "query")
            if not q:
                return self._send_error("Missing 'q' parameter")
            search_path = self._get_param(params, "path", self.working_dir)
            limit = self._get_param(params, "limit", "20")
            extension = self._get_param(params, "extension") or self._get_param(params, "type")

            args = ["file", q, "--path", search_path, "--limit", limit]
            if extension:
                args.extend(["--extension", extension])

            result = run_cs(args)
            self._send_json(result)

        elif path == "/symbol":
            name = self._get_param(params, "name")
            if not name:
                return self._send_error("Missing 'name' parameter")
            search_path = self._get_param(params, "path", self.working_dir)

            result = run_cs_json(["symbol", name, "--path", search_path])
            self._send_json(result)

        elif path == "/refs":
            name = self._get_param(params, "name")
            if not name:
                return self._send_error("Missing 'name' parameter")
            search_path = self._get_param(params, "path", self.working_dir)

            result = run_cs_json(["refs", name, "--path", search_path])
            self._send_json(result)

        elif path == "/callers":
            name = self._get_param(params, "name")
            if not name:
                return self._send_error("Missing 'name' parameter")
            search_path = self._get_param(params, "path", self.working_dir)

            result = run_cs_json(["callers", name, "--path", search_path])
            self._send_json(result)

        elif path == "/symbols":
            search_path = self._get_param(params, "path", self.working_dir)
            limit = self._get_param(params, "limit", "50")
            symbol_type = self._get_param(params, "type") or self._get_param(params, "symbol_type")
            extension = self._get_param(params, "extension")

            args = ["symbols", "--path", search_path, "--limit", limit]
            if symbol_type:
                args.extend(["--symbol-type", symbol_type])
            if extension:
                args.extend(["--extension", extension])

            result = run_cs_json(args)
            self._send_json(result)

        elif path == "/context":
            q = self._get_param(params, "q") or self._get_param(params, "query")
            if not q:
                return self._send_error("Missing 'q' parameter")
            search_path = self._get_param(params, "path", self.working_dir)
            max_items = self._get_param(params, "max_items", "20")

            result = run_cs_json(["context", q, "--path", search_path, "--max-items", max_items])
            self._send_json(result)

        elif path == "/pack":
            desc = self._get_param(params, "description") or self._get_param(params, "q")
            if not desc:
                return self._send_error("Missing 'description' or 'q' parameter")
            search_path = self._get_param(params, "path", self.working_dir)
            budget = self._get_param(params, "budget", "8000")

            result = run_cs_json(["pack", desc, "--path", search_path, "--budget", budget])
            self._send_json(result)

        elif path == "/trace":
            symbol = self._get_param(params, "symbol") or self._get_param(params, "name")
            if not symbol:
                return self._send_error("Missing 'symbol' or 'name' parameter")
            search_path = self._get_param(params, "path", self.working_dir)
            depth = self._get_param(params, "depth", "5")

            result = run_cs_json(["trace", symbol, "--path", search_path])
            self._send_json(result)

        elif path == "/stats":
            search_path = self._get_param(params, "path", self.working_dir)

            result = run_cs_json(["stats", "--path", search_path])
            self._send_json(result)

        elif path == "/where":
            name = self._get_param(params, "name")
            if not name:
                return self._send_error("Missing 'name' parameter")
            search_path = self._get_param(params, "path", self.working_dir)

            result = run_cs_json(["where", name, "--path", search_path])
            self._send_json(result)

        else:
            self._send_error(f"Unknown endpoint: {path}", 404)


# ---------------------------------------------------------------------------
# Server runner
# ---------------------------------------------------------------------------

def run_server(port: int, path: str):
    """Start the CodeScope HTTP bridge server."""
    abs_path = os.path.abspath(path)
    CodeScopeHandler.working_dir = abs_path

    server = HTTPServer(("127.0.0.1", port), CodeScopeHandler)

    print(f"\n  CodeScope HTTP Bridge", flush=True)
    print(f"  " + "=" * 50, flush=True)
    print(f"  Listening on: http://127.0.0.1:{port}", flush=True)
    print(f"  Working dir:  {abs_path}", flush=True)
    print(f"  CS binary:    {CS_BIN}", flush=True)
    print(f"", flush=True)
    print(f"  Endpoints:", flush=True)
    print(f"    GET /health                         Health check", flush=True)
    print(f"    GET /search?q=<pattern>&path=<dir>  Content search", flush=True)
    print(f"    GET /files?q=<pattern>&path=<dir>   File search", flush=True)
    print(f"    GET /symbol?name=<name>&path=<dir>  Find symbols", flush=True)
    print(f"    GET /refs?name=<name>&path=<dir>    Find references", flush=True)
    print(f"    GET /callers?name=<name>&path=<dir> Find callers", flush=True)
    print(f"    GET /symbols?path=<dir>&limit=50    List symbols", flush=True)
    print(f"    GET /context?q=<topic>&path=<dir>   Get context", flush=True)
    print(f"    GET /pack?description=<desc>        Pack for LLM", flush=True)
    print(f"    GET /trace?symbol=<name>&path=<dir> Trace calls", flush=True)
    print(f"    GET /stats?path=<dir>               Repo statistics", flush=True)
    print(f"    GET /where?name=<name>&path=<dir>   Find definitions", flush=True)
    print(f"", flush=True)
    print(f"  Press Ctrl+C to stop", flush=True)
    print(f"", flush=True)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print(f"\n  Server stopped.", flush=True)
        server.server_close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="CodeScope HTTP Bridge for AI Agents")
    parser.add_argument("--port", type=int, default=4567, help="HTTP port (default: 4567)")
    parser.add_argument("--path", default=".", help="Working directory (default: .)")
    parser.add_argument("--cs-bin", default=None, help="Path to cs binary")
    args = parser.parse_args()

    if args.cs_bin:
        CS_BIN = args.cs_bin

    run_server(args.port, args.path)
