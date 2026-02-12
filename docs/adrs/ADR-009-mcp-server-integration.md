## ADR-009: Obsidian 연동을 위한 MCP Server 구현

### 1. Context
- 추출된 어휘를 Obsidian Vault와 동기화할 필요가 있음.
- Obsidian 플러그인 개발 대신 표준화된 프로토콜 활용 희망.
- AI 에이전트(Claude, Cursor 등)와의 통합을 통한 학습 워크플로우 자동화 필요.

### 2. Decision
- **MCP(Model Context Protocol)** 서버를 구현하여 Obsidian 및 AI 에이전트와 통신한다.
- `voca-integration` 크레이트에 MCP 서버 모듈 구현.
- stdio 모드로 동작하여 Obsidian MCP 플러그인과 연동.

### 3. Rationale (선정 이유)
- **표준 프로토콜:** Anthropic이 제안한 MCP는 AI 에이전트 생태계의 표준으로 자리잡는 중.
- **양방향 통신:** 단순 파일 내보내기를 넘어 Tools/Resources를 통한 상호작용 가능.
- **확장성:** Claude Code, Cursor, Zed 등 다양한 AI 도구와 호환.
- **Obsidian 통합:** Obsidian MCP 플러그인을 통해 노트 앱 내에서 직접 에이전트 호출 가능.

### 4. Critical View (비판적 시각)
- **프로토콜 성숙도:** MCP는 아직 초기 단계로, 스펙 변경 가능성 있음.
- **구현 복잡도:** JSON-RPC 기반 프로토콜 직접 구현 필요.
- **디버깅 어려움:** stdio 모드는 로그 확인 및 디버깅이 까다로움.
- **Obsidian 의존:** MCP 플러그인이 Obsidian에 설치되어 있어야 함.

### 5. Implementation Details

#### 모듈 구조
```
crates/integration/src/mcp/
├── mod.rs          # 모듈 진입점
├── server.rs       # McpServer 구현
├── handlers.rs     # 요청 핸들러
└── protocol.rs     # JSON-RPC 타입 정의
```

#### 지원 기능 (현재)
| Type | Name | Description |
|------|------|-------------|
| Tool | `get_vocabulary` | 저장된 어휘 조회 |
| Tool | `search_vocabulary` | 키워드로 어휘 검색 |
| Resource | `vocab://today` | 오늘의 단어 리소스 |

#### CLI 실행
```bash
spread mcp   # stdio 모드로 MCP 서버 시작
```

#### Obsidian 설정 예시
```json
{
  "mcpServers": {
    "voca-agent": {
      "command": "spread",
      "args": ["mcp"]
    }
  }
}
```

### 6. Future Evolution (개선 방향)
- **Tools 확장:**
  - `add_manual_word`: 수동 단어 추가
  - `get_learning_stats`: 학습 통계 조회
  - `export_to_anki`: Anki 덱 내보내기
- **Resources 확장:**
  - `vocab://random`: 랜덤 단어
  - `vocab://week`: 이번 주 단어
- **HTTP 모드:** stdio 외에 HTTP/SSE 모드 지원으로 원격 접속 가능.
- **인증:** API 키 기반 인증으로 보안 강화.
- **AI 워크플로우:** Claude가 학습 진도를 파악하고 맞춤형 예문을 생성하는 에이전틱 워크플로우 구현.
