## ADR-008: 알림 채널로 Telegram Bot API 선정

### 1. Context
- 일일 학습 어휘를 사용자에게 푸시 알림으로 전달할 필요가 있음.
- 별도 앱 없이 기존에 사용하는 메신저를 통해 알림 수신 희망.
- 서버리스/헤드리스 환경에서 동작 가능한 알림 채널 필요.

### 2. Decision
- **Telegram Bot API**를 알림 채널로 채택한다.
- `voca-notify` 크레이트에서 `TelegramClient` 및 `Notifier` 구현.
- 매일 학습한 단어 중 랜덤 3개를 MarkdownV2 형식으로 발송.

### 3. Rationale (선정 이유)
- **무료:** Telegram Bot API는 완전 무료, 메시지 수 제한 없음.
- **간편한 설정:** BotFather로 봇 생성 후 토큰만 있으면 바로 사용 가능.
- **리치 포맷:** MarkdownV2로 bold, italic, code block 등 풍부한 포맷 지원.
- **크로스 플랫폼:** 모바일/데스크톱/웹 모든 환경에서 알림 수신 가능.
- **API 안정성:** 오래된 API로 변경이 적고 문서화가 잘 되어 있음.

### 4. Critical View (비판적 시각)
- **Telegram 의존성:** Telegram을 사용하지 않는 사용자는 별도 채널 필요.
- **보안 우려:** Bot Token이 노출되면 누구나 메시지 발송 가능.
- **MarkdownV2 이스케이프:** 특수문자 이스케이프가 까다로움.
- **단방향 통신:** 현재는 알림만 발송, 사용자 응답 처리 미구현.

### 5. Implementation Details

#### 환경 변수
```bash
TELEGRAM_BOT_TOKEN=123456:ABC-DEF...
TELEGRAM_CHAT_ID=987654321
```

#### API 엔드포인트
```
POST https://api.telegram.org/bot{token}/sendMessage
```

#### 메시지 포맷 예시
```
📚 *Today's Vocabulary*

1. *ubiquitous*
   📖 _present everywhere at the same time_
   > "Cloud computing has become ubiquitous in modern software architecture."

2. *ephemeral*
   📖 _lasting for a very short time_
   > "Container instances are ephemeral by design."
```

#### CLI 명령
```bash
spread notify           # 오늘의 단어 3개
spread notify --all     # 전체 단어
spread notify --test    # 테스트 모드
```

### 6. Future Evolution (개선 방향)
- **Slack/Discord 지원:** `NotifierPort` 트레이트 추상화로 다른 채널 추가 용이하게 구조화.
- **양방향 통신:** Telegram Webhook으로 사용자 명령 수신 (퀴즈 모드 등).
- **스케줄링:** cron 또는 systemd timer와 연동하여 매일 특정 시간 자동 발송.
- **사용자 설정:** 발송 단어 개수, 시간대 등 설정 가능하도록 개선.
