## ADR-001: 핵심 프로그래밍 언어로 Rust 선정

### 1. Context
- `spread`는 24시간 백그라운드에서 실행되는 Headless 에이전트임.
- 시스템 리소스(CPU/Mem) 효율성이 중요하며, 동시성 처리(크롤링)가 필수적임.
### 2. Decision
- **Rust**를 주 언어로 채택한다.
- Python(생산성)이나 Go(실용성) 대신, 컴파일 타임 안정성과 메모리 효율성이 보장되는 Rust를 선택한다.
### 3. Critical View (비판적 시각)
- **생산성 저하:** 단순한 스크립트 작성에 비해 컴파일 시간과 타입 시스템(Borrow Checker)과의 씨름으로 초기 개발 속도가 느릴 수 있음.
- **오버 엔지니어링:** 텍스트 수집/가공 로직에 Rust의 엄격한 메모리 관리는 과한 스펙일 수 있음. (String 처리의 복잡함 등)
- **생태계 파편화:** Python의 LangChain 등에 비해 AI 관련 라이브러리(LLM 연동 등)가 상대적으로 빈약하거나 로우 레벨임.
### 4. Future Evolution (개선 방향)
- **Rapid Prototyping:** 초기에는 `unwrap()`, `clone()`을 허용하여 개발 속도를 높이고, 안정화 단계에서 리팩토링한다.
- **Python 혼용 고려:** 만약 LLM 오케스트레이션 로직이 지나치게 복잡해질 경우, 핵심 코어는 Rust로 두고 상위 로직을 Python 바인딩(`PyO3`)으로 분리하는 **Hybrid Architecture**를 고려한다.