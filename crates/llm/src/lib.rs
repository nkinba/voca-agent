use async_trait::async_trait;
use voca_core::{CoreError, LlmPort, Vocabulary};

/// Mock LLM engine that returns sample vocabularies for testing.
/// Will be replaced with RigLlmEngine when voca-llm is fully implemented.
pub struct MockLlmEngine;

impl MockLlmEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockLlmEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmPort for MockLlmEngine {
    async fn extract(&self, text: &str) -> Result<Vec<Vocabulary>, CoreError> {
        // Mock implementation: extract some sample words from the text
        let words = extract_sample_words(text);

        let vocabularies: Vec<Vocabulary> = words
            .into_iter()
            .map(|word| Vocabulary {
                word: word.clone(),
                definition: format!("Mock definition for '{}'", word),
                context_sentence: format!("This is a sample context sentence containing {}.", word),
                source_url: String::new(), // Will be filled by the caller
            })
            .collect();

        Ok(vocabularies)
    }
}

/// Extract sample "difficult" words from text (mock implementation)
fn extract_sample_words(text: &str) -> Vec<String> {
    // Simple heuristic: words longer than 8 characters, up to 5 words
    text.split_whitespace()
        .filter(|word| word.len() > 8)
        .filter(|word| word.chars().all(|c| c.is_alphabetic()))
        .take(5)
        .map(|s| s.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_llm_extract() {
        let llm = MockLlmEngine::new();
        let text = "The unprecedented technological advancement revolutionized communication.";

        let result = llm.extract(text).await;
        assert!(result.is_ok());

        let vocabs = result.unwrap();
        assert!(!vocabs.is_empty());

        for vocab in &vocabs {
            assert!(!vocab.word.is_empty());
            assert!(!vocab.definition.is_empty());
        }
    }

    #[tokio::test]
    async fn test_mock_llm_empty_text() {
        let llm = MockLlmEngine::new();
        let text = "";

        let result = llm.extract(text).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
