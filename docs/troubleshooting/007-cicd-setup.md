# CI/CD 설정 트러블슈팅 기록

## 개요

CI/CD 파이프라인(GitHub Actions) 설정 과정에서 발생한 문제와 해결 방법을 기록합니다.

## 문제 1: GitHub Action 이름 오류

### 증상
```
Error: Unable to resolve action dtolnay/rust-action, repository not found
```

### 원인
잘못된 액션 이름 사용: `dtolnay/rust-action@stable`

### 해결
올바른 액션 이름으로 수정: `dtolnay/rust-toolchain@stable`

### 교훈
GitHub Actions 사용 시 공식 저장소에서 정확한 액션 이름을 확인할 것.

---

## 문제 2: cargo fmt 실패 (로컬에서는 통과)

### 증상
- 로컬에서 `cargo fmt --check` 통과
- CI에서 `cargo fmt --check` 실패
- CI 로그에 로컬에 없는 파일 경로 표시:
  ```
  Diff in /Users/runner/work/voca-agent/voca-agent/crates/integration/src/mcp/handlers.rs
  Diff in /Users/runner/work/voca-agent/voca-agent/crates/storage/src/lib.rs
  ```

### 원인
**GitHub Actions PR 동작 방식 이해 필요**

GitHub Actions는 PR에서 CI를 실행할 때 **임시 병합 커밋**을 생성하여 실행합니다:

```
main:        A --- B --- C (crates/integration 추가)
                    \
feature:             D --- E (CI workflow 추가)
                          \
GitHub CI 실행 환경:         [C + E 병합된 상태]
```

- `main` 브랜치에는 `crates/integration/`, `crates/storage/` 등 새 코드가 있었음
- 현재 feature 브랜치에는 해당 코드가 없었음
- GitHub CI는 둘을 병합한 상태에서 `cargo fmt --check` 실행
- `main`에서 온 코드가 포맷팅되지 않은 상태였기 때문에 실패

### 해결
```bash
git fetch origin main
git merge origin/main
cargo fmt --all
git commit -m "style: Apply cargo fmt formatting to all crates"
git push
```

### 교훈
1. PR 생성 전에 `main` 브랜치를 병합하여 최신 상태 유지
2. CI가 도입되기 전에 병합된 코드는 포맷팅되지 않았을 수 있음 (초기 설정 비용)
3. CI가 활성화된 후에는 모든 코드가 포맷팅된 상태로 병합되므로 이 문제는 재발하지 않음

---

## 문제 3: clippy ptr_arg 린트 오류

### 증상
```
error: writing `&PathBuf` instead of `&Path` involves a new object where a slice will do
   --> app/src/main.rs:195:60
    |
195 | async fn export_to_obsidian(storage: &SqliteStorage, path: &PathBuf) {
    |                                                            ^^^^^^^^
```

### 원인
Clippy의 `ptr_arg` 린트: 함수 인자로 `&PathBuf` 대신 `&Path`를 사용하는 것이 더 효율적이고 유연함.

- `&PathBuf`: 특정 타입에 의존
- `&Path`: `PathBuf`, `&str`, `String` 등 다양한 타입에서 변환 가능

### 해결
```rust
// Before
async fn export_to_obsidian(storage: &SqliteStorage, path: &PathBuf)

// After
use std::path::{Path, PathBuf};
async fn export_to_obsidian(storage: &SqliteStorage, path: &Path)
```

### 교훈
Clippy 경고를 에러로 처리(`-D warnings`)하면 코드 품질이 향상됨.

---

## 예방책: pre-commit 설정

커밋 전에 자동으로 fmt, clippy를 실행하도록 pre-commit 설정:

### 설치
```bash
# macOS
brew install pre-commit

# pip
pip install pre-commit
```

### 활성화
```bash
pre-commit install
```

### 설정 파일 (.pre-commit-config.yaml)
```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --all -- --check
        language: system
        files: \.rs$
        pass_filenames: false

      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy -- -D warnings
        language: system
        files: \.rs$
        pass_filenames: false
```

이 설정으로 `git commit` 시 자동으로 fmt와 clippy가 실행되어, CI에서 실패하기 전에 로컬에서 문제를 발견할 수 있습니다.
