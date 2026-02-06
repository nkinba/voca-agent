# Micro-PRD: CI/CD Pipeline with GitHub Actions

## 1. Goal
- **Quality Gate:** PR 단계에서 Rust 코드 스타일, 린트 에러, 컴파일 에러, 테스트 실패를 자동으로 차단한다.
- **Release Automation:** 버전 태그(`v*`)가 푸시되면 자동으로 Release 바이너리를 빌드하여 GitHub에 업로드한다.

## 2. Infrastructure
- **Platform:** GitHub Actions
- **Runner:** `macos-latest` (Mac mini M4와 동일한 Apple Silicon 환경 보장)

## 3. Workflows Specifications

### 3.1. `ci.yml` (Pull Request Check)
- **Trigger:**
  - `push` to `main`
  - `pull_request` to `main`
- **Steps:**
  1. **Checkout Code**
  2. **Install Rust Toolchain:** `stable` profile.
  3. **Cache Dependencies:** `swatinem/rust-cache` Action 사용 (Rust 빌드 속도 핵심).
  4. **Check Formatting:** `cargo fmt --all -- --check`
  5. **Lint:** `cargo clippy -- -D warnings` (경고를 에러로 취급)
  6. **Run Tests:** `cargo test`
     - *Note:* `GEMINI_API_KEY`가 필요한 테스트는 GitHub Secrets에 키가 없으면 자동으로 Skip 되거나, Secrets를 주입해야 함.

### 3.2. `release.yml` (Build & Upload)
- **Trigger:** `push` tags (`v*`)
- **Steps:**
  1. **Checkout & Cache**
  2. **Build Release Binary:** `cargo build --release`
  3. **Create Release:** `softprops/action-gh-release` Action 사용.
  4. **Upload Artifact:** `target/release/voca-app` 바이너리를 릴리즈 에셋으로 업로드.

## 4. GitHub Repository Settings (Pre-requisites)
- **Secrets:** `GEMINI_API_KEY` 등록 필요 (테스트용).
- **Permissions:** Actions가 Releases를 생성할 수 있도록 `Read and write permissions` 설정 확인.

## 5. Agent Instruction
1. 프로젝트 루트에 `.github/workflows/` 폴더를 생성한다.
2. `ci.yml`을 작성하여 기본적인 코드 품질 검사를 자동화한다.
3. `release.yml`을 작성하여 배포 자동화를 구축한다.
4. `rust-cache`를 반드시 적용하여 불필요한 빌드 시간을 줄인다.