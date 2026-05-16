//! TUI Live Preview — Interactive symbol/file browser with live preview.
//!
//! Split-screen TUI with:
//! - Left panel: List of symbols/files (fuzzy-searchable)
//! - Right panel: Live preview of the selected item's source code
//! - Bottom bar: Status line with navigation hints

use std::io::{self, Write};
use termion::cursor::Goto;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

/// A preview item that can be displayed.
#[derive(Debug, Clone)]
pub struct PreviewItem {
    pub title: String,
    pub file_path: String,
    pub line: usize,
    pub preview_lines: Vec<String>,
}

/// Run the interactive preview TUI.
/// Shows a list of items on the left and a live preview on the right.
pub fn run_preview(items: Vec<PreviewItem>) -> Option<PreviewItem> {
    if items.is_empty() {
        return None;
    }

    let selected = _run_tui_preview(items);
    selected
}

/// Extract a window of lines around the target line from the file content.
pub fn extract_preview(file_path: &str, target_line: usize, context: usize) -> Vec<String> {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return vec!["(file not readable)".to_string()],
    };

    let lines: Vec<&str> = content.lines().collect();
    let start = target_line.saturating_sub(context + 1);
    let end = (target_line + context).min(lines.len());

    if start >= end {
        return vec!["(line out of range)".to_string()];
    }

    (start..end)
        .enumerate()
        .map(|(i, idx)| {
            let line_num = start + i + 1;
            let line_content = lines[idx];
            let marker = if line_num == target_line { ">>" } else { "  " };
            format!("{} {:>4} | {}", marker, line_num, line_content)
        })
        .collect()
}

fn _run_tui_preview(items: Vec<PreviewItem>) -> Option<PreviewItem> {
    // This is a simplified implementation that works without external TUI deps.
    // For the full ratatui implementation, see tui.rs.
    // This provides a basic interactive selector with preview.

    let mut cursor: usize = 0;
    let terminal_height = termion::terminal_size().unwrap().1 as usize;
    let list_width = 40;

    // Enable raw mode for terminal input handling
    let _raw = io::stdout().into_raw_mode().unwrap();
    let mut stdout = io::stdout();

    loop {
        // Clear and redraw
        write!(stdout, "{}{}", termion::clear::All, Goto(1, 1)).ok();

        // Draw header
        writeln!(stdout, "{} CodeScope — Live Preview ({} items) {}",
            termion::color::Fg(termion::color::Cyan),
            items.len(),
            termion::style::Reset).ok();
        writeln!(stdout, "{}", "─".repeat(list_width + 1)).ok();

        // Draw list panel (left side)
        let visible_count = terminal_height.saturating_sub(4);
        let scroll = cursor.saturating_sub(visible_count / 2);
        let end = (scroll + visible_count).min(items.len());

        for i in scroll..end {
            let item = &items[i];
            let marker = if i == cursor { ">" } else { " " };
            let style_str = if i == cursor {
                format!("{}", termion::color::Fg(termion::color::Yellow))
            } else {
                String::new()
            };
            let title = if item.title.len() > list_width - 3 {
                format!("{}...", &item.title[..list_width - 6])
            } else {
                item.title.clone()
            };
            writeln!(stdout, "{}{} {:<width$}{}", marker, style_str, title, termion::style::Reset, width = list_width - 3).ok();
        }

        // Draw divider
        writeln!(stdout, "{}", "│").ok();

        // Draw preview panel (right side) — show preview of selected item
        if let Some(item) = items.get(cursor) {
            let preview_lines = &item.preview_lines;
            let preview_count = preview_lines.len().min(visible_count);
            for line in &preview_lines[..preview_count] {
                writeln!(stdout, "{}", line).ok();
            }
        }

        // Draw footer
        writeln!(stdout, "{}", termion::cursor::Goto(1, terminal_height as u16)).ok();
        write!(stdout, "{} ↑/↓ navigate  Enter select  q quit {}",
            termion::color::Fg(termion::color::Green),
            termion::style::Reset).ok();
        stdout.flush().ok();

        // Handle input
        if let Some(Ok(key)) = io::stdin().keys().next() {
            match key {
                Key::Up | Key::Char('k') => cursor = cursor.saturating_sub(1),
                Key::Down | Key::Char('j') => cursor = (cursor + 1).min(items.len() - 1),
                Key::Char('q') | Key::Ctrl('c') => return None,
                Key::Char('\n') => return Some(items[cursor].clone()),
                Key::PageUp => cursor = cursor.saturating_sub(visible_count),
                Key::PageDown => cursor = (cursor + visible_count).min(items.len() - 1),
                Key::Home => cursor = 0,
                Key::End => cursor = items.len() - 1,
                _ => {}
            }
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_preview() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(&file_path, "line1\nline2\nTARGET_LINE\nline4\nline5\n").unwrap();

        let preview = extract_preview(file_path.to_str().unwrap(), 3, 1);
        assert!(preview.iter().any(|l| l.contains("TARGET_LINE")));
        assert!(preview.iter().any(|l| l.contains(">>")));
    }

    #[test]
    fn test_extract_preview_context() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        std::fs::write(&file_path, "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n").unwrap();

        let preview = extract_preview(file_path.to_str().unwrap(), 5, 2);
        // Should show lines 2-7 (5-2=3 to 5+2=7)
        assert!(preview.iter().any(|l| l.contains("c")));
        assert!(preview.iter().any(|l| l.contains("g")));
    }

    #[test]
    fn test_run_preview_empty() {
        let result = run_preview(vec![]);
        assert!(result.is_none());
    }
}
