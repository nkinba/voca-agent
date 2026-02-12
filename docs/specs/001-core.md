# Micro-PRD: Core Domain Layer

## 1. Goal
- 프로젝트(`spread`) 전체에서 사용할 공통 데이터 모델(Struct)과 추상 인터페이스(Trait)를 정의한다.
- 이 모듈은 외부 의존성을 최소화하고, 순수 비즈니스 로직의 타입을 제공해야 한다.
- `crates/core` 내부에 구현하며, 패키지명은 `spread-core`로 한다.

## 2. Dependencies
- `serde`: 데이터 직렬화/역직렬화 (features = ["derive"])
- `chrono`: 날짜/시간 처리 (features = ["serde"])
- `thiserror`: 에러 타입 정의
- `async-trait`: 비동기 트레이트 지원 (선택 사항, trait에 async fn 사용 시 필요)

## 3. Data Models (`src/model.rs`)
모든 모델은 `Debug`, `Clone`, `Serialize`, `Deserialize`를 derive 해야 한다.

### 3.1. Article (수집된 원문)
- **url**: `String` (Unique ID 역할)
- **title**: `String`
- **content**: `String` (본문 텍스트)
- **source**: `SourceType` (Enum: RSS, Manual, Youtube)
- **published_at**: `chrono::DateTime<Utc>`
- **collected_at**: `chrono::DateTime<Utc>`

### 3.2. Vocabulary (추출된 단어)
- **word**: `String`
- **definition**: `String` (LLM이 요약한 뜻)
- **context_sentence**: `String` (원문에서 발췌한 예문)
- **source_url**: `String` (Article의 url 참조)

## 4. Interfaces / Traits (`src/port.rs`)
시스템의 결합도를 낮추기 위해 핵심 동작을 Trait로 정의한다. 모든 함수는 `async`여야 한다. (필요시 `#[async_trait]` 매크로 활용)

### 4.1. FetcherPort
외부에서 데이터를 가져오는 역할.
```rust
pub trait FetcherPort: Send + Sync {
    /// URL에서 아티클을 수집하여 반환
    async fn fetch(&self, url: &str) -> Result<Article, CoreError>;
}
```

### 4.2. StoragePort
데이터를 영구 저장소(DB)에 저장하고 조회하는 역할.

```Rust
pub trait StoragePort: Send + Sync {
    /// URL 중복 검사 (true면 이미 존재)
    async fn exists(&self, url: &str) -> Result<bool, CoreError>;
    
    /// 아티클 저장
    async fn save_article(&self, article: &Article) -> Result<(), CoreError>;
    
    /// 추출된 단어장 저장
    async fn save_vocab(&self, vocab: &Vocabulary) -> Result<(), CoreError>;
}
```

### 5. Error Handling (src/error.rs)
시스템 전반의 에러를 통합 관리하는 CoreError Enum 정의.

```Rust
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Parsing error: {0}")]
    Parse(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}
```

### 6. Implementation Steps (Agent Instruction)
1. crates/core/Cargo.toml을 수정하여 패키지명을 spread-core로 설정하고, 위 의존성을 추가한다.
2. src/error.rs, src/model.rs, src/port.rs 파일을 각각 생성한다.
3. src/lib.rs에서 위 모듈들을 pub mod로 공개(Re-export)한다.
4. 구현체(impl)는 작성하지 않는다. 오직 타입과 인터페이스 정의에만 집중한다.
