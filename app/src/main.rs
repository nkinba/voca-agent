mod workflow;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use voca_fetcher::RssFetcher;
use voca_integration::{MarkdownExporter, McpServer};
use voca_llm::MockLlmEngine;
use voca_storage::SqliteStorage;

/// Default RSS feed URLs for testing
const DEFAULT_FEED_URLS: &[&str] = &["https://blog.rust-lang.org/feed.xml"];

/// Default SQLite database path
const DEFAULT_DB_URL: &str = "sqlite:voca-agent.db?mode=rwc";

#[derive(Parser)]
#[command(name = "voca-agent")]
#[command(about = "Vocabulary collection agent with Obsidian and MCP integration")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the vocabulary collection pipeline (default)
    Run {
        /// Obsidian vault path for exporting vocabulary
        #[arg(long)]
        obsidian_path: Option<PathBuf>,
    },
    /// Start MCP server (stdio mode)
    Mcp,
    /// Export all vocabulary to Obsidian
    Export {
        /// Obsidian vault path
        #[arg(long, required = true)]
        obsidian_path: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Mcp) => run_mcp_server().await,
        Some(Commands::Export { obsidian_path }) => run_export(obsidian_path).await,
        Some(Commands::Run { obsidian_path }) => run_pipeline(obsidian_path).await,
        None => run_pipeline(None).await,
    }
}

async fn run_pipeline(obsidian_path: Option<PathBuf>) {
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

            // Export to Obsidian if path is provided
            if let Some(path) = obsidian_path {
                info!(path = %path.display(), "Exporting vocabulary to Obsidian");
                export_to_obsidian(&storage, &path).await;
            }
        }
        Err(e) => {
            error!(error = %e, "Pipeline failed");
        }
    }
}

async fn run_mcp_server() {
    info!("Starting MCP server");

    let storage = match SqliteStorage::new(DEFAULT_DB_URL).await {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to initialize storage");
            return;
        }
    };

    let server = McpServer::new(storage);
    if let Err(e) = server.run().await {
        error!(error = %e, "MCP server error");
    }
}

async fn run_export(obsidian_path: PathBuf) {
    info!(path = %obsidian_path.display(), "Exporting vocabulary to Obsidian");

    let storage = match SqliteStorage::new(DEFAULT_DB_URL).await {
        Ok(s) => s,
        Err(e) => {
            error!(error = %e, "Failed to initialize storage");
            return;
        }
    };

    export_to_obsidian(&storage, &obsidian_path).await;
}

async fn export_to_obsidian(storage: &SqliteStorage, path: &PathBuf) {
    use voca_core::port::StoragePort;

    let vocabs = match storage.get_all_vocab().await {
        Ok(v) => v,
        Err(e) => {
            error!(error = %e, "Failed to get vocabulary");
            return;
        }
    };

    if vocabs.is_empty() {
        info!("No vocabulary to export");
        return;
    }

    let exporter = match MarkdownExporter::new(path) {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to create exporter");
            return;
        }
    };

    match exporter.export_batch(&vocabs) {
        Ok(paths) => {
            info!(count = paths.len(), "Exported vocabulary files");
        }
        Err(e) => {
            error!(error = %e, "Failed to export vocabulary");
        }
    }
}
