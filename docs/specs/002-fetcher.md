# Micro-PRD: RSS Fetcher Module

## 1. Goal
- `spread-core`에 정의된 `FetcherPort` 트레이트를 구현하는 구체적인 로직을 작성한다.
- 지정된 RSS URL에서 데이터를 가져와 `Article` 도메인 모델로 변환한다.
- 패키지명: `spread-fetcher`

## 2. Dependencies (`crates/fetcher/Cargo.toml`)
- `spread-core`: { path = "../core" }
- `reqwest`: { version = "0.11", features = ["json", "rustls-tls"] }
- `rss`: "2.0"
- `async-trait`: "0.1" (Trait 구현용)
- `thiserror`: "1.0"

## 3. Specifications

### 3.1. Struct Definition
- **Name:** `RssFetcher`
- **Field:** `client: reqwest::Client` (HTTP 클라이언트를 재사용하기 위함)
- **Constructor:** `pub fn new() -> Self`

### 3.2. Trait Implementation (`FetcherPort`)
`voca-core::port::FetcherPort`를 `RssFetcher`에 구현한다.

```rust
#[async_trait]
impl FetcherPort for RssFetcher {
    async fn fetch(&self, url: &str) -> Result<Article, CoreError> {
        // 1. reqwest로 GET 요청
        // 2. 응답 본문(Bytes)을 rss::Channel::read_from으로 파싱
        // 3. 가장 최신 Item 하나만 추출 (혹은 리스트 처리가 필요하면 로직 조정, 일단은 1:1 매핑 가정)
        // 4. Item -> voca_core::model::Article 변환
        // 5. 에러 발생 시 CoreError::Network 또는 CoreError::Parse로 래핑
    }
}
```

### 3.3. Mapping Logic (RSS Item -> Article)
- Title: Item title -> Article title
- Content: Item description 또는 content -> Article content (HTML 태그 제거는 선택사항, 일단 Raw String 저장)
- Date: pub_date -> chrono::DateTime 파싱 (RFC2822 포맷 처리)
- Source: SourceType::RSS 고정

### 4. Testing Requirements
- Mock Server: 실제 네트워크 요청 없이 테스트해야 함. mockito를 쓰거나, 로컬의 샘플 XML 문자열을 파싱하는 단위 테스트(Unit Test)를 작성할 것.
- Error Case: 잘못된 URL이나 깨진 XML 입력 시 CoreError::Parse가 반환되는지 검증.

### 5. Agent Instruction
1. Cargo.toml 의존성을 설정한다.
2. src/lib.rs에 RssFetcher 구조체를 만들고 FetcherPort를 구현한다.
3. spread-core의 타입을 사용해야 하므로 use spread_core::model::*; 등을 적절히 활용한다.