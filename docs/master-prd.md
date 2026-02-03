# 📑 Master PRD: Voca-Agent (Headless Vocabulary Builder)

## 1. Project Overview
**Voca-Agent**는 사용자가 직접 단어를 입력하는 수고를 덜어주는 **Headless AI 에이전트**입니다. 사용자가 관심 있는 영문 기술 블로그나 뉴스(RSS)를 백그라운드에서 주기적으로 모니터링하고, **TOEFL Actual Test 수준의 어휘**를 AI로 추출하여, **Obsidian Vault**에 자동으로 동기화합니다.

### 1.1. Core Value Proposition
- **Automated Capture:** RSS 및 웹 크롤링을 통한 무중단 정보 수집.
- **Intelligent Processing:** LLM을 활용해 단순 사전적 정의가 아닌 '문맥(Context)' 기반의 어휘 및 예문 추출.
- **Seamless Integration:** 사용자의 지식 관리 도구(Obsidian)와 파일 시스템 레벨(MCP)에서 통합.
- **Performance & Stability:** Rust 기반의 메모리 안전하고 효율적인 시스템 구축.

---

## 2. System Architecture
이 프로젝트는 **Rust Workspace**를 활용한 모듈러 모놀리스(Modular Monolith) 구조를 따르며, 추후 MSA나 FaaS로의 확장을 고려한 **Port & Adapter (Hexagonal) 아키텍처**를 지향합니다.

### 2.1. Tech Stack
- **Language:** Rust (Edition 2021)
- **Runtime:** Tokio (Async/Await)
- **Database:** SQLite (via `sqlx`) - 로컬 파일 기반, 중복 방지 및 이력 관리
- **Network:** `reqwest`, `rss`, `scraper`
- **AI Integration:** OpenAI API / Anthropic API (via `reqwest` or `async-openai`)
- **Interface:** CLI (Command Line) & MCP (Model Context Protocol) Server

### 2.2. Directory Structure (Workspace)
```text
voca-agent/
├── Cargo.toml          # Workspace Definition
├── app/                # [Binary] Orchestrator & Scheduler
└── crates/
    ├── voca-core/      # [Lib] Domain Models, Traits, Errors (Pure Rust)
    ├── voca-fetcher/   # [Lib] RSS/Web Crawling Logic (Impl FetcherPort)
    ├── voca-storage/   # [Lib] SQLite Persistence (Impl StoragePort)
    └── voca-llm/       # [Lib] AI Extraction Logic (To be implemented)
```

---

## 3. Module Specifications (Summary of Micro-PRDs)

### 3.1. `voca-core` (The Constitution)
시스템의 데이터 모델과 인터페이스를 정의합니다. 외부 라이브러리 의존성을 최소화합니다.
- **Models:**
    - `Article`: 수집된 원문 정보 (URL, Title, Content, PublishedAt).
    - `Vocabulary`: 추출된 학습 정보 (Word, Definition, Context Sentence).
- **Traits (Ports):**
    - `FetcherPort`: `async fn fetch(url) -> Result<Article>`
    - `StoragePort`: `async fn exists(url) -> bool`, `async fn save(...)`
    - `LlmPort`: `async fn extract(text) -> Vec<Vocabulary>` (예정)

### 3.2. `voca-fetcher` (The Collector)
외부 세계의 데이터를 가져와 Core 모델로 변환합니다.
- **Dependency:** `reqwest`, `rss`
- **Function:** RSS 피드를 파싱하여 새로운 글을 감지하고, 본문을 긁어옵니다.
- **Key Logic:** HTML 태그 제거 및 텍스트 정제(Sanitization).

### 3.3. `voca-storage` (The Memory)
데이터의 영속성을 담당하며 중복 수집을 방지합니다.
- **Dependency:** `sqlx` (SQLite)
- **Schema:**
    - `articles`: 수집 이력 관리 (URL이 Primary Key).
    - `vocabularies`: 추출된 단어 저장.
- **Key Logic:** `INSERT OR IGNORE`를 활용한 효율적인 중복 처리.

### 3.4. `app` (The Brain)
각 모듈을 조립(Wiring)하고 스케줄링합니다.
- **Role:** 설정 파일(`config.toml`) 로드, 스케줄러(Cron) 실행, 에러 로깅.
- **Workflow:**
    1.  `voca-fetcher`로 RSS 조회.
    2.  `voca-storage`에서 URL 중복 확인.
    3.  (New) `voca-fetcher`로 본문 크롤링.
    4.  (New) `voca-llm`으로 단어 추출.
    5.  `voca-storage`에 결과 저장 및 Obsidian Markdown 생성.

---

## 4. Development Workflow & Infrastructure

### 4.1. Development Environment
- **Platform:** macOS / Linux (via Tailscale + SSH + tmux)
- **Task Management:** Vibe Kanban
- **Code Assistant:** Claude Code (Agent)

### 4.2. Git Strategy (Strict: Worktree + PR)
- **Remote:** GitHub Repository (`origin`) 연결 필수.
- **Parallel Work:** `git worktree`를 사용하여 브랜치별 독립 작업 공간 운용.
- **Merge Policy:** **No Local Merge.**
    1. 모든 작업은 Feature Branch에서 수행 (각 Worktree에서).
    2. 작업 완료 시 `git push origin <branch>`.
    3. GitHub Web UI에서 **PR(Pull Request)** 생성 및 Code Review 후 Merge.
    4. 로컬 Main(`app` 폴더)은 `git pull`로 동기화.

---

## 5. Roadmap & Milestones

### Phase 1: Foundation (Current)
- [x] Rust Workspace 구조 세팅.
- [x] `voca-core` 정의 (Domain Model).
- [In-Progress] `voca-fetcher` 구현 (RSS Parsing).
- [In-Progress] `voca-storage` 구현 (SQLite).

### Phase 2: Intelligence (Next)
- [ ] `voca-llm` 모듈 구현 (Prompt Engineering & API Client).
- [ ] TOEFL 수준 단어 필터링 로직 구현.
- [ ] `app`에서 Fetcher -> Storage -> LLM 파이프라인 연결 및 통합 테스트.

### Phase 3: Integration
- [ ] Obsidian용 Markdown (`.md`) 생성기 구현.
- [ ] MCP Server 인터페이스 연동 (Obsidian에서 에이전트 호출).
- [ ] Docker Container 빌드 및 배포 자동화.

---

## 6. References
- **ADR Docs:** `docs/ADR.md` (기술적 의사결정 기록)
- **Micro-PRDs:** `docs/specs/*.md` (모듈별 상세 명세)