# 미친감자 펫 개발 문서

Windows 11용 Tauri 2 데스크톱 펫 MVP를 **코드, 프로젝트, 디자인 시스템과 캐릭터 자산이 전혀 없는 상태에서 24시간 안에 구현**하기 위한 실행 문서 모음이다. 사람과 AI 코딩 도구 모두 이 폴더를 작업 기준으로 사용할 수 있다.

## 가장 먼저 읽을 문서

1. [00-project-brief.md](./00-project-brief.md) — 제품 목표와 핵심 사용자 흐름
2. [01-scope-and-priorities.md](./01-scope-and-priorities.md) — 24시간 범위와 포기 기준
3. [05-implementation-roadmap.md](./05-implementation-roadmap.md) — 실제 작업 순서
4. [06-ai-agent-work-orders.md](./06-ai-agent-work-orders.md) — AI에게 단계별로 줄 작업 지시
5. [13-progress-board.md](./13-progress-board.md) — 진행 상태 기록
6. [17-session-handoff.md](./17-session-handoff.md) — 다음 세션이 이어서 작업하기 위한 최신 인수인계
7. [18-todo-pomodoro-spec.md](./18-todo-pomodoro-spec.md) — 다음 추가 기능인 투두리스트·뽀모도로 연동 확정 명세

## 문서 지도

| 문서 | 용도 |
|---|---|
| `00-project-brief.md` | 제품과 성공 기준 이해 |
| `01-scope-and-priorities.md` | P0/P1/P2 범위 통제 |
| `02-technology-stack.md` | 사용할 기술과 금지할 선택 |
| `03-architecture.md` | 모듈 경계와 실행 흐름 |
| `04-file-structure.md` | 파일별 책임과 생성 순서 |
| `05-implementation-roadmap.md` | 24시간 시간표와 단계별 게이트 |
| `06-ai-agent-work-orders.md` | 복사해서 사용할 AI 작업 프롬프트 |
| `07-contracts-and-data.md` | commands, events, 설정 데이터 계약 |
| `08-pet-behavior-spec.md` | 행동 우선순위, 이동과 물리 규칙 |
| `09-windows-safety.md` | Windows API와 창 보호 정책 |
| `10-character-pack-spec.md` | 캐릭터 자산과 manifest 규격 |
| `11-test-plan.md` | 단위·통합·수동 테스트 계획 |
| `12-release-checklist.md` | 빌드, 배포 후보와 인수인계 |
| `13-progress-board.md` | 작업 현황과 이슈 기록 |
| `14-decision-log.md` | 설계 결정 기록 |
| `15-troubleshooting.md` | 자주 막히는 문제와 대응 |
| `16-definition-of-done.md` | 최종 완료 판정표 |
| `17-session-handoff.md` | 최신 구현 상태, 검증 결과와 정확한 다음 작업 |
| `18-todo-pomodoro-spec.md` | 투두 CRUD, 집중 연결, 전체 완료 축하와 구현 게이트 |

## AI 작업 규칙

- 한 번에 전체를 구현시키지 말고 `06-ai-agent-work-orders.md`의 작업 0부터 순서대로 수행한다.
- 각 작업이 끝날 때 빌드와 관련 테스트를 통과시킨다.
- 신규 Tauri 프로젝트 생성부터 시작한다. 예상과 달리 코드가 발견되면 덮어쓰지 말고 먼저 사용자에게 보고한다.
- 핵심 경로가 완성되기 전에는 카드, 액세서리, 시각 효과를 추가하지 않는다.
- 창 최소화는 기본 꺼짐이고, 불확실한 창은 절대 조작하지 않는다.
- 실제 창 제목, 키 입력, URL과 사용 앱 기록은 저장하거나 로그에 남기지 않는다.
- 작업 결과와 남은 문제를 `13-progress-board.md`에 기록한다.
- 모든 세션은 루트 `AGENTS.md`와 `17-session-handoff.md`를 먼저 읽고, 종료 전에 두 진행 문서를 갱신한다.

## 핵심 개발 경로

```text
프로젝트 실행
  → 트레이와 투명 펫 창
  → Idle/Walk
  → Click/Drag/Thrown
  → 뽀모도로
  → 방해 창 감지
  → 경고와 대상 재검증
  → 안전한 최소화
  → 긴급 중지
  → 테스트와 릴리스 빌드
```

## 문서 우선순위

문서끼리 내용이 충돌하면 다음 순서를 따른다.

1. 사용자의 최신 요청
2. `01-scope-and-priorities.md`의 안전·범위 규칙
3. `07-contracts-and-data.md`의 데이터 계약
4. `03-architecture.md`의 모듈 경계
5. 나머지 참고 문서
