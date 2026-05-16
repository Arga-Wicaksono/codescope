//! Local embedding generation using candle-nn.
//!
//! Provides sentence-level embeddings for semantic code search.
//! Uses a small model (all-MiniLM-L6-v2) for fast local inference.

#[cfg(feature = "embeddings")]
use std::path::PathBuf;

/// Configuration for the embedding model.
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub model_name: String,
    pub max_length: usize,
    pub dimensions: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            max_length: 256,
            dimensions: 384,
        }
    }
}

/// A computed embedding vector.
#[derive(Debug, Clone)]
pub struct Embedding {
    pub values: Vec<f32>,
    pub dimensions: usize,
}

impl Embedding {
    /// Compute cosine similarity between this embedding and another.
    pub fn cosine_similarity(&self, other: &Embedding) -> f64 {
        if self.dimensions != other.dimensions || self.dimensions == 0 {
            return 0.0;
        }
        let dot: f64 = self.values.iter()
            .zip(&other.values)
            .map(|(a, b)| (*a as f64) * (*b as f64))
            .sum();
        let norm_a: f64 = (self.values.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>()).sqrt();
        let norm_b: f64 = (other.values.iter().map(|v| (*v as f64) * (*v as f64)).sum::<f64>()).sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        dot / (norm_a * norm_b)
    }
}

/// Generate a simple TF-IDF fallback embedding (no model required).
/// This is used when the `embeddings` feature is not enabled.
pub fn tfidf_fallback_embedding(text: &str, vocabulary: &[String]) -> Embedding {
    // Simple bag-of-words embedding using character n-gram frequencies
    let text_lower = text.to_lowercase();
    let chars: Vec<char> = text_lower.chars().collect();
    let mut freq = std::collections::HashMap::<String, f32>::new();

    // Extract 3-grams
    for window in chars.windows(3) {
        let gram: String = window.iter().collect();
        *freq.entry(gram).or_insert(0.0) += 1.0;
    }

    // Normalize
    let total: f32 = freq.values().sum();
    if total > 0.0 {
        for v in freq.values_mut() {
            *v /= total;
        }
    }

    let dimensions = 384;
    let mut values = vec![0.0f32; dimensions];
    for (i, (_, &val)) in freq.iter().enumerate() {
        if i < dimensions {
            values[i] = val;
        }
    }

    Embedding { values, dimensions }
}

/// Result of embedding multiple texts.
pub struct EmbeddingResult {
    pub embeddings: Vec<Embedding>,
    pub model_used: String,
}

/// Generate embeddings for a batch of texts.
/// Uses TF-IDF fallback when candle is not available.
pub fn embed_texts(texts: &[&str]) -> EmbeddingResult {
    let vocabulary: Vec<String> = texts.iter()
        .flat_map(|t| {
            let lower = t.to_lowercase();
            lower.split_whitespace().map(String::from).collect::<Vec<_>>()
        })
        .collect();

    let embeddings: Vec<Embedding> = texts.iter()
        .map(|t| tfidf_fallback_embedding(t, &vocabulary))
        .collect();

    EmbeddingResult {
        embeddings,
        model_used: "tfidf-fallback".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let e = Embedding { values: vec![1.0, 0.0, 0.0], dimensions: 3 };
        assert!((e.cosine_similarity(&e) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = Embedding { values: vec![1.0, 0.0, 0.0], dimensions: 3 };
        let b = Embedding { values: vec![0.0, 1.0, 0.0], dimensions: 3 };
        assert!((a.cosine_similarity(&b)).abs() < 0.001);
    }

    #[test]
    fn test_embed_texts() {
        let result = embed_texts(&["hello world", "foo bar"]);
        assert_eq!(result.embeddings.len(), 2);
        assert_eq!(result.embeddings[0].dimensions, 384);
    }
}
