# 06. AI 코딩 작업 지시서

이 문서의 각 작업 블록을 AI 코딩 도구에 하나씩 전달한다. 이전 작업의 테스트가 통과한 뒤 다음 작업을 준다.

## 공통 지시문

```text
빈 작업 폴더에서 Windows 11용 Tauri 2 + Rust + 순수 TypeScript 데스크톱 펫을 신규 개발한다.
코드, 프로젝트와 캐릭터 자산이 없으며 `docs` 폴더의 문서만 작업 기준으로 존재한다.

작업 전:
1. 현재 작업 폴더와 직전 단계 결과를 확인한다.
2. 직전 단계에서 생성된 코드를 보존한다.
3. 이번 작업에 필요한 파일만 생성하거나 수정한다.

작업 중:
- 창 최소화는 기본 off로 유지한다.
- 대상이 불확실하거나 보호 대상이면 조작하지 않는다.
- 실제 창 제목, URL, 키 입력과 사용 이력을 저장하거나 로그하지 않는다.
- unsafe Win32 코드는 infrastructure/windows에 제한한다.

작업 후:
1. 포맷, 관련 테스트와 빌드를 실행한다.
2. 변경 파일, 동작, 테스트 결과와 남은 위험을 보고한다.
3. progress board에 상태를 반영한다.
```

## 작업 0 — 환경 확인과 신규 프로젝트 생성

```text
공통 지시문을 따른다.
현재 폴더에 애플리케이션 코드가 없다는 전제로 시작해라.
Windows, Node.js, npm, Rust/Cargo, MSVC build tools, Windows SDK, WebView2와 Git 사용 가능 여부를 확인해라.
Tauri 2 + Vite + 순수 TypeScript 프로젝트를 새로 만들고 기본 앱을 dev 모드로 실행해라.
기본 typecheck, lint, test와 Tauri build 명령을 package scripts에 정리하고 생성 직후 기준 상태를 보고해라.
외부 캐릭터 자산은 받지 말고 이후 CSS placeholder를 사용할 준비만 해라.
```

게이트: 빈 폴더에서 만든 기본 Tauri 앱이 Windows에서 실행된다.

## 작업 1 — 앱 셸

```text
pet, settings, timer 창 label과 트레이 메뉴를 구현해라.
pet 창은 무테·투명·항상 위이며 전체 화면 크기가 아니어야 한다.
Ctrl+Shift+F12 전역 단축키 골격을 추가하고 등록 실패를 사용자에게 표시해라.
캐릭터 자산이 없으므로 저작권 문제가 없는 CSS 감자 도형을 placeholder로 표시해라.
아직 이동, 타이머와 창 최소화는 구현하지 마라.
```

게이트: tray에서 펫/설정/타이머를 열고 펫을 숨길 수 있다.

## 작업 2 — 설정 기반

```text
AppSettings, 기본 안전값, validation, schemaVersion과 JSON 원자 저장을 구현해라.
손상된 설정은 timestamp가 포함된 별도 파일로 보관하고 기본값으로 시작해라.
get_bootstrap_state와 save_settings command, 최소 settings UI를 연결해라.
실제 창 정보는 어떤 설정이나 로그에도 포함하지 마라.
```

게이트: 설정 변경이 재실행 뒤 복원되고 손상 JSON 테스트가 통과한다.

## 작업 3 — 펫 이동

```text
placeholder character manifest와 Idle/Walk 상태를 구현해라.
Windows monitor work area를 사용해 목적지를 선택하고 펫의 최소 24px가 화면 안에 남도록 해라.
Rust 도메인과 Windows adapter를 분리하고 모니터 정보는 테스트 가능하게 추상화해라.
```

게이트: 단일/다중 모니터에서 펫이 작업표시줄 영역을 피해 이동한다.

## 작업 4 — 클릭·드래그·던지기

```text
펫 클릭, pointer capture 기반 드래그와 놓기 속도 계산을 구현해라.
빠르게 놓으면 최대 속력, 중력, 감속, 경계 반사와 최대 3초 제한을 적용한 Thrown 상태로 전환해라.
Dragged가 모든 자동 행동보다 우선하고 Dragged/Thrown 중에는 창 개입이 불가능해야 한다.
physics 계산은 순수 함수와 단위 테스트로 작성해라.
```

게이트: 느린 놓기/던지기가 구분되고 펫이 3초 안에 멈춘다.

## 작업 5 — 뽀모도로

```text
Stopped, Focus, ShortBreak, LongBreak와 Paused를 지원하는 순수 상태 머신을 구현해라.
start, pause, resume, skip, stop command와 최소 timer UI를 연결해라.
네 번째 Focus 완료 뒤 LongBreak로 가고 Focus가 아니면 감지 서비스가 동작하지 않게 해라.
```

게이트: 축약 시간으로 전체 phase와 pause/resume 테스트가 통과한다.

## 작업 6 — 전경 창과 규칙

```text
ForegroundWindowSource trait, fake와 Windows 구현을 추가해라.
프로세스 파일명과 창 제목 포함 문자열 규칙을 대소문자 무시로 비교해라.
Focus 중에만 1초 polling하고 snapshot의 실제 제목과 경로는 저장하거나 로그하지 마라.
아직 창을 최소화하지 말고 detection 결과까지만 구현해라.
```

게이트: fake와 실제 테스트 창에서 일치/불일치가 확인된다.

## 작업 7 — 안전한 개입

```text
grace, cooldown과 InterventionService를 구현해라.
펫 경고 후 실행 직전에 새 snapshot을 얻어 hwnd, 규칙 일치와 보호 여부를 다시 검사해라.
모든 조건이 유지될 때만 Kick 충돌 프레임에서 ShowWindow(SW_MINIMIZE)를 1회 호출해라.
대상 변경, Focus 종료, Dragged, 설정 off와 긴급 중지에서 즉시 취소해라.
앱 자체 창, 알 수 없는/높은 권한 창, 시스템 핵심 창과 전체 화면 창은 보호해라.
```

게이트: 경고 중 Alt+Tab하면 새 창을 최소화하지 않는다.

## 작업 8 — 긴급 중지

```text
Ctrl+Shift+F12와 tray 메뉴의 emergency stop을 전체 시스템에 연결해라.
펫/카드/말풍선을 숨기고, 타이머를 일시정지하며, monitor와 pending intervention을 취소해라.
tray와 펫 다시 시작 메뉴는 유지해라.
중복 호출해도 안전한 idempotent 동작으로 만들고 통합 테스트를 추가해라.
```

게이트: 개입 어느 단계에서 호출해도 최소화가 일어나지 않는다.

## 작업 9 — P1 선택

```text
P0 테스트가 모두 통과하는지 먼저 확인해라.
남은 시간 안에 다음 중 최대 두 개만 구현해라: 말풍선/대사, 춤, 7회 클릭 이스터에그, 비활동 경고.
기존 핵심 흐름을 깨뜨리는 변경은 하지 마라.
```

## 작업 10 — 최종 검증

```text
새 기능을 추가하지 마라.
전체 포맷, lint, typecheck, Rust test, UI test, clippy와 release build를 실행해라.
Windows 수동 검사표를 따라 검증 가능한 항목을 점검하고, 자동 확인할 수 없는 항목은 명시해라.
README에 실행 방법, 설정 위치, 긴급 중지, 개인정보 원칙과 알려진 제한을 정리해라.
```
