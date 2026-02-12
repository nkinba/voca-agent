# 📑 Master PRD: Spread

> AI-powered vocabulary extraction agent for TOEFL-level English learning

## 1. Project Overview

**Spread**는 사용자가 직접 단어를 입력하는 수고를 덜어주는 **Headless AI 에이전트**입니다. 사용자가 관심 있는 영문 기술 블로그나 뉴스(RSS/Atom)를 백그라운드에서 모니터링하고, **TOEFL Actual Test 수준의 어휘**를 AI로 추출하여, **Obsidian Vault** 및 **Telegram**으로 자동 동기화합니다.

### 1.1. Core Value Proposition
- **Automated Capture:** RSS/Atom 피드를 통한 무중단 정보 수집
- **Intelligent Processing:** Gemini 2.5 Flash LLM을 활용한 문맥 기반 어휘 추출
- **Seamless Integration:** Obsidian MCP 연동 및 Telegram 일일 알림
- **Performance & Stability:** Rust 기반의 메모리 안전하고 효율적인 시스템

---

## 2. System Architecture

**Hexagonal Architecture (Ports & Adapters)** 기반의 모듈러 모놀리스 구조.

### 2.1. Tech Stack

| 영역 | 기술 | 비고 |
|------|------|------|
| Language | Rust (Edition 2021) | ADR-001 |
| Runtime | Tokio (Async/Await) | ADR-004 |
| Database | SQLite + sqlx | ADR-002 |
| Feed Parsing | feed-rs | ADR-010 (RSS/Atom/JSON Feed) |
| HTML Parsing | scraper | - |
| LLM | Gemini 2.5 Flash | ADR-005 |
| Notification | Telegram Bot API | ADR-008 |
| Integration | MCP Server (stdio) | ADR-009 |
| CLI | clap | - |
| CI/CD | GitHub Actions | ADR-006 |
| Distribution | Homebrew Tap | ADR-007 |

### 2.2. Directory Structure

```text
spread/
├── Cargo.toml              # Workspace Definition
├── Formula/                # Homebrew Formula
│   └── spread.rb
├── app/                    # [Binary] CLI & Orchestrator
│   └── src/
│       ├── main.rs         # CLI entrypoint (clap)
│       └── workflow.rs     # Pipeline orchestration
└── crates/
    ├── core/               # Domain Models & Ports
    │   └── src/
    │       ├── model.rs    # Article, Vocabulary, SourceType
    │       ├── port.rs     # FetcherPort, StoragePort, LlmPort
    │       └── error.rs    # CoreError
    ├── fetcher/            # RSS/Atom Feed Collector
    ├── storage/            # SQLite Persistence
    ├── llm/                # Gemini LLM Integration
    ├── notify/             # Telegram Notification
    └── integration/        # Obsidian Export & MCP Server
        └── src/
            ├── mcp/        # MCP Server implementation
            └── obsidian/   # Markdown Exporter
```

### 2.3. Data Flow

```
RSS/Atom Feed URLs
       ↓
[Fetcher] → FeedItem (title, url, published_at)
       ↓
[Storage] exists() → 중복 확인
       ↓ (새 URL만)
[Fetcher] fetch_body() → HTML → Text
       ↓
[LLM] extract() → Vec<Vocabulary>
       ↓
[Storage] save_article() + save_vocab()
       ↓
SQLite DB ←→ [MCP Server] ←→ Obsidian
       ↓
[Notify] → Telegram (Daily)
```

---

## 3. Module Specifications

### 3.1. `spread-core` (Domain Layer)

시스템의 핵심 도메인 모델과 포트(인터페이스) 정의.

**Models:**
- `Article`: 수집된 원문 (url, title, content, source, published_at, collected_at)
- `Vocabulary`: 추출된 어휘 (word, definition, context_sentence, source_url)
- `SourceType`: RSS | Manual | Youtube

**Ports (Traits):**
```rust
#[async_trait]
pub trait FetcherPort: Send + Sync {
    async fn fetch(&self, url: &str) -> Result<Article, CoreError>;
}

#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn exists(&self, url: &str) -> Result<bool, CoreError>;
    async fn save_article(&self, article: &Article) -> Result<(), CoreError>;
    async fn save_vocab(&self, vocab: &Vocabulary) -> Result<(), CoreError>;
    async fn get_all_vocab(&self) -> Result<Vec<Vocabulary>, CoreError>;
    async fn search_vocab(&self, query: &str) -> Result<Vec<Vocabulary>, CoreError>;
    async fn get_today_vocab(&self) -> Result<Vec<Vocabulary>, CoreError>;
    async fn get_random_vocab(&self) -> Result<Option<Vocabulary>, CoreError>;
}

#[async_trait]
pub trait LlmPort: Send + Sync {
    async fn extract(&self, text: &str) -> Result<Vec<Vocabulary>, CoreError>;
}
```

### 3.2. `spread-fetcher` (Feed Collector)

RSS/Atom/JSON Feed를 파싱하여 기사 목록 수집.

- **Library:** `feed-rs` (자동 포맷 감지)
- **HTML Parsing:** `scraper` (main content 추출)
- **Content Selectors:** `article`, `main`, `[role="main"]`, `.content`, `.post-content`

### 3.3. `spread-storage` (Persistence)

SQLite 기반 데이터 영속성.

**Schema:**
```sql
CREATE TABLE articles (
    url TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    source TEXT NOT NULL,
    published_at DATETIME NOT NULL,
    collected_at DATETIME NOT NULL
);

CREATE TABLE vocabularies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    word TEXT NOT NULL,
    definition TEXT NOT NULL,
    context_sentence TEXT NOT NULL,
    source_url TEXT NOT NULL,
    FOREIGN KEY (source_url) REFERENCES articles(url)
);
```

### 3.4. `spread-llm` (AI Extraction)

Gemini 2.5 Flash를 활용한 TOEFL 수준 어휘 추출.

- **Model:** `gemini-2.5-flash`
- **Prompt:** TOEFL 시험 작문자 역할, CEFR C1/C2 레벨 단어만 추출
- **Filtering:** 3자 이하 단어 제외, 104개 불용어 필터링
- **Output:** JSON 배열 (`word`, `definition`, `context_sentence`)

### 3.5. `spread-notify` (Telegram)

일일 학습 어휘 Telegram 알림.

- **API:** Telegram Bot API (sendMessage)
- **Format:** MarkdownV2
- **Daily Words:** 기본 3개 (설정 가능)

### 3.6. `spread-integration` (Obsidian & MCP)

**Obsidian Exporter:**
- Markdown 파일 생성 (tera 템플릿)
- Vault 직접 파일 쓰기

**MCP Server:**
- stdio 모드로 Obsidian과 양방향 통신
- Tools/Resources 노출

---

## 4. CLI Interface

```bash
# 기본 파이프라인 실행
spread run [--obsidian-path PATH]

# MCP 서버 시작
spread mcp

# Obsidian 내보내기
spread export --obsidian-path PATH

# Telegram 알림
spread notify [--all] [--test]
```

**환경 변수:**
```bash
GEMINI_API_KEY=...              # LLM API (필수)
TELEGRAM_BOT_TOKEN=...          # Telegram Bot
TELEGRAM_CHAT_ID=...            # 수신 Chat ID
OBSIDIAN_VAULT_PATH=...         # Obsidian Vault 경로
RUST_LOG=info                   # 로깅 레벨
```

---

## 5. Development & Deployment

### 5.1. Development Environment
- **Platform:** macOS / Linux
- **Task Management:** Vibe Kanban
- **Code Assistant:** Claude Code

### 5.2. Git Strategy
- **Branching:** Feature Branch + PR
- **Parallel Work:** `git worktree` 활용
- **Merge Policy:** No Local Merge, GitHub PR 필수

### 5.3. CI/CD (GitHub Actions)
- **ci.yml:** PR 시 cargo fmt, clippy, test 실행
- **release.yml:** Tag push 시 Release 자동 생성

### 5.4. Distribution
```bash
# Homebrew 설치
brew tap nkinba/tap
brew install spread

# 소스 빌드
cargo build --release
```

---

## 6. Roadmap & Milestones

### Phase 1: Foundation ✅
- [x] Rust Workspace 구조 세팅
- [x] `voca-core` 도메인 모델 정의
- [x] `voca-fetcher` RSS 파싱 (rss → feed-rs 마이그레이션)
- [x] `voca-storage` SQLite 구현

### Phase 2: Intelligence ✅
- [x] `voca-llm` Gemini 2.5 Flash 연동
- [x] TOEFL 수준 단어 필터링 로직
- [x] 파이프라인 통합 (Fetcher → LLM → Storage)

### Phase 3: Integration ✅
- [x] Obsidian Markdown Exporter
- [x] MCP Server 기본 구현
- [x] Telegram 일일 알림

### Phase 4: Distribution ✅
- [x] GitHub Actions CI/CD
- [x] Homebrew Tap 배포

### Phase 5: Expansion (Next)
- [ ] MCP Server 고도화 (Tools/Resources 확장)
- [ ] TUI 인터페이스 (Ratatui)
- [ ] 리스닝 영역 확장 (오디오 입력, STT)

---

## 7. Future Directions

### 7.1. MCP Server 고도화 (우선순위 1)
기존 MCP 구현을 확장하여 AI 에이전트 연동 강화.

| Tool/Resource | 설명 |
|---------------|------|
| `get_vocabulary` | 학습 단어 조회 |
| `search_vocabulary` | 키워드 검색 |
| `get_learning_stats` | 학습 진도/통계 |
| `add_manual_word` | 수동 단어 추가 |
| `get_today_words` | 오늘의 단어 |

### 7.2. TUI 인터페이스 (우선순위 2)
Ratatui 기반 터미널 UI.

- 단어장 브라우징/검색
- 플래시카드 학습 모드
- 학습 진도 대시보드

### 7.3. 리스닝 확장 (우선순위 3)
오디오 입력 소스 지원.

- `AudioPort` 추상화
- STT 연동 (Whisper API)
- `SourceType::Audio` 추가
- 시스템 오디오 캡처 (cpal/audiotee)

---

## 8. References

### ADR Documents
| ADR | 주제 | 파일 |
|-----|------|------|
| 001 | Rust 언어 선정 | `docs/adrs/ADR-001-rust-language.md` |
| 002 | SQLite 데이터베이스 | `docs/adrs/ADR-002-sqlite-database.md` |
| 003 | Vibe Kanban 워크플로우 | `docs/adrs/ADR-003-vibe-kanban-workflow.md` |
| 004 | Tokio/Reqwest 스택 | `docs/adrs/ADR-004-tokio-reqwest-stack.md` |
| 005 | Gemini Flash LLM | `docs/adrs/ADR-005-gemini-flash-llm.md` |
| 006 | GitHub Actions CI/CD | `docs/adrs/ADR-006-github-actions-cicd.md` |
| 007 | Homebrew 배포 | `docs/adrs/ADR-007-homebrew-distribution.md` |
| 008 | Telegram 알림 | `docs/adrs/ADR-008-telegram-notification.md` |
| 009 | MCP Server 연동 | `docs/adrs/ADR-009-mcp-server-integration.md` |
| 010 | feed-rs RSS 파싱 | `docs/adrs/ADR-010-feed-rs-rss-parsing.md` |

### Micro-PRDs
- `docs/specs/PRD-001-core.md` ~ `docs/specs/PRD-009-telegram.md`
