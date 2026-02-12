# Micro-PRD: Homebrew Tap Distribution

## 1. Goal
- 사용자가 `brew install <user-name>/tap/spread` 명령어로 쉽게 설치할 수 있도록 지원한다.
- CI/CD 파이프라인(`release.yml`)과 연동하여, 새 버전이 나올 때마다 Formula를 자동으로 업데이트한다.

## 2. Infrastructure
- **New Repository:** `homebrew-tap` (또는 `homebrew-voca`)라는 이름의 Public Repository 생성 필요.

## 3. Specifications (`spread.rb`)
- **Formula Definition:**
  - `desc`: "Headless TOEFL Vocabulary Builder for Developers"
  - `homepage`: GitHub Repo URL
  - `url`: Release Tarball URL
  - `sha256`: Release Asset의 해시값
  - `bin.install`: 바이너리를 시스템 경로에 설치.

## 4. Automation (Optional but Recommended)
- 기존 `release.yml`의 후속 작업으로, Release가 성공하면 `homebrew-tap` 리포지토리의 `spread.rb` 파일 내용(URL, SHA256)을 갱신해서 Commit/Push 하는 Step 추가.
  - *Tool:* `mislav/bump-homebrew-formula-action` 등을 활용 가능.

## 5. Agent Instruction
1. Ruby 문법으로 된 기본적인 Homebrew Formula 파일(`spread.rb`)을 작성해줘.
2. 이 파일을 담을 별도의 Git 리포지토리(`homebrew-tap`)를 생성하고 Push 하는 가이드를 줘.
3. 로컬에서 `brew install --build-from-source ./spread.rb`로 설치 테스트를 진행해줘.