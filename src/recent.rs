use colored::Colorize;

pub fn run_recent(
    path: &str,
    exclude: Option<&str>,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    hidden: bool,
    no_ignore: bool,
    since: Option<&str>,
    limit: Option<usize>,
    interactive: bool,
    open: bool,
    json: bool,
) -> Result<i32, String> {
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };
    let extensions_ref = extensions.as_deref();

    let mut files: Vec<(String, std::time::SystemTime)> = Vec::new();

    let mut builder = ignore::WalkBuilder::new(path);
    builder.hidden(!hidden);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);

    if let Some(exclude_dirs) = exclude {
        for dir in exclude_dirs.split(',') {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                builder.add_custom_ignore_filename(trimmed);
            }
        }
    }

    for entry in builder.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();

        if let Some(exts) = extensions_ref {
            let matches = exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)));
            if !matches { continue; }
        }

        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                files.push((entry.path().to_string_lossy().to_string(), modified));
            }
        }
    }

    files.sort_by(|a, b| b.1.cmp(&a.1));

    // Apply since filter
    if let Some(since_str) = since {
        let duration = parse_relative_time(since_str);
        if let Some(dur) = duration {
            let cutoff = std::time::SystemTime::now() - dur;
            files.retain(|(_, mtime)| *mtime >= cutoff);
        }
    }

    let max = limit.unwrap_or(20);
    files.truncate(max);

    if json {
        let json_output = serde_json::json!({
            "tool": "codescope",
            "command": "recent",
            "count": files.len(),
            "results": files.iter().map(|(path, _)| path).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        return Ok(0);
    }

    if files.is_empty() {
        eprintln!("{}", "No recently modified files found.".yellow());
        return Ok(1);
    }

    let separator = "─".repeat(50);
    eprintln!("{} Recently modified files", ">>".cyan());
    eprintln!("{}", separator.dimmed());

    for (file_path, modified) in &files {
        let display = file_path.strip_prefix(path).unwrap_or(file_path);
        let display = display.trim_start_matches('/').trim_start_matches('\\');
        let time_ago = format_time_ago(modified);
        eprintln!("  {} {}", display.green(), format!("({} ago)", time_ago).dimmed());
    }

    eprintln!("{}", separator.dimmed());
    eprintln!("{} {} files", "✓".green(), files.len());

    if open && !files.is_empty() {
        let editor = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| "vim".to_string());
        let _ = std::process::Command::new(&editor).arg(&files[0].0).status();
    }

    Ok(0)
}

fn parse_relative_time(s: &str) -> Option<std::time::Duration> {
    let s = s.trim();
    if s.ends_with('h') || s.ends_with("hour") || s.ends_with("hours") {
        let num: u64 = s.trim_end_matches(|c| c == 'h' || c == ' ').trim().parse().ok()?;
        Some(std::time::Duration::from_secs(num * 3600))
    } else if s.ends_with('m') || s.ends_with("min") || s.ends_with("mins") {
        let num: u64 = s.trim_end_matches(|c: char| c == 'm' || c == 'i' || c == 'n' || c == 's' || c == ' ').trim().parse().ok()?;
        Some(std::time::Duration::from_secs(num * 60))
    } else if s.ends_with('d') || s.ends_with("day") || s.ends_with("days") {
        let num: u64 = s.trim_end_matches(|c: char| c == 'd' || c == 'a' || c == 'y' || c == 's' || c == ' ').trim().parse().ok()?;
        Some(std::time::Duration::from_secs(num * 86400))
    } else {
        None
    }
}

fn format_time_ago(modified: &std::time::SystemTime) -> String {
    match modified.elapsed() {
        Ok(dur) => {
            let secs = dur.as_secs();
            if secs < 60 { format!("{}s", secs) }
            else if secs < 3600 { format!("{}m", secs / 60) }
            else if secs < 86400 { format!("{}h", secs / 3600) }
            else { format!("{}d", secs / 86400) }
        }
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_recent_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn a() {}").unwrap();
        fs::write(dir.path().join("b.rs"), "fn b() {}").unwrap();

        let result = run_recent(dir.path().to_str().unwrap(), None, None, None, false, true, None, Some(10), false, false, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_parse_relative_time() {
        assert!(parse_relative_time("2h").is_some());
        assert!(parse_relative_time("30m").is_some());
        assert!(parse_relative_time("1d").is_some());
    }
}
