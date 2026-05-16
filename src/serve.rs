//! MCP (Model Context Protocol) Server and HTTP API.
//!
//! Makes CodeScope consumable by any AI system through two interfaces:
//!
//! 1. **MCP Server** (`cs serve --mcp`) - JSON-RPC 2.0 over stdin/stdout.
//!    Compatible with Claude Desktop, Cursor, and any MCP-compatible agent.
//!
//! 2. **HTTP API** (`cs serve --http [port]`) - RESTful HTTP server.
//!    Endpoints for search, context, symbol, trace, pack, stats.

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::net::TcpListener;
use std::thread;

// ---------------------------------------------------------------------------
// MCP Protocol types (JSON-RPC 2.0)
// ---------------------------------------------------------------------------

/// JSON-RPC 2.0 request from the client.
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response to the client.
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// Large result threshold in bytes for streaming progress notifications.
const STREAMING_THRESHOLD: usize = 10_240;

/// Emit a progress notification to stderr for large results.
fn emit_streaming_progress(tool_name: &str, result_size: usize) {
    eprintln!(
        "{} Streaming: tool '{}' produced {} bytes, sending result...",
        ">>".cyan(),
        tool_name.yellow(),
        result_size
    );
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// MCP tool definition.
#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

/// MCP server info for initialization response.
#[derive(Debug, Serialize)]
struct McpServerInfo {
    name: String,
    version: String,
}

/// MCP capabilities.
#[derive(Debug, Serialize)]
struct McpCapabilities {
    tools: McpToolsCapability,
    #[serde(skip_serializing_if = "Option::is_none")]
    streaming: Option<McpStreamingCapability>,
}

#[derive(Debug, Serialize)]
struct McpToolsCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    list_changed: Option<bool>,
}

#[derive(Debug, Serialize)]
struct McpStreamingCapability {
    supported: bool,
}

// ---------------------------------------------------------------------------
// MCP tool definitions
// ---------------------------------------------------------------------------

fn get_mcp_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "search_files".to_string(),
            description: "Search for files by name with fuzzy matching".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Search pattern for file names" },
                    "path": { "type": "string", "description": "Directory to search (default: .)" },
                    "extension": { "type": "string", "description": "Filter by file extension" },
                    "limit": { "type": "integer", "description": "Maximum results (default: 20)" }
                },
                "required": ["pattern"]
            }),
        },
        McpTool {
            name: "search_content".to_string(),
            description: "Search for content inside files (text, regex, or exact match)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Search pattern" },
                    "path": { "type": "string", "description": "Directory to search (default: .)" },
                    "extension": { "type": "string", "description": "Filter by file extension" },
                    "regex": { "type": "boolean", "description": "Use regex matching" },
                    "exact": { "type": "boolean", "description": "Use exact string matching" },
                    "limit": { "type": "integer", "description": "Maximum results (default: 20)" }
                },
                "required": ["pattern"]
            }),
        },
        McpTool {
            name: "find_symbol".to_string(),
            description: "Find where a symbol (function, class, struct, etc.) is defined".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Symbol name to find" },
                    "path": { "type": "string", "description": "Directory to search (default: .)" },
                    "symbol_type": { "type": "string", "description": "Filter by kind (function, class, struct, enum, trait, etc.)" }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "find_references".to_string(),
            description: "Find all references to a symbol (not definitions)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Symbol name" },
                    "path": { "type": "string", "description": "Directory to search (default: .)" }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "find_callers".to_string(),
            description: "Find all functions that call a specific function".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Function name" },
                    "path": { "type": "string", "description": "Directory to search (default: .)" }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "list_symbols".to_string(),
            description: "List all symbols in a file or directory".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File or directory path" },
                    "symbol_type": { "type": "string", "description": "Filter by kind" },
                    "limit": { "type": "integer", "description": "Maximum results (default: 100)" }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: "get_context".to_string(),
            description: "Extract relevant context for a topic (files, symbols, dependencies)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic to extract context for" },
                    "path": { "type": "string", "description": "Directory (default: .)" },
                    "extension": { "type": "string", "description": "Filter by file extension" },
                    "limit": { "type": "integer", "description": "Maximum context items (default: 20)" }
                },
                "required": ["topic"]
            }),
        },
        McpTool {
            name: "pack_context".to_string(),
            description: "Pack context into token-efficient format for LLM prompts".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "description": { "type": "string", "description": "Description of what context is needed" },
                    "path": { "type": "string", "description": "Directory (default: .)" },
                    "budget": { "type": "integer", "description": "Token budget (default: 8000)" }
                },
                "required": ["description"]
            }),
        },
        McpTool {
            name: "trace_symbol".to_string(),
            description: "Trace execution flow through function calls".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Function name to trace" },
                    "path": { "type": "string", "description": "Directory (default: .)" },
                    "depth": { "type": "integer", "description": "Trace depth (default: 5)" }
                },
                "required": ["name"]
            }),
        },
        McpTool {
            name: "repo_stats".to_string(),
            description: "Get repository statistics (language breakdown, file counts)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory (default: .)" }
                },
                "required": []
            }),
        },
    ]
}

// ---------------------------------------------------------------------------
// MCP tool execution
// ---------------------------------------------------------------------------

fn execute_tool(name: &str, params: &serde_json::Value) -> Result<serde_json::Value, String> {
    match name {
        "search_files" => {
            let pattern = params["pattern"].as_str().ok_or("missing 'pattern'")?;
            let path = params["path"].as_str().unwrap_or(".");
            let extension = params["extension"].as_str();
            let limit = params["limit"].as_u64().map(|l| l as usize);

            let mut results = Vec::new();
            let _ = crate::file_search::collect_file_results_raw(pattern, path, None, extension, false, true, true, None, &mut results);
            if let Some(l) = limit { results.truncate(l); }

            let json_results: Vec<serde_json::Value> = results
                .iter()
                .map(|(filename, full_path, score)| serde_json::json!({
                    "filename": filename,
                    "path": full_path,
                    "score": score,
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "search_content" => {
            let pattern = params["pattern"].as_str().ok_or("missing 'pattern'")?;
            let path = params["path"].as_str().unwrap_or(".");
            let extension = params["extension"].as_str();
            let is_regex = params["regex"].as_bool().unwrap_or(false);
            let is_exact = params["exact"].as_bool().unwrap_or(false);
            let limit = params["limit"].as_u64().map(|l| l as usize);

            let mode = if is_regex {
                crate::types::MatchMode::Regex
            } else if is_exact {
                crate::types::MatchMode::Exact
            } else {
                crate::types::MatchMode::Fuzzy
            };

            let mut results = Vec::new();
            let _ = crate::content_search::collect_content_results_raw(pattern, path, extension, mode, None, true, true, 0, None, false, &mut results);
            if let Some(l) = limit { results.truncate(l); }

            let json_results: Vec<serde_json::Value> = results
                .iter()
                .map(|(file, path, line, content, score)| serde_json::json!({
                    "file": file,
                    "path": path,
                    "line": line,
                    "content": content,
                    "score": score,
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "find_symbol" => {
            let name = params["name"].as_str().ok_or("missing 'name'")?;
            let path = params["path"].as_str().unwrap_or(".");
            let symbol_type = params["symbol_type"].as_str();

            let result = crate::symbol::run_symbol(name, path, None, None, None, symbol_type, true, None, true)?;
            // The function already prints JSON, but in MCP mode we need to capture it.
            // For MCP, we'll re-collect the data.
            let symbols = crate::symbol::collect_symbol_results(name, path, None, None, None, symbol_type, true, None)?;
            let json_results: Vec<serde_json::Value> = symbols
                .iter()
                .map(|s| serde_json::json!({
                    "name": s.name,
                    "kind": s.kind.to_string(),
                    "file": s.file,
                    "line": s.line,
                    "language": s.language,
                    "signature": s.signature,
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "find_references" => {
            let name = params["name"].as_str().ok_or("missing 'name'")?;
            let path = params["path"].as_str().unwrap_or(".");

            let refs = crate::symbol::collect_ref_results(name, path, None, None, None, true, None)?;
            let json_results: Vec<serde_json::Value> = refs
                .iter()
                .map(|(path, line, content, lang)| serde_json::json!({
                    "file": path,
                    "line": line,
                    "content": content,
                    "language": lang,
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "find_callers" => {
            let name = params["name"].as_str().ok_or("missing 'name'")?;
            let path = params["path"].as_str().unwrap_or(".");

            let callers = crate::symbol::collect_caller_results(name, path, None, None, None, true, None)?;
            let json_results: Vec<serde_json::Value> = callers
                .iter()
                .map(|c| serde_json::json!({
                    "caller_name": c.0,
                    "caller_file": c.1,
                    "caller_line": c.2,
                    "call_site_line": c.3,
                    "call_context": c.4,
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "list_symbols" => {
            let path = params["path"].as_str().ok_or("missing 'path'")?;
            let symbol_type = params["symbol_type"].as_str();
            let limit = params["limit"].as_u64().map(|l| l as usize);

            let symbols = crate::symbol::collect_all_symbols(path, None, None, None, symbol_type, true, None, limit)?;
            let json_results: Vec<serde_json::Value> = symbols
                .iter()
                .map(|s| serde_json::json!({
                    "name": s.name,
                    "kind": s.kind.to_string(),
                    "file": s.file,
                    "line": s.line,
                    "language": s.language,
                    "signature": s.signature,
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "get_context" => {
            let topic = params["topic"].as_str().ok_or("missing 'topic'")?;
            let path = params["path"].as_str().unwrap_or(".");
            let extension = params["extension"].as_str();
            let limit = params["limit"].as_u64().map(|l| l as usize);

            let items = crate::context::collect_context_items(topic, path, None, extension, true, None, limit.unwrap_or(20))?;
            let json_results: Vec<serde_json::Value> = items
                .iter()
                .map(|item| serde_json::json!({
                    "source_type": "context".to_string(),
                    "file": "context_item".to_string(),
                    "line": None::<usize>,
                    "symbol_name": None::<String>,
                    "language": "context".to_string(),
                    "relevance": 0.0,
                    "snippet": String::new(),
                    "reason": String::new(),
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "pack_context" => {
            let description = params["description"].as_str().ok_or("missing 'description'")?;
            let path = params["path"].as_str().unwrap_or(".");
            let budget = params["budget"].as_u64().map(|l| l as usize);

            let pack_result = crate::context::collect_packed_context(description, path, None, None, true, None, budget)?;
            Ok(pack_result)
        }

        "trace_symbol" => {
            let name = params["name"].as_str().ok_or("missing 'name'")?;
            let path = params["path"].as_str().unwrap_or(".");
            let depth = params["depth"].as_u64().map(|l| l as usize);

            let trace = crate::context::collect_trace_steps(name, path, None, None, true, None, depth)?;
            let json_results: Vec<serde_json::Value> = trace
                .iter()
                .map(|step| serde_json::json!({
                    "step": step.step,
                    "name": step.name,
                    "kind": step.kind,
                    "file": step.file,
                    "line": step.line,
                    "signature": step.signature,
                    "depth": step.depth,
                }))
                .collect();

            Ok(serde_json::json!({ "results": json_results, "count": json_results.len() }))
        }

        "repo_stats" => {
            let path = params["path"].as_str().unwrap_or(".");
            let stats = crate::stats::collect_stats(path, None, None)?;

            Ok(serde_json::json!({
                "results": stats,
                "count": stats.len(),
            }))
        }

        _ => Err(format!("Unknown tool: {}", name)),
    }
}

// ---------------------------------------------------------------------------
// MCP Server (stdio)
// ---------------------------------------------------------------------------

/// Run MCP server over stdin/stdout (JSON-RPC 2.0).
fn run_mcp_server() {
    eprintln!("{}", "CodeScope MCP Server starting...".cyan());
    eprintln!("{}", "Communicating via JSON-RPC 2.0 on stdin/stdout".dimmed());
    eprintln!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let json_str = match serde_json::to_string(&response) {
                    Ok(s) => s,
                    Err(e) => format!("{{\"error\":\"serialization failed: {}\"}}", e),
                };
                let _ = writeln!(stdout, "{}", json_str);
                let _ = stdout.flush();
                continue;
            }
        };

        let response = handle_mcp_request(&request);
        let json_str = match serde_json::to_string(&response) {
            Ok(s) => s,
            Err(e) => format!("{{\"error\":\"serialization failed: {}\"}}", e),
        };
        let _ = writeln!(stdout, "{}", json_str);
        let _ = stdout.flush();
    }
}

fn handle_mcp_request(request: &JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => {
            let result = serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {},
                    "streaming": { "supported": true }
                },
                "serverInfo": {
                    "name": "codescope",
                    "version": env!("CARGO_PKG_VERSION")
                }
            });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            }
        }

        "notifications/initialized" => {
            // Acknowledgment notification, no response needed
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(serde_json::json!({})),
                error: None,
            }
        }

        "tools/list" => {
            let tools = get_mcp_tools();
            let result = serde_json::json!({ "tools": tools });
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            }
        }

        "tools/call" => {
            let params = match &request.params {
                Some(p) => p,
                None => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing params".to_string(),
                            data: None,
                        }),
                    };
                }
            };

            let tool_name = match params["name"].as_str() {
                Some(n) => n,
                None => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing tool name".to_string(),
                            data: None,
                        }),
                    };
                }
            };

            let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

            let tool_result = execute_tool(tool_name, &arguments);

            // Check for large results and emit streaming progress
            if let Ok(ref res) = tool_result {
                let res_str = serde_json::to_string(res).unwrap_or_default();
                if res_str.len() > STREAMING_THRESHOLD {
                    emit_streaming_progress(tool_name, res_str.len());
                }
            }

            match tool_result {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(result),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: e,
                        data: None,
                    }),
                },
            }
        }

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}

// ---------------------------------------------------------------------------
// HTTP API Server
// ---------------------------------------------------------------------------

/// Check if the request has `stream=true` query parameter.
fn is_streaming_request(query: &HashMap<String, String>) -> bool {
    query.get("stream").map(|v| v == "true" || v == "1").unwrap_or(false)
}

/// Build a streaming HTTP response (chunked transfer encoding).
/// Returns (response_header, body_chunks) where each chunk is a line of data.
fn build_streaming_response(status: u16, reason: &str, tool_result: &serde_json::Value) -> String {
    let result_str = serde_json::to_string_pretty(tool_result).unwrap_or_default();

    // Split large results into chunks of ~4KB
    let chunk_size = 4096;
    let mut chunks = Vec::new();
    if result_str.len() > chunk_size {
        let bytes = result_str.as_bytes();
        let mut offset = 0;
        while offset < bytes.len() {
            let end = std::cmp::min(offset + chunk_size, bytes.len());
            // Try to break at a newline
            let mut break_at = end;
            if end < bytes.len() {
                for i in (offset..end).rev() {
                    if bytes[i] == b'\n' {
                        break_at = i + 1;
                        break;
                    }
                }
            }
            let chunk_str = String::from_utf8_lossy(&bytes[offset..break_at]).to_string();
            let partial = offset + chunk_str.len() < bytes.len();
            chunks.push(serde_json::json!({
                "partial": partial,
                "data": chunk_str.trim_end(),
            }));
            offset += chunk_str.len();
        }
    } else {
        chunks.push(serde_json::json!({
            "partial": false,
            "data": result_str,
        }));
    }

    let body: String = chunks.iter()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/x-ndjson\r\nTransfer-Encoding: chunked\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}\r\n0\r\n\r\n",
        status, reason, body
    )
}

fn handle_http_request(request_line: &str, _headers: &[String], working_dir: &str) -> String {
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return http_response(400, "Bad Request", None);
    }

    let method = parts[0];
    let path = parts[1];

    // Only handle GET requests
    if method != "GET" {
        return http_response(405, "Method Not Allowed", Some(&json_error("Only GET requests are supported")));
    }

    // Parse query parameters
    let (endpoint, query) = parse_path_and_query(path);
    let streaming = is_streaming_request(&query);

    let response = match endpoint.as_str() {
        "/search" => {
            let q = query.get("q").or_else(|| query.get("query"));
            let q = match q {
                Some(v) => v.as_str(),
                None => return http_response(400, "Bad Request", Some(&json_error("Missing 'q' parameter"))),
            };
            let search_path = query.get("path").map(|v| v.as_str()).unwrap_or(working_dir);
            let ext = query.get("type").or_else(|| query.get("extension")).map(|v| v.as_str());
            let limit = query.get("limit").and_then(|v| v.parse::<usize>().ok());

            let mode = crate::types::MatchMode::Fuzzy;
            match execute_tool("search_content", &serde_json::json!({
                "pattern": q,
                "path": search_path,
                "extension": ext,
                "limit": limit,
            })) {
                Ok(result) => {
                    if streaming { build_streaming_response(200, "OK", &result) }
                    else { http_response(200, "OK", Some(&result.to_string())) }
                }
                Err(e) => http_response(500, "Internal Server Error", Some(&json_error(&e))),
            }
        }

        "/context" => {
            let q = query.get("q").or_else(|| query.get("query"));
            let q = match q {
                Some(v) => v.as_str(),
                None => return http_response(400, "Bad Request", Some(&json_error("Missing 'q' parameter"))),
            };
            let search_path = query.get("path").map(|v| v.as_str()).unwrap_or(working_dir);
            let budget = query.get("tokens").and_then(|v| v.parse::<usize>().ok());

            match execute_tool("pack_context", &serde_json::json!({
                "description": q,
                "path": search_path,
                "budget": budget,
            })) {
                Ok(result) => {
                    if streaming { build_streaming_response(200, "OK", &result) }
                    else { http_response(200, "OK", Some(&result.to_string())) }
                }
                Err(e) => http_response(500, "Internal Server Error", Some(&json_error(&e))),
            }
        }

        "/symbol" => {
            let name = query.get("name");
            let name = match name {
                Some(v) => v.as_str(),
                None => return http_response(400, "Bad Request", Some(&json_error("Missing 'name' parameter"))),
            };
            let search_path = query.get("path").map(|v| v.as_str()).unwrap_or(working_dir);

            match execute_tool("find_symbol", &serde_json::json!({
                "name": name,
                "path": search_path,
            })) {
                Ok(result) => {
                    if streaming { build_streaming_response(200, "OK", &result) }
                    else { http_response(200, "OK", Some(&result.to_string())) }
                }
                Err(e) => http_response(500, "Internal Server Error", Some(&json_error(&e))),
            }
        }

        "/trace" => {
            let symbol = query.get("symbol");
            let symbol = match symbol {
                Some(v) => v.as_str(),
                None => return http_response(400, "Bad Request", Some(&json_error("Missing 'symbol' parameter"))),
            };
            let search_path = query.get("path").map(|v| v.as_str()).unwrap_or(working_dir);
            let depth = query.get("depth").and_then(|v| v.parse::<usize>().ok());

            match execute_tool("trace_symbol", &serde_json::json!({
                "name": symbol,
                "path": search_path,
                "depth": depth,
            })) {
                Ok(result) => {
                    if streaming { build_streaming_response(200, "OK", &result) }
                    else { http_response(200, "OK", Some(&result.to_string())) }
                }
                Err(e) => http_response(500, "Internal Server Error", Some(&json_error(&e))),
            }
        }

        "/pack" => {
            let desc = query.get("description").or_else(|| query.get("q"));
            let desc = match desc {
                Some(v) => v.as_str(),
                None => return http_response(400, "Bad Request", Some(&json_error("Missing 'description' or 'q' parameter"))),
            };
            let search_path = query.get("path").map(|v| v.as_str()).unwrap_or(working_dir);
            let budget = query.get("budget").and_then(|v| v.parse::<usize>().ok());

            match execute_tool("pack_context", &serde_json::json!({
                "description": desc,
                "path": search_path,
                "budget": budget,
            })) {
                Ok(result) => {
                    if streaming { build_streaming_response(200, "OK", &result) }
                    else { http_response(200, "OK", Some(&result.to_string())) }
                }
                Err(e) => http_response(500, "Internal Server Error", Some(&json_error(&e))),
            }
        }

        "/stats" => {
            let search_path = query.get("path").map(|v| v.as_str()).unwrap_or(working_dir);

            match execute_tool("repo_stats", &serde_json::json!({
                "path": search_path,
            })) {
                Ok(result) => {
                    if streaming { build_streaming_response(200, "OK", &result) }
                    else { http_response(200, "OK", Some(&result.to_string())) }
                }
                Err(e) => http_response(500, "Internal Server Error", Some(&json_error(&e))),
            }
        }

        "/" | "/health" => {
            http_response(200, "OK", Some(&serde_json::json!({
                "tool": "codescope",
                "version": env!("CARGO_PKG_VERSION"),
                "status": "ok",
                "streaming": true,
                "endpoints": ["/search", "/context", "/symbol", "/trace", "/pack", "/stats", "/health"]
            }).to_string()))
        }

        _ => http_response(404, "Not Found", Some(&json_error("Endpoint not found"))),
    };

    response
}

fn parse_path_and_query(request_path: &str) -> (String, HashMap<String, String>) {
    let mut query = HashMap::new();

    if let Some((path, qs)) = request_path.split_once('?') {
        for pair in qs.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                query.insert(
                    urlencoding::decode(key).unwrap_or_default().to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string(),
                );
            }
        }
        (path.to_string(), query)
    } else {
        (request_path.to_string(), query)
    }
}

fn http_response(status: u16, reason: &str, body: Option<&str>) -> String {
    let body_str = body.unwrap_or("");
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        status,
        reason,
        body_str.len(),
        body_str,
    )
}

fn json_error(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

fn run_http_server(port: u16, working_dir: String) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{} Failed to bind to {}: {}", "Error:".red().bold(), addr, e);
            std::process::exit(1);
        }
    };

    eprintln!("{}", "CodeScope HTTP API Server".cyan().bold());
    eprintln!("{}", "─".repeat(50).dimmed());
    eprintln!("  {} http://{}", "Listening on:".green(), addr);
    eprintln!("  {}  {}", "Working dir:".dimmed(), working_dir);
    eprintln!();
    eprintln!("  {} Endpoints:", "Available:".bold());
    eprintln!("    {} /search?q=auth&type=rust", "GET".yellow());
    eprintln!("    {} /context?q=authentication&tokens=4000", "GET".yellow());
    eprintln!("    {} /symbol?name=UserService", "GET".yellow());
    eprintln!("    {} /trace?symbol=login_user&depth=5", "GET".yellow());
    eprintln!("    {} /pack?description=auth+bug&budget=8000", "GET".yellow());
    eprintln!("    {} /stats", "GET".yellow());
    eprintln!("    {} /health", "GET".yellow());
    eprintln!();
    eprintln!("  {} Press Ctrl+C to stop", "Tip:".yellow());
    eprintln!();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let wd = working_dir.clone();
                thread::spawn(move || {
                    use std::io::Read;
                    stream.set_nonblocking(false).ok();
                    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
                    stream.set_write_timeout(Some(std::time::Duration::from_secs(5))).ok();

                    // Read the entire request (up to 16KB) then parse
                    let mut buf = [0u8; 16384];
                    let n = match stream.read(&mut buf) {
                        Ok(0) => return,
                        Ok(n) => n,
                        Err(_) => return,
                    };

                    let raw = String::from_utf8_lossy(&buf[..n]);
                    let mut lines = raw.lines();
                    let request_line = lines.next().unwrap_or("").to_string();
                    let headers: Vec<String> = lines.take_while(|l| !l.is_empty()).map(|l| l.trim().to_string()).collect();

                    let response = handle_http_request(&request_line, &headers, &wd);
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                });
            }
            Err(e) => {
                eprintln!("{} Connection error: {}", "Error:".red().bold(), e);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the serve command.
pub fn run_serve(mcp: bool, http: bool, port: u16, path: &str) -> Result<(), String> {
    if mcp {
        run_mcp_server();
        Ok(())
    } else if http {
        let working_dir = std::path::absolute(path)
            .map_err(|e| format!("Invalid path '{}': {}", path, e))?
            .to_string_lossy()
            .to_string();
        run_http_server(port, working_dir);
        Ok(())
    } else {
        Err("Please specify --mcp or --http".to_string())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_response() {
        let response = http_response(200, "OK", Some(&json_error("test")));
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("application/json"));
    }

    #[test]
    fn test_parse_path_and_query() {
        let (path, query) = parse_path_and_query("/search?q=auth&type=rust");
        assert_eq!(path, "/search");
        assert_eq!(query.get("q").unwrap(), "auth");
        assert_eq!(query.get("type").unwrap(), "rust");
    }

    #[test]
    fn test_parse_path_no_query() {
        let (path, query) = parse_path_and_query("/health");
        assert_eq!(path, "/health");
        assert!(query.is_empty());
    }

    #[test]
    fn test_json_error() {
        let err = json_error("something went wrong");
        assert!(err.contains("something went wrong"));
        let parsed: serde_json::Value = serde_json::from_str(&err).unwrap();
        assert_eq!(parsed["error"], "something went wrong");
    }

    #[test]
    fn test_mcp_tools_non_empty() {
        let tools = get_mcp_tools();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "search_files"));
        assert!(tools.iter().any(|t| t.name == "get_context"));
        assert!(tools.iter().any(|t| t.name == "pack_context"));
    }

    #[test]
    fn test_mcp_tool_schemas_valid() {
        let tools = get_mcp_tools();
        for tool in &tools {
            let schema = &tool.input_schema;
            assert_eq!(schema["type"], "object");
        }
    }

    #[test]
    fn test_handle_mcp_request_initialize() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: None,
        };
        let response = handle_mcp_request(&request);
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], "2024-11-05");
        assert_eq!(result["serverInfo"]["name"], "codescope");
    }

    #[test]
    fn test_handle_mcp_request_tools_list() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };
        let response = handle_mcp_request(&request);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result["tools"].as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_handle_mcp_request_unknown_method() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(3)),
            method: "nonexistent".to_string(),
            params: None,
        };
        let response = handle_mcp_request(&request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[test]
    fn test_mcp_initialize_includes_streaming_capability() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(42)),
            method: "initialize".to_string(),
            params: None,
        };
        let response = handle_mcp_request(&request);
        let result = response.result.unwrap();
        assert_eq!(result["capabilities"]["streaming"]["supported"], true);
    }

    #[test]
    fn test_is_streaming_request() {
        let mut q = HashMap::new();
        assert!(!is_streaming_request(&q));
        q.insert("stream".to_string(), "true".to_string());
        assert!(is_streaming_request(&q));
        q.insert("stream".to_string(), "1".to_string());
        assert!(is_streaming_request(&q));
        q.insert("stream".to_string(), "false".to_string());
        assert!(!is_streaming_request(&q));
    }

    #[test]
    fn test_build_streaming_response_small() {
        let data = serde_json::json!({"results": [], "count": 0});
        let response = build_streaming_response(200, "OK", &data);
        assert!(response.contains("Transfer-Encoding: chunked"));
        assert!(response.contains("application/x-ndjson"));
        assert!(response.contains("\"partial\":false"));
    }

    #[test]
    fn test_build_streaming_response_large() {
        // Build a large result > 4KB to trigger chunking
        let big_array: Vec<String> = (0..500).map(|i| format!("line {}: padding to make this result very long so it exceeds the chunk size", i)).collect();
        let data = serde_json::json!({"results": big_array, "count": 500});
        let response = build_streaming_response(200, "OK", &data);
        assert!(response.contains("Transfer-Encoding: chunked"));
        // Should have multiple partial chunks
        let partial_count = response.matches("\"partial\":true").count();
        assert!(partial_count > 0, "Expected at least one partial chunk for large response");
        // Last chunk should be partial:false
        assert!(response.contains("\"partial\":false"));
    }

    #[test]
    fn test_streaming_threshold_constant() {
        assert_eq!(STREAMING_THRESHOLD, 10_240);
    }
}
