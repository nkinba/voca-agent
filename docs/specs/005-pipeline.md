# Micro-PRD: App Pipeline Integration

## 1. Goal
- `fetcher`, `storage`, `llm` 모듈을 `app` 바이너리(`main.rs`)에서 연결한다.
- 주기적 실행(Loop) 또는 단발성 실행 흐름을 제어하고, 에러가 발생해도 전체 프로세스가 죽지 않도록(Graceful Handling) 한다.

## 2. Dependencies (`app/Cargo.toml`)
- `voca-fetcher`: { workspace = true }
- `voca-storage`: { workspace = true }
- `voca-llm`: { workspace = true }
- `tokio`: { workspace = true }
- `tracing`: { workspace = true }

## 3. Workflow Logic (`app/src/main.rs` or `workflow.rs`)

### 3.1. Initialization
- `.env` 로드.
- `SqliteStorage`, `RssFetcher`, `RigLlmEngine` 인스턴스 생성.

### 3.2. Execution Pipeline (Sequential Flow)
1. **Fetch Feed:** RSS URL 리스트(설정 파일 또는 하드코딩)를 순회하며 `fetcher.fetch_feed(url)` 호출.
2. **Deduplication:** 가져온 각 Item의 URL에 대해 `storage.exists(url)` 확인.
    - 이미 있으면 `continue` (Skip).
3. **Fetch Body:** 새로운 URL이면 `fetcher.fetch_body(url)`로 본문 획득.
4. **AI Extract:** 본문 텍스트를 `llm.extract(body)`에 전달.
    - **Rate Limiting:** API 비용 및 제한을 고려하여 호출 사이 `tokio::time::sleep` (예: 2초) 추가.
5. **Persist:**
    - `storage.save_article(article)` (원문 저장)
    - `storage.save_vocab(vocab)` (단어 저장 - 트랜잭션 처리 권장하지만 필수는 아님)
6. **Logging:** 각 단계별 성공/실패 여부를 `tracing::info!` / `error!`로 기록.

## 4. Exception Handling
- **LLM 추출 실패 시:** 해당 아티클은 저장하되, 단어는 비어있는 상태로 넘어가거나, 에러 로그를 남기고 다음 아티클로 진행 (Panic 금지).
- **Network 실패 시:** 재시도(Retry) 로직은 각 라이브러리에 위임하거나, 다음 주기에 다시 시도하도록 둔다.

## 5. Agent Instruction
1. `app/src/workflow.rs` 모듈을 만들어 비즈니스 로직을 분리한다.
2. `main` 함수에서는 의존성 주입(Dependency Injection) 후 workflow를 실행한다.
3. `voca-llm` 구현이 아직 안 끝났을 경우를 대비해, `MockLlmEngine`을 임시로 만들어 컴파일이 되게 한다. (또는 `RigLlmEngine`의 껍데기만 사용)