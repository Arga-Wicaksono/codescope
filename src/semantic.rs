//! Semantic search using a lightweight TF-IDF approach (no external ML library needed).
//!
//! Tokenizes source files into words, computes TF-IDF vectors, and ranks
//! documents by cosine similarity against the query.

use colored::Colorize;
use ignore::WalkBuilder;
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::embeddings;
use crate::utils::Timer;
use crate::validate;

// ---------------------------------------------------------------------------
// Stop words
// ---------------------------------------------------------------------------

/// Hardcoded list of common English stop words plus common programming keywords.
fn stop_words() -> HashSet<&'static str> {
    let words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "can", "shall", "to", "of", "in", "for",
        "on", "with", "at", "by", "from", "as", "into", "through", "during",
        "before", "after", "above", "below", "between", "out", "off", "over",
        "under", "again", "further", "then", "once", "here", "there", "when",
        "where", "why", "how", "all", "each", "every", "both", "few", "more",
        "most", "other", "some", "such", "no", "nor", "not", "only", "own",
        "same", "so", "than", "too", "very", "just", "because", "but", "and",
        "or", "if", "while", "that", "this", "it", "its", "fn", "pub", "let",
        "mut", "use", "mod", "impl", "struct", "enum", "trait", "def", "class",
        "self", "return", "import", "from", "const", "var", "function",
        "package", "new", "nil", "true", "false",
    ];
    HashSet::from(words)
}

// ---------------------------------------------------------------------------
// Tokenisation
// ---------------------------------------------------------------------------

/// Split a string into lowercase tokens, stripping punctuation and filtering
/// stop words.
fn tokenize(text: &str, stops: &HashSet<&str>) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .filter(|s| !stops.contains(s.as_str()) && s.len() > 1)
        .collect()
}

// ---------------------------------------------------------------------------
// TF-IDF types
// ---------------------------------------------------------------------------

/// A document in the corpus, keeping its file path and content.
struct Document {
    path: String,
    lines: Vec<(usize, String)>, // (line_number, line_text)
    all_text: String,
}

/// Result entry for the caller.
struct SemanticResult {
    path: String,
    line: usize,
    content: String,
    score: f64,
}

// ---------------------------------------------------------------------------
// TF-IDF computation
// ---------------------------------------------------------------------------

/// Compute term frequency for a list of tokens.
fn term_frequency(tokens: &[String]) -> HashMap<String, f64> {
    let mut tf: HashMap<String, f64> = HashMap::new();
    if tokens.is_empty() {
        return tf;
    }
    for token in tokens {
        *tf.entry(token.clone()).or_insert(0.0) += 1.0;
    }
    let total = tokens.len() as f64;
    for v in tf.values_mut() {
        *v /= total;
    }
    tf
}

/// Compute IDF across a corpus of token lists.
/// IDF(t) = ln(N / (1 + df(t))) where df(t) = number of documents containing t.
fn inverse_document_frequency(
    doc_token_sets: &[HashSet<String>],
    all_terms: &HashSet<String>,
) -> HashMap<String, f64> {
    let n = doc_token_sets.len() as f64;
    let mut idf: HashMap<String, f64> = HashMap::new();
    for term in all_terms {
        let df = doc_token_sets.iter().filter(|ts| ts.contains(term)).count() as f64;
        idf.insert(term.clone(), (n / (1.0 + df)).ln());
    }
    idf
}

/// Compute cosine similarity between two sparse vectors represented as
/// `HashMap<String, f64>`.
fn cosine_similarity(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let mut dot = 0.0_f64;
    for (term, va) in a {
        if let Some(vb) = b.get(term) {
            dot += va * vb;
        }
    }
    let mag_a: f64 = a.values().map(|v| v * v).sum::<f64>().sqrt();
    let mag_b: f64 = b.values().map(|v| v * v).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run semantic search over files in `path`, ranking results by TF-IDF cosine
/// similarity with the query.
pub fn run_semantic(
    query: &str,
    path: &str,
    file_type: Option<crate::types::FileType>,
    extension: Option<&str>,
    no_ignore: bool,
    depth: Option<usize>,
    limit: Option<usize>,
    json: bool,
    vector: bool,
) -> Result<i32, String> {
    validate::validate_pattern(query)?;
    validate::validate_path(path)?;

    let timer = Timer::new();
    let stops = stop_words();
    let effective_limit = limit.unwrap_or(20);

    // Resolve extensions
    let extensions: Option<Vec<&str>> = match (file_type, extension) {
        (Some(ft), _) => Some(ft.extensions().to_vec()),
        (_, Some(ext)) => Some(vec![ext]),
        _ => None,
    };

    // 1. Walk directory and collect files
    let mut builder = WalkBuilder::new(path);
    builder.git_ignore(!no_ignore);
    builder.git_global(!no_ignore);
    builder.git_exclude(!no_ignore);
    if let Some(d) = depth {
        builder.max_depth(Some(d));
    }

    let files: Vec<String> = builder
        .build()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.file_type()?.is_file() {
                return None;
            }
            let p = entry.path().to_string_lossy().to_string();

            if let Some(ref exts) = extensions {
                let file_name = entry.file_name().to_string_lossy();
                let matches = exts.iter().any(|ext| file_name.ends_with(&format!(".{}", ext)));
                if !matches {
                    return None;
                }
            }

            // Skip binary-like files (heuristic: skip if we can't read as UTF-8)
            if let Ok(content) = fs::read_to_string(&p) {
                if content.len() > 5_000_000 {
                    return None; // skip files > 5 MB
                }
            } else {
                return None;
            }

            Some(p)
        })
        .collect();

    if files.is_empty() {
        if json {
            let out = serde_json::json!({
                "tool": "codescope",
                "command": "semantic",
                "query": query,
                "count": 0,
                "results": [],
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            eprintln!("{}", "No searchable files found.".yellow());
        }
        return Ok(1);
    }

    // 2. Build document corpus
    let mut documents: Vec<Document> = Vec::new();
    for fp in &files {
        if let Ok(content) = fs::read_to_string(fp) {
            let lines: Vec<(usize, String)> = content
                .lines()
                .enumerate()
                .map(|(i, l)| (i + 1, l.to_string()))
                .collect();
            documents.push(Document {
                path: fp.clone(),
                all_text: content,
                lines,
            });
        }
    }

    // ── Vector search path ──
    if vector {
        return run_vector_semantic(query, &documents, effective_limit, json, timer);
    }

    // ── TF-IDF search path (original) ──

    // 3. Tokenize all documents
    let doc_tokens: Vec<Vec<String>> = documents
        .iter()
        .map(|d| tokenize(&d.all_text, &stops))
        .collect();

    let doc_token_sets: Vec<HashSet<String>> = doc_tokens
        .iter()
        .map(|t| t.iter().cloned().collect())
        .collect();

    // Collect all unique terms across the entire corpus
    let all_terms: HashSet<String> = doc_token_sets.iter().flatten().cloned().collect();

    // 4. Tokenize the query and merge into the term universe
    let query_tokens = tokenize(query, &stops);
    if query_tokens.is_empty() {
        if json {
            let out = serde_json::json!({
                "tool": "codescope",
                "command": "semantic",
                "query": query,
                "count": 0,
                "results": [],
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            eprintln!("{}", "Query contains only stop words — no meaningful terms to search.".yellow());
        }
        return Ok(1);
    }

    let mut extended_terms: HashSet<String> = all_terms.clone();
    for t in &query_tokens {
        extended_terms.insert(t.clone());
    }

    // 5. Compute IDF (include query terms in corpus for IDF)
    let mut idf_doc_sets = doc_token_sets.clone();
    let query_set: HashSet<String> = query_tokens.iter().cloned().collect();
    idf_doc_sets.push(query_set.clone());
    let idf = inverse_document_frequency(&idf_doc_sets, &extended_terms);

    // 6. Compute TF-IDF for query
    let query_tf = term_frequency(&query_tokens);
    let query_tfidf: HashMap<String, f64> = query_tf
        .iter()
        .map(|(term, tf)| {
            let idf_val = idf.get(term).copied().unwrap_or(0.0);
            (term.clone(), tf * idf_val)
        })
        .collect();

    // 7. Score each document and find the best-matching line within it
    let mut results: Vec<SemanticResult> = Vec::new();

    for (idx, doc) in documents.iter().enumerate() {
        // Skip documents with no overlap at all
        let doc_set = &doc_token_sets[idx];
        if query_set.iter().all(|t| !doc_set.contains(t)) {
            continue;
        }

        // Whole-document score for ranking
        let doc_tf = term_frequency(&doc_tokens[idx]);
        let doc_tfidf: HashMap<String, f64> = doc_tf
            .iter()
            .map(|(term, tf)| {
                let idf_val = idf.get(term).copied().unwrap_or(0.0);
                (term.clone(), tf * idf_val)
            })
            .collect();

        let doc_score = cosine_similarity(&query_tfidf, &doc_tfidf);
        if doc_score < 0.01 {
            continue;
        }

        // Find the best-matching line in this document
        let mut best_line_score = 0.0_f64;
        let mut best_line_num = 1;
        let mut best_line_text = String::new();

        for (line_num, line_text) in &doc.lines {
            let line_tokens = tokenize(line_text, &stops);
            if line_tokens.is_empty() {
                continue;
            }
            // Check at least one query term is present
            let line_set: HashSet<&str> = line_tokens.iter().map(|s| s.as_str()).collect();
            if !query_tokens.iter().any(|t| line_set.contains(t.as_str())) {
                continue;
            }
            let line_tf = term_frequency(&line_tokens);
            let line_tfidf: HashMap<String, f64> = line_tf
                .iter()
                .map(|(term, tf)| {
                    let idf_val = idf.get(term).copied().unwrap_or(0.0);
                    (term.clone(), tf * idf_val)
                })
                .collect();
            let line_score = cosine_similarity(&query_tfidf, &line_tfidf);
            if line_score > best_line_score {
                best_line_score = line_score;
                best_line_num = *line_num;
                best_line_text = line_text.clone();
            }
        }

        // Use the document score if no individual line matched
        let display_score = if best_line_score > 0.0 {
            best_line_score
        } else {
            doc_score
        };
        let display_line = if best_line_score > 0.0 {
            best_line_num
        } else {
            doc.lines.first().map(|(n, _)| *n).unwrap_or(1)
        };
        let display_content = if best_line_text.is_empty() {
            doc.lines.first().map(|(_, t)| t.clone()).unwrap_or_default()
        } else {
            best_line_text
        };

        results.push(SemanticResult {
            path: doc.path.clone(),
            line: display_line,
            content: display_content,
            score: display_score,
        });
    }

    // 8. Rank by score descending
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(effective_limit);

    let elapsed = timer.elapsed_secs();

    // 9. Output
    if json {
        let out = serde_json::json!({
            "tool": "codescope",
            "command": "semantic",
            "query": query,
            "count": results.len(),
            "elapsed_secs": elapsed,
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "path": r.path,
                    "line": r.line,
                    "content": r.content,
                    "score": r.score,
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        let separator = "─".repeat(50);
        eprintln!(
            "{} Semantic search: '{}' in {}",
            ">>".cyan(),
            query.cyan(),
            path
        );
        eprintln!("{}", separator.dimmed());

        if results.is_empty() {
            eprintln!("{}", "No semantically similar results found.".yellow());
        } else {
            for (i, r) in results.iter().enumerate() {
                let score_pct = (r.score * 100.0).round() as usize;
                let score_color = if score_pct >= 50 {
                    "green"
                } else if score_pct >= 20 {
                    "yellow"
                } else {
                    "dimmed"
                };
                let score_str = format!("{:.2}", r.score);
                eprintln!(
                    "  {} {} {}:{}",
                    format!("{:3}", i + 1).dimmed(),
                    match score_color {
                        "green" => score_str.green().bold(),
                        "yellow" => score_str.yellow(),
                        _ => score_str.dimmed(),
                    },
                    r.path.cyan(),
                    r.line.to_string().dimmed(),
                );
                eprintln!("      {}", r.content);
            }
            eprintln!("{}", separator.dimmed());
            eprintln!(
                "{} Found {} result(s) in {:.3}s",
                "✓".green(),
                results.len().to_string().green(),
                elapsed
            );
        }
    }

    Ok(if results.is_empty() { 1 } else { 0 })
}

// ---------------------------------------------------------------------------
// Vector-based semantic search
// ---------------------------------------------------------------------------

/// Run semantic search using vector embeddings (cosine similarity).
fn run_vector_semantic(
    query: &str,
    documents: &[Document],
    limit: usize,
    json: bool,
    timer: Timer,
) -> Result<i32, String> {
    // Build texts: query + one text per document (first 2000 chars)
    let doc_snippets: Vec<&str> = documents.iter().map(|d| {
        if d.all_text.len() > 2000 {
            &d.all_text[..2000]
        } else {
            &d.all_text
        }
    }).collect();

    let all_texts: Vec<&str> = std::iter::once(query)
        .chain(doc_snippets.iter().copied())
        .collect();

    // Generate embeddings for all texts
    let result = embeddings::embed_texts(&all_texts);
    let query_embedding = &result.embeddings[0];

    // Score each document by cosine similarity
    let mut scored: Vec<(usize, f64)> = documents
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let sim = query_embedding.cosine_similarity(&result.embeddings[i + 1]);
            (i, sim)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    let elapsed = timer.elapsed_secs();
    let model_used = &result.model_used;

    // Build results
    let results: Vec<SemanticResult> = scored
        .iter()
        .map(|&(idx, score)| {
            let doc = &documents[idx];
            let (line, content) = doc.lines.first()
                .map(|(n, t)| (*n, t.clone()))
                .unwrap_or((1, String::new()));
            SemanticResult {
                path: doc.path.clone(),
                line,
                content,
                score,
            }
        })
        .collect();

    // Output
    if json {
        let out = serde_json::json!({
            "tool": "codescope",
            "command": "semantic",
            "query": query,
            "model": model_used,
            "count": results.len(),
            "elapsed_secs": elapsed,
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "path": r.path,
                    "line": r.line,
                    "content": r.content,
                    "score": r.score,
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        let separator = "─".repeat(50);
        eprintln!(
            "{} Semantic vector search: '{}' (model: {})",
            ">>".cyan(),
            query.cyan(),
            model_used
        );
        eprintln!("{}", separator.dimmed());

        if results.is_empty() {
            eprintln!("{}", "No semantically similar results found.".yellow());
        } else {
            for (i, r) in results.iter().enumerate() {
                let score_pct = (r.score * 100.0).round() as usize;
                let score_color = if score_pct >= 50 {
                    "green"
                } else if score_pct >= 20 {
                    "yellow"
                } else {
                    "dimmed"
                };
                let score_str = format!("{:.4}", r.score);
                eprintln!(
                    "  {} {} {}:{}",
                    format!("{:3}", i + 1).dimmed(),
                    match score_color {
                        "green" => score_str.green().bold(),
                        "yellow" => score_str.yellow(),
                        _ => score_str.dimmed(),
                    },
                    r.path.cyan(),
                    r.line.to_string().dimmed(),
                );
                eprintln!("      {}", r.content);
            }
            eprintln!("{}", separator.dimmed());
            eprintln!(
                "{} Found {} result(s) in {:.3}s",
                "✓".green(),
                results.len().to_string().green(),
                elapsed
            );
        }
    }

    Ok(if results.is_empty() { 1 } else { 0 })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: create a temporary directory with some test files.
    fn setup_test_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("server.rs"),
            "fn handle_request(req: &Request) -> Response {\n    parse_headers(req)\n    route_request(req)\n}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("database.rs"),
            "fn connect_database(url: &str) -> Connection {\n    let pool = build_pool(url);\n    pool.get()\n}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("utils.rs"),
            "fn parse_headers(req: &Request) -> Headers {\n    let raw = req.raw_headers();\n    Headers::from(raw)\n}\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn test_semantic_search_returns_results() {
        let dir = setup_test_dir();
        let result = run_semantic(
            "database connection pool",
            dir.path().to_str().unwrap(),
            None,
            None,
            true,
            None,
            Some(5),
            false,
            false,
        );
        assert!(result.is_ok());
        // Should find at least one result since "database.rs" has database/pool terms
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_semantic_search_no_results_for_unrelated_query() {
        let dir = setup_test_dir();
        let result = run_semantic(
            "quantum physics teleportation",
            dir.path().to_str().unwrap(),
            None,
            None,
            true,
            None,
            Some(5),
            false,
            false,
        );
        assert!(result.is_ok());
        // No overlap between query terms and corpus
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_semantic_search_json_output() {
        let dir = setup_test_dir();
        let result = run_semantic(
            "database connection pool",
            dir.path().to_str().unwrap(),
            None,
            None,
            true,
            None,
            Some(5),
            true,
            false,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_stop_words_filtering() {
        let stops = stop_words();
        assert!(stops.contains("the"));
        assert!(stops.contains("fn"));
        assert!(stops.contains("function"));
        assert!(stops.contains("self"));
        assert!(stops.contains("return"));
        assert!(stops.contains("and"));
        assert!(stops.contains("or"));
        // Regular words should not be in stop words
        assert!(!stops.contains("database"));
        assert!(!stops.contains("server"));
        assert!(!stops.contains("handler"));
    }

    #[test]
    fn test_tokenization() {
        let stops = stop_words();
        let tokens = tokenize("fn handle_request(req: &Request) -> Response {", &stops);
        // "fn" is a stop word; "handle_request" splits into "handle" + "request";
        // "req" is only 3 chars but > 1; "Request" lowercased to "request"
        assert!(!tokens.contains(&"fn".to_string()));
        assert!(tokens.contains(&"handle".to_string()));
        assert!(tokens.contains(&"request".to_string()));
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let mut v: HashMap<String, f64> = HashMap::new();
        v.insert("hello".to_string(), 0.5);
        v.insert("world".to_string(), 0.5);
        // Cosine similarity of a vector with itself should be ~1.0
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let mut a: HashMap<String, f64> = HashMap::new();
        a.insert("foo".to_string(), 1.0);
        let mut b: HashMap<String, f64> = HashMap::new();
        b.insert("bar".to_string(), 1.0);
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_term_frequency() {
        let tokens = vec!["hello".to_string(), "world".to_string(), "hello".to_string()];
        let tf = term_frequency(&tokens);
        assert!((tf.get("hello").unwrap() - 2.0 / 3.0).abs() < 1e-9);
        assert!((tf.get("world").unwrap() - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_inverse_document_frequency() {
        let sets = vec![
            ["alpha".to_string(), "beta".to_string()].into_iter().collect(),
            ["alpha".to_string(), "gamma".to_string()].into_iter().collect(),
            ["delta".to_string()].into_iter().collect(),
        ];
        let all: HashSet<String> = ["alpha", "beta", "gamma", "delta"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let idf = inverse_document_frequency(&sets, &all);
        // alpha appears in 2 of 3 docs → IDF = ln(3/(1+2)) = ln(1) = 0
        assert!((idf.get("alpha").unwrap()).abs() < 1e-9);
        // beta appears in 1 of 3 docs → IDF = ln(3/(1+1)) = ln(1.5) > 0
        assert!(*idf.get("beta").unwrap() > 0.0);
        // delta appears in 1 of 3 docs → same IDF as beta
        assert_eq!(
            idf.get("beta").unwrap(),
            idf.get("delta").unwrap()
        );
    }
}
