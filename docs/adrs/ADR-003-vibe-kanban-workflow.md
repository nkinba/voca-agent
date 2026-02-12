## ADR-003: Vibe Kanban과 AI Agent(Claude Code) 기반 워크플로우

### 1. Context
- 1인 개발자 체제에서 기획, 개발, 검증을 모두 수행해야 함.
- 컨텍스트 스위칭 비용을 줄이고, AI 에이전트에게 명확한 작업 단위(Task)를 할당할 필요가 있음.
    
### 2. Decision
- **Vibe Kanban**을 로컬 태스크 관리 도구로 사용하며, 각 카드를 **Micro-PRD** 단위로 관리한다.
- **Claude Code** (CLI Agent)를 실행 엔진으로 사용하며, `git worktree`와 연계하여 병렬 개발을 수행한다.
### 3. Critical View (비판적 시각)
- **도구 파편화:** GitHub Issues/Projects와 Vibe Kanban 간의 연동이 자동으로 되지 않아, 이력 관리가 이중화될 수 있음.
- **로컬 의존성:** Vibe Kanban 데이터가 로컬 파일에만 존재할 경우, 작업 환경 이동(카페, 집, 회사) 시 동기화가 번거로울 수 있음.
- **협업 불가:** 추후 팀 단위 프로젝트로 확장 시 Vibe Kanban은 공유가 어려움.

### 4. Future Evolution (개선 방향)
- **Sync Automation:** 작업이 완료된 카드는 GitHub API를 통해 Issue를 닫거나 PR에 링크하는 스크립트를(Rust로) 작성하여 자동화한다.
- **Single Source of Truth:** 프로젝트가 커지면 GitHub Projects를 메인으로 하고, Vibe Kanban은 개인의 "투두 리스트" 용도로 범위를 축소한다.