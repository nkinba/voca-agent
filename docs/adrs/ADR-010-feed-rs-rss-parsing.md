## ADR-010: RSS/Atom 파싱 라이브러리로 feed-rs 선정

### 1. Context
- 초기 구현에서 `rss` 크레이트를 사용하여 RSS 피드를 파싱함.
- 일부 기술 블로그(GitHub Blog 등)가 Atom 피드만 제공하여 파싱 실패 발생.
- RSS 2.0, Atom, JSON Feed 등 다양한 포맷을 통합 지원할 필요 있음.

### 2. Decision
- **`feed-rs`** 크레이트로 교체하여 RSS/Atom/JSON Feed를 통합 파싱한다.
- `rss` 크레이트 의존성 제거.
- 피드 포맷 자동 감지(Auto-detection) 기능 활용.

### 3. Rationale (선정 이유)
- **통합 파싱:** RSS 1.0, RSS 2.0, Atom 0.9/1.0, JSON Feed 1.x 모두 단일 API로 처리.
- **자동 감지:** `feed_rs::parser::parse()` 함수가 포맷을 자동 판별.
- **풍부한 메타데이터:** 저자, 카테고리, 미디어 등 확장 필드 접근 가능.
- **활발한 유지보수:** 2024년 기준 활발히 업데이트되는 크레이트.

### 4. Critical View (비판적 시각)
- **의존성 증가:** `rss` 대비 더 많은 transitive 의존성 포함.
- **API 차이:** 기존 `rss` 크레이트와 API가 달라 마이그레이션 작업 필요.
- **에러 핸들링:** 파싱 실패 시 에러 메시지가 덜 직관적일 수 있음.
- **메모리 사용:** 모든 포맷 지원으로 인해 약간의 오버헤드 가능.

### 5. Implementation Details

#### 의존성 변경
```toml
# Before
[dependencies]
rss = "2"

# After
[dependencies]
feed-rs = "2"
```

#### 파싱 코드
```rust
use feed_rs::parser;

pub async fn fetch_feed(&self, feed_url: &str) -> Result<Vec<FeedItem>, CoreError> {
    let response = self.client.get(feed_url).send().await?;
    let body = response.bytes().await?;

    // 자동 포맷 감지
    let feed = parser::parse(&body[..])
        .map_err(|e| CoreError::Parse(format!("Feed parse error: {}", e)))?;

    let items = feed.entries.into_iter().map(|entry| {
        FeedItem {
            url: entry.links.first().map(|l| l.href.clone()).unwrap_or_default(),
            title: entry.title.map(|t| t.content).unwrap_or_default(),
            published_at: entry.published.unwrap_or_else(Utc::now),
        }
    }).collect();

    Ok(items)
}
```

#### 지원 피드 포맷
| 포맷 | 예시 사이트 |
|------|-----------|
| RSS 2.0 | 대부분의 블로그 |
| Atom 1.0 | GitHub Blog, Medium |
| JSON Feed | 일부 모던 블로그 |

### 6. Future Evolution (개선 방향)
- **OPML Import:** OPML 파일에서 피드 목록 일괄 추가 기능.
- **피드 검증:** 피드 URL 유효성 검사 및 자동 피드 발견(Auto-discovery).
- **캐싱:** ETag/Last-Modified 헤더를 활용한 조건부 요청으로 대역폭 절약.
- **에러 복구:** 일시적 파싱 실패 시 재시도 로직 추가.
