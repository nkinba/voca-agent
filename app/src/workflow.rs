use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use voca_core::{Article, LlmPort, SourceType, StoragePort, Vocabulary};
use voca_fetcher::RssFetcher;

/// Rate limiting delay between LLM API calls (in seconds)
const LLM_RATE_LIMIT_DELAY_SECS: u64 = 2;

/// Execute the vocabulary extraction pipeline for a list of RSS feed URLs.
///
/// Pipeline flow:
/// 1. Fetch Feed: Get all items from RSS feed
/// 2. Deduplication: Check if URL already exists in storage
/// 3. Fetch Body: Get article body content
/// 4. AI Extract: Extract vocabularies using LLM
/// 5. Persist: Save article and vocabularies to storage
pub async fn run_pipeline<S, L>(
    feed_urls: &[&str],
    fetcher: &RssFetcher,
    storage: &S,
    llm: &L,
) -> Result<PipelineStats, PipelineError>
where
    S: StoragePort,
    L: LlmPort,
{
    let mut stats = PipelineStats::default();

    for feed_url in feed_urls {
        info!(feed_url, "Fetching RSS feed");

        let feed_items = match fetcher.fetch_feed(feed_url).await {
            Ok(items) => items,
            Err(e) => {
                error!(feed_url, error = %e, "Failed to fetch feed");
                stats.feed_errors += 1;
                continue;
            }
        };

        info!(feed_url, item_count = feed_items.len(), "Fetched feed items");

        for item in feed_items {
            stats.total_items += 1;

            // Step 2: Deduplication check
            match storage.exists(&item.url).await {
                Ok(true) => {
                    info!(url = %item.url, "Article already exists, skipping");
                    stats.skipped_duplicates += 1;
                    continue;
                }
                Ok(false) => {
                    // New article, proceed
                }
                Err(e) => {
                    error!(url = %item.url, error = %e, "Failed to check if article exists");
                    stats.storage_errors += 1;
                    continue;
                }
            }

            // Step 3: Fetch body content
            let body = match fetcher.fetch_body(&item.url).await {
                Ok(content) => {
                    if content.is_empty() {
                        warn!(url = %item.url, "Fetched empty body content");
                    }
                    content
                }
                Err(e) => {
                    error!(url = %item.url, error = %e, "Failed to fetch body");
                    stats.fetch_errors += 1;
                    continue;
                }
            };

            // Step 4: AI Extract vocabularies
            let vocabularies = match llm.extract(&body).await {
                Ok(vocabs) => {
                    info!(url = %item.url, vocab_count = vocabs.len(), "Extracted vocabularies");
                    vocabs
                }
                Err(e) => {
                    // LLM failure: save article without vocabularies
                    warn!(url = %item.url, error = %e, "LLM extraction failed, saving article without vocabularies");
                    stats.llm_errors += 1;
                    Vec::new()
                }
            };

            // Step 5: Persist article
            let article = Article {
                url: item.url.clone(),
                title: item.title.clone(),
                content: body,
                source: SourceType::RSS,
                published_at: item.published_at,
                collected_at: Utc::now(),
            };

            if let Err(e) = storage.save_article(&article).await {
                error!(url = %item.url, error = %e, "Failed to save article");
                stats.storage_errors += 1;
                continue;
            }

            info!(url = %item.url, title = %item.title, "Saved article");
            stats.articles_saved += 1;

            // Step 5: Persist vocabularies
            for vocab in vocabularies {
                let vocab_with_source = Vocabulary {
                    source_url: item.url.clone(),
                    ..vocab
                };

                if let Err(e) = storage.save_vocab(&vocab_with_source).await {
                    error!(word = %vocab_with_source.word, error = %e, "Failed to save vocabulary");
                    stats.storage_errors += 1;
                } else {
                    stats.vocabularies_saved += 1;
                }
            }

            // Rate limiting: sleep between LLM calls
            sleep(Duration::from_secs(LLM_RATE_LIMIT_DELAY_SECS)).await;
        }
    }

    info!(
        articles_saved = stats.articles_saved,
        vocabularies_saved = stats.vocabularies_saved,
        skipped_duplicates = stats.skipped_duplicates,
        "Pipeline completed"
    );

    Ok(stats)
}

/// Statistics collected during pipeline execution
#[derive(Debug, Default)]
pub struct PipelineStats {
    pub total_items: usize,
    pub articles_saved: usize,
    pub vocabularies_saved: usize,
    pub skipped_duplicates: usize,
    pub feed_errors: usize,
    pub fetch_errors: usize,
    pub llm_errors: usize,
    pub storage_errors: usize,
}

/// Pipeline error type
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum PipelineError {
    #[error("Initialization error: {0}")]
    Init(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use voca_core::CoreError;

    struct MockStorage {
        existing_urls: Vec<String>,
    }

    #[async_trait]
    impl StoragePort for MockStorage {
        async fn exists(&self, url: &str) -> Result<bool, CoreError> {
            Ok(self.existing_urls.contains(&url.to_string()))
        }

        async fn save_article(&self, _article: &Article) -> Result<(), CoreError> {
            Ok(())
        }

        async fn save_vocab(&self, _vocab: &Vocabulary) -> Result<(), CoreError> {
            Ok(())
        }
    }

    struct MockLlm;

    #[async_trait]
    impl LlmPort for MockLlm {
        async fn extract(&self, _text: &str) -> Result<Vec<Vocabulary>, CoreError> {
            Ok(vec![Vocabulary {
                word: "test".to_string(),
                definition: "a test word".to_string(),
                context_sentence: "This is a test.".to_string(),
                source_url: String::new(),
            }])
        }
    }

    #[test]
    fn test_pipeline_stats_default() {
        let stats = PipelineStats::default();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.articles_saved, 0);
    }
}
