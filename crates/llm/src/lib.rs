use async_trait::async_trait;
use rig::completion::Prompt;
use rig::providers::openai;
use serde::{Deserialize, Serialize};

use voca_core::{CoreError, LlmPort, Vocabulary};

const SYSTEM_PROMPT: &str = r#"You are a strict TOEFL exam creator. Identify 3-5 distinct English words from the text that are CEFR Level C1 or C2.
Ignore common words. For each word, provide:
1. The word itself (lemma form).
2. A concise definition in English suitable for academic context.
3. The specific sentence from the text where it was used (context).

Output must be valid JSON array with the following structure:
[
  {
    "word": "string",
    "definition": "string",
    "context_sentence": "string"
  }
]

Only output the JSON array, no other text."#;

#[derive(Debug, Serialize, Deserialize)]
struct ExtractedWord {
    word: String,
    definition: String,
    context_sentence: String,
}

pub struct RigLlmEngine {
    client: openai::Client,
    model: String,
}

impl RigLlmEngine {
    pub fn new() -> Result<Self, CoreError> {
        dotenvy::dotenv().ok();

        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| CoreError::Llm("OPENAI_API_KEY not found in environment".to_string()))?;

        let client = openai::Client::new(&api_key);

        Ok(Self {
            client,
            model: "gpt-4o-mini".to_string(),
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
            "above", "below", "to", "from", "up", "down", "in", "out", "on", "off", "over", "under",
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

impl Default for RigLlmEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create RigLlmEngine with default settings")
    }
}

#[async_trait]
impl LlmPort for RigLlmEngine {
    async fn extract(&self, text: &str) -> Result<Vec<Vocabulary>, CoreError> {
        let agent = self
            .client
            .agent(&self.model)
            .preamble(SYSTEM_PROMPT)
            .build();

        let user_prompt = format!(
            "Extract vocabulary words from the following text:\n\n{}",
            text
        );

        let response = agent
            .prompt(user_prompt)
            .await
            .map_err(|e| CoreError::Llm(e.to_string()))?;

        let extracted: Vec<ExtractedWord> =
            serde_json::from_str(&response).map_err(|e| CoreError::Parse(e.to_string()))?;

        Ok(self.filter_words(extracted, ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_short_words() {
        let engine = RigLlmEngine {
            client: openai::Client::new("test"),
            model: "test".to_string(),
        };

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
        let engine = RigLlmEngine {
            client: openai::Client::new("test"),
            model: "test".to_string(),
        };

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
}
