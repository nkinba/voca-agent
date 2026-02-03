## ADR-002: 로컬 데이터베이스로 SQLite 선정

### 1. Context
- 수집된 URL의 중복 방지(Deduplication)와 단어장 저장이 필요함.
- 서버가 아닌 로컬/개인용 장비에서 구동되므로, 별도의 데몬 프로세스(Docker 등) 관리를 최소화해야 함.
### 2. Decision
- **SQLite**를 사용하며, ORM 대신 **`sqlx`** (Compile-time Checked Queries)를 사용한다
- 파일 기반 DB로 배포 및 관리를 단순화한다.
### 3. Critical View (비판적 시각)
- **동시성 쓰기 제한:** SQLite는 기본적으로 **Single Writer** 구조임. 만약 `Fetcher` 에이전트를 수십 개 띄워 병렬 수집을 할 경우 `Database Locked` 에러가 발생할 수 있음.
- **벡터 검색의 한계:** 추후 "의미 기반 검색(Semantic Search)" 기능을 넣고 싶을 때, SQLite의 벡터 익스텐션(`sqlite-vss`)은 설정이 까다롭거나 성능 제약이 있을 수 있음.
- **클라우드 확장성:** 추후 AWS Lambda 등으로 이전 시, 파일 시스템 기반 DB는 공유가 어려워 아키텍처를 뜯어고쳐야 함.
### 4. Future Evolution (개선 방향)
- **WAL Mode:** 동시성 문제를 완화하기 위해 SQLite의 **WAL(Write-Ahead Logging) 모드**를 활성화한다.
- **Migration Path:** `sqlx`는 PostgreSQL도 지원하므로, 트래픽이 늘어나거나 벡터 검색이 필요해지면 **PostgreSQL(+pgvector)** 로 마이그레이션한다. 코드 변경은 쿼리 문법의 미세한 수정 외에는 최소화된다.