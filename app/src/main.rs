mod workflow;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use voca_fetcher::RssFetcher;
use voca_llm::MockLlmEngine;
use voca_storage::SqliteStorage;

/// Default RSS feed URLs for testing
const DEFAULT_FEED_URLS: &[&str] = &[
    "https://blog.rust-lang.org/feed.xml",
];

/// Default SQLite database path
const DEFAULT_DB_URL: &str = "sqlite:voca-agent.db?mode=rwc";

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting voca-agent pipeline");

    // Initialize dependencies
    let fetcher = RssFetcher::new();
    let llm = MockLlmEngine::new();

    let storage = match SqliteStorage::new(DEFAULT_DB_URL).await {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to initialize storage");
            return;
        }
    };

    info!("Initialized all dependencies");

    // Run the pipeline
    match workflow::run_pipeline(DEFAULT_FEED_URLS, &fetcher, &storage, &llm).await {
        Ok(stats) => {
            info!(
                articles_saved = stats.articles_saved,
                vocabularies_saved = stats.vocabularies_saved,
                skipped = stats.skipped_duplicates,
                "Pipeline completed successfully"
            );
        }
        Err(e) => {
            error!(error = %e, "Pipeline failed");
        }
    }
}
