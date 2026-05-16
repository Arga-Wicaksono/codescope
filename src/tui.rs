//! Terminal User Interface (TUI) for CodeScope.
//!
//! A modern terminal UI built with **ratatui** and **crossterm** that provides:
//! - File/symbol browser panel
//! - Live code preview with syntax highlighting
//! - Search bar with fuzzy matching
//! - AI context sidebar (relevance score, token estimate)
//! - Vim-style keyboard navigation
//! - Split view for diff/comparison
//!
//! # Entry point
//!
//! ```bash
//! cs tui                    # Open TUI in current directory
//! cs tui --path src/        # Open TUI scoped to a directory
//! cs tui --type rust        # Filter by language
//! ```
//!
//! # Key bindings
//!
//! | Key | Action |
//! |-----|--------|
//! | `/` | Focus search bar |
//! | `Esc` | Unfocus / go back |
//! | `Tab` | Switch panels (files ↔ preview) |
//! | `↑/j` | Move up |
//! | `↓/k` | Move down |
//! | `g` | Jump to top |
//! | `G` | Jump to bottom |
//! | `Enter` | Open selected file / jump to definition |
//! | `o` | Open in `$EDITOR` |
//! | `s` | Toggle symbol view |
//! | `c` | Toggle context sidebar |
//! | `f` | Toggle file filter |
//! | `n` | Next search result |
//! | `N` | Previous search result |
//! | `1-5` | Switch tabs |
//! | `?` | Show help |
//! | `q` | Quit |

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ignore::WalkBuilder;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Tabs, Wrap,
    },
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single file entry displayed in the TUI.
#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    rel_path: String,
    extension: String,
    size: u64,
    /// Line count (lazily computed)
    lines: Option<usize>,
}

/// A symbol entry displayed in the symbol panel.
#[derive(Debug, Clone)]
struct SymbolEntry {
    name: String,
    kind: String,
    file: String,
    line: usize,
    snippet: String,
}

/// Which panel is currently focused.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Search,
    Files,
    Symbols,
    Preview,
    Context,
}

/// Which tab is active in the left panel.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Files,
    Symbols,
}

/// The mode the TUI is in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Search,
    Help,
}

/// Search mode: files, content, or symbols.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SearchKind {
    Files,
    Content,
    Symbols,
}

// ---------------------------------------------------------------------------
// App State
// ---------------------------------------------------------------------------

/// The main application state for the TUI.
pub struct App {
    // ── Core state ───────────────────────────────────────────────────────
    root_path: PathBuf,
    mode: Mode,
    focus: Focus,
    tab: Tab,
    should_quit: bool,

    // ── Search ───────────────────────────────────────────────────────────
    search_query: String,
    search_kind: SearchKind,
    search_cursor: usize,

    // ── File list ────────────────────────────────────────────────────────
    file_entries: Vec<FileEntry>,
    filtered_files: Vec<usize>,
    file_table_state: TableState,
    file_filter: Option<String>,
    extension_filter: Option<String>,
    file_scroll: usize,

    // ── Symbol list ──────────────────────────────────────────────────────
    symbol_entries: Vec<SymbolEntry>,
    filtered_symbols: Vec<usize>,
    symbol_table_state: TableState,
    symbol_scroll: usize,

    // ── Preview ──────────────────────────────────────────────────────────
    preview_file: Option<String>,
    preview_content: Vec<String>,
    preview_scroll: usize,
    preview_highlight_line: Option<usize>,

    // ── Context sidebar ──────────────────────────────────────────────────
    show_context: bool,
    context_lines: Vec<String>,
    context_scroll: usize,

    // ── Timing ───────────────────────────────────────────────────────────
    last_search_time: Option<f64>,
    total_files: usize,
    total_symbols: usize,

    // ── Message ──────────────────────────────────────────────────────────
    status_message: String,
    status_time: Instant,
}

impl App {
    /// Create a new App rooted at `path`.
    pub fn new(path: &str, file_type: Option<&str>) -> Result<Self, String> {
        let root = std::fs::canonicalize(path)
            .map_err(|e| format!("Cannot resolve path '{}': {}", path, e))?;

        let extension_filter = file_type.and_then(|ft| match ft {
            "rust" => Some("rs".to_string()),
            "python" | "py" => Some("py".to_string()),
            "js" | "javascript" => Some("js".to_string()),
            "ts" | "typescript" => Some("ts".to_string()),
            "go" => Some("go".to_string()),
            "java" => Some("java".to_string()),
            "c" => Some("c".to_string()),
            "cpp" => Some("cpp".to_string()),
            _ => None,
        });

        let mut app = Self {
            root_path: root.clone(),
            mode: Mode::Normal,
            focus: Focus::Files,
            tab: Tab::Files,
            should_quit: false,
            search_query: String::new(),
            search_kind: SearchKind::Files,
            search_cursor: 0,
            file_entries: Vec::new(),
            filtered_files: Vec::new(),
            file_table_state: TableState::default(),
            file_filter: None,
            extension_filter,
            file_scroll: 0,
            symbol_entries: Vec::new(),
            filtered_symbols: Vec::new(),
            symbol_table_state: TableState::default(),
            symbol_scroll: 0,
            preview_file: None,
            preview_content: Vec::new(),
            preview_scroll: 0,
            preview_highlight_line: None,
            show_context: false,
            context_lines: Vec::new(),
            context_scroll: 0,
            last_search_time: None,
            total_files: 0,
            total_symbols: 0,
            status_message: String::new(),
            status_time: Instant::now(),
        };

        app.scan_files()?;
        app.scan_symbols()?;

        Ok(app)
    }

    // ── File scanning ────────────────────────────────────────────────────

    fn scan_files(&mut self) -> Result<(), String> {
        let start = Instant::now();
        let mut entries = Vec::new();

        let mut builder = WalkBuilder::new(&self.root_path);
        builder.git_ignore(true).git_global(true).git_exclude(true).hidden(false);

        for entry in builder.build().filter_map(|e| e.ok()) {
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                continue;
            }
            let path = entry.path().to_path_buf();
            let rel = path
                .strip_prefix(&self.root_path)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if rel.is_empty() {
                continue;
            }

            if let Some(ref ext_filter) = self.extension_filter {
                let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase());
                if ext.as_deref() != Some(ext_filter.as_str()) {
                    continue;
                }
            }

            let extension = path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

            entries.push(FileEntry {
                path,
                rel_path: rel,
                extension,
                size,
                lines: None,
            });
        }

        // Sort by path
        entries.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));

        self.total_files = entries.len();
        self.filtered_files = (0..entries.len()).collect();
        self.file_entries = entries;

        let elapsed = start.elapsed().as_secs_f64();
        self.set_status(format!(
            "Scanned {} files in {:.3}s",
            self.total_files, elapsed
        ));

        Ok(())
    }

    // ── Symbol scanning ──────────────────────────────────────────────────

    fn scan_symbols(&mut self) -> Result<(), String> {
        use regex::Regex;

        let start = Instant::now();
        let mut symbols = Vec::new();

        let patterns: &[(&str, &str, &str)] = &[
            // (extension, kind, pattern)
            ("rs", "fn", r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)"),
            ("rs", "struct", r"(?:pub\s+)?struct\s+(\w+)"),
            ("rs", "enum", r"(?:pub\s+)?enum\s+(\w+)"),
            ("rs", "trait", r"(?:pub\s+)?trait\s+(\w+)"),
            ("rs", "impl", r"impl\s+(\w+)"),
            ("py", "fn", r"(?:async\s+)?def\s+(\w+)"),
            ("py", "class", r"class\s+(\w+)"),
            ("js", "fn", r"(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+(\w+)"),
            ("ts", "fn", r"(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+(\w+)"),
            ("go", "fn", r"func\s+(?:\(\w+\s+\*?\w+\)\s+)?(\w+)"),
            ("go", "struct", r"type\s+(\w+)\s+struct"),
            ("go", "interface", r"type\s+(\w+)\s+interface"),
            ("java", "class", r"(?:public|private|protected)?\s*class\s+(\w+)"),
            ("java", "interface", r"(?:public|private|protected)?\s*interface\s+(\w+)"),
            ("c", "fn", r"\w+\s+(\w+)\s*\([^)]*\)\s*\{?"),
            ("cpp", "class", r"(?:class|struct)\s+(\w+)"),
        ];

        let compiled: Vec<(String, String, Regex)> = patterns
            .iter()
            .filter_map(|(ext, kind, pat)| {
                Regex::new(pat).ok().map(|re| ((*ext).to_string(), (*kind).to_string(), re))
            })
            .collect();

        for entry in &self.file_entries {
            let ext = entry.extension.to_lowercase();
            let content = match fs::read_to_string(&entry.path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            for (ext_pat, kind, re) in &compiled {
                if ext != *ext_pat {
                    continue;
                }
                for cap in re.find_iter(&content) {
                    if let Some(caps) = re.captures(cap.as_str()) {
                        if let Some(name_match) = caps.get(1) {
                            let name = name_match.as_str().to_string();
                            let line_num = content[..cap.start()].matches('\n').count() + 1;
                            let lines: Vec<&str> = content.lines().collect();
                            let snippet = lines
                                .get(line_num.saturating_sub(1))
                                .unwrap_or(&"")
                                .chars()
                                .take(80)
                                .collect();

                            symbols.push(SymbolEntry {
                                name,
                                kind: kind.clone(),
                                file: entry.rel_path.clone(),
                                line: line_num,
                                snippet,
                            });
                        }
                    }
                }
            }
        }

        symbols.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.file.cmp(&b.file)));
        self.total_symbols = symbols.len();
        self.filtered_symbols = (0..symbols.len()).collect();
        self.symbol_entries = symbols;

        let elapsed = start.elapsed().as_secs_f64();
        self.set_status(format!(
            "Found {} symbols in {:.3}s",
            self.total_symbols, elapsed
        ));

        Ok(())
    }

    // ── Search ───────────────────────────────────────────────────────────

    fn search(&mut self) {
        if self.search_query.is_empty() {
            // Reset filters
            self.filtered_files = (0..self.file_entries.len()).collect();
            self.filtered_symbols = (0..self.symbol_entries.len()).collect();
            return;
        }

        let start = Instant::now();
        let query = self.search_query.to_lowercase();

        match self.search_kind {
            SearchKind::Files => {
                let matcher = SkimMatcherV2::default();

                let mut scored: Vec<(usize, i64)> = self
                    .file_entries
                    .iter()
                    .enumerate()
                    .filter_map(|(i, e)| {
                        let score = matcher.fuzzy_match(&e.rel_path, &self.search_query)?;
                        Some((i, score))
                    })
                    .collect();
                scored.sort_by(|a, b| b.1.cmp(&a.1));
                self.filtered_files = scored.iter().map(|(i, _)| *i).collect();
            }
            SearchKind::Content => {
                // Search content and mark matching files
                let mut matching_indices = Vec::new();
                for (i, entry) in self.file_entries.iter().enumerate() {
                    if let Ok(content) = fs::read_to_string(&entry.path) {
                        if content.to_lowercase().contains(&query) {
                            matching_indices.push(i);
                        }
                    }
                }
                self.filtered_files = matching_indices;
            }
            SearchKind::Symbols => {
                let matcher = SkimMatcherV2::default();

                let mut scored: Vec<(usize, i64)> = self
                    .symbol_entries
                    .iter()
                    .enumerate()
                    .filter_map(|(i, s)| {
                        let score = matcher.fuzzy_match(&s.name, &self.search_query)?;
                        Some((i, score))
                    })
                    .collect();
                scored.sort_by(|a, b| b.1.cmp(&a.1));
                self.filtered_symbols = scored.iter().map(|(i, _)| *i).collect();
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        self.last_search_time = Some(elapsed);

        let count = match self.search_kind {
            SearchKind::Files | SearchKind::Content => self.filtered_files.len(),
            SearchKind::Symbols => self.filtered_symbols.len(),
        };
        self.set_status(format!(
            "Found {} results in {:.3}s [{}]",
            count,
            elapsed,
            match self.search_kind {
                SearchKind::Files => "files",
                SearchKind::Content => "content",
                SearchKind::Symbols => "symbols",
            }
        ));
    }

    // ── Preview ──────────────────────────────────────────────────────────

    fn load_preview(&mut self, file_idx: Option<usize>, symbol_idx: Option<usize>) {
        let path = if let Some(si) = symbol_idx {
            if let Some(sym) = self.symbol_entries.get(si) {
                self.preview_highlight_line = Some(sym.line);
                format!("{}/{}", self.root_path.display(), sym.file)
            } else {
                return;
            }
        } else if let Some(fi) = file_idx {
            if let Some(entry) = self.file_entries.get(fi) {
                self.preview_highlight_line = None;
                entry.path.to_string_lossy().to_string()
            } else {
                return;
            }
        } else {
            return;
        };

        match fs::read_to_string(&path) {
            Ok(content) => {
                self.preview_file = Some(path);
                self.preview_content = content.lines().map(String::from).collect();
                self.preview_scroll = 0;

                // Scroll to highlight line
                if let Some(line) = self.preview_highlight_line {
                    if line > 5 {
                        self.preview_scroll = line.saturating_sub(3);
                    }
                }

                self.update_context();
            }
            Err(e) => {
                self.set_status(format!("Cannot read file: {}", e));
            }
        }
    }

    fn update_context(&mut self) {
        if !self.show_context {
            return;
        }
        let file = match &self.preview_file {
            Some(f) => f.clone(),
            None => return,
        };

        let mut ctx = Vec::new();
        ctx.push("Context Analysis".to_string());
        ctx.push(format!("{}", "─".repeat(30)));
        ctx.push(format!("File: {}", file));

        let content = self.preview_content.join("\n");
        let chars: usize = content.len();
        let tokens = chars / 4;
        ctx.push(format!("Lines: {}", self.preview_content.len()));
        ctx.push(format!("Chars: {}", chars));
        ctx.push(format!("Tokens (~): {}", tokens));

        // Find symbols in this file
        let file_symbols: Vec<&SymbolEntry> = self
            .symbol_entries
            .iter()
            .filter(|s| file.ends_with(&s.file))
            .collect();
        ctx.push(format!(""));
        ctx.push(format!("Symbols ({}):", file_symbols.len()));
        for sym in file_symbols.iter().take(20) {
            ctx.push(format!("  [{}] {} (L{})", sym.kind, sym.name, sym.line));
        }

        self.context_lines = ctx;
        self.context_scroll = 0;
    }

    // ── Status ───────────────────────────────────────────────────────────

    fn set_status(&mut self, msg: String) {
        self.status_message = msg;
        self.status_time = Instant::now();
    }

    // ── Actions ──────────────────────────────────────────────────────────

    fn open_in_editor(&self) {
        let _file = match self.focus {
            Focus::Files => self
                .filtered_files
                .get(self.file_table_state.selected().unwrap_or(0))
                .and_then(|i| self.file_entries.get(*i))
                .map(|e| e.path.to_string_lossy().to_string()),
            Focus::Symbols => self
                .filtered_symbols
                .get(self.symbol_table_state.selected().unwrap_or(0))
                .and_then(|i| self.symbol_entries.get(*i))
                .map(|s| format!("{}/{}:{}", self.root_path.display(), s.file, s.line)),
            _ => None,
        };

        if let Some(path) = _file {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            // We need to pause the TUI to run the editor
            let _ = std::process::Command::new(&editor).arg(&path).status();
        }
    }

    // ── Event handling ───────────────────────────────────────────────────

    fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Help => {
                match key.code {
                    KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                        self.mode = Mode::Normal;
                    }
                    _ => {}
                }
                return;
            }
            Mode::Search => {
                self.handle_search_key(key);
                return;
            }
            Mode::Normal => {}
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                self.focus = Focus::Search;
                self.search_query.clear();
                self.search_cursor = 0;
            }
            KeyCode::Char('?') => {
                self.mode = Mode::Help;
            }
            KeyCode::Tab => {
                match self.focus {
                    Focus::Files | Focus::Symbols => self.focus = Focus::Preview,
                    Focus::Preview => {
                        self.focus = match self.tab {
                            Tab::Files => Focus::Files,
                            Tab::Symbols => Focus::Symbols,
                        }
                    }
                    _ => self.focus = Focus::Files,
                }
            }
            KeyCode::Char('1') => {
                self.tab = Tab::Files;
                self.focus = Focus::Files;
            }
            KeyCode::Char('2') => {
                self.tab = Tab::Symbols;
                self.focus = Focus::Symbols;
            }
            KeyCode::Char('3') => {
                self.focus = Focus::Preview;
            }
            KeyCode::Char('s') => {
                self.tab = match self.tab {
                    Tab::Files => Tab::Symbols,
                    Tab::Symbols => Tab::Files,
                };
                self.focus = match self.tab {
                    Tab::Files => Focus::Files,
                    Tab::Symbols => Focus::Symbols,
                };
            }
            KeyCode::Char('c') => {
                self.show_context = !self.show_context;
                if self.show_context {
                    self.update_context();
                }
            }
            KeyCode::Char('f') => {
                // Cycle search kind
                self.search_kind = match self.search_kind {
                    SearchKind::Files => SearchKind::Content,
                    SearchKind::Content => SearchKind::Symbols,
                    SearchKind::Symbols => SearchKind::Files,
                };
                self.set_status(format!("Search mode: {:?}", self.search_kind));
                if !self.search_query.is_empty() {
                    self.search();
                }
            }
            KeyCode::Char('o') => {
                self.open_in_editor();
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.move_selection(-1);
            }
            KeyCode::Char('n') => {
                self.move_selection(1);
            }
            KeyCode::Char('g') => {
                self.jump_to_top();
            }
            KeyCode::Char('G') => {
                self.jump_to_bottom();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1);
            }
            KeyCode::Enter => {
                self.select_current();
            }
            KeyCode::Char(' ') => {
                self.preview_scroll =
                    self.preview_scroll.saturating_add(10).min(self.preview_content.len().saturating_sub(1));
            }
            KeyCode::Char('b') => {
                self.preview_scroll = self.preview_scroll.saturating_sub(10);
            }
            KeyCode::Esc => {
                if self.search_query.is_empty() {
                    self.should_quit = true;
                } else {
                    self.search_query.clear();
                    self.search();
                }
            }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                self.search();
                self.mode = Mode::Normal;
                self.focus = match self.search_kind {
                    SearchKind::Files => Focus::Files,
                    SearchKind::Content => Focus::Files,
                    SearchKind::Symbols => Focus::Symbols,
                };
                // Load preview for first result
                if !self.filtered_files.is_empty() {
                    self.file_table_state.select(Some(0));
                    self.load_preview(self.filtered_files.first().copied(), None);
                } else if !self.filtered_symbols.is_empty() {
                    self.symbol_table_state.select(Some(0));
                    self.load_preview(None, self.filtered_symbols.first().copied());
                }
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.focus = Focus::Files;
            }
            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' => {
                self.search_query.clear();
                self.search_cursor = 0;
            }
            KeyCode::Char(c) => {
                self.search_query.insert(self.search_cursor, c);
                self.search_cursor += 1;
            }
            KeyCode::Backspace => {
                if self.search_cursor > 0 {
                    self.search_cursor -= 1;
                    self.search_query.remove(self.search_cursor);
                }
            }
            KeyCode::Delete => {
                if self.search_cursor < self.search_query.len() {
                    self.search_query.remove(self.search_cursor);
                }
            }
            KeyCode::Left => {
                self.search_cursor = self.search_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.search_cursor < self.search_query.len() {
                    self.search_cursor += 1;
                }
            }
            KeyCode::Home => {
                self.search_cursor = 0;
            }
            KeyCode::End => {
                self.search_cursor = self.search_query.len();
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, direction: i32) {
        match self.focus {
            Focus::Files => {
                let len = self.filtered_files.len();
                if len == 0 {
                    return;
                }
                let current = self.file_table_state.selected().unwrap_or(0);
                let new_pos = (current as i32 + direction).max(0).min(len as i32 - 1) as usize;
                self.file_table_state.select(Some(new_pos));

                // Auto-load preview
                if let Some(&idx) = self.filtered_files.get(new_pos) {
                    self.load_preview(Some(idx), None);
                }
            }
            Focus::Symbols => {
                let len = self.filtered_symbols.len();
                if len == 0 {
                    return;
                }
                let current = self.symbol_table_state.selected().unwrap_or(0);
                let new_pos = (current as i32 + direction).max(0).min(len as i32 - 1) as usize;
                self.symbol_table_state.select(Some(new_pos));

                // Auto-load preview
                if let Some(&idx) = self.filtered_symbols.get(new_pos) {
                    self.load_preview(None, Some(idx));
                }
            }
            Focus::Preview => {
                let max = self.preview_content.len().saturating_sub(1);
                self.preview_scroll = (self.preview_scroll as i32 + direction)
                    .max(0)
                    .min(max as i32) as usize;
            }
            _ => {}
        }
    }

    fn jump_to_top(&mut self) {
        match self.focus {
            Focus::Files => {
                if !self.filtered_files.is_empty() {
                    self.file_table_state.select(Some(0));
                }
            }
            Focus::Symbols => {
                if !self.filtered_symbols.is_empty() {
                    self.symbol_table_state.select(Some(0));
                }
            }
            Focus::Preview => {
                self.preview_scroll = 0;
            }
            _ => {}
        }
    }

    fn jump_to_bottom(&mut self) {
        match self.focus {
            Focus::Files => {
                let last = self.filtered_files.len().saturating_sub(1);
                self.file_table_state.select(Some(last));
            }
            Focus::Symbols => {
                let last = self.filtered_symbols.len().saturating_sub(1);
                self.symbol_table_state.select(Some(last));
            }
            Focus::Preview => {
                self.preview_scroll = self.preview_content.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    fn select_current(&mut self) {
        match self.focus {
            Focus::Files => {
                if let Some(&idx) = self
                    .filtered_files
                    .get(self.file_table_state.selected().unwrap_or(0))
                {
                    self.load_preview(Some(idx), None);
                }
            }
            Focus::Symbols => {
                if let Some(&idx) = self
                    .filtered_symbols
                    .get(self.symbol_table_state.selected().unwrap_or(0))
                {
                    self.load_preview(None, Some(idx));
                }
            }
            Focus::Preview => {
                self.open_in_editor();
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// UI Rendering
// ---------------------------------------------------------------------------

fn render_app(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // ── Main layout ──────────────────────────────────────────────────────
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Search bar + tabs
            Constraint::Min(1),      // Main content
            Constraint::Length(1),   // Status bar
        ])
        .split(size);

    // ── Top bar ──────────────────────────────────────────────────────────
    render_top_bar(f, app, main_chunks[0]);

    // ── Main content area ────────────────────────────────────────────────
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if app.show_context {
            vec![
                Constraint::Percentage(30),  // Left panel
                Constraint::Percentage(45),  // Preview
                Constraint::Percentage(25),  // Context sidebar
            ]
        } else {
            vec![
                Constraint::Percentage(35),  // Left panel
                Constraint::Percentage(65),  // Preview
            ]
        })
        .split(main_chunks[1]);

    // ── Left panel ───────────────────────────────────────────────────────
    render_left_panel(f, app, content_chunks[0]);

    // ── Preview panel ────────────────────────────────────────────────────
    render_preview(f, app, content_chunks[1]);

    // ── Context sidebar ──────────────────────────────────────────────────
    if app.show_context {
        render_context_sidebar(f, app, content_chunks[2]);
    }

    // ── Status bar ───────────────────────────────────────────────────────
    render_status_bar(f, app, main_chunks[2]);

    // ── Help overlay ─────────────────────────────────────────────────────
    if app.mode == Mode::Help {
        render_help(f, size);
    }
}

fn render_top_bar(f: &mut Frame, app: &App, area: Rect) {
    let tabs = Tabs::new(
        vec![
            Span::styled(
                format!(" Files ({}) ", app.filtered_files.len()),
                if app.tab == Tab::Files {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(
                format!(" Symbols ({}) ", app.filtered_symbols.len()),
                if app.tab == Tab::Symbols {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ],
    )
    .block(
        Block::default()
            .title(Span::styled(
                " CodeScope ",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ))
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL),
    )
    .select(if app.tab == Tab::Files { 0 } else { 1 })
    .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, area);
}

fn render_left_panel(f: &mut Frame, app: &mut App, area: Rect) {
    match app.tab {
        Tab::Files => render_file_list(f, app, area),
        Tab::Symbols => render_symbol_list(f, app, area),
    }
}

fn render_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focus == Focus::Files && app.mode != Mode::Search;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Row::new(vec![
        Cell::from("File").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Size").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Ext").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = app
        .filtered_files
        .iter()
        .map(|&idx| {
            let entry = &app.file_entries[idx];
            let name = entry
                .rel_path
                .chars()
                .take(40)
                .collect::<String>();
            let size_str = format_size(entry.size);
            let ext_style = ext_color(&entry.extension);

            Row::new(vec![
                Cell::from(Span::raw(name)),
                Cell::from(Span::styled(size_str, Style::default().fg(Color::DarkGray))),
                Cell::from(Span::styled(
                    entry.extension.clone(),
                    ext_style,
                )),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(65), Constraint::Length(7), Constraint::Length(6)])
        .header(header)
        .block(
            Block::default()
                .title(Span::styled(" Files ", border_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.file_table_state);

    // Scrollbar
    let total = app.filtered_files.len();
    let selected = app.file_table_state.selected().unwrap_or(0);
    let visible = area.height.saturating_sub(2) as usize;
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(total)
        .position(selected)
        .viewport_content_length(visible);
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_symbol_list(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focus == Focus::Symbols && app.mode != Mode::Search;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Row::new(vec![
        Cell::from("Symbol").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Kind").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("Location").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = app
        .filtered_symbols
        .iter()
        .map(|&idx| {
            let sym = &app.symbol_entries[idx];
            let kind_style = kind_color(&sym.kind);

            Row::new(vec![
                Cell::from(Span::styled(
                    sym.name.clone(),
                    Style::default().fg(Color::White),
                )),
                Cell::from(Span::styled(sym.kind.clone(), kind_style)),
                Cell::from(Span::styled(
                    format!("{}:{}", sym.file, sym.line),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Length(8),
            Constraint::Percentage(62),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(Span::styled(" Symbols ", border_style))
            .borders(Borders::ALL)
            .border_style(border_style),
    )
    .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
    .highlight_spacing(HighlightSpacing::Always)
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.symbol_table_state);

    // Scrollbar
    let total = app.filtered_symbols.len();
    let selected = app.symbol_table_state.selected().unwrap_or(0);
    let visible = area.height.saturating_sub(2) as usize;
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(total)
        .position(selected)
        .viewport_content_length(visible);
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Preview && app.mode != Mode::Search;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let file_name = app
        .preview_file
        .as_deref()
        .and_then(|p| std::path::Path::new(p).file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("No file selected");

    let lines: Vec<Line> = if app.preview_content.is_empty() {
        vec![Line::from(Span::styled(
            "Select a file or symbol to preview",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        app.preview_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line_num = i + 1;
                let num_str = format!("{:>4} ", line_num);
                let is_highlight = app.preview_highlight_line == Some(line_num);
                let is_keyword_line = is_rust_keyword_line(line);

                let num_style = if is_highlight {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let content_style = if is_highlight {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else if is_keyword_line {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default().fg(Color::Gray)
                };

                Line::from(vec![
                    Span::styled(num_str, num_style),
                    Span::styled(line.clone(), content_style),
                ])
            })
            .collect()
    };

    let mut scrollbar_state = ScrollbarState::new(app.preview_content.len())
        .position(app.preview_scroll)
        .viewport_content_length(area.height.saturating_sub(2) as usize);

    let preview = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", file_name), border_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll as u16, 0));

    f.render_widget(preview, area);

    // Scrollbar
    if app.preview_content.len() > 1 {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn render_context_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let border_style = Style::default().fg(Color::Magenta);
    let lines: Vec<Line> = if app.context_lines.is_empty() {
        vec![Line::from("No context")]
    } else {
        app.context_lines
            .iter()
            .map(|l| Line::from(Span::raw(l.clone())))
            .collect()
    };

    let preview = Paragraph::new(lines)
        .block(
            Block::default()
                .title(Span::styled(" Context ", border_style))
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true })
        .scroll((app.context_scroll as u16, 0));

    f.render_widget(preview, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let search_label = match app.search_kind {
        SearchKind::Files => "[Files]",
        SearchKind::Content => "[Content]",
        SearchKind::Symbols => "[Symbols]",
    };

    let mode_label = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Search => "SEARCH",
        Mode::Help => "HELP",
    };

    let status = format!(
        " {} | {} | {} files | {} symbols | {}",
        mode_label,
        search_label,
        app.total_files,
        app.total_symbols,
        app.status_message,
    );

    let status_bar = Paragraph::new(Span::styled(
        status,
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray),
    ));

    f.render_widget(status_bar, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            " CodeScope TUI — Keyboard Shortcuts ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Navigation", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(""),
        ]),
        Line::from("  j/Down      Move down"),
        Line::from("  k/Up        Move up"),
        Line::from("  g           Jump to top"),
        Line::from("  G           Jump to bottom"),
        Line::from("  Tab         Switch panels"),
        Line::from("  Space       Scroll preview down"),
        Line::from("  b           Scroll preview up"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Search", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(""),
        ]),
        Line::from("  /           Focus search bar"),
        Line::from("  Enter       Execute search"),
        Line::from("  Esc         Cancel search / quit"),
        Line::from("  f           Cycle search mode (files/content/symbols)"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Tabs", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(""),
        ]),
        Line::from("  1           File browser tab"),
        Line::from("  2           Symbol browser tab"),
        Line::from("  3           Preview panel"),
        Line::from("  s           Toggle files/symbols"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Actions", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(""),
        ]),
        Line::from("  Enter       Select / open file"),
        Line::from("  o           Open in $EDITOR"),
        Line::from("  c           Toggle context sidebar"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Other", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(""),
        ]),
        Line::from("  ?           Show this help"),
        Line::from("  q           Quit"),
        Line::from(""),
        Line::from(" Press ? or Esc to close"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(Span::styled(" Help ", Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    // Center the help overlay
    let help_width = 45.min(area.width.saturating_sub(4));
    let help_height = 35.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(help_width)) / 2;
    let y = (area.height.saturating_sub(help_height)) / 2;

    let help_area = Rect::new(x, y, help_width, help_height);
    f.render_widget(Clear, help_area);
    f.render_widget(help_paragraph, help_area);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}K", bytes as f64 / KB as f64)
    } else {
        format!("{}", bytes)
    }
}

fn ext_color(ext: &str) -> Style {
    match ext {
        "rs" => Style::default().fg(Color::Rgb(222, 165, 132)),  // Rust orange
        "py" => Style::default().fg(Color::Rgb(255, 212, 59)),   // Python yellow
        "js" | "jsx" | "mjs" => Style::default().fg(Color::Rgb(247, 223, 30)), // JS yellow
        "ts" | "tsx" => Style::default().fg(Color::Rgb(49, 120, 198)),  // TS blue
        "go" => Style::default().fg(Color::Rgb(0, 173, 216)),    // Go cyan
        "java" | "kt" => Style::default().fg(Color::Rgb(237, 83, 46)),  // Java red
        "c" | "cpp" | "h" | "hpp" => Style::default().fg(Color::Rgb(104, 159, 56)), // C green
        "md" | "rst" => Style::default().fg(Color::Rgb(63, 127, 95)),  // Doc green
        "toml" | "yaml" | "yml" | "json" => Style::default().fg(Color::Rgb(173, 127, 168)), // Config purple
        _ => Style::default().fg(Color::Gray),
    }
}

fn kind_color(kind: &str) -> Style {
    match kind {
        "fn" => Style::default().fg(Color::Rgb(86, 182, 194)),    // Cyan
        "struct" => Style::default().fg(Color::Rgb(222, 165, 132)), // Rust orange
        "enum" => Style::default().fg(Color::Rgb(209, 154, 102)),   // Brown
        "trait" => Style::default().fg(Color::Rgb(139, 233, 253)),  // Light cyan
        "impl" => Style::default().fg(Color::Rgb(139, 233, 253)),   // Light cyan
        "class" => Style::default().fg(Color::Rgb(250, 179, 135)),  // Light orange
        "interface" => Style::default().fg(Color::Rgb(137, 220, 235)), // Sky
        _ => Style::default().fg(Color::Gray),
    }
}

fn is_rust_keyword_line(line: &str) -> bool {
    let trimmed = line.trim();
    let keywords = [
        "pub ", "pub(crate) ", "pub(super) ", "fn ", "struct ", "enum ",
        "trait ", "impl ", "mod ", "use ", "type ", "const ", "static ",
        "async fn", "pub async fn", "macro_rules!",
    ];
    keywords.iter().any(|kw| trimmed.starts_with(kw))
}

// ---------------------------------------------------------------------------
// Main loop
// ---------------------------------------------------------------------------

/// Run the TUI application.
pub fn run_tui(path: &str, file_type: Option<&str>) -> Result<(), String> {
    // Setup terminal
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Create app
    let mut app = App::new(path, file_type)?;

    // Restore terminal on panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Run the main loop
        loop {
            terminal
                .draw(|f| render_app(f, &mut app))
                .map_err(|e| format!("Render error: {}", e))
                .ok();

            // Poll for events with timeout
            let poll_duration = if app.mode == Mode::Normal {
                Duration::from_millis(100)
            } else {
                Duration::from_millis(16) // ~60fps in search mode
            };

            if event::poll(poll_duration).map_err(|e| format!("Event poll error: {}", e))? {
                if let Event::Key(key) =
                    event::read().map_err(|e| format!("Event read error: {}", e))?
                {
                    app.handle_key(key);

                    if app.should_quit {
                        break;
                    }
                }
            }
        }

        Ok(())
    }));

    // Restore terminal
    disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| format!("Failed to leave alternate screen: {}", e))?;
    terminal
        .show_cursor()
        .map_err(|e| format!("Failed to show cursor: {}", e))?;

    match result {
        Ok(inner) => inner,
        Err(_) => Err("TUI panicked".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0");
        assert_eq!(format_size(512), "512");
        assert_eq!(format_size(1024), "1K");
        assert_eq!(format_size(1536), "2K");
        assert_eq!(format_size(1048576), "1.0M");
        assert_eq!(format_size(1073741824), "1.0G");
    }

    #[test]
    fn test_ext_color() {
        assert!(ext_color("rs").fg == Some(Color::Rgb(222, 165, 132)));
        assert!(ext_color("py").fg == Some(Color::Rgb(255, 212, 59)));
        assert!(ext_color("unknown").fg == Some(Color::Gray));
    }

    #[test]
    fn test_kind_color() {
        assert!(kind_color("fn").fg == Some(Color::Rgb(86, 182, 194)));
        assert!(kind_color("struct").fg == Some(Color::Rgb(222, 165, 132)));
    }

    #[test]
    fn test_is_rust_keyword_line() {
        assert!(is_rust_keyword_line("pub fn hello() {"));
        assert!(is_rust_keyword_line("struct MyStruct {"));
        assert!(!is_rust_keyword_line("    // comment"));
        assert!(!is_rust_keyword_line("    let x = 5;"));
    }
}
