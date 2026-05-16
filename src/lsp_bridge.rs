//! Lightweight LSP bridge for editor integration.
//!
//! Feature Request #2: "Lightweight LSP bridge for editor integration"
//!
//! Listens on a TCP port and translates basic LSP protocol requests into
//! CodeScope (`cs`) CLI commands, returning the results as LSP responses.
//!
//! Supported requests:
//! - `initialize` → server capabilities
//! - `textDocument/completion` → `cs content` (symbol completions)
//! - `textDocument/definition` → `cs where` (go-to-definition)
//! - `textDocument/references` → `cs refs` (find-references)
//! - `textDocument/hover` → `cs context` (hover information)
//! - `textDocument/documentSymbol` → `cs where` (file symbols)
//!
//! # Usage
//!
//! ```sh
//! cs lsp-bridge --port 8765
//! ```
//!
//! Then configure your editor to connect to `localhost:8765` via TCP transport.

use colored::Colorize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

// ────────────────────────────────────────────────────────────────────────────
// Public entry point
// ────────────────────────────────────────────────────────────────────────────

/// Start the LSP bridge TCP server and block until interrupted.
pub fn run_lsp_bridge(port: u16) -> Result<(), String> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

    eprintln!(
        "{} CodeScope LSP Bridge listening on {}",
        "✓".green().bold(),
        addr.cyan()
    );
    eprintln!(
        "  {} Press Ctrl+C to stop",
        "Tip:".yellow()
    );
    eprintln!(
        "  {} Connect your editor's LSP client via TCP transport",
        "Info:".dimmed()
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                eprintln!(
                    "{} Client connected from {}",
                    ">>".cyan(),
                    stream
                        .peer_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| "unknown".to_string())
                );
                if let Err(e) = handle_connection(stream) {
                    eprintln!("{} Connection error: {}", "✗".red(), e);
                }
                eprintln!("{} Client disconnected", ">>".dimmed());
            }
            Err(e) => {
                eprintln!("{} Accept error: {}", "✗".red(), e);
            }
        }
    }

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Connection handling
// ────────────────────────────────────────────────────────────────────────────

/// Handle a single client connection. Reads LSP messages, dispatches them,
/// and writes LSP responses until the client disconnects.
fn handle_connection(mut stream: std::net::TcpStream) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);

    loop {
        // Read the next LSP message (or return on EOF)
        let raw = match read_lsp_message(&mut reader) {
            Ok(Some(body)) => body,
            Ok(None) => {
                // Client disconnected gracefully
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        // Parse the JSON-RPC message
        let request: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{} Failed to parse LSP message: {}", "✗".red(), e);
                continue;
            }
        };

        let response = handle_request(&request);

        // Write the response back to the client
        write_lsp_message(&mut stream, &response)?;
    }
}

// ────────────────────────────────────────────────────────────────────────────
// LSP protocol I/O
// ────────────────────────────────────────────────────────────────────────────

/// Read a single LSP message from the stream.
///
/// LSP messages are framed as:
/// ```text
/// Content-Length: <length>\r\n
/// \r\n
/// <JSON body>
/// ```
fn read_lsp_message<R: BufRead>(reader: &mut R) -> Result<Option<String>, String> {
    let mut content_length: Option<usize> = None;
    let mut header_lines = Vec::new();

    // Read headers until empty line
    loop {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|e| format!("Read error: {}", e))?;

        if bytes_read == 0 {
            // EOF
            return Ok(None);
        }

        let trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
        if trimmed.is_empty() {
            break;
        }

        header_lines.push(trimmed.to_string());
        if trimmed.to_lowercase().starts_with("content-length:") {
            let len_str = trimmed["content-length:".len()..].trim();
            content_length = len_str.parse::<usize>().ok();
        }
    }

    let length = content_length.ok_or("Missing Content-Length header in LSP message")?;

    // Read exactly `length` bytes
    let mut body = vec![0u8; length];
    reader
        .read_exact(&mut body)
        .map_err(|e| format!("Failed to read message body: {}", e))?;

    Ok(Some(String::from_utf8_lossy(&body).to_string()))
}

/// Write an LSP message (response) to the stream.
fn write_lsp_message<W: Write>(writer: &mut W, value: &Value) -> Result<(), String> {
    let body = serde_json::to_string(value).map_err(|e| format!("Failed to serialize response: {}", e))?;

    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer
        .write_all(header.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;
    writer
        .write_all(body.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;
    writer
        .flush()
        .map_err(|e| format!("Flush error: {}", e))?;

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// Request dispatch
// ────────────────────────────────────────────────────────────────────────────

/// Dispatch an incoming JSON-RPC request and return the response.
fn handle_request(request: &Value) -> Value {
    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    let id = request.get("id").cloned();

    eprintln!("{} LSP request: {}", ">>".cyan(), method);

    let result = match method {
        "initialize" => handle_initialize(request),
        "initialized" => {
            // No response needed for the initialized notification
            return json!(null);
        }
        "textDocument/completion" => handle_completion(request),
        "textDocument/definition" => handle_definition(request),
        "textDocument/references" => handle_references(request),
        "textDocument/hover" => handle_hover(request),
        "textDocument/documentSymbol" => handle_document_symbol(request),
        "shutdown" => json!(null),
        _ => {
            // Unknown method — return an error
            let params = request.get("params").cloned().unwrap_or(json!(null));
            return make_error_response(
                id,
                -32601,
                &format!("Method not found: {}", method),
                &params,
            );
        }
    };

    make_success_response(id, result)
}

/// Build a successful JSON-RPC response.
fn make_success_response(id: Option<Value>, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

/// Build a JSON-RPC error response.
fn make_error_response(id: Option<Value>, code: i64, message: &str, data: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
            "data": data,
        }
    })
}

// ────────────────────────────────────────────────────────────────────────────
// LSP request handlers
// ────────────────────────────────────────────────────────────────────────────

/// Handle the `initialize` request — return server capabilities.
fn handle_initialize(_request: &Value) -> Value {
    json!({
        "capabilities": {
            "completionProvider": {
                "resolveProvider": false,
                "triggerCharacters": [".", ":"]
            },
            "definitionProvider": true,
            "referencesProvider": true,
            "hoverProvider": true,
            "documentSymbolProvider": true,
            "textDocumentSync": {
                "openClose": true,
                "change": 1
            }
        },
        "serverInfo": {
            "name": "codescope-lsp-bridge",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

/// Handle `textDocument/completion` — use `cs content` for symbol completions.
fn handle_completion(request: &Value) -> Value {
    let (file_path, _position, word) = extract_text_document_params(request);

    // Try to find the word under cursor; if empty, return empty list
    if word.is_empty() {
        return json!({ "isIncomplete": false, "items": [] });
    }

    // Use `cs content` to find symbols matching the prefix
    let dir = std::path::Path::new(&file_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let output = run_cs_command(&[
        "content",
        &word,
        "--path",
        &dir,
        "--limit",
        "10",
        "--json",
    ]);

    let mut items: Vec<Value> = Vec::new();

    if let Ok(output) = &output {
        if let Ok(json_val) = serde_json::from_str::<Value>(output) {
            if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                for result in results {
                    let content = result
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");
                    let line = result
                        .get("line")
                        .and_then(|l| l.as_u64())
                        .unwrap_or(0);
                    // Extract a reasonable label from the content line
                    let label = extract_label_from_line(content);
                    items.push(json!({
                        "label": label,
                        "kind": 6,  // Field / Variable
                        "detail": format!("line {}", line),
                        "data": { "file": file_path, "line": line },
                    }));
                }
            }
        }
    }

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

/// Handle `textDocument/definition` — use `cs where` for go-to-definition.
fn handle_definition(request: &Value) -> Value {
    let (file_path, _position, word) = extract_text_document_params(request);

    if word.is_empty() {
        return json!(null);
    }

    let dir = std::path::Path::new(&file_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let output = run_cs_command(&[
        "where",
        &word,
        "--path",
        &dir,
        "--json",
    ]);

    if let Ok(output) = &output {
        if let Ok(json_val) = serde_json::from_str::<Value>(output) {
            if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                if let Some(first) = results.first() {
                    let path = first
                        .get("path")
                        .and_then(|p| p.as_str())
                        .unwrap_or(&file_path);
                    let line = first
                        .get("line")
                        .and_then(|l| l.as_u64())
                        .unwrap_or(1);
                    return json!({
                        "uri": path_to_uri(path, &dir),
                        "range": {
                            "start": { "line": line.saturating_sub(1), "character": 0 },
                            "end": { "line": line.saturating_sub(1), "character": 100 }
                        }
                    });
                }
            }
        }
    }

    json!(null)
}

/// Handle `textDocument/references` — use `cs content` for find-references.
fn handle_references(request: &Value) -> Value {
    let (file_path, _position, word) = extract_text_document_params(request);

    if word.is_empty() {
        return json!([]);
    }

    let dir = std::path::Path::new(&file_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let output = run_cs_command(&[
        "content",
        &word,
        "--path",
        &dir,
        "--exact",
        "--limit",
        "50",
        "--json",
    ]);

    let mut locations: Vec<Value> = Vec::new();

    if let Ok(output) = &output {
        if let Ok(json_val) = serde_json::from_str::<Value>(output) {
            if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                for result in results {
                    let path = result
                        .get("path")
                        .and_then(|p| p.as_str())
                        .unwrap_or(&file_path);
                    let line = result
                        .get("line")
                        .and_then(|l| l.as_u64())
                        .unwrap_or(1);
                    let content = result
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");
                    let end_char = content.len() as u64;

                    locations.push(json!({
                        "uri": path_to_uri(path, &dir),
                        "range": {
                            "start": { "line": line.saturating_sub(1), "character": 0 },
                            "end": { "line": line.saturating_sub(1), "character": end_char }
                        }
                    }));
                }
            }
        }
    }

    json!(locations)
}

/// Handle `textDocument/hover` — use `cs content` with context for hover information.
fn handle_hover(request: &Value) -> Value {
    let (file_path, position, word) = extract_text_document_params(request);

    if word.is_empty() {
        return json!(null);
    }

    let dir = std::path::Path::new(&file_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // First try `cs where` for definition info
    let def_output = run_cs_command(&["where", &word, "--path", &dir, "--json"]);

    // Also search content for usage context
    let content_output = run_cs_command(&[
        "content",
        &word,
        "--path",
        &dir,
        "--limit",
        "5",
        "--json",
    ]);

    let mut hover_parts: Vec<String> = Vec::new();

    if let Ok(output) = &def_output {
        if let Ok(json_val) = serde_json::from_str::<Value>(output) {
            if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                if let Some(first) = results.first() {
                    let content = first
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");
                    let lang = first
                        .get("language")
                        .and_then(|l| l.as_str())
                        .unwrap_or("unknown");
                    let path = first
                        .get("path")
                        .and_then(|p| p.as_str())
                        .unwrap_or("");
                    hover_parts.push(format!("**Definition** ({})\n`{}`\n`{}`", lang, content, path));
                }
            }
        }
    }

    if let Ok(output) = &content_output {
        if let Ok(json_val) = serde_json::from_str::<Value>(output) {
            if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                if !results.is_empty() {
                    let usages: Vec<String> = results
                        .iter()
                        .take(5)
                        .filter_map(|r| {
                            let path = r.get("path")?.as_str()?;
                            let line = r.get("line")?.as_u64()?;
                            let content = r.get("content")?.as_str()?;
                            Some(format!("  `{}:{}  {}`", path, line, content))
                        })
                        .collect();
                    if !usages.is_empty() {
                        hover_parts.push(format!("**Usages** ({}):\n{}", results.len(), usages.join("\n")));
                    }
                }
            }
        }
    }

    if hover_parts.is_empty() {
        return json!(null);
    }

    let hover_text = hover_parts.join("\n\n");

    json!({
        "contents": {
            "kind": "markdown",
            "value": hover_text,
        },
        "range": {
            "start": { "line": position.0, "character": position.1 },
            "end": { "line": position.0, "character": position.1 + (word.len() as u64) }
        }
    })
}

/// Handle `textDocument/documentSymbol` — use `cs where` for file symbols.
fn handle_document_symbol(request: &Value) -> Value {
    let (file_path, _position, _word) = extract_text_document_params(request);

    // Search for all definitions in the specific file
    let dir = std::path::Path::new(&file_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    let output = run_cs_command(&["where", ".", "--path", &dir, "--json"]);

    let mut symbols: Vec<Value> = Vec::new();

    if let Ok(output) = &output {
        if let Ok(json_val) = serde_json::from_str::<Value>(output) {
            if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                for result in results {
                    let path = result
                        .get("path")
                        .and_then(|p| p.as_str())
                        .unwrap_or("");
                    let line = result
                        .get("line")
                        .and_then(|l| l.as_u64())
                        .unwrap_or(1);
                    let content = result
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");
                    let lang = result
                        .get("language")
                        .and_then(|l| l.as_str())
                        .unwrap_or("");

                    // Only include symbols from the requested file
                    let fn_part = file_path.rsplit(|c| c == '/' || c == '\\').next();
                    let pn_part = path.rsplit(|c| c == '/' || c == '\\').next();
                    let same_filename = fn_part.zip(pn_part).map(|(a, b)| a == b).unwrap_or(false);
                    if !path.contains(&file_path)
                        && !file_path.contains(path)
                        && !same_filename
                    {
                        continue;
                    }

                    let (name, kind) = parse_symbol_name_and_kind(content, lang);
                    symbols.push(json!({
                        "name": name,
                        "kind": kind,
                        "detail": content,
                        "location": {
                            "uri": path_to_uri(path, &dir),
                            "range": {
                                "start": { "line": line.saturating_sub(1), "character": 0 },
                                "end": { "line": line.saturating_sub(1), "character": content.len() as u64 }
                            }
                        }
                    }));
                }
            }
        }
    }

    json!(symbols)
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Extract common text document parameters from an LSP request:
/// (file_path, (line, character), word_under_cursor)
fn extract_text_document_params(request: &Value) -> (String, (u64, u64), String) {
    let empty_params = json!({});
    let params = request.get("params").unwrap_or(&empty_params);

    // File path from textDocument.uri
    let uri = params
        .get("textDocument")
        .and_then(|td| td.get("uri"))
        .and_then(|u| u.as_str())
        .unwrap_or("file:///untitled");
    let file_path = uri_to_path(uri);

    // Position
    let default_position = json!({"line": 0, "character": 0});
    let position = params.get("position").unwrap_or(&default_position);
    let line = position
        .get("line")
        .and_then(|l| l.as_u64())
        .unwrap_or(0);
    let character = position
        .get("character")
        .and_then(|c| c.as_u64())
        .unwrap_or(0);

    // Context: if available, extract the word near the cursor
    let word = params
        .get("context")
        .and_then(|c| c.get("triggerCharacter"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    let word = if word.is_empty() {
        // Try to extract a word from the position using textDocument content
        params
            .get("textDocument")
            .and_then(|td| td.get("text"))
            .and_then(|t| t.as_str())
            .map(|text| extract_word_at_position(text, line as usize, character as usize))
            .unwrap_or_default()
    } else {
        word.to_string()
    };

    (file_path, (line, character), word)
}

/// Extract a word at the given cursor position from text.
fn extract_word_at_position(text: &str, line: usize, col: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if line >= lines.len() {
        return String::new();
    }

    let line_str = lines[line];
    let col = col.min(line_str.len());

    // Find word boundaries
    let start = line_str[..col]
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_alphanumeric() && *c != '_' && *c != ':')
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    let end = line_str[col..]
        .char_indices()
        .find(|(_, c)| !c.is_alphanumeric() && *c != '_' && *c != ':')
        .map(|(i, _)| col + i)
        .unwrap_or(line_str.len());

    line_str[start..end].to_string()
}

/// Extract a reasonable completion label from a code line.
fn extract_label_from_line(line: &str) -> String {
    let trimmed = line.trim();

    // Try to extract the function/struct name
    let patterns = [
        r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)",
        r"(?:pub\s+)?struct\s+(\w+)",
        r"(?:pub\s+)?enum\s+(\w+)",
        r"(?:pub\s+)?trait\s+(\w+)",
        r"(?:pub\s+)?type\s+(\w+)",
        r"(?:pub\s+)?mod\s+(\w+)",
        r"(?:pub\s+)?(?:const|static)\s+(\w+)",
        r"let\s+(?:mut\s+)?(\w+)",
        r"(?:fn|class|def)\s+(\w+)",
        r"(\w+)\s*[:=]",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(cap) = re.captures(trimmed) {
                if let Some(m) = cap.get(1) {
                    return m.as_str().to_string();
                }
            }
        }
    }

    // Fallback: use the first word or first N chars
    trimmed
        .split_whitespace()
        .next()
        .unwrap_or(trimmed)
        .chars()
        .take(40)
        .collect()
}

/// Parse a symbol name and LSP SymbolKind from a definition line.
fn parse_symbol_name_and_kind(content: &str, lang: &str) -> (String, u64) {
    let trimmed = content.trim();

    // Rust
    if lang == "rust" || trimmed.contains("fn ") || trimmed.contains("struct ") {
        if trimmed.contains("fn ") {
            if let Some(name) = extract_identifier_after(trimmed, "fn ") {
                let kind = if trimmed.contains("pub fn") { 6 } else { 6 }; // Function
                return (name, kind);
            }
        }
        if trimmed.contains("struct ") {
            if let Some(name) = extract_identifier_after(trimmed, "struct ") {
                return (name, 23); // Struct
            }
        }
        if trimmed.contains("enum ") {
            if let Some(name) = extract_identifier_after(trimmed, "enum ") {
                return (name, 23); // Enum
            }
        }
        if trimmed.contains("trait ") {
            if let Some(name) = extract_identifier_after(trimmed, "trait ") {
                return (name, 11); // Interface (trait)
            }
        }
        if trimmed.contains("mod ") {
            if let Some(name) = extract_identifier_after(trimmed, "mod ") {
                return (name, 2); // Module
            }
        }
        if trimmed.contains("impl ") {
            if let Some(name) = extract_identifier_after(trimmed, "impl ") {
                return (name, 8); // Class (impl block)
            }
        }
        if trimmed.contains("const ") || trimmed.contains("static ") {
            if let Some(name) = extract_identifier_after(trimmed, "const ")
                .or_else(|| extract_identifier_after(trimmed, "static "))
            {
                return (name, 14); // Constant
            }
        }
    }

    // Python
    if lang == "python" {
        if trimmed.contains("def ") {
            if let Some(name) = extract_identifier_after(trimmed, "def ") {
                return (name, 6); // Function
            }
        }
        if trimmed.contains("class ") {
            if let Some(name) = extract_identifier_after(trimmed, "class ") {
                return (name, 5); // Class
            }
        }
    }

    // Fallback: extract first identifier-like token
    let first_word = trimmed
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .find(|s| !s.is_empty())
        .unwrap_or("unknown");

    (first_word.to_string(), 13) // Variable as fallback
}

/// Extract the first identifier after a keyword.
fn extract_identifier_after(text: &str, keyword: &str) -> Option<String> {
    let pos = text.find(keyword)?;
    let rest = &text[pos + keyword.len()..];
    let id: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

/// Convert a file URI to a filesystem path.
fn uri_to_path(uri: &str) -> String {
    if uri.starts_with("file://") {
        urlencoding::decode(&uri[7..])
            .map(|s| s.to_string())
            .unwrap_or_else(|_| uri[7..].to_string())
    } else {
        uri.to_string()
    }
}

/// Convert a filesystem path to a file URI.
fn path_to_uri(path: &str, base_dir: &str) -> String {
    let abs = if std::path::Path::new(path).is_absolute() {
        path.to_string()
    } else {
        format!("{}/{}", base_dir.trim_end_matches('/'), path.trim_start_matches("./"))
    };
    format!("file://{}", abs)
}

/// Run a `cs` command as a subprocess and return its stdout output.
fn run_cs_command(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("cs")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run cs {:?}: {}", args, e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!(
            "cs {:?} exited with {:?}",
            args,
            output.status.code()
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_lsp_message() {
        let body = r#"{"jsonrpc":"2.0","method":"test"}"#;
        let input = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let mut reader = BufReader::new(input.as_bytes());

        let result = read_lsp_message(&mut reader).unwrap();
        assert!(result.is_some());
        let body_out = result.unwrap();
        let parsed: Value = serde_json::from_str(&body_out).unwrap();
        assert_eq!(parsed["method"], "test");
    }

    #[test]
    fn test_read_lsp_message_eof() {
        let input = "";
        let mut reader = BufReader::new(input.as_bytes());

        let result = read_lsp_message(&mut reader).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_write_lsp_message() {
        let value = json!({"jsonrpc": "2.0", "id": 1, "result": null});
        let mut buf = Vec::new();
        write_lsp_message(&mut buf, &value).unwrap();

        let output = String::from_utf8_lossy(&buf);
        assert!(output.contains("Content-Length:"));
        assert!(output.contains("jsonrpc"));
    }

    #[test]
    fn test_handle_initialize() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": 12345,
                "rootUri": "file:///tmp/project"
            }
        });

        let response = handle_request(&request);
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);

        let caps = &response["result"]["capabilities"];
        assert!(caps["definitionProvider"].as_bool().unwrap_or(false));
        assert!(caps["hoverProvider"].as_bool().unwrap_or(false));
        assert!(caps["referencesProvider"].as_bool().unwrap_or(false));
        assert!(caps["documentSymbolProvider"].as_bool().unwrap_or(false));
        assert!(caps["completionProvider"].is_object());
    }

    #[test]
    fn test_handle_unknown_method() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "textDocument/nonexistent",
        });

        let response = handle_request(&request);
        assert!(response.get("error").is_some());
        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn test_extract_word_at_position() {
        let text = "fn hello_world() {\n    let x = 42;\n}\n";
        assert_eq!(extract_word_at_position(text, 0, 5), "hello_world");
        assert_eq!(extract_word_at_position(text, 1, 8), "x");
        assert_eq!(extract_word_at_position(text, 0, 0), "fn");
    }

    #[test]
    fn test_extract_word_at_position_edge() {
        let text = "hello";
        assert_eq!(extract_word_at_position(text, 0, 0), "hello");
        assert_eq!(extract_word_at_position(text, 0, 5), "hello");
        assert_eq!(extract_word_at_position(text, 5, 0), ""); // out of bounds line
    }

    #[test]
    fn test_extract_label_from_line() {
        assert_eq!(extract_label_from_line("pub fn search_files() {"), "search_files");
        assert_eq!(extract_label_from_line("struct Config {"), "Config");
        assert_eq!(extract_label_from_line("    let mut results: Vec<String> = Vec::new();"), "results");
    }

    #[test]
    fn test_parse_symbol_name_and_kind() {
        assert_eq!(parse_symbol_name_and_kind("pub fn main() {}", "rust"), ("main".to_string(), 6));
        assert_eq!(parse_symbol_name_and_kind("struct Config {}", "rust"), ("Config".to_string(), 23));
        assert_eq!(parse_symbol_name_and_kind("def hello():", "python"), ("hello".to_string(), 6));
        assert_eq!(parse_symbol_name_and_kind("class MyClass:", "python"), ("MyClass".to_string(), 5));
    }

    #[test]
    fn test_uri_to_path() {
        assert_eq!(uri_to_path("file:///tmp/project/src/main.rs"), "/tmp/project/src/main.rs");
        assert_eq!(uri_to_path("/tmp/file.rs"), "/tmp/file.rs");
    }

    #[test]
    fn test_path_to_uri() {
        let uri = path_to_uri("/tmp/file.rs", "/tmp");
        assert_eq!(uri, "file:///tmp/file.rs");
        // Relative path with base dir
        let uri_rel = path_to_uri("file.rs", "/tmp");
        assert_eq!(uri_rel, "file:///tmp/file.rs");
    }

    #[test]
    fn test_make_success_response() {
        let resp = make_success_response(Some(json!(1)), json!("hello"));
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"], "hello");
    }

    #[test]
    fn test_make_error_response() {
        let resp = make_error_response(Some(json!(1)), -32601, "not found", &json!(null));
        assert_eq!(resp["error"]["code"], -32601);
        assert_eq!(resp["error"]["message"], "not found");
    }

    #[test]
    fn test_handle_initialized_notification() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });

        let response = handle_request(&request);
        assert_eq!(response, json!(null));
    }

    #[test]
    fn test_extract_identifier_after() {
        assert_eq!(extract_identifier_after("pub fn search_files(", "fn "), Some("search_files".to_string()));
        assert_eq!(extract_identifier_after("struct Config {", "struct "), Some("Config".to_string()));
        assert_eq!(extract_identifier_after("impl Foo {", "impl "), Some("Foo".to_string()));
        assert_eq!(extract_identifier_after("", "fn "), None);
    }

    #[test]
    fn test_handle_document_symbol_request_format() {
        // Just verify that the handler returns proper structure
        let request = json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "textDocument/documentSymbol",
            "params": {
                "textDocument": { "uri": "file:///tmp/test.rs" },
                "position": { "line": 0, "character": 0 }
            }
        });

        let response = handle_request(&request);
        assert_eq!(response["id"], 10);
        // Result should be an array (even if empty since cs might not be available in test)
        assert!(response["result"].is_array());
    }

    #[test]
    fn test_handle_hover_request_format() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": "file:///tmp/test.rs", "text": "fn foo() {}\n" },
                "position": { "line": 0, "character": 3 },
                "context": { "triggerCharacter": "f" }
            }
        });

        let response = handle_request(&request);
        assert_eq!(response["id"], 11);
        // Result could be null if cs command fails, which is acceptable in test
    }
}
