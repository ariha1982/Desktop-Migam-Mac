# 05. 완전 신규 개발 기준 24시간 구현 로드맵

## 1. 전제

다음 항목이 하나도 없다고 가정한다.

- 소스 코드와 Git 저장소
- Tauri/Vite/TypeScript 프로젝트
- 창, 트레이와 설정 코드
- 캐릭터 이미지, 애니메이션, 아이콘과 효과음
- 테스트와 CI
- 설치 프로그램과 코드 서명

개발 PC는 Windows 11 64비트이며 인터넷 연결과 프로그램 설치 권한이 있다고 가정한다. 24시간은 휴식을 제외한 실제 작업 예산이다. Visual Studio C++ Build Tools처럼 설치 시간이 긴 필수 도구가 없으면 그 시간만큼 P0 범위를 더 줄여야 한다.

## 2. 24시간 결과물

24시간 종료 시 다음 세로 단면을 실행 가능한 release build로 남긴다.

```text
앱 실행
  → tray와 placeholder 펫 표시
  → Idle/Walk/Drag/Throw
  → 설정에서 방해 규칙 추가 및 opt-in
  → Focus 시작
  → 대상 창 감지와 유예 시간
  → 경고
  → 대상 재검증
  → Kick placeholder 연출과 해당 창 최소화
  → 긴급 중지
```

완성 캐릭터, 카드, 액세서리, 자동 시작과 화려한 UI는 결과물의 조건이 아니다.

## 3. 개발 원칙

- 각 구간 종료 시 앱이 다시 실행되어야 한다.
- 첫 2시간 안에 빈 프로젝트를 dev 모드로 실행한다.
- 첫 4시간 30분 안에 release build를 한 번 시도해 도구 문제를 앞에서 발견한다.
- UI 미관보다 안전한 개입과 취소 경로를 우선한다.
- placeholder는 외부 자산 없이 CSS 도형이나 직접 만든 단순 이미지로 만든다.
- 18시간까지 핵심 세로 단면이 완성되지 않으면 부가 기능을 모두 제거한다.
- 20시간 이후에는 신규 기능을 추가하지 않는다.

## 4. 시간대별 구현 계획

| 시간 | 단계 | 핵심 작업 | 통과 조건 |
|---:|---|---|---|
| 0:00~1:00 | 환경 준비 | 필수 도구 확인, 작업 폴더와 Git 생성, 범위 고정 | 버전과 설치 누락 목록 기록 |
| 1:00~2:00 | 프로젝트 생성 | Tauri 2 + Vite + 순수 TS scaffold, 첫 dev 실행 | 빈 앱이 Windows에서 실행 |
| 2:00~3:30 | 앱 셸 | window label, tray, pet/settings/timer 창, 기본 CSS placeholder | tray에서 각 창 표시/숨김 |
| 3:30~4:30 | 기반 구조 | Rust 계층, AppState, error, command/event 골격, 첫 release build | release build 또는 명확한 환경 이슈 |
| 4:30~6:00 | 설정 | 기본값, validation, JSON load/save, settings 최소 UI | 재실행 후 설정 복원 |
| 6:00~8:00 | 펫 이동 | manifest 최소 규격, Idle/Walk, monitor work area와 clamp | 펫이 화면 안에서 이동 |
| 8:00~9:30 | 상호작용 | click, pointer capture, drag, throw 순수 물리 | 던진 뒤 3초 안에 착지 |
| 9:30~11:00 | 뽀모도로 | 상태 머신, start/pause/resume/skip/stop, timer UI | 축약 시간으로 phase 순환 |
| 11:00~12:30 | 규칙 UI | rule CRUD, opt-in, grace/cooldown validation | 규칙 저장과 재실행 복원 |
| 12:30~14:30 | Windows 감지 | Win32 adapter, snapshot, matcher, 보호 판정 골격 | 테스트 창 감지, 아직 최소화 안 함 |
| 14:30~17:30 | 안전 개입 | grace, warning, fresh 재검증, Kick impact, 최소화 1회, 취소 | Alt+Tab 시 오작동 없음 |
| 17:30~18:30 | 긴급 중지 | hotkey, tray stop, cancellation, resume | 어느 단계에서도 개입 취소 |
| 18:30~20:00 | 자동 테스트 | domain 및 핵심 integration tests, 오류 경로 | 핵심 테스트와 clippy 통과 |
| 20:00~22:00 | 실제 환경 검증 | DPI, 듀얼 모니터, 브라우저, 관리자 창, soak 시작 | 수동 검사 결과 기록 |
| 22:00~23:00 | 수정·안정화 | S0/S1 수정, CPU/메모리와 종료 경로 확인 | 미해결 S0/S1 없음 |
| 23:00~24:00 | 릴리스·인수인계 | 최종 release build, README, 알려진 제한, 진행판 | 다시 빌드 가능한 산출물 |

## 5. 단계별 세부 작업

### 단계 0 — 환경과 빈 저장소: 0:00~1:00

확인 항목:

- Windows 11 64비트
- Node.js LTS와 npm
- Rust stable, Cargo와 MSVC target
- Microsoft C++ Build Tools와 Windows SDK
- WebView2 Runtime
- Git

작업:

1. 프로젝트 폴더를 만든다.
2. Git 저장소를 초기화하고 `.gitignore`를 준비한다.
3. `docs` 폴더에 이 문서 묶음을 복사한다.
4. 도구 버전을 `docs/environment.md`에 기록한다.
5. 설치 실패가 있으면 즉시 차단 요소로 기록한다.

게이트: Tauri scaffold를 실행할 수 있는 상태.

### 단계 1 — 프로젝트 생성: 1:00~2:00

1. Tauri 2 + Vite + 순수 TypeScript 템플릿으로 프로젝트를 만든다.
2. package install과 Rust dependency fetch를 완료한다.
3. 기본 앱을 dev 모드로 실행한다.
4. 기본 테스트/포맷 명령을 package scripts에 등록한다.
5. 생성 직후 기준 상태를 기록한다.

게이트: 기본 앱이 Windows 창으로 실행되고 재실행 가능.

### 단계 2 — 앱 셸: 2:00~3:30

1. 창 label을 `pet`, `settings`, `timer`로 고정한다.
2. pet 창을 256×256, 무테, 투명, 항상 위로 만든다.
3. 외부 자산 없이 CSS 감자 도형을 표시한다.
4. tray 메뉴에 펫 표시/숨김, 타이머, 설정, 긴급 중지, 종료를 만든다.
5. settings/timer는 한 화면의 단순 HTML form으로 시작한다.

게이트: tray에서 세 창을 열고 닫을 수 있으며 pet 창이 전체 화면을 덮지 않음.

### 단계 3 — 기반 구조와 조기 release 확인: 3:30~4:30

1. `domain`, `application`, `infrastructure`, `presentation` 폴더를 필요한 만큼만 만든다.
2. AppState와 공통 오류 타입을 만든다.
3. `get_bootstrap_state` command와 event emitter 골격을 연결한다.
4. 창별 capability를 최소 권한으로 분리한다.
5. release build를 한 번 실행한다.

게이트: release build 성공. 실패하면 원인이 앱 로직인지 개발환경인지 분리되어 있음.

### 단계 4 — 설정: 4:30~6:00

1. 안전 기본값을 Rust `Default`로 정의한다.
2. app data 경로의 `settings.json`을 load한다.
3. 범위를 validation하고 누락 field에 기본값을 넣는다.
4. tmp write 후 교체하는 원자 저장을 구현한다.
5. 손상 JSON은 별도 파일로 보관한다.
6. 최소 settings UI와 `save_settings`를 연결한다.

게이트: scale 또는 timer 값을 바꾸고 앱을 재실행해도 복원됨.

### 단계 5 — 펫 이동: 6:00~8:00

1. CSS placeholder 또는 한 프레임 이미지로 idle/walk를 표현한다.
2. PetBehavior와 우선순위를 만든다.
3. Windows monitor work area adapter를 구현한다.
4. work area 안에서 임의 목적지를 선택한다.
5. Idle/Walk를 30fps 이하로 갱신한다.
6. 최소 24px가 화면 안에 남도록 clamp한다.

게이트: 10분 이동 중 화면 밖 영구 유실이 없음.

### 단계 6 — 드래그와 던지기: 8:00~9:30

1. pointer down에서 capture하고 Dragged로 전환한다.
2. 최근 5~8개 좌표와 시간을 저장한다.
3. pointer up의 최근 80~120ms로 속도를 계산한다.
4. 임계값 이상이면 Thrown으로 전환한다.
5. gravity, friction, bounce, speed clamp와 3초 제한을 순수 함수로 만든다.
6. 물리 단위 테스트를 추가한다.

게이트: 느리게 놓기와 던지기가 구분되고 화면 경계를 벗어나지 않음.

### 단계 7 — 뽀모도로: 9:30~11:00

1. 순수 상태 머신부터 테스트한다.
2. Focus, ShortBreak, LongBreak, Paused를 구현한다.
3. start/pause/resume/skip/stop command를 만든다.
4. timer 창에 상태, 남은 시간과 최소 controls를 표시한다.
5. Focus Running 여부를 단일 진실 공급원으로 사용한다.

게이트: 테스트용 10초 설정으로 phase 전환과 pause/resume 확인.

### 단계 8 — 규칙 설정: 11:00~12:30

1. rule name, process name, title contains, enabled를 편집한다.
2. process/title 중 하나 이상을 요구한다.
3. grace 5초~10분, cooldown 30초~60분을 검증한다.
4. focus guard와 minimize opt-in을 명확한 checkbox로 둔다.
5. 설정에 사이트 감지 제한을 표시한다.

게이트: 규칙 생성/수정/삭제/비활성화가 저장됨.

### 단계 9 — Windows 감지: 12:30~14:30

1. `ForegroundWindowSource` trait과 fake를 만든다.
2. `GetForegroundWindow`, PID, process filename, title을 읽는 adapter를 만든다.
3. process/title 규칙을 대소문자 무시로 비교한다.
4. 앱 자체 창, 읽을 수 없는 창과 전체 화면 창을 우선 보호한다.
5. Focus Running 중에만 1초 polling한다.
6. detection 상태까지만 확인하고 아직 최소화하지 않는다.

게이트: 직접 연 테스트용 메모장/브라우저를 정확히 감지하며 실제 제목을 로그하지 않음.

### 단계 10 — 안전한 개입: 14:30~17:30

1. 동일 hwnd와 rule이 grace 동안 유지되는지 확인한다.
2. intervention ID와 cancellation token을 만든다.
3. FocusWarning과 취소 기회를 표시한다.
4. 실행 직전에 fresh snapshot을 다시 읽는다.
5. hwnd, rule 일치, Focus, opt-in과 보호 여부를 모두 재검사한다.
6. Kick placeholder impact 시점에 최소화를 한 번 호출한다.
7. 성공/실패와 관계없이 cooldown을 적용한다.
8. 전경 변경, Focus 종료와 Dragged에서 취소한다.

게이트: 경고 중 Alt+Tab하면 새 창과 이전 창 모두 최소화되지 않음.

### 단계 11 — 긴급 중지: 17:30~18:30

1. `Ctrl+Shift+F12`를 등록한다.
2. 같은 기능을 tray에도 둔다.
3. atomic emergency flag와 전체 cancellation을 연결한다.
4. pet/말풍선을 숨기고 timer를 pause한다.
5. tray와 `펫 다시 시작`만 남긴다.
6. 중복 호출에도 안전하게 만든다.

게이트: 감지, warning, Kick 어느 시점에서도 최소화를 막음.

### 단계 12 — 테스트와 안정화: 18:30~23:00

- Pomodoro, rules, behavior, physics, settings 단위 테스트
- warning→재검증→minimize fake 통합 테스트
- foreground 변경, protected window와 emergency 취소 테스트
- typecheck, lint, rustfmt, clippy
- 100/150/200% DPI
- 가능한 경우 듀얼 모니터
- Chrome/Edge 제목 규칙
- 관리자 창 안전 실패
- 30분 soak와 작업 관리자 CPU/RSS 관찰

게이트: S0/S1 버그 없음. 잘못된 창 최소화가 한 번이라도 발생하면 개입 기능을 release에서 비활성화한다.

### 단계 13 — 릴리스: 23:00~24:00

1. 최종 release build를 만든다.
2. 새로 만든 artifact를 직접 실행한다.
3. README에 실행·빌드·설정 위치·긴급 중지·제한을 적는다.
4. 수동 검사 결과와 미해결 S2/S3를 기록한다.
5. `13-progress-board.md`에 마지막 인수인계를 쓴다.

게이트: 다른 작업자가 문서만 보고 build와 실행을 재현할 수 있음.

## 6. 강제 중단선

| 시점 | 미달 상태 | 즉시 조치 |
|---:|---|---|
| 2시간 | 기본 Tauri 앱 미실행 | 기능 개발 중단, toolchain 문제 해결 또는 범위 재협의 |
| 4.5시간 | release build 불가 | 원인 고정 전 P1/P2 금지 |
| 8시간 | 안정적인 Walk 불가 | animation과 multi-monitor 고도화 제거, 주 모니터 clamp만 유지 |
| 12.5시간 | 설정/rule UI 미완성 | UI를 원시 form으로 단순화 |
| 14.5시간 | 실제 창 snapshot 불가 | 최소화 구현 금지, fake 기반 데모와 blocker 보고 |
| 18시간 | 안전 개입 세로 단면 미완성 | 모든 P1/P2 제거, 오직 안전 경로와 테스트에 집중 |
| 20시간 | release build 또는 핵심 테스트 실패 | 신규 기능 금지, 수정만 수행 |

## 7. 신규 개발에서 의도적으로 포기하는 항목

24시간 기본 일정에는 다음을 넣지 않는다.

- 완성 캐릭터 애니메이션
- 카드 운반
- 액세서리 UI
- 커서 추적
- 춤과 다중 대사
- 클릭 이스터에그
- 비활동 경고
- 자동 시작과 Windows 알림
- 알림음
- 픽셀 단위 hit-test 완성
- CI/CD, 코드 서명과 자동 업데이트

20시간 이전에 P0 release build와 테스트가 모두 통과한 경우에만 대사 또는 이스터에그 중 하나를 추가한다.

## 8. 24시간 성공 판정

- 빈 폴더에서 생성한 프로젝트가 release build된다.
- 외부 캐릭터 자산 없이 placeholder로 핵심 행동을 시연한다.
- 집중 세션과 방해 규칙이 동작한다.
- 경고 후 동일 대상만 안전하게 최소화한다.
- 모든 취소 조건과 긴급 중지가 동작한다.
- 설정이 로컬에 안전하게 저장된다.
- 핵심 자동 테스트와 Windows 수동 검사 결과가 남아 있다.
- 실제 창 제목, URL, 키 입력과 사용 이력이 저장되지 않는다.

P1/P2가 없어도 위 조건을 만족하면 24시간 MVP 성공이다.
