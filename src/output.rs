//! Terminal output formatting for codescope.

use colored::Colorize;

/// Configure color output.
pub fn configure_colors(no_color: bool) {
    if no_color {
        colored::control::set_override(false);
    }
}

/// Print file search results to stdout.
pub fn print_file_results(results: &[(String, String, i64)], query: &str, path: &str, elapsed_secs: f64) {
    let separator = "─".repeat(50);
    eprintln!("{} Searching files: '{}' in {}", ">>".cyan(), query.cyan(), path);
    eprintln!("{}", separator.dimmed());
    for (i, (filename, full_path, score)) in results.iter().enumerate() {
        eprintln!("  {} [{}] {}", format!("{:3}", i + 1).dimmed(), score.to_string().yellow(), full_path.green());
    }
    if !results.is_empty() {
        eprintln!("{}", separator.dimmed());
        eprintln!("{} Found {} file(s) in {:.3}s", "✓".green(), results.len().to_string().green(), elapsed_secs);
    } else {
        eprintln!("{}", "No files found.".yellow());
    }
}

/// Print file search results as JSON.
pub fn print_file_results_json(results: &[(String, String, i64)], query: &str, elapsed_secs: f64) {
    let json_output = serde_json::json!({
        "tool": "codescope",
        "version": env!("CARGO_PKG_VERSION"),
        "query": query,
        "source": "filesystem",
        "count": results.len(),
        "elapsed_secs": elapsed_secs,
        "results": results.iter().map(|(filename, path, score)| {
            serde_json::json!({
                "filename": filename,
                "path": path,
                "score": score,
            })
        }).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}

/// Print content search results to stdout.
pub fn print_content_results(results: &[(String, String, usize, String, i64)], query: &str, path: &str, show_line_numbers: bool, context_lines: usize, elapsed_secs: f64) {
    let separator = "─".repeat(50);
    eprintln!("{} Searching content: '{}' in {}", ">>".cyan(), query.cyan(), path);
    eprintln!("{}", separator.dimmed());

    // Group by file
    let mut current_file = String::new();
    for (file, _full_path, line, content, score) in results {
        if *file != current_file {
            current_file = file.clone();
            eprintln!("  {}", file.cyan());
        }
        if show_line_numbers {
            eprintln!("    {:>4}: {}", line.to_string().dimmed(), content);
        } else {
            eprintln!("    {}", content);
        }
    }

    if !results.is_empty() {
        eprintln!("{}", separator.dimmed());
        eprintln!("{} Found {} matches in {:.3}s", "✓".green(), results.len().to_string().green(), elapsed_secs);
    } else {
        eprintln!("{}", "No matches found.".yellow());
    }
}

/// Print content search results as JSON.
pub fn print_content_results_json(results: &[(String, String, usize, String, i64)], query: &str, elapsed_secs: f64) {
    let json_output = serde_json::json!({
        "tool": "codescope",
        "version": env!("CARGO_PKG_VERSION"),
        "query": query,
        "source": "filesystem",
        "count": results.len(),
        "elapsed_secs": elapsed_secs,
        "results": results.iter().map(|(file, path, line, content, score)| {
            serde_json::json!({
                "file": file,
                "path": path,
                "line": line,
                "content": content,
                "score": score,
            })
        }).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}

/// Print content count results.
pub fn print_content_count(results: &[(String, usize)]) {
    let separator = "─".repeat(50);
    eprintln!("{}", separator.dimmed());
    for (file, count) in results {
        eprintln!("  {} — {} matches", file.green(), count.to_string().yellow());
    }
    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} files with matches", "✓".green(), results.len());
}

/// Print web search results.
pub fn print_web_results(results: &[(String, String, String)]) {
    let separator = "─".repeat(50);
    for (i, (title, url, snippet)) in results.iter().enumerate() {
        eprintln!("  {}. {}", format!("{:2}", i + 1).dimmed(), title.green().bold());
        eprintln!("     {}", url.cyan());
        eprintln!("     {}", snippet.dimmed());
        if i < results.len() - 1 {
            eprintln!("{}", separator.dimmed());
        }
    }
}

/// Print web results as JSON.
pub fn print_web_results_json(results: &[(String, String, String)], query: &str, elapsed_secs: f64) {
    let json_output = serde_json::json!({
        "tool": "codescope",
        "version": env!("CARGO_PKG_VERSION"),
        "query": query,
        "source": "web",
        "count": results.len(),
        "elapsed_secs": elapsed_secs,
        "results": results.iter().map(|(title, url, snippet)| {
            serde_json::json!({
                "title": title,
                "url": url,
                "snippet": snippet,
            })
        }).collect::<Vec<_>>()
    });
    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}

/// Print content replace results (dry run).
pub fn print_replace_results(changes: &[(String, usize, String, String)]) {
    let separator = "─".repeat(50);
    eprintln!("{} Replace preview (dry run)", ">>".yellow());
    eprintln!("{}", separator.dimmed());
    for (file, line, old, new) in changes {
        eprintln!("  {}:{}  {} → {}", file.cyan(), line.to_string().dimmed(), old.red(), new.green());
    }
    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} replacement(s) (use --write to apply)", "✓".yellow(), changes.len());
}
