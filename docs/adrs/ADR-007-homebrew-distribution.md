## ADR-007: macOS 배포 채널로 Homebrew Tap 선정

### 1. Context
- macOS 사용자를 위한 간편한 설치 방법이 필요함.
- 바이너리 직접 다운로드는 사용자 경험이 좋지 않고, 업데이트 관리가 어려움.
- macOS 개발자들에게 친숙한 패키지 관리자를 통한 배포 필요.

### 2. Decision
- **Homebrew Tap**을 통해 macOS용 바이너리를 배포한다.
- 별도 Tap 저장소(`nkinba/homebrew-tap`) 또는 프로젝트 내 `Formula/` 디렉토리 활용.
- GitHub Release의 tarball을 소스로 사용하는 Formula 작성.

### 3. Rationale (선정 이유)
- **사용자 친숙성:** macOS 개발자 대부분이 Homebrew 사용 중.
- **설치 간편성:** `brew install spread` 한 줄로 설치 완료.
- **업데이트 용이:** `brew upgrade` 명령으로 자동 업데이트.
- **의존성 관리:** Homebrew가 의존성(OpenSSL 등)을 자동 처리.

### 4. Critical View (비판적 시각)
- **Tap 관리:** 별도 저장소 관리 필요, Formula 업데이트 자동화 필요.
- **macOS Only:** Linux/Windows 사용자는 다른 방법 필요.
- **빌드 시간:** 소스 빌드 시 Rust 컴파일 시간이 길어 사용자 경험 저하 가능.
- **Apple Silicon:** arm64/x86_64 바이너리 모두 제공 필요.

### 5. Implementation Details

#### Formula 구조 (`Formula/spread.rb`)
```ruby
class Spread < Formula
  desc "AI-powered vocabulary extraction agent"
  homepage "https://github.com/nkinba/voca-agent"
  url "https://github.com/nkinba/voca-agent/archive/refs/tags/v#{version}.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "app")
  end
end
```

#### 설치 명령
```bash
brew tap nkinba/tap
brew install spread
```

### 6. Future Evolution (개선 방향)
- **Pre-built Binary:** 소스 빌드 대신 미리 컴파일된 바이너리 배포로 설치 시간 단축.
- **Homebrew Core 등록:** 사용자 수 증가 시 공식 Homebrew Core 등록 검토.
- **Linux 지원:** Linuxbrew 지원 또는 별도 패키지 매니저(apt, dnf) 배포.
- **Cask 고려:** GUI 앱 추가 시 Homebrew Cask로 전환.
