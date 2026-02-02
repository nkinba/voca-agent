### 📄 Micro-PRD 003: SQLite Storage Implementation

**파일명:** `docs/specs/003-storage.md`

```markdown
# Micro-PRD: SQLite Storage Module

## 1. Goal
- `voca-core`에 정의된 `StoragePort` 트레이트를 구현하여 영구 저장소를 관리한다.
- SQLite 데이터베이스를 사용하여 데이터를 저장하고 중복을 방지한다.
- 패키지명: `voca-storage`

## 2. Dependencies (`crates/storage/Cargo.toml`)
- `voca-core`: { path = "../core" }
- `sqlx`: { version = "0.7", features = ["runtime-tokio", "sqlite", "macros"] }
- `async-trait`: "0.1"
- `thiserror`: "1.0"

## 3. Specifications

### 3.1. Schema Design (Initialize)
- **Table 1: `articles`**
  - `url` (TEXT PRIMARY KEY)
  - `title` (TEXT)
  - `content` (TEXT)
  - `source` (TEXT)
  - `published_at` (DATETIME)
  - `collected_at` (DATETIME)
- **Table 2: `vocabularies`**
  - `id` (INTEGER PRIMARY KEY AUTOINCREMENT)
  - `word` (TEXT)
  - `definition` (TEXT)
  - `source_url` (TEXT, Foreign Key to articles.url)

### 3.2. Struct Definition
- **Name:** `SqliteStorage`
- **Field:** `pool: sqlx::SqlitePool`
- **Constructor:** - `pub async fn new(db_url: &str) -> Result<Self, CoreError>`
  - 생성 시점에 `CREATE TABLE IF NOT EXISTS` 쿼리를 실행하여 테이블을 자동 생성해야 함.

### 3.3. Trait Implementation (`StoragePort`)
`voca-core::port::StoragePort`를 `SqliteStorage`에 구현한다.

```rust
#[async_trait]
impl StoragePort for SqliteStorage {
    async fn exists(&self, url: &str) -> Result<bool, CoreError> {
        // SELECT count(*) FROM articles WHERE url = ?
    }

    async fn save_article(&self, article: &Article) -> Result<(), CoreError> {
        // INSERT OR IGNORE INTO articles ...
    }
    
    async fn save_vocab(&self, vocab: &Vocabulary) -> Result<(), CoreError> {
        // INSERT INTO vocabularies ...
    }
}
```

## 4. Testing Requirements
- In-Memory Test: 파일 DB 대신 메모리 DB(sqlite::memory:)를 사용하여 테스트 속도를 높이고 잔여 파일을 남기지 말 것.
- CRUD Test: 저장 후 exists가 true를 반환하는지, 저장된 데이터를 다시 불러올 수 있는지 검증.

## 5. Agent Instruction
- Cargo.toml 의존성을 설정한다.
- src/schema.rs (선택 사항) 또는 lib.rs 내부에 테이블 생성 쿼리를 상수로 정의한다.
- sqlx::query! 매크로를 사용하여 컴파일 타임 쿼리 검증을 시도하되, 에이전트 환경에서 .env 파일 설정이 복잡하면 일반 sqlx::query 함수를 사용해도 무방하다.