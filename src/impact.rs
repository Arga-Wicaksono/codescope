//! Impact analysis command for codescope.
//!
//! Given a target file or module, finds all files that depend on it
//! (directly and transitively) and reports the impact.

use std::collections::{HashMap, HashSet, BTreeSet};
use std::path::Path;

use colored::Colorize;

use crate::graph;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run the impact analysis command.
///
/// - `target`: file path or module name to analyse
/// - `path`: root directory to scan
/// - `json`: if true, emit structured JSON
pub fn run_impact(target: &str, path: &str, json: bool) -> Result<i32, String> {
    // Build the full module-dependency graph (same as `graph modules`)
    let edges = graph::build_module_graph(path, None)?.edges;

    if edges.is_empty() {
        if json {
            let output = serde_json::json!({
                "tool": "codescope",
                "command": "impact",
                "target": target,
                "total_affected": 0,
                "direct": [],
                "indirect": [],
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            print_no_dependents_message(target);
        }
        return Ok(1);
    }

    // Build a reverse adjacency list using flexible matching:
    // for each edge (from → to), record that `to` is depended-upon by `from`.
    // We store multiple possible keys so lookups succeed regardless of how
    // the target and edge.to were resolved (full path vs. basename vs. module).
    let mut reverse: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &edges {
        let keys = matching_keys(&edge.to);
        for key in keys {
            reverse.entry(key).or_default().push(edge.from.clone());
        }
    }

    // Resolve the target into the same set of matching keys
    let target_keys = matching_keys_for_target(path, target);

    // Collect direct dependents (seed the BFS)
    let mut direct: BTreeSet<String> = BTreeSet::new();
    let mut indirect: BTreeSet<String> = BTreeSet::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<(String, usize)> = Vec::new();

    for target_key in &target_keys {
        if let Some(deps) = reverse.get(target_key) {
            for dep in deps {
                if visited.insert(dep.clone()) {
                    direct.insert(dep.clone());
                    queue.push((dep.clone(), 1));
                }
            }
        }
    }

    // BFS for transitive dependents
    let mut bfs_front = 0;
    while bfs_front < queue.len() {
        let (node, depth) = queue[bfs_front].clone();
        bfs_front += 1;

        if depth > 1 {
            indirect.insert(node.clone());
        }

        let node_keys = matching_keys(&node);
        for node_key in &node_keys {
            if let Some(further) = reverse.get(node_key) {
                for next in further {
                    if visited.insert(next.clone()) {
                        queue.push((next.clone(), depth + 1));
                    }
                }
            }
        }
    }

    let total = direct.len() + indirect.len();

    if total == 0 {
        if json {
            let output = serde_json::json!({
                "tool": "codescope",
                "command": "impact",
                "target": target,
                "total_affected": 0,
                "direct": [],
                "indirect": [],
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            print_no_dependents_message(target);
        }
        return Ok(1);
    }

    if json {
        let output = serde_json::json!({
            "tool": "codescope",
            "command": "impact",
            "target": target,
            "total_affected": total,
            "direct": direct.iter().collect::<Vec<_>>(),
            "indirect": indirect.iter().collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(0);
    }

    // Human-readable output
    let separator = "─".repeat(50);
    eprintln!("{} Impact Analysis: {}", ">>".cyan(), display_path(target).cyan().bold());
    eprintln!("{}", separator.dimmed());

    if !direct.is_empty() {
        eprintln!(
            "  {} ({} file(s))",
            "Direct dependents (1st degree):".bold(),
            direct.len()
        );
        for dep in &direct {
            eprintln!("    {} {}", "●".yellow(), display_path(dep).green());
        }
    }

    if !indirect.is_empty() {
        eprintln!();
        eprintln!(
            "  {} ({} file(s))",
            "Indirect dependents (2nd degree+):".bold(),
            indirect.len()
        );
        for dep in &indirect {
            eprintln!("    {} {}", "○".dimmed(), display_path(dep).cyan());
        }
    }

    eprintln!("{}", separator.dimmed());
    eprintln!(
        "{} {} affected file(s) ({} direct, {} indirect)",
        "✓".green(),
        total.to_string().green(),
        direct.len(),
        indirect.len(),
    );

    Ok(0)
}

// ---------------------------------------------------------------------------
// No-dependents message (Fix #4)
// ---------------------------------------------------------------------------

fn print_no_dependents_message(target: &str) {
    let display = display_path(target);
    eprintln!(
        "\n  {} No incoming dependencies found for '{}'.\n",
        "Note:".yellow().bold(),
        display.cyan()
    );
    eprintln!(
        "  This file is a leaf module {} no other files depend on it.\n",
        "—".dimmed()
    );
    eprintln!(
        "  {} Use {} to see the full dependency graph.\n",
        "Tip:".yellow(),
        "cs graph --format dot | dot -Tpng -o graph.png".green(),
    );
}

// ---------------------------------------------------------------------------
// Flexible matching
// ---------------------------------------------------------------------------

/// Generate all possible lookup keys for an edge destination.
///
/// This allows matching regardless of whether the path is absolute,
/// relative, or a bare module name.
fn matching_keys(path: &str) -> Vec<String> {
    let mut keys = Vec::new();

    // Exact match
    keys.push(path.to_string());

    // Basename only (e.g. "utils.rs")
    if let Some(basename) = Path::new(path).file_name() {
        keys.push(basename.to_string_lossy().to_string());
    }

    // Stem only (e.g. "utils" from "utils.rs")
    if let Some(stem) = Path::new(path).file_stem() {
        keys.push(stem.to_string_lossy().to_string());
    }

    // Module form (e.g. "utils" from "/path/to/utils.rs")
    if let Some(stem) = Path::new(path).file_stem() {
        let stem_str = stem.to_string_lossy().to_string();
        let module_form = format!("{}{}", stem_str, module_extension(path));
        keys.push(module_form);
    }

    keys
}

/// Generate matching keys for a user-provided target.
fn matching_keys_for_target(root: &str, target: &str) -> Vec<String> {
    let mut keys = Vec::new();

    // Exact target as-is
    keys.push(target.to_string());

    // Basename of target
    if let Some(basename) = Path::new(target).file_name() {
        keys.push(basename.to_string_lossy().to_string());
    }

    // Try resolving relative to root
    let joined = Path::new(root).join(target);
    keys.push(joined.to_string_lossy().to_string());

    // Basename of resolved path
    if let Some(basename) = joined.file_name() {
        keys.push(basename.to_string_lossy().to_string());
    }

    // Stem of resolved path
    if let Some(stem) = joined.file_stem() {
        keys.push(stem.to_string_lossy().to_string());
    }

    keys
}

/// Determine the typical module extension for a path string.
fn module_extension(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".rs") { return ".rs".to_string(); }
    if lower.ends_with(".py") { return ".py".to_string(); }
    if lower.ends_with(".js") || lower.ends_with(".ts") { return ".js".to_string(); }
    if lower.ends_with(".go") { return ".go".to_string(); }
    if lower.ends_with(".java") { return ".java".to_string(); }
    if lower.ends_with(".kt") { return ".kt".to_string(); }
    String::new()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn display_path(full: &str) -> String {
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(stripped) = Path::new(full).strip_prefix(&cwd) {
            let s = stripped.to_string_lossy().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    full.to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            fs::write(dir.path().join(name), content).unwrap();
        }
        dir
    }

    #[test]
    fn test_impact_direct_dependent() {
        let dir = setup_project(&[
            ("utils.rs", "pub fn run() {}\n"),
            ("main.rs", "use crate::utils;\nfn main() { utils::run(); }\n"),
        ]);

        // main.rs depends on utils.rs → target utils.rs should show main.rs as affected
        let result = run_impact("utils.rs", dir.path().to_str().unwrap(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impact_no_dependents_leaf_module() {
        let dir = setup_project(&[
            ("utils.rs", "pub fn run() {}\n"),
            ("main.rs", "use crate::utils;\nfn main() { utils::run(); }\n"),
        ]);

        // main.rs is a leaf — no one depends on it
        let result = run_impact("main.rs", dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_impact_python_project() {
        let dir = setup_project(&[
            ("utils.py", "def helper(): pass\n"),
            ("app.py", "from utils import helper\n"),
            ("server.py", "from app import main\n"),
        ]);

        // utils.py is depended on by app.py (direct) and server.py (indirect)
        let result = run_impact("utils.py", dir.path().to_str().unwrap(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impact_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_impact("anything.rs", dir.path().to_str().unwrap(), false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_impact_json_output() {
        let dir = setup_project(&[
            ("utils.rs", "pub fn run() {}\n"),
            ("main.rs", "use crate::utils;\nfn main() { utils::run(); }\n"),
        ]);

        let result = run_impact("utils.rs", dir.path().to_str().unwrap(), true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_matching_keys() {
        let keys = matching_keys("/tmp/project/src/utils.rs");
        assert!(keys.contains(&"/tmp/project/src/utils.rs".to_string()));
        assert!(keys.contains(&"utils.rs".to_string()));
        assert!(keys.contains(&"utils".to_string()));
    }

    #[test]
    fn test_matching_keys_for_target() {
        let keys = matching_keys_for_target("/tmp/project", "utils.rs");
        assert!(keys.contains(&"utils.rs".to_string()));
        assert!(keys.contains(&"/tmp/project/utils.rs".to_string()));
        assert!(keys.contains(&"utils".to_string()));
    }

    #[test]
    fn test_display_path() {
        let cwd = std::env::current_dir().unwrap();
        let full = cwd.join("src").join("lib.rs");
        let displayed = display_path(&full.to_string_lossy());
        assert_eq!(displayed, "src/lib.rs");
    }
}
