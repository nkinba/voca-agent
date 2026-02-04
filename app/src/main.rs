use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Hello, voca-agent!");

    // TODO: Pipeline 구현 (PRD-005)
    // 1. SqliteStorage 초기화
    // 2. RssFetcher 초기화
    // 3. LlmEngine 초기화 (PRD-004 구현 후)
    // 4. RSS URL 순회 및 Article fetch
    // 5. 중복 체크 및 저장
    // 6. LLM 기반 Vocabulary 추출
}
