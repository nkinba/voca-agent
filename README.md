# Voca-Agent

RSS 피드와 기술 블로그를 모니터링하여 TOEFL 수준의 영어 어휘를 추출하는 헤드리스 AI 에이전트입니다.

## 프로젝트 개요

- **RSS/웹 크롤링**: 기술 블로그 및 뉴스 피드 자동 수집
- **LLM 기반 어휘 추출**: Gemini 1.5 Flash를 활용한 문맥 기반 어휘 분석
- **Obsidian 연동**: MCP 서버를 통한 Obsidian Vault 동기화 (예정)

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
├── app/                    # 메인 바이너리 (Orchestrator)
│   └── src/main.rs
├── crates/
│   ├── core/               # 도메인 모델 및 인터페이스 (Ports)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── model.rs    # Article, Vocabulary, SourceType
│   │       ├── port.rs     # FetcherPort, StoragePort, LlmPort
│   │       └── error.rs    # CoreError
│   ├── fetcher/            # RSS 수집 모듈
│   │   └── src/lib.rs      # RssFetcher
│   └── storage/            # SQLite 저장소 모듈
│       └── src/lib.rs      # SqliteStorage
└── docs/                   # 문서
    ├── master-prd.md       # 마스터 PRD
    ├── adrs/               # Architecture Decision Records
    └── specs/              # 모듈별 상세 스펙 (PRD-001 ~ 005)
```

## 개발 현황

| 모듈 | PRD | 상태 |
|------|-----|------|
| voca-core | PRD-001 | ✅ 완료 |
| voca-fetcher | PRD-002 | ✅ 완료 |
| voca-storage | PRD-003 | ✅ 완료 |
| voca-llm | PRD-004 | ⬜ 예정 |
| Pipeline | PRD-005 | ⬜ 예정 |

## 환경 설정

### 필수 요구사항

- Rust 1.70+ (권장: 최신 stable)
- SQLite 3.x

### 설치

```bash
# Rust 설치 (이미 설치된 경우 생략)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 프로젝트 클론
git clone https://github.com/nkinba/voca-agent.git
cd voca-agent
```

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
# 개발 모드 실행
cargo run

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

## 문서

- [Master PRD](docs/master-prd.md) - 전체 프로젝트 요구사항
- [ADR-001](docs/adrs/ADR-001.md) - Rust 선택 배경
- [ADR-002](docs/adrs/ADR-002.md) - SQLite 선택 배경
- [PRD-001 Core](docs/specs/001-core.md) - 도메인 모델 스펙
- [PRD-002 Fetcher](docs/specs/002-fetcher.md) - RSS 수집 모듈 스펙
- [PRD-003 Storage](docs/specs/003-storage.md) - 저장소 모듈 스펙
- [PRD-004 LLM](docs/specs/004-llm.md) - LLM 연동 스펙
- [PRD-005 Pipeline](docs/specs/005-pipeline.md) - 파이프라인 오케스트레이션 스펙

## 라이선스

MIT License
