# Spread (Voca-Agent)

RSS 피드와 기술 블로그를 모니터링하여 TOEFL 수준의 영어 어휘를 추출하는 헤드리스 AI 에이전트입니다.

## 프로젝트 개요

- **RSS/웹 크롤링**: 기술 블로그 및 뉴스 피드 자동 수집
- **LLM 기반 어휘 추출**: Gemini 2.5 Flash를 활용한 문맥 기반 어휘 분석
- **Obsidian 연동**: MCP 서버를 통한 Obsidian Vault 동기화
- **Telegram 알림**: 일일 어휘 알림 발송
- **Homebrew 배포**: macOS용 Homebrew Tap 지원

## 기술 스택

- **언어**: Rust (Edition 2021)
- **비동기 런타임**: Tokio
- **데이터베이스**: SQLite (sqlx)
- **HTTP 클라이언트**: reqwest
- **RSS 파싱**: rss crate

## 프로젝트 구조

```
voca-agent/
├── Cargo.toml              # Workspace 설정
├── Formula/                # Homebrew Formula
│   └── spread.rb
├── app/                    # 메인 바이너리 (Orchestrator)
│   └── src/
│       ├── main.rs         # CLI 엔트리포인트
│       └── workflow.rs     # 파이프라인 워크플로우
├── crates/
│   ├── core/               # 도메인 모델 및 인터페이스 (Ports)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── model.rs    # Article, Vocabulary, SourceType
│   │       ├── port.rs     # FetcherPort, StoragePort, LlmPort
│   │       └── error.rs    # CoreError
│   ├── fetcher/            # RSS 수집 모듈
│   │   └── src/lib.rs      # RssFetcher
│   ├── storage/            # SQLite 저장소 모듈
│   │   └── src/lib.rs      # SqliteStorage
│   ├── llm/                # LLM 연동 모듈
│   │   └── src/lib.rs      # GeminiLlmEngine, MockLlmEngine
│   ├── integration/        # 외부 연동 모듈
│   │   └── src/
│   │       ├── mcp/        # MCP 서버
│   │       └── obsidian/   # Obsidian 내보내기
│   └── notify/             # 알림 모듈
│       └── src/lib.rs      # TelegramClient, Notifier
├── .github/
│   └── workflows/
│       ├── ci.yml          # CI 파이프라인
│       └── release.yml     # 릴리스 자동화
└── docs/                   # 문서
    ├── master-prd.md       # 마스터 PRD
    ├── adrs/               # Architecture Decision Records
    └── specs/              # 모듈별 상세 스펙 (PRD-001 ~ 009)
```

## 개발 현황

| 모듈 | PRD | 상태 |
|------|-----|------|
| voca-core | PRD-001 | ✅ 완료 |
| voca-fetcher | PRD-002 | ✅ 완료 |
| voca-storage | PRD-003 | ✅ 완료 |
| voca-llm | PRD-004 | ✅ 완료 |
| Pipeline | PRD-005 | ✅ 완료 |
| voca-integration | PRD-006 | ✅ 완료 |
| CI/CD | PRD-007 | ✅ 완료 |
| Homebrew 배포 | PRD-008 | ✅ 완료 |
| Telegram 알림 | PRD-009 | ✅ 완료 |

## 환경 설정

### 필수 요구사항

- Rust 1.70+ (권장: 최신 stable)
- SQLite 3.x

### 설치

```bash
# Homebrew를 통한 설치 (macOS)
brew tap nkinba/tap
brew install spread

# 또는 소스에서 빌드
git clone https://github.com/nkinba/voca-agent.git
cd voca-agent
cargo build --release
```

### 환경 변수 설정

`.env` 파일을 프로젝트 루트에 생성하거나 환경 변수를 직접 설정합니다.

```bash
# .env 파일 예시

# [필수] Gemini API 키 (LLM 어휘 추출에 사용)
GEMINI_API_KEY=your_gemini_api_key

# [선택] Obsidian 연동
OBSIDIAN_VAULT_PATH=/path/to/your/vault
OBSIDIAN_NOTE_PATH=/path/to/your/vault/vocabulary.md
OBSIDIAN_INBOX_PATH=Inbox  # VAULT_PATH 기준 상대 경로

# [선택] Telegram 알림
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
```

### 외부 API 키 발급

#### Gemini API Key

1. [Google AI Studio](https://aistudio.google.com/apikey)에서 API 키 발급
2. `GEMINI_API_KEY` 환경 변수에 설정

#### Telegram Bot Token

1. Telegram에서 [@BotFather](https://t.me/BotFather)와 대화
2. `/newbot` 명령으로 봇 생성
3. 발급된 토큰을 `TELEGRAM_BOT_TOKEN`에 설정
4. 봇과 대화 시작 후 [getUpdates API](https://api.telegram.org/bot<TOKEN>/getUpdates)로 `chat_id` 확인
5. `TELEGRAM_CHAT_ID`에 설정

## 빌드 및 실행

### 빌드

```bash
# 개발 빌드
cargo build

# 릴리스 빌드 (최적화 + LTO)
cargo build --release
```

### 실행

```bash
# 어휘 수집 파이프라인 실행 (기본)
cargo run
# 또는
spread run

# MCP 서버 모드 (Obsidian 연동)
spread mcp

# Obsidian으로 어휘 내보내기
spread export --obsidian-path /path/to/vault

# Telegram 알림 발송
spread notify           # 오늘의 어휘
spread notify --all     # 전체 어휘
spread notify --test    # 테스트 모드

# 로그 레벨 설정 (RUST_LOG 환경변수)
RUST_LOG=info cargo run
RUST_LOG=debug cargo run
RUST_LOG=voca_fetcher=debug cargo run
```

## 테스트

### 전체 테스트 실행

```bash
cargo test
```

### 개별 크레이트 테스트

```bash
# core 모듈 테스트
cargo test -p voca-core

# fetcher 모듈 테스트
cargo test -p voca-fetcher

# storage 모듈 테스트
cargo test -p voca-storage

# llm 모듈 테스트
cargo test -p voca-llm

# notify 모듈 테스트
cargo test -p voca-notify

# integration 모듈 테스트
cargo test -p voca-integration
```

### 테스트 출력 표시

```bash
# 표준 출력 포함
cargo test -- --nocapture

# 특정 테스트만 실행
cargo test test_parse_rss_channel
cargo test test_article_crud
```

### 현재 테스트 커버리지

**voca-fetcher** (4개 테스트)
- `test_parse_rss_channel`: RSS 채널 파싱 검증
- `test_parse_empty_rss`: 빈 피드 처리
- `test_parse_invalid_xml`: 잘못된 XML 에러 처리
- `test_rfc2822_date_parsing`: 날짜 파싱 검증

**voca-storage** (3개 테스트)
- `test_article_crud`: Article 저장 및 조회
- `test_vocabulary_crud`: Vocabulary 저장
- `test_duplicate_article_ignored`: 중복 Article 무시 (INSERT OR IGNORE)

### 수동 통합 테스트 (파일 DB)

단위 테스트는 인메모리 DB(`sqlite::memory:`)를 사용합니다. 실제 파일 기반 DB로 테스트하려면:

```bash
# SQLite CLI로 직접 확인
sqlite3 /tmp/voca_test.db

# 테이블 스키마 확인
.schema

# 데이터 조회
SELECT * FROM articles;
SELECT * FROM vocabularies;

# 종료
.quit
```

**SqliteStorage 사용 예시** (Rust 코드):

```rust
use voca_storage::SqliteStorage;
use voca_core::{Article, Vocabulary, SourceType, StoragePort};
use chrono::Utc;

#[tokio::main]
async fn main() {
    // 파일 기반 DB 생성 (mode=rwc: read/write/create)
    let storage = SqliteStorage::new("sqlite:/tmp/voca_test.db?mode=rwc")
        .await
        .expect("Failed to create storage");

    let article = Article {
        url: "https://example.com/test".to_string(),
        title: "Test".to_string(),
        content: "Content".to_string(),
        source: SourceType::RSS,
        published_at: Utc::now(),
        collected_at: Utc::now(),
    };

    // 저장 및 확인
    storage.save_article(&article).await.unwrap();
    assert!(storage.exists(&article.url).await.unwrap());
}
```

## 코드 품질

### 포맷팅

```bash
cargo fmt
```

### 린팅

```bash
cargo clippy
```

### 타입 검사

```bash
cargo check
```

## 핵심 도메인 모델

### Article

```rust
pub struct Article {
    pub url: String,              // 원본 URL (Primary Key)
    pub title: String,            // 제목
    pub content: String,          // 본문 내용
    pub source: SourceType,       // RSS | Manual | Youtube
    pub published_at: DateTime<Utc>,  // 발행일
    pub collected_at: DateTime<Utc>,  // 수집일
}
```

### Vocabulary

```rust
pub struct Vocabulary {
    pub word: String,             // 단어
    pub definition: String,       // 정의
    pub context_sentence: String, // 문맥 문장
    pub source_url: String,       // 출처 Article URL (FK)
}
```

## 포트 (인터페이스)

### FetcherPort

```rust
#[async_trait]
pub trait FetcherPort: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<Article, CoreError>;
}
```

### StoragePort

```rust
#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn exists(&self, url: &str) -> Result<bool, CoreError>;
    async fn save_article(&self, article: &Article) -> Result<(), CoreError>;
    async fn save_vocab(&self, vocab: &Vocabulary) -> Result<(), CoreError>;
}
```

### LlmPort

```rust
#[async_trait]
pub trait LlmPort: Send + Sync {
    async fn extract(&self, text: &str) -> Result<Vec<Vocabulary>, CoreError>;
}
```

## 데이터베이스 스키마

```sql
-- Article 테이블
CREATE TABLE articles (
    url TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    source TEXT NOT NULL,
    published_at DATETIME NOT NULL,
    collected_at DATETIME NOT NULL
);

-- Vocabulary 테이블
CREATE TABLE vocabularies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    word TEXT NOT NULL,
    definition TEXT NOT NULL,
    context_sentence TEXT NOT NULL,
    source_url TEXT NOT NULL,
    FOREIGN KEY (source_url) REFERENCES articles(url)
);
```

## GitHub Actions 설정

CI/CD 파이프라인 실행을 위해 다음 시크릿을 GitHub 저장소 설정에 추가해야 합니다.

### Repository Secrets 설정

**Settings > Secrets and variables > Actions > Repository secrets**

| Secret 이름 | 용도 | 필수 |
|------------|------|------|
| `GEMINI_API_KEY` | CI 테스트 실행 시 LLM 통합 테스트 | 선택 |
| `HOMEBREW_TAP_TOKEN` | 릴리스 시 Homebrew Tap 자동 업데이트 | 선택 |

### CI 워크플로우 (`.github/workflows/ci.yml`)

- main 브랜치 push/PR 시 자동 실행
- 포맷 검사 (`cargo fmt`)
- 린트 검사 (`cargo clippy`)
- 테스트 실행 (`cargo test`)

### Release 워크플로우 (`.github/workflows/release.yml`)

- `v*` 태그 push 시 자동 실행
- 릴리스 바이너리 빌드
- GitHub Release 생성
- Homebrew Tap 자동 업데이트 (HOMEBREW_TAP_TOKEN 필요)

## 문서

- [Master PRD](docs/master-prd.md) - 전체 프로젝트 요구사항
- [ADR-001](docs/adrs/ADR-001.md) - Rust 선택 배경
- [ADR-002](docs/adrs/ADR-002.md) - SQLite 선택 배경
- [PRD-001 Core](docs/specs/001-core.md) - 도메인 모델 스펙
- [PRD-002 Fetcher](docs/specs/002-fetcher.md) - RSS 수집 모듈 스펙
- [PRD-003 Storage](docs/specs/003-storage.md) - 저장소 모듈 스펙
- [PRD-004 LLM](docs/specs/004-llm.md) - LLM 연동 스펙
- [PRD-005 Pipeline](docs/specs/005-pipeline.md) - 파이프라인 오케스트레이션 스펙
- [PRD-006 Integration](docs/specs/006-integration.md) - 외부 연동 스펙
- [PRD-007 CI/CD](docs/specs/007-cicd.md) - CI/CD 스펙
- [PRD-008 Homebrew](docs/specs/008-homebrew_dist.md) - Homebrew 배포 스펙
- [PRD-009 Telegram](docs/specs/009-telegram_notify.md) - Telegram 알림 스펙

## 라이선스

MIT License
