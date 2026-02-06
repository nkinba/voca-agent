# Micro-PRD: Telegram Notification (Micro-Habit)

## 1. Goal
- 수집된 단어 중 '오늘의 단어' 3개를 선정하여 사용자의 텔레그램으로 발송한다.
- 사용자가 앱을 켜지 않아도 학습이 이루어지게 한다 (Zero-Friction).
- 패키지명: `voca-notify`

## 2. Dependencies (`crates/notify/Cargo.toml`)
- `voca-core`: { path = "../core" }
- `voca-storage`: { path = "../storage" }
- `reqwest`: { workspace = true } (HTTP 요청용)
- `serde`: { workspace = true }
- `serde_json`: { workspace = true }
- `teloxide`: (선택사항) 봇 기능이 복잡해지면 도입하되, 단순 발송은 `reqwest`로 충분함. **초기엔 `reqwest` 권장.**

## 3. Specifications

### 3.1. Telegram Bot Setup
- **Bot Token:** `@BotFather`를 통해 발급 (`TELEGRAM_BOT_TOKEN`).
- **Chat ID:** 메시지를 받을 사용자의 ID (`TELEGRAM_CHAT_ID`).

### 3.2. Logic (`Notifier`)
1. **Fetch:** `voca-storage`에서 오늘 날짜(`created_at`)의 단어 조회.
2. **Select:** 랜덤하게 3개 선정 (단어가 3개 미만이면 전체).
3. **Format:** 가독성 좋은 메시지 포맷팅 (MarkdownV2 지원).
   ```text
   📚 *Today's Vocabulary*

   1. **Ephemeral**
      📖 _Lasting for a very short time._
      > "Fashions are ephemeral, changing with every season."

   2. ...
   
   [Open in Obsidian](obsidian://open?...)

 4. **Send:** Telegram API (POST /sendMessage) 호출.   

## 4. Execution Flow
- `app`의 스케줄러(Cron)가 매일 아침 8시에 `Notifier::run()`을 호출하도록 설정.
- 또는 CLI 명령어로 테스트: `voca-agent notify --test`

## 5. Agent Instruction
- `crates/notify` 모듈을 생성한다.
- `reqwest`를 사용하여 텔레그램 메시지를 보내는 심플한 클라이언트를 구현한다.
- `.env`에서 토큰과 챗 ID를 로드하고, 없을 경우 우아하게 기능을 Skip(Log warning)하도록 처리한다.