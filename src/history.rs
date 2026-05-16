use colored::Colorize;

use crate::types::HistoryEntry;
use std::path::PathBuf;

const MAX_ENTRIES: usize = 1000;

/// Get the history file path.
pub fn history_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codescope_history.jsonl"))
}

/// Get the current timestamp as ISO 8601 string.
pub fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Record a search entry to the history file.
pub fn record_entry(entry: &HistoryEntry) {
    let path = match history_path() {
        Some(p) => p,
        None => return,
    };

    if let Ok(json) = serde_json::to_string(entry) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = writeln!(file, "{}", json);
        }
    }
}

/// Read history entries from the history file.
pub fn read_entries() -> Vec<HistoryEntry> {
    let path = match history_path() {
        Some(p) if p.exists() => p,
        _ => return Vec::new(),
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let entries: Vec<HistoryEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    entries.into_iter().rev().take(MAX_ENTRIES).collect()
}

/// Show search history.
pub fn show_history(limit: Option<usize>, json: bool) -> Result<(), String> {
    let entries = read_entries();
    let max = limit.unwrap_or(20);
    let entries: Vec<_> = entries.into_iter().take(max).collect();

    if json {
        let results_json: Vec<serde_json::Value> = entries
            .iter()
            .map(|entry| {
                serde_json::to_value(crate::output_schema::HistoryResultItem {
                    timestamp: entry.timestamp.clone(),
                    command: entry.command.clone(),
                    pattern: entry.pattern.clone(),
                    path: entry.path.clone(),
                    results: entry.results,
                    elapsed_secs: entry.elapsed_secs,
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "history", ".", "filesystem", entries.len(), 0.0,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
    } else {
        if entries.is_empty() {
            eprintln!("{}", "No search history yet.".yellow());
            return Ok(());
        }
        eprintln!("{}", "  Recent searches:".bold());
        eprintln!("  {}", "─".repeat(50).dimmed());
        for entry in &entries {
            eprintln!(
                "  {} {} {}  {}",
                entry.timestamp.dimmed(),
                format!("[{}]", entry.command).green(),
                entry.pattern.cyan(),
                format!("({:.3}s)", entry.elapsed_secs).dimmed()
            );
        }
        eprintln!("  {}", "─".repeat(50).dimmed());
        eprintln!("  {} {} entries", "✓".green(), entries.len());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrono_timestamp() {
        let ts = chrono_timestamp();
        assert!(!ts.is_empty());
        assert!(ts.parse::<u64>().is_ok());
    }
}
