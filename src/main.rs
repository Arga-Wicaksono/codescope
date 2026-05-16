//! CodeScope (`cs`) — Repository Intelligence Engine for AI & Developers.

use std::process;

use clap::Parser;
use colored::Colorize;

use codescope::{
    across, cache, cli, config, content_search, context, explain,
    file_search, graph, history, impact, lsp_bridge, open, output,
    output_schema, recent, rewrite, semantic, serve, stats, symbol,
    types, utils, validate, where_cmd, embeddings, plugin_wasm,
};

#[cfg(feature = "interactive")]
use codescope::interactive;

#[cfg(feature = "web-search")]
use codescope::web_search;

use cli::{Cli, Commands, ShellName, CacheAction};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Resolve `--type` or `--extension` into a unified `Option<Vec<&str>>`.
fn resolve_extensions(
    file_type: Option<types::FileType>,
    extension: Option<&str>,
) -> Option<Vec<&str>> {
    match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    }
}

fn print_banner() {
    eprintln!(
        "\n  {} {} — Repository Intelligence Engine\n",
        "cs".bold().cyan(),
        format!("v{}", VERSION).dimmed()
    );
}

fn print_branded_banner() {
    let art = r#"
     ____                 __
    / __/___  ____  _____/ /_
   / /_/ __ \/ __ \/ ___/ __/
  / __/ /_/ / / / / /__/ /_
 /_/  \____/_/ /_/\___/\__/
    "#;
    eprintln!("{}", art.dimmed());
    eprintln!(
        "  {} {} — Repository Intelligence Engine\n",
        "cs".bold().cyan(),
        format!("v{}", VERSION).yellow()
    );
    eprintln!("  {} Run {} for full help\n",
        "Quick start:".bold(),
        "cs help".green()
    );
    eprintln!("  {:<24} {}", "cs file <pattern>".green(), "Search files by name");
    eprintln!("  {:<24} {}", "cs content <pattern>".green(), "Search inside files");
    eprintln!("  {:<24} {}", "cs symbol <name>".green(), "Find symbol definitions");
    eprintln!("  {:<24} {}", "cs where <name>".green(), "Find definitions");
    eprintln!("  {:<24} {}", "cs context <topic>".green(), "Extract context for AI");
    eprintln!("  {:<24} {}", "cs graph".green(), "Dependency graph");
    eprintln!("  {:<24} {}", "cs impact <target>".green(), "Impact analysis");
    eprintln!("  {:<24} {}", "cs semantic <query>".green(), "Semantic (TF-IDF) search");
    eprintln!("  {:<24} {}", "cs rewrite <instruction>".green(), "AI-powered rewrite");
    eprintln!("  {:<24} {}", "cs serve --mcp".green(), "MCP server for AI agents");
    eprintln!();
}

#[allow(dead_code)]
fn print_help() -> i32 {
    println!("\n{}", "cs — Repository Intelligence Engine".bold().cyan());
    println!("{}\n", format!("Version {}", VERSION).dimmed());

    println!("{}", "SEARCH & NAVIGATION:".bold());
    println!("  {} <pattern>    Search files by name", "file".green());
    println!("  {} <pattern>    Search content inside files", "content".green());
    println!("  {} <pattern>    Semantic search (TF-IDF)", "semantic".green());
    println!("  {} <pattern>    Search the web", "web".green());
    println!("  {} <pattern>    Find + open in editor", "open".green());
    println!("  {} [options]    Recently modified files", "recent".green());
    println!("  {} <pattern>    Cross-repository search", "across".green());

    println!("\n{}", "SYMBOL INTELLIGENCE:".bold());
    println!("  {} <name>       Find definitions", "where".green());
    println!("  {} <name>       Find symbol with metadata", "symbol".green());
    println!("  {} <name>       Find all references", "refs".green());
    println!("  {} <name>       Find callers of a function", "callers".green());
    println!("  {} [path]       List all symbols", "symbols".green());

    println!("\n{}", "CONTEXT ENGINE:".bold());
    println!("  {} <topic>      Extract context for AI", "context".green());
    println!("  {} <desc>       Pack context for LLM prompts", "pack".green());
    println!("  {} <symbol>     Trace execution flow", "trace".green());

    println!("\n{}", "DEPENDENCY GRAPH:".bold());
    println!("  {}              Module dependency graph", "graph".green());
    println!("  {} <target>     Impact analysis", "impact".green());

    println!("\n{}", "AI & INTEGRATION:".bold());
    println!("  {} <instruction> AI-powered rewrite", "rewrite".green());
    println!("  {} [--mcp|--http] MCP/HTTP server", "serve".green());
    println!("  {} [port]       LSP bridge for editors", "lsp-bridge".green());

    println!("\n{}", "DEVELOPER TOOLS:".bold());
    println!("  {}              File statistics", "stats".green());
    println!("  {} <pattern>    Explain regex pattern", "explain".green());
    println!("  {} [options]    Search history", "history".green());
    println!("  {}              Cache management", "cache".green());
    println!("  {}              Configuration", "config".green());
    println!();

    0
}

fn print_completions(shell: ShellName) -> i32 {
    use clap::CommandFactory;
    use clap_complete::{generate, shells::{Bash, Elvish, Fish, PowerShell, Zsh}};

    let mut app = Cli::command();

    match shell {
        ShellName::Bash => generate(Bash, &mut app, "cs", &mut std::io::stdout()),
        ShellName::Zsh => generate(Zsh, &mut app, "cs", &mut std::io::stdout()),
        ShellName::Fish => generate(Fish, &mut app, "cs", &mut std::io::stdout()),
        ShellName::PowerShell => generate(PowerShell, &mut app, "cs", &mut std::io::stdout()),
        ShellName::Elvish => generate(Elvish, &mut app, "cs", &mut std::io::stdout()),
    }
    0
}

fn print_config_info(cfg: &config::Config) -> i32 {
    println!("\n{}", "cs — Configuration".bold().cyan());
    println!("{}", "─".repeat(50).dimmed());
    println!("  {} {:?}", "default_limit:".dimmed(), cfg.default_limit);
    println!("  {} {:?}", "default_depth:".dimmed(), cfg.default_depth);
    println!("  {} {:?}", "default_exclude:".dimmed(), cfg.default_exclude);
    println!("  {} {:?}", "default_extension:".dimmed(), cfg.default_extension);
    println!("  {} {:?}", "color:".dimmed(), cfg.color);
    println!("  {} {:?}", "web_timeout:".dimmed(), cfg.web_timeout);
    println!("  {} {:?}", "interactive:".dimmed(), cfg.interactive);
    println!("{}", "─".repeat(50).dimmed());

    if let Some(path) = config::default_config_path() {
        println!("  {} {}", "Config path:".dimmed(), path.display().to_string().cyan());
        let exists = path.exists();
        println!("  {} {}",
            "File exists:".dimmed(),
            if exists { "yes".green() } else { "no (using defaults)".yellow() }
        );
    }

    println!("\n  {} Create: echo '{{\"default_limit\": 50}}' > ~/.codescope.json",
        "Tip:".yellow());
    println!();
    0
}

/// Helper to handle command results with helpful error messages.
fn handle_result(result: Result<i32, String>) -> i32 {
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            2
        }
    }
}

fn main() {
    let cfg = config::load_config();
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // FIX #3: Provide helpful suggestions on invalid commands
            let err_str = e.to_string();
            if err_str.contains("Unrecognized subcommand") {
                // Extract the unknown command name
                let unknown = err_str
                    .lines()
                    .find(|l| l.contains("Unrecognized subcommand"))
                    .and_then(|l| l.split('"').nth(1))
                    .unwrap_or("");
                eprintln!("{}", validate::unknown_command_help(unknown));
                process::exit(2);
            }
            eprintln!("{}", err_str);
            process::exit(2);
        }
    };

    let no_color = cli.no_color || cfg.color == Some(false);
    output::configure_colors(no_color);

    if cli.verbose {
        print_banner();
    }

    let default_limit = cfg.default_limit.unwrap_or(20);
    let default_web_timeout = cfg.web_timeout.unwrap_or(10);

    let command = cli.command.unwrap_or(Commands::Config);

    let exit_code = match command {
        Commands::File {
            pattern, path, exclude, extension, file_type,
            hidden, case_insensitive, case_sensitive,
            no_ignore, depth, limit, json, interactive,
        } => {
            let effective_limit = limit.unwrap_or(default_limit);
            let effective_exclude = exclude.as_deref();
            let effective_case = utils::resolve_case_insensitive(&pattern, case_insensitive, case_sensitive);
            let extensions = resolve_extensions(file_type, extension.as_deref());
            let extensions_ref = extensions.as_deref();
            let search_path = path.as_deref().unwrap_or(".");

            match file_search::search_files(
                &pattern, search_path, effective_exclude, extensions_ref,
                hidden, effective_case, no_ignore, depth, effective_limit, json,
            ) {
                Ok(true) if interactive && !json => {
                    #[cfg(feature = "interactive")]
                    {
                        match file_search::collect_file_results(&pattern, search_path, effective_exclude, extensions_ref, hidden, effective_case, no_ignore, depth) {
                            Ok(mut results) => {
                                results.truncate(effective_limit);
                                match interactive::interactive_file_select(&results) {
                                    Some(selected) => { interactive::print_file_selection(&selected); 0 }
                                    None => 1,
                                }
                            }
                            Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                        }
                    }
                    #[cfg(not(feature = "interactive"))]
                    { eprintln!("{} Interactive mode requires the 'interactive' feature", "Error:".red().bold()); 2 }
                }
                Ok(found) => if found { 0 } else { 1 },
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Content {
            pattern, path, extension, file_type, regex, exact,
            replace, write, count, invert, exclude,
            case_insensitive, case_sensitive, no_ignore,
            line_number, context, depth, limit, json, interactive,
        } => {
            let effective_limit = limit.unwrap_or(default_limit);
            let effective_exclude = exclude.as_deref();
            let mode = content_search::resolve_match_mode(regex, exact);
            let effective_case = utils::resolve_case_insensitive(&pattern, case_insensitive, case_sensitive);
            let extensions = resolve_extensions(file_type, extension.as_deref());
            let extensions_ref = extensions.as_deref();
            let search_path = path.as_deref().unwrap_or(".");

            if let Some(replacement) = &replace {
                match content_search::search_content_replace(&pattern, search_path, extensions_ref, mode, effective_exclude, effective_case, no_ignore, depth, replacement, write, json) {
                    Ok(found) => if found { 0 } else { 1 },
                    Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                }
            } else if count {
                if content_search::stdin_has_data() {
                    match content_search::search_content_stdin(&pattern, mode, effective_case, line_number, context, effective_limit, json, invert, true) {
                        Ok(found) => if found { 0 } else { 1 },
                        Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                    }
                } else {
                    match content_search::search_content_count(&pattern, search_path, extensions_ref, mode, effective_exclude, effective_case, no_ignore, depth, json, invert) {
                        Ok(found) => if found { 0 } else { 1 },
                        Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                    }
                }
            } else if content_search::stdin_has_data() {
                match content_search::search_content_stdin(&pattern, mode, effective_case, line_number, context, effective_limit, json, invert, false) {
                    Ok(found) => if found { 0 } else { 1 },
                    Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                }
            } else {
                match content_search::search_content(&pattern, search_path, extensions_ref, mode, effective_exclude, effective_case, no_ignore, line_number, context, depth, effective_limit, json, invert) {
                    Ok(true) if interactive && !json => {
                        #[cfg(feature = "interactive")]
                        {
                            match content_search::collect_content_results(&pattern, search_path, extensions_ref, mode, effective_exclude, effective_case, no_ignore, context, depth, invert) {
                                Ok(mut results) => {
                                    results.truncate(effective_limit);
                                    match interactive::interactive_content_select(&results) {
                                        Some((file, line, preview)) => { interactive::print_content_selection(&file, line, &preview); 0 }
                                        None => 1,
                                    }
                                }
                                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                            }
                        }
                        #[cfg(not(feature = "interactive"))]
                        { eprintln!("{} Interactive mode requires the 'interactive' feature", "Error:".red().bold()); 2 }
                    }
                    Ok(found) => if found { 0 } else { 1 },
                    Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                }
            }
        }

        Commands::Open {
            pattern, path, exclude, extension, file_type,
            hidden, case_insensitive, case_sensitive,
            no_ignore, depth, line, interactive, json,
        } => {
            let effective_exclude = exclude.as_deref();
            let effective_case = utils::resolve_case_insensitive(&pattern, case_insensitive, case_sensitive);
            let extensions = resolve_extensions(file_type, extension.as_deref());
            let extensions_ref = extensions.as_deref();
            let search_path = path.as_deref().unwrap_or(".");
            handle_result(open::run_open(&pattern, search_path, effective_exclude, extensions_ref, hidden, effective_case, no_ignore, depth, interactive, line, json))
        }

        Commands::Recent {
            path, exclude, file_type, extension, hidden,
            no_ignore, since, limit, interactive, open, json,
        } => {
            handle_result(recent::run_recent(&path, exclude.as_deref(), file_type, extension.as_deref(), hidden, no_ignore, since.as_deref(), limit, interactive, open, json))
        }

        Commands::Where {
            name, path, exclude, file_type, extension,
            no_ignore, depth, interactive, open, json,
        } => {
            handle_result(where_cmd::run_where(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, interactive, open, json))
        }

        Commands::Explain { pattern, json } => {
            handle_result(explain::run_explain(&pattern, json))
        }

        Commands::History { limit, json } => {
            match history::show_history(limit, json) {
                Ok(_) => 0,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        #[cfg(feature = "web-search")]
        Commands::Web { query, limit, timeout, json, interactive } => {
            let effective_limit = limit.unwrap_or(10);
            let effective_timeout = timeout.unwrap_or(default_web_timeout);

            match web_search::search_web(&query, effective_limit, effective_timeout, json) {
                Ok(true) if interactive && !json => {
                    #[cfg(feature = "interactive")]
                    {
                        match web_search::collect_web_results(&query, effective_limit, effective_timeout) {
                            Ok(results) => {
                                match interactive::interactive_web_select(&results) {
                                    Some(url) => { interactive::print_web_selection(&url); 0 }
                                    None => 1,
                                }
                            }
                            Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                        }
                    }
                    #[cfg(not(feature = "interactive"))]
                    { eprintln!("{} Interactive mode requires the 'interactive' feature", "Error:".red().bold()); 2 }
                }
                Ok(found) => if found { 0 } else { 1 },
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Across {
            pattern, repos, workspace, repos_file,
            file_type, extension, regex, exact,
            limit, json, interactive,
        } => {
            handle_result(across::run_across(&pattern, repos.as_deref(), workspace.as_deref(), repos_file.as_deref(), file_type, extension.as_deref(), regex, exact, limit, json, interactive))
        }

        Commands::Stats { path, file_type, extension, json } => {
            match stats::run_stats(&path, file_type, extension.as_deref(), json) {
                Ok(_) => 0,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        // ── Symbol Intelligence ──

        Commands::Symbol {
            name, path, exclude, file_type, extension,
            symbol_type, no_ignore, depth, json,
        } => {
            handle_result(symbol::run_symbol(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), symbol_type.as_deref(), no_ignore, depth, json))
        }

        Commands::Refs {
            name, path, exclude, file_type, extension,
            no_ignore, depth, json,
        } => {
            handle_result(symbol::run_refs(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, json))
        }

        Commands::Callers {
            name, path, exclude, file_type, extension,
            no_ignore, depth, json,
        } => {
            handle_result(symbol::run_callers(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, json))
        }

        Commands::Symbols {
            path, exclude, file_type, extension,
            symbol_type, no_ignore, depth, limit, json,
        } => {
            handle_result(symbol::run_symbols(&path, exclude.as_deref(), file_type, extension.as_deref(), symbol_type.as_deref(), no_ignore, depth, limit, json))
        }

        // ── Context Engine ──

        Commands::Context {
            topic, path, exclude, file_type, extension,
            no_ignore, depth, limit, json,
        } => {
            handle_result(context::run_context(&topic, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, limit, json))
        }

        Commands::Pack {
            description, path, exclude, file_type, extension,
            no_ignore, depth, budget, json,
        } => {
            handle_result(context::run_pack(&description, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, budget, json))
        }

        Commands::Trace {
            name, path, exclude, file_type, extension,
            no_ignore, depth, json,
        } => {
            handle_result(context::run_trace(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, None, depth, json))
        }

        // ── Dependency Graph ──

        Commands::Graph {
            path, graph_type, depth, format, json,
        } => {
            handle_result(graph::run_graph(&path, &graph_type, depth, &format, json))
        }

        Commands::Impact { target, path, json } => {
            handle_result(impact::run_impact(&target, &path, json))
        }

        // ── Serve ──

        Commands::Serve { mcp, http, port, path } => {
            match serve::run_serve(mcp, http, port, &path) {
                Ok(_) => 0,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        // ── Semantic Search ──

        Commands::Semantic {
            query, path, file_type, extension,
            no_ignore, depth, limit, json, vector,
        } => {
            handle_result(semantic::run_semantic(&query, &path, file_type, extension.as_deref(), no_ignore, depth, limit, json, vector))
        }

        // ── Cache ──

        Commands::Cache { action } => {
            let mgr = cache::CacheManager::new(24, 100);
            match action {
                CacheAction::Stats => {
                    let s = mgr.stats();
                    match serde_json::to_string_pretty(&s) {
                        Ok(json) => println!("{}", json),
                        Err(e) => eprintln!("{} Failed to serialize cache stats: {}", "Error:".red().bold(), e),
                    }
                    0
                }
                CacheAction::Clear => {
                    handle_result(mgr.clear().map(|_| 0))
                }
                CacheAction::Cleanup => {
                    match mgr.cleanup() {
                        Ok(n) => {
                            eprintln!("{} Removed {} expired entries", "✓".green(), n);
                            0
                        }
                        Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
                    }
                }
            }
        }

        // ── AI Rewrite ──

        Commands::Rewrite {
            instruction, path, symbol, file_type, extension,
            no_ignore, depth, model, budget, dry_run, write, json,
        } => {
            handle_result(rewrite::run_rewrite(&instruction, &path, symbol.as_deref(), file_type, extension.as_deref(), no_ignore, depth, model.as_deref(), budget, dry_run, write, json))
        }

        // ── LSP Bridge ──

        Commands::LspBridge { port } => {
            match lsp_bridge::run_lsp_bridge(port) {
                Ok(_) => 0,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        // ── Misc ──

        Commands::Completions { shell } => print_completions(shell),

        Commands::Config => {
            print_branded_banner();
            print_config_info(&cfg);
            0
        }

        Commands::Schema { command } => {
            // Stub: schema command — for now just show available commands
            if let Some(cmd) = command {
                eprintln!("{} Schema for '{}' — use {} to see JSON output examples",
                    ">>".cyan(), cmd.green(), format!("cs {} -j", cmd).yellow());
            } else {
                eprintln!("{}", "Available schemas:".bold());
                for cmd in &["file", "content", "symbol", "refs", "callers", "symbols", "context", "pack", "trace", "graph", "impact", "stats", "semantic", "recent", "across", "where"] {
                    eprintln!("  {}", cmd.green());
                }
            }
            0
        }
    };

    process::exit(exit_code);
}
