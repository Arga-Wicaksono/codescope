use colored::Colorize;

use dialoguer::{MultiSelect, Select};

pub fn interactive_file_select(results: &[(String, String, i64)]) -> Option<String> {
    let items: Vec<String> = results.iter().map(|(name, path, _score)| {
        format!("{} ({})", name, path)
    }).collect();

    if items.is_empty() {
        return None;
    }

    Select::new()
        .with_prompt("Select a file")
        .items(&items)
        .default(0)
        .interact()
        .ok()
        .map(|idx| results[idx].1.clone())
}

pub fn interactive_content_select(results: &[(String, String, usize, String, i64)]) -> Option<(String, usize, String)> {
    let items: Vec<String> = results.iter().map(|(file, _path, line, content, _score)| {
        format!("{}:{}  {}", file, line, content)
    }).collect();

    if items.is_empty() {
        return None;
    }

    Select::new()
        .with_prompt("Select a match")
        .items(&items)
        .default(0)
        .interact()
        .ok()
        .map(|idx| {
            let (file, _, line, content, _) = &results[idx];
            (file.clone(), *line, content.clone())
        })
}

pub fn interactive_web_select(results: &[(String, String, String)]) -> Option<String> {
    let items: Vec<String> = results.iter().map(|(title, url, snippet)| {
        format!("{} — {}", title, url)
    }).collect();

    if items.is_empty() {
        return None;
    }

    Select::new()
        .with_prompt("Select a result")
        .items(&items)
        .default(0)
        .interact()
        .ok()
        .map(|idx| results[idx].1.clone())
}

pub fn print_file_selection(path: &str) {
    eprintln!("\n  {} {}", "Selected:".green().bold(), path.cyan());
    eprintln!("  {} vim {}", "Open:".dimmed(), path);
    eprintln!("  {} code -g {}", "Open:".dimmed(), path);
}

pub fn print_content_selection(file: &str, line: usize, content: &str) {
    eprintln!("\n  {} {}:{}", "Selected:".green().bold(), file.cyan(), line.to_string().yellow());
    eprintln!("  {}", content);
    eprintln!("  {} vim +{} {}", "Open:".dimmed(), line, file);
}

pub fn print_web_selection(url: &str) {
    eprintln!("\n  {} {}", "Selected:".green().bold(), url.cyan());
    eprintln!("  {} open {}", "Open:".dimmed(), url);
}
