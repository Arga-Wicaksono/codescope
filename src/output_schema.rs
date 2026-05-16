//! Structured JSON output schema for all codescope commands.
//!
//! This module defines the **canonical** JSON envelope and per-command result
//! item structs. Every command's `--json` output MUST go through the
//! [`envelope`] function so that field names and the overall shape stay
//! consistent forever.
//!
//! # Stability contract
//!
//! Field names in the envelope and result items are FIXED. Renaming or
//! removing a field is a breaking change and requires a major version bump.

use serde::Serialize;

// ---------------------------------------------------------------------------
// Per-command result item structs
// ---------------------------------------------------------------------------

/// Result item for `cs file`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FileResultItem {
    pub filename: String,
    pub path: String,
    pub score: i64,
    pub extension: String,
    pub size_bytes: u64,
}

/// Result item for `cs content`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentResultItem {
    pub file: String,
    pub path: String,
    pub line: usize,
    pub content: String,
    pub score: i64,
    pub language: Option<String>,
}

/// Result item for `cs content --replace`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ReplaceResultItem {
    pub file: String,
    pub path: String,
    pub line: usize,
    pub old: String,
    #[serde(rename = "new")]
    pub new_val: String,
}

/// Result item for `cs content --count`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CountResultItem {
    pub file: String,
    pub path: String,
    pub count: usize,
}

/// Result item for `cs web`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WebResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Result item for `cs where`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WhereResultItem {
    pub path: String,
    pub line: usize,
    pub content: String,
    pub language: String,
    pub kind: String,
}

/// Result item for `cs stats`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StatsResultItem {
    pub language: String,
    pub files: usize,
    pub lines: usize,
    pub bytes: u64,
    pub percentage: f64,
}

/// Result item for `cs recent`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RecentResultItem {
    pub path: String,
    pub modified: String,
    pub size_bytes: u64,
    pub extension: String,
}

/// Result item for `cs across`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AcrossResultItem {
    pub repo: String,
    pub file: String,
    pub path: String,
    pub line: usize,
    pub content: String,
}

/// Result item for `cs open`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenResultItem {
    pub file: String,
    pub path: String,
    pub score: i64,
    pub line: Option<usize>,
}

/// Result item for `cs explain`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ExplainResultItem {
    pub token: String,
    pub description: String,
}

/// Result item for `cs history`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct HistoryResultItem {
    pub timestamp: String,
    pub command: String,
    pub pattern: String,
    pub path: String,
    pub results: usize,
    pub elapsed_secs: f64,
}

// ---------------------------------------------------------------------------
// Envelope builder
// ---------------------------------------------------------------------------

/// Build the standard JSON envelope that wraps every command's output.
///
/// Every codescope command MUST call this function (or one of the thin
/// wrappers below) so the output shape stays consistent.
///
/// # Arguments
///
/// * `command`   – The sub-command name (`"file"`, `"content"`, `"web"`, …).
/// * `query`     – The search term / description supplied by the user.
/// * `source`    – Where the data came from: `"filesystem"`, `"stdin"`, `"web"`.
/// * `count`     – Number of items in the `results` array.
/// * `elapsed`   – Wall-clock seconds the command took.
/// * `results`   – A `serde_json::Value` (typically an array of serialisable items).
pub fn envelope(
    command: &str,
    query: &str,
    source: &str,
    count: usize,
    elapsed: f64,
    results: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "tool": "codescope",
        "version": env!("CARGO_PKG_VERSION"),
        "command": command,
        "query": query,
        "source": source,
        "count": count,
        "elapsed_secs": elapsed,
        "results": results,
    })
}

/// Build a standard envelope **with extra root-level fields**.
///
/// Some commands need additional scalar fields next to the standard envelope
/// keys (e.g. `cs stats` adds `total_files` / `total_lines`,
/// `cs content --replace` adds `dry_run`).
///
/// The `extra` map is merged into the envelope **after** the standard fields,
/// so it can override nothing — it only adds.
pub fn envelope_with_extra(
    command: &str,
    query: &str,
    source: &str,
    count: usize,
    elapsed: f64,
    results: serde_json::Value,
    extra: serde_json::Value,
) -> serde_json::Value {
    let mut env = serde_json::json!({
        "tool": "codescope",
        "version": env!("CARGO_PKG_VERSION"),
        "command": command,
        "query": query,
        "source": source,
        "count": count,
        "elapsed_secs": elapsed,
        "results": results,
    });

    // Merge extra key-value pairs into the envelope.
    if let serde_json::Value::Object(map) = extra {
        if let serde_json::Value::Object(env_map) = &mut env {
            env_map.extend(map);
        }
    }

    env
}

// ---------------------------------------------------------------------------
// Pretty-print helper
// ---------------------------------------------------------------------------

/// Print a `serde_json::Value` as pretty JSON to stdout.
pub fn print_json(output: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(output).unwrap());
}

// ---------------------------------------------------------------------------
// Embedded JSON Schemas
// ---------------------------------------------------------------------------
//
// Each schema is stored as a `const &str` containing a draft-07 JSON Schema
// that describes the full envelope + command-specific result items.
//
// The public [`get_schema`] function looks up the right one by command name.

/// Return the JSON Schema for a given command name, or `None` if unknown.
///
/// Supported names: `"file"`, `"content"`, `"content-replace"`,
/// `"content-count"`, `"web"`, `"where"`, `"stats"`, `"recent"`, `"across"`,
/// `"open"`, `"explain"`, `"history"`.
pub fn get_schema(command: &str) -> Option<serde_json::Value> {
    let raw = match command {
        "file" => SCHEMA_FILE,
        "content" => SCHEMA_CONTENT,
        "content-replace" => SCHEMA_CONTENT_REPLACE,
        "content-count" => SCHEMA_CONTENT_COUNT,
        "web" => SCHEMA_WEB,
        "where" => SCHEMA_WHERE,
        "stats" => SCHEMA_STATS,
        "recent" => SCHEMA_RECENT,
        "across" => SCHEMA_ACROSS,
        "open" => SCHEMA_OPEN,
        "explain" => SCHEMA_EXPLAIN,
        "history" => SCHEMA_HISTORY,
        _ => return None,
    };
    Some(serde_json::from_str(raw).expect("embedded schema must be valid JSON"))
}

// ---- individual schemas as const strings --------------------------------

const SCHEMA_FILE: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs file output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "file" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["filename","path","score","extension","size_bytes"],
        "properties": {
          "filename":    { "type": "string" },
          "path":        { "type": "string" },
          "score":       { "type": "integer" },
          "extension":   { "type": "string" },
          "size_bytes":  { "type": "integer", "minimum": 0 }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_CONTENT: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs content output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "content" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["file","path","line","content","score"],
        "properties": {
          "file":      { "type": "string" },
          "path":      { "type": "string" },
          "line":      { "type": "integer", "minimum": 1 },
          "content":   { "type": "string" },
          "score":     { "type": "integer" },
          "language":  { "type": ["string","null"] }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_CONTENT_REPLACE: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs content --replace output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","dry_run","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "content" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "dry_run":       { "type": "boolean" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["file","path","line","old","new"],
        "properties": {
          "file":  { "type": "string" },
          "path":  { "type": "string" },
          "line":  { "type": "integer", "minimum": 1 },
          "old":   { "type": "string" },
          "new":   { "type": "string" }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_CONTENT_COUNT: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs content --count output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "content" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["file","path","count"],
        "properties": {
          "file":  { "type": "string" },
          "path":  { "type": "string" },
          "count": { "type": "integer", "minimum": 0 }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_WEB: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs web output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "web" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "const": "web" },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["title","url","snippet"],
        "properties": {
          "title":   { "type": "string" },
          "url":     { "type": "string", "format": "uri" },
          "snippet": { "type": "string" }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_WHERE: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs where output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "where" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["path","line","content","language","kind"],
        "properties": {
          "path":      { "type": "string" },
          "line":      { "type": "integer", "minimum": 1 },
          "content":   { "type": "string" },
          "language":  { "type": "string" },
          "kind":      { "type": "string" }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_STATS: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs stats output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","total_files","total_lines","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "stats" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "total_files":   { "type": "integer", "minimum": 0 },
    "total_lines":   { "type": "integer", "minimum": 0 },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["language","files","lines","bytes","percentage"],
        "properties": {
          "language":   { "type": "string" },
          "files":      { "type": "integer", "minimum": 0 },
          "lines":      { "type": "integer", "minimum": 0 },
          "bytes":      { "type": "integer", "minimum": 0 },
          "percentage": { "type": "number", "minimum": 0.0, "maximum": 100.0 }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_RECENT: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs recent output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "recent" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["path","modified","size_bytes","extension"],
        "properties": {
          "path":       { "type": "string" },
          "modified":   { "type": "string" },
          "size_bytes": { "type": "integer", "minimum": 0 },
          "extension":  { "type": "string" }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_ACROSS: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs across output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "across" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["repo","file","path","line","content"],
        "properties": {
          "repo":    { "type": "string" },
          "file":    { "type": "string" },
          "path":    { "type": "string" },
          "line":    { "type": "integer", "minimum": 1 },
          "content": { "type": "string" }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_OPEN: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs open output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "open" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "const": 1 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "minItems": 1,
      "maxItems": 1,
      "items": {
        "type": "object",
        "required": ["file","path","score"],
        "properties": {
          "file":  { "type": "string" },
          "path":  { "type": "string" },
          "score": { "type": "integer" },
          "line":  { "type": ["integer","null"], "minimum": 1 }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_EXPLAIN: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs explain output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "explain" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["token","description"],
        "properties": {
          "token":       { "type": "string" },
          "description": { "type": "string" }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

const SCHEMA_HISTORY: &str = r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "cs history output",
  "type": "object",
  "required": ["tool","version","command","query","source","count","elapsed_secs","results"],
  "properties": {
    "tool":          { "type": "string", "const": "codescope" },
    "version":       { "type": "string" },
    "command":       { "type": "string", "const": "history" },
    "query":         { "type": "string" },
    "source":        { "type": "string", "enum": ["filesystem","stdin","web"] },
    "count":         { "type": "integer", "minimum": 0 },
    "elapsed_secs":  { "type": "number" },
    "results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["timestamp","command","pattern","path","results","elapsed_secs"],
        "properties": {
          "timestamp":    { "type": "string" },
          "command":      { "type": "string" },
          "pattern":      { "type": "string" },
          "path":         { "type": "string" },
          "results":      { "type": "integer", "minimum": 0 },
          "elapsed_secs": { "type": "number" }
        },
        "additionalProperties": false
      }
    }
  },
  "additionalProperties": false
}"##;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_has_required_keys() {
        let env = envelope("file", "test", "filesystem", 3, 0.042, serde_json::json!([]));
        assert_eq!(env["tool"], "codescope");
        assert_eq!(env["command"], "file");
        assert_eq!(env["query"], "test");
        assert_eq!(env["source"], "filesystem");
        assert_eq!(env["count"], 3);
        assert_eq!(env["results"], serde_json::json!([]));
        // version must be present
        assert!(env["version"].is_string());
    }

    #[test]
    fn envelope_with_extra_merges_keys() {
        let env = envelope_with_extra(
            "stats",
            ".",
            "filesystem",
            5,
            0.1,
            serde_json::json!([{"language": "Rust"}]),
            serde_json::json!({"total_files": 10, "total_lines": 500}),
        );
        assert_eq!(env["total_files"], 10);
        assert_eq!(env["total_lines"], 500);
        assert_eq!(env["command"], "stats");
    }

    #[test]
    fn file_result_item_serialises() {
        let item = FileResultItem {
            filename: "main.rs".into(),
            path: "/src/main.rs".into(),
            score: 200,
            extension: "rs".into(),
            size_bytes: 1024,
        };
        let val = serde_json::to_value(&item).unwrap();
        assert_eq!(val["filename"], "main.rs");
        assert_eq!(val["extension"], "rs");
        assert_eq!(val["size_bytes"], 1024);
    }

    #[test]
    fn replace_result_item_uses_new_key() {
        let item = ReplaceResultItem {
            file: "a.rs".into(),
            path: "/a.rs".into(),
            line: 10,
            old: "foo".into(),
            new_val: "bar".into(),
        };
        let val = serde_json::to_value(&item).unwrap();
        // The `#[serde(rename = "new")]` attribute should produce `"new"`
        assert!(val.get("new").is_some());
        assert!(val.get("new_val").is_none());
    }

    #[test]
    fn get_schema_returns_all_commands() {
        let commands = [
            "file", "content", "content-replace", "content-count",
            "web", "where", "stats", "recent", "across",
            "open", "explain", "history",
        ];
        for cmd in &commands {
            let schema = get_schema(cmd).expect(&format!("schema for {} missing", cmd));
            assert_eq!(schema["$schema"], "http://json-schema.org/draft-07/schema#");
        }
    }

    #[test]
    fn get_schema_unknown_returns_none() {
        assert!(get_schema("nonexistent").is_none());
    }

    #[test]
    fn open_result_item_optional_line() {
        let item = OpenResultItem {
            file: "main.rs".into(),
            path: "/main.rs".into(),
            score: 100,
            line: None,
        };
        let val = serde_json::to_value(&item).unwrap();
        assert_eq!(val["line"], serde_json::Value::Null);
    }

    #[test]
    fn print_json_does_not_panic() {
        let env = envelope("file", "test", "filesystem", 0, 0.0, serde_json::json!([]));
        // Just ensure it doesn't panic
        print_json(&env);
    }
}
