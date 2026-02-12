## ADR-006: CI/CD 파이프라인으로 GitHub Actions 선정

### 1. Context
- 코드 품질 보장을 위해 자동화된 빌드/테스트/린트 파이프라인이 필요함.
- 릴리스 자동화(바이너리 빌드, Homebrew Formula 업데이트)가 필요함.
- 1인 개발 환경에서 별도 CI 서버 운영은 부담임.

### 2. Decision
- **GitHub Actions**를 CI/CD 플랫폼으로 채택한다.
- 두 개의 워크플로우 구성:
  - `ci.yml`: PR/Push 시 빌드, 테스트, 클리피(clippy), 포맷 검사
  - `release.yml`: 태그 푸시 시 멀티 플랫폼 바이너리 빌드 및 릴리스

### 3. Rationale (선정 이유)
- **GitHub 네이티브:** 코드 저장소와 동일 플랫폼에서 CI/CD 운영으로 관리 단순화.
- **무료 티어:** Public repo 무제한, Private repo도 월 2000분 무료.
- **Rust 지원:** `actions-rs` 계열 액션 및 캐싱 지원으로 빌드 시간 최적화.
- **Matrix Build:** macOS, Linux, Windows 동시 빌드 지원.

### 4. Critical View (비판적 시각)
- **빌드 시간:** Rust 프로젝트는 컴파일 시간이 길어 무료 티어 소진 가능성 있음.
- **Runner 제약:** macOS runner는 Linux 대비 할당량이 적고 느림.
- **캐시 관리:** Cargo 캐시가 효율적으로 동작하지 않으면 매번 전체 빌드 발생.
- **복잡한 워크플로우:** 멀티 플랫폼 릴리스 워크플로우는 YAML이 복잡해질 수 있음.

### 5. Implementation Details

#### CI Pipeline (`ci.yml`)
```yaml
on: [push, pull_request]
jobs:
  check:
    - cargo fmt --check
    - cargo clippy -- -D warnings
  test:
    - cargo test --workspace
  build:
    - cargo build --release
```

#### Release Pipeline (`release.yml`)
```yaml
on:
  push:
    tags: ['v*']
jobs:
  build:
    matrix:
      - macos-latest (aarch64-apple-darwin)
      - ubuntu-latest (x86_64-unknown-linux-gnu)
    steps:
      - cargo build --release
      - Upload to GitHub Release
      - Update Homebrew Formula
```

### 6. Future Evolution (개선 방향)
- **캐시 최적화:** `Swatinem/rust-cache` 액션 활용하여 의존성 캐싱 개선.
- **빌드 시간 단축:** `sccache` 도입 또는 `mold` 링커 사용 검토.
- **Windows 지원:** 현재 macOS/Linux만 지원, 필요시 Windows 타겟 추가.
- **자동 버전 관리:** `cargo-release` 또는 `release-please`로 버전 bumping 자동화.
