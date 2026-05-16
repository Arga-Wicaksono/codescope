use colored::Colorize;

pub fn run_explain(pattern: &str, json: bool) -> Result<i32, String> {
    if pattern.trim().is_empty() {
        return Err("Pattern cannot be empty".to_string());
    }

    let explanations = explain_pattern(pattern);

    if json {
        let results_json: Vec<serde_json::Value> = explanations
            .iter()
            .map(|(token, desc)| {
                serde_json::to_value(crate::output_schema::ExplainResultItem {
                    token: token.clone(),
                    description: desc.clone(),
                })
                .unwrap()
            })
            .collect();
        let output = crate::output_schema::envelope(
            "explain", pattern, "filesystem", explanations.len(), 0.0,
            serde_json::json!(results_json),
        );
        crate::output_schema::print_json(&output);
        return Ok(0);
    }

    eprintln!("{} Explaining regex: '{}'", ">>".cyan(), pattern.cyan());
    eprintln!("{}", "─".repeat(50).dimmed());

    for (token, desc) in &explanations {
        eprintln!("  {}  {}", token.green(), desc);
    }

    eprintln!("{}", "─".repeat(50).dimmed());
    eprintln!("{} {} token(s) explained", "✓".green(), explanations.len());

    Ok(0)
}

fn explain_pattern(pattern: &str) -> Vec<(String, String)> {
    let mut explanations = Vec::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            let (token, desc): (String, String) = match next {
                'd' => ("\\d".into(), "Any digit [0-9]".into()),
                'D' => ("\\D".into(), "Any non-digit [^0-9]".into()),
                'w' => ("\\w".into(), "Any word character [a-zA-Z0-9_]".into()),
                'W' => ("\\W".into(), "Any non-word character [^a-zA-Z0-9_]".into()),
                's' => ("\\s".into(), "Any whitespace (space, tab, newline)".into()),
                'S' => ("\\S".into(), "Any non-whitespace".into()),
                'n' => ("\\n".into(), "Newline".into()),
                't' => ("\\t".into(), "Tab".into()),
                'r' => ("\\r".into(), "Carriage return".into()),
                'b' => ("\\b".into(), "Word boundary".into()),
                'B' => ("\\B".into(), "Non-word boundary".into()),
                '.' => ("\\.".into(), "Literal dot (.)".into()),
                '+' => ("\\+".into(), "Literal plus (+)".into()),
                '*' => ("\\*".into(), "Literal asterisk (*)".into()),
                '?' => ("\\?".into(), "Literal question mark (?)".into()),
                '[' => ("\\[".into(), "Literal opening bracket ([)".into()),
                ']' => ("\\]".into(), "Literal closing bracket (])".into()),
                '(' => ("\\(".into(), "Literal opening parenthesis".into()),
                ')' => ("\\)".into(), "Literal closing parenthesis".into()),
                '{' => ("\\{".into(), "Literal opening brace".into()),
                '}' => ("\\}".into(), "Literal closing brace".into()),
                '^' => ("\\^".into(), "Literal caret (^)".into()),
                '$' => ("\\$".into(), "Literal dollar sign ($)".into()),
                '|' => ("\\|".into(), "Literal pipe (|)".into()),
                _ => (format!("\\{}", next), format!("Escape sequence: \\{}", next)),
            };
            explanations.push((token, desc));
            i += 2;
        } else {
            let ch = chars[i];
            let (token, desc): (String, String) = match ch {
                '.' => (".".into(), "Any character (except newline)".into()),
                '+' => ("+".into(), "One or more of the previous".into()),
                '*' => ("*".into(), "Zero or more of the previous".into()),
                '?' => ("?".into(), "Zero or one of the previous (optional)".into()),
                '^' => ("^".into(), "Start of string/line".into()),
                '$' => ("$".into(), "End of string/line".into()),
                '|' => ("|".into(), "OR — alternation".into()),
                '(' => ("(".into(), "Start of capture group".into()),
                ')' => (")".into(), "End of capture group".into()),
                '[' => ("[".into(), "Start of character class".into()),
                ']' => ("]".into(), "End of character class".into()),
                '{' => ("{".into(), "Start of quantifier".into()),
                '}' => ("}".into(), "End of quantifier".into()),
                _ => (ch.to_string(), format!("Literal '{}'", ch)),
            };
            explanations.push((token, desc));
            i += 1;
        }
    }

    explanations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_basic() {
        let result = run_explain("\\s+\\w+", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_explain_empty() {
        let result = run_explain("", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_explain_character_class() {
        let result = run_explain("[A-Z][a-z]+", false);
        assert!(result.is_ok());
    }
}
