use serde::Deserialize;
use serde_json::{json, Value};
use voca_core::model::Vocabulary;
use voca_core::port::StoragePort;

use crate::error::IntegrationError;

use super::protocol::{TextContent, ToolCallResult};

#[derive(Debug, Deserialize)]
pub struct SearchVocaArgs {
    pub query: String,
}

pub async fn search_voca<S: StoragePort>(
    storage: &S,
    args: SearchVocaArgs,
) -> Result<ToolCallResult, IntegrationError> {
    let vocabs = storage.search_vocab(&args.query).await?;

    let text = if vocabs.is_empty() {
        format!("No vocabulary found matching '{}'", args.query)
    } else {
        vocabs
            .iter()
            .map(format_vocabulary)
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    };

    Ok(ToolCallResult {
        content: vec![TextContent {
            content_type: "text".to_string(),
            text,
        }],
        is_error: None,
    })
}

pub async fn get_random_quiz<S: StoragePort>(
    storage: &S,
) -> Result<ToolCallResult, IntegrationError> {
    let vocab = storage.get_random_vocab().await?;

    let text = match vocab {
        Some(v) => {
            let quiz = json!({
                "type": "quiz",
                "word": v.word,
                "question": format!("What is the meaning of '{}'?", v.word),
                "answer": v.definition,
                "context": v.context_sentence,
                "source": v.source_url
            });
            serde_json::to_string_pretty(&quiz)?
        }
        None => "No vocabulary available for quiz. Please collect some words first.".to_string(),
    };

    Ok(ToolCallResult {
        content: vec![TextContent {
            content_type: "text".to_string(),
            text,
        }],
        is_error: None,
    })
}

pub async fn get_daily_words<S: StoragePort>(storage: &S) -> Result<Value, IntegrationError> {
    let vocabs = storage.get_today_vocab().await?;

    let text = if vocabs.is_empty() {
        "No vocabulary collected today.".to_string()
    } else {
        let header = format!("# Today's Vocabulary ({} words)\n\n", vocabs.len());
        let body = vocabs
            .iter()
            .map(format_vocabulary)
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");
        format!("{}{}", header, body)
    };

    Ok(json!({
        "contents": [{
            "uri": "voca://daily-words",
            "mimeType": "text/markdown",
            "text": text
        }]
    }))
}

fn format_vocabulary(vocab: &Vocabulary) -> String {
    format!(
        "**{}**\n\n*Definition:* {}\n\n> {}\n\nSource: {}",
        vocab.word, vocab.definition, vocab.context_sentence, vocab.source_url
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use voca_core::error::CoreError;
    use voca_core::model::Article;

    struct MockStorage {
        vocabs: Vec<Vocabulary>,
    }

    #[async_trait]
    impl StoragePort for MockStorage {
        async fn exists(&self, _url: &str) -> Result<bool, CoreError> {
            Ok(false)
        }
        async fn save_article(&self, _article: &Article) -> Result<(), CoreError> {
            Ok(())
        }
        async fn save_vocab(&self, _vocab: &Vocabulary) -> Result<(), CoreError> {
            Ok(())
        }
        async fn get_all_vocab(&self) -> Result<Vec<Vocabulary>, CoreError> {
            Ok(self.vocabs.clone())
        }
        async fn search_vocab(&self, query: &str) -> Result<Vec<Vocabulary>, CoreError> {
            Ok(self
                .vocabs
                .iter()
                .filter(|v| v.word.contains(query) || v.definition.contains(query))
                .cloned()
                .collect())
        }
        async fn get_today_vocab(&self) -> Result<Vec<Vocabulary>, CoreError> {
            Ok(self.vocabs.clone())
        }
        async fn get_random_vocab(&self) -> Result<Option<Vocabulary>, CoreError> {
            Ok(self.vocabs.first().cloned())
        }
    }

    #[tokio::test]
    async fn test_search_voca() {
        let storage = MockStorage {
            vocabs: vec![Vocabulary {
                word: "serendipity".to_string(),
                definition: "Finding good things by chance".to_string(),
                context_sentence: "It was serendipity.".to_string(),
                source_url: "https://example.com".to_string(),
            }],
        };

        let result = search_voca(
            &storage,
            SearchVocaArgs {
                query: "serendip".to_string(),
            },
        )
        .await
        .unwrap();

        assert!(result.content[0].text.contains("serendipity"));
    }

    #[tokio::test]
    async fn test_search_voca_no_results() {
        let storage = MockStorage { vocabs: vec![] };

        let result = search_voca(
            &storage,
            SearchVocaArgs {
                query: "xyz".to_string(),
            },
        )
        .await
        .unwrap();

        assert!(result.content[0].text.contains("No vocabulary found"));
    }

    #[tokio::test]
    async fn test_get_random_quiz() {
        let storage = MockStorage {
            vocabs: vec![Vocabulary {
                word: "ephemeral".to_string(),
                definition: "Lasting for a short time".to_string(),
                context_sentence: "Fame is ephemeral.".to_string(),
                source_url: "https://example.com".to_string(),
            }],
        };

        let result = get_random_quiz(&storage).await.unwrap();
        assert!(result.content[0].text.contains("ephemeral"));
        assert!(result.content[0].text.contains("quiz"));
    }

    #[tokio::test]
    async fn test_get_random_quiz_empty() {
        let storage = MockStorage { vocabs: vec![] };

        let result = get_random_quiz(&storage).await.unwrap();
        assert!(result.content[0].text.contains("No vocabulary available"));
    }
}
