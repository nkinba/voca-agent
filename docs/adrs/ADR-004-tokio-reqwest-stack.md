## ADR-004: Heavy Library Stack (Tokio, Reqwest) 채택

### 1. Context
- 비동기 I/O 처리가 필수적임.
- Rust 생태계에서 가장 표준적이고 레퍼런스가 많은 라이브러리를 선호함.
### 2. Decision
- 비동기 런타임: **`tokio`** (Full features)
- HTTP 클라이언트: **`reqwest`**
- 에러 처리: 라이브러리(`voca-core` 등)에는 **`thiserror`**, 앱(`app`)에는 **`anyhow`**
### 3. Critical View (비판적 시각)
- **Binary Size Bloat:** `tokio`와 `reqwest`는 무겁기로 유명함. 단순한 CLI 도구 치고는 컴파일된 바이너리 크기가 크고, 컴파일 속도도 느림.
- **Overkill:** 단순한 주기적 실행(Cron) 작업에 이벤트 루프 기반의 풀 비동기 런타임은 과할 수 있음. 동기식(`ureq`, `std::thread`)이 더 간단하고 빠를 수도 있음.
### 4. Future Evolution (개선 방향)
- **Feature Flag Optimization:** `Cargo.toml`에서 `default-features = false`를 적극 활용하여 불필요한 기능(예: `reqwest`의 cookie store, `tokio`의 full macros 중 미사용분)을 제거한다.
- **Lighter Alternatives:** 만약 바이너리 크기가 치명적인 문제가 된다면(AWS Lambda 등), `reqwest` 대신 **`ureq`**(3.x부터 비동기 지원 예정)나 **`smol`** 런타임으로 교체를 고려한다. 하지만 현재 단계에서는 **개발 편의성(풍부한 문서)**이 우선이다.