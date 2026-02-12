# Micro-PRD: Integration (Obsidian & MCP)

## 1. Goal
- 수집된 단어 데이터를 사용자의 **Obsidian Vault**에 Markdown 파일로 자동 생성한다.
- **Model Context Protocol (MCP)** 서버를 구현하여, 외부 AI 에이전트(Claude Desktop, Cursor)가 `spread`의 데이터베이스를 조회할 수 있게 한다.
- 패키지명: `spread-integration`

## 2. Dependencies (`crates/integration/Cargo.toml`)
- `spread-core`: { path = "../core" }
- `spread-storage`: { path = "../storage" }
- `tera`: "1.0" (Markdown 템플릿 엔진)
- `tokio`: { workspace = true }
- `serde_json`: "1.0"
- `axum`: "0.7" (MCP 서버용 HTTP/SSE - *선택사항, STDIO 방식 권장*)

## 3. Specifications

### 3.1. Obsidian Generator (`MarkdownExporter`)
- **기능:** `Daily Note` 또는 `Inbox` 폴더에 마크다운 파일 생성.
- **Template Logic (Tera):**
    ```markdown
    ---
    tag: #toefl #voca
    date: {{ today }}
    source: {{ article_url }}
    ---
    # {{ word }}
    **Definition:** {{ definition }}
    
    > {{ context_sentence }}
    
    [YouGlish로 발음 듣기](https://youglish.com/pronounce/{{ word }}/english?)
    ```
- **YouGlish Link:** 로드맵의 아이디어 반영. 단어를 쿼리 파라미터로 넣어 링크 생성.

### 3.2. MCP Server (`McpServer`)
- **Protocol:** `stdio` 기반 통신 (로컬 실행에 최적화).
- **Resources:**
    - `voca://daily-words`: 오늘의 단어 리스트 제공.
- **Tools (Prompt for Claude):**
    - `search_voca(query: str)`: 내 단어장에서 단어 뜻과 예문 검색.
    - `get_random_quiz()`: 랜덤하게 단어 하나를 뽑아 퀴즈 형태로 반환.

## 4. Execution Flow
1. `app` 실행 시 `config.toml`에서 `obsidian_path`를 읽음.
2. 수집이 끝나면 `MarkdownExporter::export()` 호출 -> 파일 생성.
3. (별도 모드) `spread mcp` 명령어로 실행 시, MCP 서버 모드로 진입하여 표준 입출력(STDIO) 대기.

## 5. Agent Instruction
1. `crates/integration`을 생성한다.
2. 템플릿 엔진(`tera`)을 이용해 Obsidian용 `.md` 파일을 생성하는 로직을 짠다.
3. `spread-core`의 모델을 `serde`로 직렬화하여 MCP 프로토콜(JSON-RPC 유사)에 맞춰 출력하는 기본 서버를 구현한다.