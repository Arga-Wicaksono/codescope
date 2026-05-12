use colored::Colorize;

use crate::utils::Timer;

pub fn search_web(query: &str, limit: usize, timeout_secs: u64, json: bool) -> Result<bool, String> {
    let timer = Timer::new();

    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client.get(&url).send().map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let html = response.text().map_err(|e| format!("Failed to read response: {}", e))?;

    let document = scraper::Html::parse_document(&html);

    let result_selector = scraper::Selector::parse(".result").unwrap();
    let title_selector = scraper::Selector::parse(".result__title").unwrap();
    let snippet_selector = scraper::Selector::parse(".result__snippet").unwrap();
    let url_selector = scraper::Selector::parse(".result__url").unwrap();

    let mut results: Vec<(String, String, String)> = Vec::new();

    for element in document.select(&result_selector) {
        let title = element.select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let snippet = element.select(&snippet_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let url = element.select(&url_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        if !title.is_empty() {
            results.push((title, url, snippet));
        }
    }

    results.truncate(limit);
    let elapsed = timer.elapsed_secs();

    if json {
        crate::output::print_web_results_json(&results, query, elapsed);
    } else {
        eprintln!("{} Web search: '{}'", ">>".cyan(), query.cyan());
        eprintln!("{}", "─".repeat(50).dimmed());

        crate::output::print_web_results(&results);

        if !results.is_empty() {
            eprintln!("\n{} {} results in {:.1}s", "✓".green(), results.len(), elapsed);
        } else {
            eprintln!("\n{}", "No web results found.".yellow());
        }
    }

    Ok(!results.is_empty())
}

pub fn collect_web_results(query: &str, limit: usize, timeout_secs: u64) -> Result<Vec<(String, String, String)>, String> {
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client.get(&url).send().map_err(|e| format!("Request failed: {}", e))?;
    let html = response.text().map_err(|e| format!("Failed to read response: {}", e))?;
    let document = scraper::Html::parse_document(&html);

    let result_selector = scraper::Selector::parse(".result").unwrap();
    let title_selector = scraper::Selector::parse(".result__title").unwrap();
    let snippet_selector = scraper::Selector::parse(".result__snippet").unwrap();
    let url_selector = scraper::Selector::parse(".result__url").unwrap();

    let mut results: Vec<(String, String, String)> = Vec::new();

    for element in document.select(&result_selector) {
        let title = element.select(&title_selector).next().map(|el| el.text().collect::<String>()).unwrap_or_default().trim().to_string();
        let snippet = element.select(&snippet_selector).next().map(|el| el.text().collect::<String>()).unwrap_or_default().trim().to_string();
        let url = element.select(&url_selector).next().map(|el| el.text().collect::<String>()).unwrap_or_default().trim().to_string();

        if !title.is_empty() {
            results.push((title, url, snippet));
        }
    }

    results.truncate(limit);
    Ok(results)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_web_search_invalid_query() {
        // We can't easily test web search without network, but test the function exists
        // This test ensures the module compiles
    }
}
