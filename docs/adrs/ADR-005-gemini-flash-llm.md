## ADR-005: LLM으로 Gemini 2.5 Flash 선정

### 1. Context
- TOEFL 수준의 영어 어휘를 텍스트에서 추출하기 위해 LLM이 필요함.
- 초기 설계에서는 OpenAI API 또는 Anthropic API를 고려했으나, 비용 효율성과 응답 속도를 재검토함.
- MVP 단계에서 빠른 반복 개발과 비용 절감이 중요함.

### 2. Decision
- **Google Gemini 2.5 Flash**를 LLM으로 채택한다.
- `reqwest`를 통해 `generativelanguage.googleapis.com` API를 직접 호출한다.
- JSON 형식으로 구조화된 어휘 데이터를 응답받는다.

### 3. Rationale (선정 이유)
- **비용 효율성:** Gemini Flash는 OpenAI GPT-4 대비 상당히 저렴한 가격 정책 제공.
- **응답 속도:** "Flash" 모델은 latency 최적화 모델로, 빠른 응답 시간 제공.
- **무료 티어:** 개발/테스트 단계에서 무료 할당량 활용 가능.
- **JSON 모드:** 구조화된 출력(Structured Output)을 안정적으로 지원.

### 4. Critical View (비판적 시각)
- **API 안정성:** Google AI API는 OpenAI 대비 변경이 잦을 수 있음 (beta 상태).
- **품질 차이:** 특정 도메인(어휘 추출)에서 GPT-4나 Claude 대비 품질이 낮을 가능성 있음.
- **Vendor Lock-in:** Google 생태계에 종속될 수 있음.
- **Rate Limit:** 무료 티어의 분당 요청 제한이 있어 대량 처리 시 throttling 필요.

### 5. Implementation Details
- **Endpoint:** `https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent`
- **Auth:** API Key via `GEMINI_API_KEY` 환경 변수
- **Prompt Strategy:** System prompt로 "TOEFL 시험 작문자" 역할 부여, CEFR C1/C2 수준 단어만 추출 지시
- **Post-processing:** 불용어(stopwords) 필터링, 3자 이하 단어 제외

### 6. Future Evolution (개선 방향)
- **LLM 교체 용이성:** `LlmPort` 트레이트를 통해 추상화되어 있으므로, 다른 LLM(Claude, GPT-4)으로 쉽게 교체 가능.
- **로컬 LLM 고려:** Ollama를 통한 로컬 LLM(Llama 3 등) 지원으로 비용 완전 제거 가능.
- **프롬프트 튜닝:** 추출 품질 향상을 위한 Few-shot 예제 추가 검토.
