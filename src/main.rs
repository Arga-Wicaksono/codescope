//! CodeScope (`cs`) — A blazing fast Rust CLI search tool.

use std::process;

use clap::Parser;
use colored::Colorize;

mod across;
mod cli;
mod config;
mod content_search;
mod explain;
mod file_search;
mod history;
mod open;
mod output;
mod output_schema;
mod recent;
mod stats;
mod types;
mod utils;
mod validate;
mod where_cmd;
mod symbol;
mod context;
mod graph;
mod serve;

#[cfg(feature = "interactive")]
mod interactive;

#[cfg(feature = "web-search")]
mod web_search;

use cli::{Cli, Commands, ShellName};

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
        "\n  {} {} — CodeScope\n",
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
    eprintln!("  {:<20} {}", "cs file <pattern>".green(), "Search files by name");
    eprintln!("  {:<20} {}", "cs content <pattern>".green(), "Search inside files");
    eprintln!("  {:<20} {}", "cs where <name>".green(), "Find definitions");
    eprintln!("  {:<20} {}", "cs open <pattern>".green(), "Find + open in editor");
    eprintln!("  {:<20} {}", "cs recent".green(), "Recently modified files");
    eprintln!("  {:<20} {}", "cs stats".green(), "Repository statistics");
    eprintln!("  {:<20} {}", "cs across <pattern>".green(), "Cross-repo search");
    eprintln!("  {:<20} {}", "cs web <query>".green(), "Search the web");
    eprintln!();
}

#[allow(dead_code)]
fn print_help() -> i32 {
    println!("\n{}", "cs — CodeScope".bold().cyan());
    println!("{}\n", format!("Version {}", VERSION).dimmed());

    println!("{}", "COMMANDS:".bold());
    println!("  {} <pattern>    Search files by name", "file".green());
    println!("  {} <pattern>    Search content inside files", "content".green());
    println!("  {} <pattern>    Search files and open in editor", "open".green());
    println!("  {} [options]    Show recently modified files", "recent".green());
    println!("  {} <name>       Find where functions/classes are defined", "where".green());
    println!("  {} <pattern>    Explain a regex pattern", "explain".green());
    println!("  {} [options]    Show search history", "history".green());
    println!("  {} <pattern>    Cross-repository search", "across".green());
    println!("  {}              File statistics", "stats".green());
    println!("  {} <shell>      Shell completions", "completions".green());
    println!("  {}              Show configuration", "config".green());
    println!();

    println!("{}", "EXIT CODES:".bold());
    println!("  0 = Results found");
    println!("  1 = No results found");
    println!("  2 = Error");
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

fn main() {
    let cfg = config::load_config();
    let cli = Cli::parse();

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

            match open::run_open(&pattern, search_path, effective_exclude, extensions_ref, hidden, effective_case, no_ignore, depth, interactive, line, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Recent {
            path, exclude, file_type, extension, hidden,
            no_ignore, since, limit, interactive, open, json,
        } => {
            match recent::run_recent(&path, exclude.as_deref(), file_type, extension.as_deref(), hidden, no_ignore, since.as_deref(), limit, interactive, open, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Where {
            name, path, exclude, file_type, extension,
            no_ignore, depth, interactive, open, json,
        } => {
            match where_cmd::run_where(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, interactive, open, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Explain { pattern, json } => {
            match explain::run_explain(&pattern, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
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
            match across::run_across(&pattern, repos.as_deref(), workspace.as_deref(), repos_file.as_deref(), file_type, extension.as_deref(), regex, exact, limit, json, interactive) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Stats { path, file_type, extension, json } => {
            match stats::run_stats(&path, file_type, extension.as_deref(), json) {
                Ok(_) => 0,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Completions { shell } => print_completions(shell),

        Commands::Config => {
            print_branded_banner();
            print_config_info(&cfg);
            0
        }

        Commands::Symbol {
            name, path, exclude, file_type, extension,
            symbol_type, no_ignore, depth, json,
        } => {
            match symbol::run_symbol(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), symbol_type.as_deref(), no_ignore, depth, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Refs {
            name, path, exclude, file_type, extension,
            no_ignore, depth, json,
        } => {
            match symbol::run_refs(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Callers {
            name, path, exclude, file_type, extension,
            no_ignore, depth, json,
        } => {
            match symbol::run_callers(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Symbols {
            path, exclude, file_type, extension,
            symbol_type, no_ignore, depth, limit, json,
        } => {
            match symbol::run_symbols(&path, exclude.as_deref(), file_type, extension.as_deref(), symbol_type.as_deref(), no_ignore, depth, limit, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Context {
            topic, path, exclude, file_type, extension,
            no_ignore, depth, max_items, json,
        } => {
            match context::run_context(&topic, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, max_items, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Pack {
            description, path, exclude, file_type, extension,
            no_ignore, depth, budget, json,
        } => {
            match context::run_pack(&description, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, budget, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Trace {
            name, path, exclude, file_type, extension,
            no_ignore, depth, max_depth, json,
        } => {
            match context::run_trace(&name, &path, exclude.as_deref(), file_type, extension.as_deref(), no_ignore, depth, max_depth, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Serve { mcp, http, port, path } => {
            match serve::run_serve(mcp, http, port, &path) {
                Ok(()) => 0,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Graph {
            path, graph_type, depth, format, json,
        } => {
            match graph::run_graph(&path, depth, &format, json, &graph_type) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Impact { target, path, json } => {
            match graph::run_impact(&path, &target, json) {
                Ok(code) => code,
                Err(e) => { eprintln!("{} {}", "Error:".red().bold(), e); 2 }
            }
        }

        Commands::Schema { command } => {
            match command.as_deref() {
                Some(cmd) => {
                    match output_schema::get_schema(cmd) {
                        Some(schema) => {
                            output_schema::print_json(&schema);
                            0
                        }
                        None => {
                            eprintln!("{} Unknown command '{}'. Valid: file, content, content-replace, content-count, web, where, stats, recent, across, open, explain, history",
                                "Error:".red().bold(), cmd);
                            2
                        }
                    }
                }
                None => {
                    // No command specified — list all available schemas
                    eprintln!("{}", "CodeScope — Available JSON Schemas".bold().cyan());
                    eprintln!("{}", "─".repeat(50).dimmed());
                    let schemas = &["file", "content", "content-replace", "content-count", "web", "where", "stats", "recent", "across", "open", "explain", "history"];
                    for s in schemas {
                        eprintln!("  {} {}", format!("cs schema {}", s).green(), format!("(cs {} --json)", if *s == "content-replace" { "content --replace X" } else if *s == "content-count" { "content --count" } else { s }).dimmed());
                    }
                    eprintln!("\n  {} Print a specific schema: {}", "Usage:".bold(), "cs schema <command>".green());
                    eprintln!();
                    0
                }
            }
        }
    };

    process::exit(exit_code);
}
