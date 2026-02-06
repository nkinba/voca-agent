use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use voca_core::{CoreError, LlmPort, Vocabulary};

const SYSTEM_PROMPT: &str = r#"You are a strict TOEFL exam creator. Identify 3-5 distinct English words from the text that are CEFR Level C1 or C2. Ignore common words. Output a JSON list of objects with the following keys:
- 'word': The lemma of the word.
- 'definition': A concise academic definition.
- 'context_sentence': The sentence from the text containing the word."#;

const GEMINI_API_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

#[derive(Debug, Serialize, Deserialize)]
struct ExtractedWord {
    word: String,
    definition: String,
    context_sentence: String,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Debug, Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Debug, Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    message: String,
}

pub struct GeminiLlmEngine {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiLlmEngine {
    pub fn new() -> Result<Self, CoreError> {
        dotenvy::dotenv().ok();

        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| CoreError::Llm("GEMINI_API_KEY not found in environment".to_string()))?;

        Ok(Self {
            api_key,
            model: "gemini-2.5-flash".to_string(),
            client: reqwest::Client::new(),
        })
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    fn filter_words(&self, words: Vec<ExtractedWord>, source_url: &str) -> Vec<Vocabulary> {
        const STOP_WORDS: &[&str] = &[
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
            "can", "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they",
            "what", "which", "who", "whom", "when", "where", "why", "how", "all", "each", "every",
            "both", "few", "more", "most", "other", "some", "such", "no", "nor", "not", "only",
            "own", "same", "so", "than", "too", "very", "just", "but", "and", "or", "if", "for",
            "with", "about", "against", "between", "into", "through", "during", "before", "after",
            "above", "below", "to", "from", "up", "down", "in", "out", "on", "off", "over",
            "under",
        ];

        words
            .into_iter()
            .filter(|w| {
                let word_lower = w.word.to_lowercase();
                w.word.len() > 3 && !STOP_WORDS.contains(&word_lower.as_str())
            })
            .map(|w| Vocabulary {
                word: w.word,
                definition: w.definition,
                context_sentence: w.context_sentence,
                source_url: source_url.to_string(),
            })
            .collect()
    }
}

#[async_trait]
impl LlmPort for GeminiLlmEngine {
    async fn extract(&self, text: &str) -> Result<Vec<Vocabulary>, CoreError> {
        let prompt = format!("{}\n\nTarget Text:\n{}", SYSTEM_PROMPT, text);

        let request_body = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: prompt }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: "application/json".to_string(),
            },
        };

        let url = format!("{}?key={}", GEMINI_API_URL, self.api_key);

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CoreError::Llm(format!(
                "Gemini API error ({}): {}",
                status, error_text
            )));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| CoreError::Parse(format!("Failed to parse Gemini response: {}", e)))?;

        if let Some(error) = gemini_response.error {
            return Err(CoreError::Llm(format!(
                "Gemini API error: {}",
                error.message
            )));
        }

        let text_response = gemini_response
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| CoreError::Parse("No content in Gemini response".to_string()))?;

        let extracted: Vec<ExtractedWord> = serde_json::from_str(&text_response)
            .map_err(|e| CoreError::Parse(format!("Failed to parse vocabulary JSON: {}", e)))?;

        Ok(self.filter_words(extracted, ""))
    }
}

/// Mock LLM engine that returns sample vocabularies for testing.
/// Use this when GEMINI_API_KEY is not available or for testing purposes.
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
                source_url: String::new(),
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

    fn create_test_engine() -> GeminiLlmEngine {
        GeminiLlmEngine {
            api_key: "test_key".to_string(),
            model: "gemini-2.5-flash".to_string(),
            client: reqwest::Client::new(),
        }
    }

    #[test]
    fn test_filter_short_words() {
        let engine = create_test_engine();

        let words = vec![
            ExtractedWord {
                word: "cat".to_string(),
                definition: "A small feline".to_string(),
                context_sentence: "The cat sat.".to_string(),
            },
            ExtractedWord {
                word: "ephemeral".to_string(),
                definition: "Lasting for a very short time".to_string(),
                context_sentence: "The ephemeral beauty of cherry blossoms.".to_string(),
            },
        ];

        let filtered = engine.filter_words(words, "https://example.com");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].word, "ephemeral");
        assert_eq!(filtered[0].source_url, "https://example.com");
    }

    #[test]
    fn test_filter_stop_words() {
        let engine = create_test_engine();

        let words = vec![
            ExtractedWord {
                word: "about".to_string(),
                definition: "On the subject of".to_string(),
                context_sentence: "About the matter.".to_string(),
            },
            ExtractedWord {
                word: "ubiquitous".to_string(),
                definition: "Present everywhere".to_string(),
                context_sentence: "Smartphones are ubiquitous.".to_string(),
            },
        ];

        let filtered = engine.filter_words(words, "");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].word, "ubiquitous");
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"[
            {
                "word": "ephemeral",
                "definition": "Lasting for a very short time",
                "context_sentence": "The ephemeral beauty of the sunset."
            }
        ]"#;

        let parsed: Result<Vec<ExtractedWord>, _> = serde_json::from_str(json);
        assert!(parsed.is_ok());

        let words = parsed.unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].word, "ephemeral");
    }

    #[test]
    fn test_gemini_request_serialization() {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: "Test prompt".to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                response_mime_type: "application/json".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("responseMimeType"));
        assert!(json.contains("application/json"));
    }

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
