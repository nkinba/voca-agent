use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Hello, voca-agent!");

    voca_core::init();
    voca_fetcher::init();
    voca_storage::init();
}
