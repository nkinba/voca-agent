mod workflow;

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use voca_fetcher::RssFetcher;
use voca_integration::{MarkdownExporter, McpServer};
use voca_llm::MockLlmEngine;
use voca_storage::SqliteStorage;

/// Default RSS feed URLs for testing
const DEFAULT_FEED_URLS: &[&str] = &["https://blog.rust-lang.org/feed.xml"];

/// Default SQLite database path
const DEFAULT_DB_URL: &str = "sqlite:voca-agent.db?mode=rwc";

/// Environment variable names
const ENV_OBSIDIAN_VAULT_PATH: &str = "OBSIDIAN_VAULT_PATH";
const ENV_OBSIDIAN_NOTE_PATH: &str = "OBSIDIAN_NOTE_PATH";
const ENV_OBSIDIAN_INBOX_PATH: &str = "OBSIDIAN_INBOX_PATH";

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
        /// Obsidian vault path for exporting vocabulary (overrides OBSIDIAN_VAULT_PATH env)
        #[arg(long)]
        obsidian_path: Option<PathBuf>,
    },
    /// Start MCP server (stdio mode)
    Mcp,
    /// Export all vocabulary to Obsidian
    Export {
        /// Obsidian vault path (overrides OBSIDIAN_VAULT_PATH env)
        #[arg(long)]
        obsidian_path: Option<PathBuf>,
    },
}

/// Get Obsidian export path from CLI arg or environment variables
fn get_obsidian_path(cli_path: Option<PathBuf>) -> Option<PathBuf> {
    // CLI arg takes precedence
    if let Some(path) = cli_path {
        return Some(path);
    }

    // Try OBSIDIAN_NOTE_PATH first (absolute path, most specific)
    if let Ok(note_path) = std::env::var(ENV_OBSIDIAN_NOTE_PATH) {
        if !note_path.is_empty() {
            info!(path = %note_path, "Using OBSIDIAN_NOTE_PATH from environment");
            return Some(PathBuf::from(note_path));
        }
    }

    // Get vault path (needed for INBOX_PATH resolution)
    let vault_path = std::env::var(ENV_OBSIDIAN_VAULT_PATH)
        .ok()
        .filter(|s| !s.is_empty());

    // Try OBSIDIAN_INBOX_PATH (relative to vault)
    if let Ok(inbox_path) = std::env::var(ENV_OBSIDIAN_INBOX_PATH) {
        if !inbox_path.is_empty() {
            if let Some(ref vault) = vault_path {
                let full_path = PathBuf::from(vault).join(&inbox_path);
                info!(path = %full_path.display(), "Using OBSIDIAN_VAULT_PATH + OBSIDIAN_INBOX_PATH");
                return Some(full_path);
            } else {
                warn!("OBSIDIAN_INBOX_PATH is set but OBSIDIAN_VAULT_PATH is not. Using INBOX_PATH as absolute path.");
                return Some(PathBuf::from(inbox_path));
            }
        }
    }

    // Fall back to OBSIDIAN_VAULT_PATH alone
    if let Some(vault) = vault_path {
        info!(path = %vault, "Using OBSIDIAN_VAULT_PATH from environment");
        return Some(PathBuf::from(vault));
    }

    None
}

#[tokio::main]
async fn main() {
    // Load .env file (ignore errors if file doesn't exist)
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Mcp) => run_mcp_server().await,
        Some(Commands::Export { obsidian_path }) => {
            let path = get_obsidian_path(obsidian_path);
            if let Some(p) = path {
                run_export(p).await;
            } else {
                error!("No Obsidian path provided. Use --obsidian-path or set OBSIDIAN_VAULT_PATH/OBSIDIAN_NOTE_PATH in .env");
            }
        }
        Some(Commands::Run { obsidian_path }) => {
            run_pipeline(get_obsidian_path(obsidian_path)).await
        }
        None => run_pipeline(get_obsidian_path(None)).await,
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
            } else {
                warn!("No Obsidian path configured. Set OBSIDIAN_VAULT_PATH or OBSIDIAN_NOTE_PATH in .env to auto-export");
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

async fn export_to_obsidian(storage: &SqliteStorage, path: &Path) {
    use voca_core::port::StoragePort;

    // Verify path exists
    if !path.exists() {
        error!(path = %path.display(), "Obsidian path does not exist");
        return;
    }

    if !path.is_dir() {
        error!(path = %path.display(), "Obsidian path is not a directory");
        return;
    }

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
