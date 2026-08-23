# Windows 데스크톱 펫 MVP — 24시간 개발 실행 계획

문서 버전: 1.0
작성일: 2026-08-23
대상 플랫폼: Windows 11 64비트
개발 기준: Tauri 2 + Rust 기반 기존 앱 확장
원본 설계: `2026-08-23-windows-desktop-pet-mvp-design.md`

## 1. 문서 목적

이 문서는 원본 제품·기술 명세를 실제 개발 작업으로 옮기기 위한 24시간짜리 구현 계획이다. 개발자가 별도 구조 설계 없이 바로 프로젝트를 만들 수 있도록 다음을 고정한다.

- 24시간 안에 달성할 현실적인 범위와 제외 범위
- 프런트엔드, Rust, Windows API 기술 스택
- 디렉터리와 파일 단위 책임
- 핵심 데이터 구조, 명령, 이벤트와 실행 흐름
- 시간대별 구현 순서, 테스트 항목과 완료 기준
- 자산이 준비되지 않은 상태의 대체 전략

24시간은 1명의 개발자가 이미 Rust, TypeScript와 Tauri의 기본 사용법을 알고 있으며, 개발 PC에서 기존 앱이 빌드되는 상태를 전제로 한다. 기존 코드가 없거나 빌드가 깨진 상태라면 첫 2시간을 프로젝트 복구에 사용하고 후순위 기능을 줄인다.

## 2. 24시간 목표

### 2.1 반드시 완성할 세로 단면

다음 사용자 흐름이 처음부터 끝까지 동작하면 24시간 MVP 성공으로 본다.

1. 앱을 실행하면 트레이 아이콘과 투명 펫 창이 나타난다.
2. 펫이 현재 모니터의 작업 영역에서 Idle과 Walk를 반복한다.
3. 펫을 클릭하고 드래그할 수 있으며 빠르게 놓으면 관성 이동 후 멈춘다.
4. 타이머 창에서 집중 세션을 시작, 일시정지, 재개, 중지할 수 있다.
5. 집중 중 등록된 프로세스명 또는 창 제목이 유예 시간 이상 전경에 있으면 경고한다.
6. 대상 창이 여전히 동일할 때만 발차기 연출과 함께 그 창을 최소화한다.
7. `Ctrl+Shift+F12`를 누르면 오버레이와 개입이 즉시 중지되고 타이머가 일시정지된다.
8. 설정과 방해 규칙은 앱 재실행 후 복원된다.

### 2.2 24시간 내 기능 우선순위

| 우선순위 | 기능 | 판단 기준 |
|---|---|---|
| P0 | 트레이, 펫 오버레이, 걷기, 드래그, 던지기 | 제품 정체성과 기본 조작 |
| P0 | 뽀모도로 기본 상태 전환 | 집중 기능의 기준 상태 |
| P0 | 전경 창 감지, 규칙 대조, 경고 후 최소화 | 핵심 차별 기능 |
| P0 | 긴급 중지, 기본 보호 규칙, 로컬 설정 저장 | 안전성 |
| P1 | 말풍선, 클릭 반응, 춤, 7회 클릭 이스터에그 | 펫의 개성 |
| P1 | 비활동 경고와 순수 발차기 연출 | 집중 보조 기능 |
| P2 | WinDbg/Hyper-V 테마 카드 | 일정이 남을 때 구현 |
| P2 | 액세서리 레이어 | 구조만 먼저 만들고 UI는 단순화 가능 |
| P2 | Windows 알림, 자동 시작, 알림음 | 핵심 경로 완료 후 구현 |

### 2.3 24시간 데모에서 허용하는 단순화

- 완성 캐릭터 대신 256×256 투명 PNG/WebP 자리표시자 프레임을 사용한다.
- 설정 UI는 한 화면으로 만들고 고급 스타일링과 애니메이션을 생략한다.
- 사이트 감지는 브라우저 주소가 아닌 창 제목 포함 문자열만 지원한다.
- 카드 이미지는 상표 로고 대신 자체 제작한 `DBG`, `VM` 텍스트 카드로 대체한다.
- 창 최소화 보호는 보수적으로 적용한다. 확실하지 않은 창은 건드리지 않는다.
- 자동 업데이트, 코드 서명, 설치 프로그램 고도화와 24시간 내구 테스트 완료는 후속 작업으로 둔다.
- 카드용 별도 네이티브 창이 일정 위험이 되면 펫 창 내부의 임시 레이어로 구현하되 인터페이스는 분리한다.

> 현재 구현 결정(2026-08-23): 이 문서의 React/TSX 예시는 순수 TypeScript + DOM API와 `.ts` 모듈로 대체한다. 세부 기준은 `02-technology-stack.md`와 ADR-007을 우선한다.

## 3. 기술 스택

의존성의 정확한 패치 버전은 작성 시점의 최신 호환 버전을 설치하고 lockfile로 고정한다. 하루짜리 구현 중에는 패키지 일괄 업그레이드를 하지 않는다.

### 3.1 애플리케이션

| 영역 | 선택 | 사용 목적 |
|---|---|---|
| 데스크톱 셸 | Tauri 2 | 다중 창, 트레이, Rust 명령, 이벤트, 번들링 |
| 시스템 로직 | Rust stable | 타이머, 행동 상태, 규칙 판정, Windows API 호출 |
| 비동기 런타임 | Tokio | 주기 작업, 취소 가능한 모니터 루프, 타이머 |
| 프런트엔드 | React + TypeScript | 펫 렌더러, 타이머와 설정 UI |
| 빌드 | Vite | 빠른 개발 서버와 Tauri 연동 |
| 상태 관리 | React Context + `useReducer` | 화면 수가 적어 별도 전역 상태 라이브러리 불필요 |
| 스타일 | CSS Modules 또는 단일 기능별 CSS | 투명 창과 간단한 설정 UI에 충분 |
| 직렬화 | Serde + serde_json | 설정, 캐릭터 manifest, 명령 payload |
| 오류 | thiserror + anyhow | 도메인 오류와 애플리케이션 경계 오류 구분 |
| 로깅 | tracing + tracing-subscriber | 창 제목을 제외한 로컬 진단 로그 |
| 식별자 | uuid | 방해 규칙 ID |
| 시간 | std::time::Instant + chrono | 경과 시간은 단조 시계, 저장 시각은 로컬 날짜 |

### 3.2 Tauri 기능

| 기능 | 구성 |
|---|---|
| 트레이 | Tauri 내장 tray API |
| 전역 단축키 | Tauri global-shortcut 플러그인 |
| 알림 | Tauri notification 플러그인, P2 |
| 자동 시작 | Tauri autostart 플러그인, P2 |
| 파일 경로 | Tauri path API로 앱 데이터 디렉터리 결정 |
| 설정 저장 | 직접 구현한 JSON 원자 저장; 범용 store 플러그인은 사용하지 않음 |

설정은 임시 파일에 먼저 쓴 후 rename하여 원본을 교체한다. 이렇게 하면 저장 도중 종료되어도 기존 설정을 잃을 가능성을 줄일 수 있다.

### 3.3 Windows 연동

Rust의 `windows` 크레이트를 사용하고 필요한 Win32 feature만 활성화한다.

| 기능 | Windows API |
|---|---|
| 현재 전경 창 | `GetForegroundWindow` |
| 프로세스 ID | `GetWindowThreadProcessId` |
| 실행 파일명 | 프로세스 핸들 + `QueryFullProcessImageNameW` |
| 창 제목 | `GetWindowTextLengthW`, `GetWindowTextW` |
| 창 최소화 | `ShowWindow(hwnd, SW_MINIMIZE)` |
| 마지막 사용자 입력 | `GetLastInputInfo` |
| 창 상태/가시성 | `IsWindow`, `IsWindowVisible`, `IsIconic` |
| 창 클래스 | `GetClassNameW` |
| 모니터와 작업 영역 | `EnumDisplayMonitors`, `GetMonitorInfoW` |
| DPI | Tauri가 제공하는 logical/physical 좌표를 우선 사용하고 필요 시 monitor scale factor 사용 |

안전 원칙은 다음과 같다.

- `HWND`는 저장하지 않고 경고 시작과 실행 직전에 다시 검증한다.
- 관리자 권한 앱 등 열 수 없는 프로세스는 감지 실패로 처리하고 최소화하지 않는다.
- 실제 창 제목과 사용 앱 이력은 파일이나 로그에 기록하지 않는다.
- 최소화 기능은 기본 꺼짐이며 규칙이 존재하고 사용자가 명시적으로 켠 경우만 동작한다.

### 3.4 테스트와 품질 도구

| 도구 | 목적 |
|---|---|
| Rust 내장 테스트 | 도메인 상태, 물리, 규칙, 타이머 테스트 |
| Vitest + React Testing Library | UI reducer와 주요 컴포넌트 테스트 |
| ESLint + Prettier | TypeScript 정적 검사와 포맷 |
| rustfmt + Clippy | Rust 포맷과 정적 검사 |
| 수동 Windows 체크리스트 | 투명 창, DPI, 다중 모니터, 창 최소화 검증 |

## 4. 프로젝트 파일 구조

기존 앱이 있다면 아래 구조로 점진적으로 이동한다. 24시간 동안 전체 리팩터링을 먼저 하지 말고, 새 기능부터 경계를 지켜 추가한다.

```text
desktop-pet/
├─ package.json
├─ package-lock.json                 # npm 사용 기준; 기존 패키지 매니저 유지 가능
├─ vite.config.ts
├─ tsconfig.json
├─ index.html
├─ README.md
├─ docs/
│  ├─ architecture.md                # 실행 흐름과 설계 결정
│  ├─ manual-test-checklist.md        # Windows 수동 검사 결과
│  └─ privacy.md                      # 수집하지 않는 데이터 명시
├─ public/
│  └─ icons/
│     └─ tray.ico
├─ src/
│  ├─ main.tsx                        # label별 React 진입점 선택
│  ├─ AppRouter.tsx                   # pet/settings/timer/card 창 라우팅
│  ├─ shared/
│  │  ├─ api/
│  │  │  ├─ commands.ts              # invoke 래퍼
│  │  │  ├─ events.ts                # listen/unlisten 래퍼
│  │  │  └─ contracts.ts             # Rust DTO와 맞춘 TS 타입
│  │  ├─ hooks/
│  │  │  ├─ useAppEvent.ts
│  │  │  └─ useWindowLabel.ts
│  │  └─ styles/
│  │     ├─ reset.css
│  │     └─ tokens.css
│  ├─ pet/
│  │  ├─ PetApp.tsx                  # 투명 펫 창 루트
│  │  ├─ PetSprite.tsx               # 프레임, 방향, 액세서리 합성
│  │  ├─ SpeechBubble.tsx
│  │  ├─ CardOverlay.tsx             # 카드 창 분리 전 대체 가능
│  │  ├─ usePetInteraction.ts        # 클릭/드래그/놓기 속도 계산
│  │  ├─ spriteResolver.ts           # 상태 → 프레임 URL
│  │  └─ pet.module.css
│  ├─ timer/
│  │  ├─ TimerApp.tsx
│  │  ├─ TimerControls.tsx
│  │  └─ timer.module.css
│  ├─ settings/
│  │  ├─ SettingsApp.tsx
│  │  ├─ PetSettingsSection.tsx
│  │  ├─ PomodoroSettingsSection.tsx
│  │  ├─ FocusGuardSection.tsx
│  │  ├─ DistractionRuleEditor.tsx
│  │  ├─ SafetySection.tsx
│  │  ├─ settingsReducer.ts
│  │  └─ settings.module.css
│  └─ card/
│     ├─ CardApp.tsx
│     └─ card.module.css
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ Cargo.lock
│  ├─ tauri.conf.json
│  ├─ capabilities/
│  │  ├─ pet.json                    # 펫 창의 최소 권한
│  │  ├─ settings.json               # 설정 창 invoke 권한
│  │  └─ timer.json
│  ├─ icons/
│  ├─ resources/
│  │  ├─ characters/
│  │  │  └─ placeholder/
│  │  │     ├─ manifest.json
│  │  │     ├─ idle/
│  │  │     ├─ walk/
│  │  │     ├─ dragged/
│  │  │     ├─ thrown/
│  │  │     ├─ kick/
│  │  │     ├─ speak/
│  │  │     ├─ dance/
│  │  │     └─ accessories/
│  │  └─ cards/
│  │     ├─ debugger.webp
│  │     └─ virtual-machine.webp
│  └─ src/
│     ├─ main.rs                     # run() 호출만 담당
│     ├─ lib.rs                      # Tauri Builder와 전체 조립
│     ├─ app_state.rs                # 공유 서비스와 cancellation 상태
│     ├─ error.rs                    # 외부 전달용 AppError
│     ├─ domain/
│     │  ├─ mod.rs
│     │  ├─ behavior.rs              # 행동 상태, 우선순위와 전이 규칙
│     │  ├─ physics.rs               # 던지기 적분, 감속, 경계 반사
│     │  ├─ pomodoro.rs              # 순수 타이머 상태 머신
│     │  ├─ distraction.rs           # 규칙과 일치 판정
│     │  ├─ settings.rs              # 설정 모델, 기본값과 검증
│     │  ├─ character.rs             # manifest와 fallback 규칙
│     │  └─ events.rs                # AppEvent 도메인 이벤트
│     ├─ application/
│     │  ├─ mod.rs
│     │  ├─ behavior_scheduler.rs    # 우선순위, 랜덤 행동, 취소
│     │  ├─ motion_service.rs        # 목적지 선택과 60/30fps 위치 업데이트
│     │  ├─ pomodoro_service.rs      # tick, 저장, phase 완료 이벤트
│     │  ├─ distraction_monitor.rs   # 1초 polling, 유예/쿨다운
│     │  ├─ intervention_service.rs  # 이동→경고→재검증→최소화
│     │  ├─ idle_monitor.rs          # 비활동 경고 횟수 제한
│     │  ├─ interaction_service.rs   # click/drag/throw 이벤트 처리
│     │  └─ settings_service.rs      # 검증, migration, 원자 저장
│     ├─ infrastructure/
│     │  ├─ mod.rs
│     │  ├─ clock.rs                 # 테스트 가능한 Clock trait
│     │  ├─ filesystem.rs            # 설정 저장 구현
│     │  ├─ random.rs                # 테스트 가능한 RandomSource trait
│     │  └─ windows/
│     │     ├─ mod.rs
│     │     ├─ foreground_window.rs
│     │     ├─ window_minimizer.rs
│     │     ├─ protected_window.rs
│     │     ├─ idle_time.rs
│     │     └─ monitor_layout.rs
│     ├─ presentation/
│     │  ├─ mod.rs
│     │  ├─ commands.rs              # Tauri command 등록 집합
│     │  ├─ pet_commands.rs
│     │  ├─ timer_commands.rs
│     │  ├─ settings_commands.rs
│     │  ├─ tray.rs
│     │  ├─ hotkey.rs
│     │  ├─ windows.rs               # 창 생성, 표시, 숨김과 이동
│     │  └─ emitter.rs               # 프런트 이벤트 전송 어댑터
│     └─ tests/
│        ├─ distraction_flow.rs
│        ├─ emergency_stop.rs
│        └─ settings_recovery.rs
└─ scripts/
   ├─ validate-character-pack.ps1
   └─ smoke-test.ps1
```

### 4.1 계층 의존 규칙

```text
presentation → application → domain
                         ↘ infrastructure (trait 구현)
```

- `domain`은 Tauri, Win32, React를 알지 못한다.
- `application`은 사용 사례의 순서를 조정하며 직접 Win32 함수를 호출하지 않는다.
- `infrastructure/windows`만 `unsafe` Win32 호출을 가진다.
- `presentation`은 Tauri 명령과 이벤트를 도메인 타입으로 변환한다.
- 프런트엔드는 판단 로직을 갖지 않고 표시와 포인터 입력 수집에 집중한다.

## 5. 핵심 데이터 모델

### 5.1 설정

```rust
struct AppSettings {
    schema_version: u32,
    pet: PetSettings,
    pomodoro: PomodoroSettings,
    focus_guard: FocusGuardSettings,
    emergency_hotkey: String,
}

struct DistractionRule {
    id: Uuid,
    name: String,
    enabled: bool,
    process_name: Option<String>,
    title_contains: Option<String>,
    grace_seconds: u32,
    cooldown_seconds: u32,
}
```

검증 규칙:

- 프로세스명과 제목 문자열 중 하나 이상 필요
- 앞뒤 공백 제거, 빈 문자열은 `None` 처리
- 프로세스명 비교는 대소문자를 무시하고 파일명만 비교
- 제목 포함 비교는 기본적으로 Unicode 소문자 변환 후 비교
- 시간과 크기 값은 제품 명세의 허용 범위로 제한
- 알 수 없는 JSON 필드는 무시하고 누락 필드는 기본값 적용

### 5.2 펫 상태

```rust
enum PetBehavior {
    Idle,
    Walk { target: Point },
    ChaseCursor { until: Instant },
    Speak { line_id: String },
    Dance,
    CarryCard { card_id: String },
    FocusWarning { intervention_id: Uuid },
    Kick { intervention_id: Uuid },
    Dragged,
    Thrown { velocity: Vector },
}
```

모든 행동에는 `priority()`, `is_interruptible()`과 종료 조건을 둔다. Dragged는 최우선이며 Dragged/Thrown 동안 모든 창 개입을 취소한다.

### 5.3 전경 창 스냅샷

```rust
struct WindowSnapshot {
    hwnd: isize,              // 메모리에서만 사용
    process_id: u32,
    process_name: Option<String>,
    title: Option<String>,
    class_name: Option<String>,
    is_visible: bool,
    is_minimized: bool,
}
```

스냅샷은 직렬화하거나 로그에 남기지 않는다. 진단 로그에는 `rule_id`, 성공 여부와 오류 범주만 기록한다.

### 5.4 뽀모도로

```rust
enum PomodoroPhase { Stopped, Focus, ShortBreak, LongBreak }
enum RunState { Running, Paused }

struct PomodoroState {
    phase: PomodoroPhase,
    run_state: RunState,
    phase_duration: Duration,
    elapsed_before_pause: Duration,
    started_at: Option<Instant>,
    completed_focus_count: u32,
}
```

화면 표시용 남은 시간은 250ms~1초마다 계산하되 저장 기준은 `Instant` 경과값이다. 재실행 복구용으로는 현재 상태와 종료 예정 wall-clock을 별도로 저장하고 자동 재개하지 않는다.

## 6. 프런트엔드–Rust 계약

### 6.1 Tauri commands

| 명령 | 입력 | 출력 | 호출 창 |
|---|---|---|---|
| `get_bootstrap_state` | 없음 | 설정, 타이머, 펫 상태 | 전체 |
| `save_settings` | `AppSettingsDto` | 정규화된 설정 | settings |
| `start_pomodoro` | 선택 phase | `PomodoroDto` | timer/settings |
| `pause_pomodoro` | 없음 | `PomodoroDto` | timer/tray |
| `resume_pomodoro` | 없음 | `PomodoroDto` | timer/tray |
| `skip_pomodoro` | 없음 | `PomodoroDto` | timer |
| `stop_pomodoro` | 없음 | `PomodoroDto` | timer |
| `pet_clicked` | timestamp | 없음 | pet |
| `pet_drag_started` | pointer point | 없음 | pet |
| `pet_drag_moved` | screen point | 없음 | pet |
| `pet_drag_ended` | velocity | 없음 | pet |
| `cancel_intervention` | intervention ID | 없음 | pet |
| `emergency_stop` | 없음 | 없음 | tray/settings |
| `resume_pet` | 없음 | 없음 | tray/settings |

명령 입력은 모두 Rust에서 다시 검증한다. capability 파일은 각 창이 필요한 명령만 호출하도록 최소 권한으로 나눈다.

### 6.2 Rust → 프런트 이벤트

| 이벤트 | payload | 사용처 |
|---|---|---|
| `pet://behavior-changed` | behavior, direction, animation | pet |
| `pet://position-changed` | x, y, monitor ID | pet |
| `pet://speech` | text, duration | pet |
| `pet://card` | show/hide, card ID | pet/card |
| `timer://state-changed` | phase, run state, remaining | timer/settings |
| `focus://warning` | intervention ID, seconds | pet |
| `focus://cancelled` | intervention ID | pet |
| `app://emergency-stopped` | 없음 | 전체 |
| `settings://changed` | 정규화된 설정 | 전체 |

이벤트 listener는 컴포넌트 해제 시 반드시 unlisten한다. 고빈도 좌표 이벤트가 부담되면 Rust가 창 자체를 이동하고 React에는 행동 변화만 보낸다.

## 7. 주요 실행 흐름

### 7.1 앱 시작

1. 앱 데이터 디렉터리를 확인한다.
2. `settings.json`을 읽고 schema migration과 범위 검증을 수행한다.
3. 손상된 파일은 `settings.corrupt-<timestamp>.json`으로 보관하고 안전한 기본값을 사용한다.
4. placeholder 캐릭터 manifest를 검증한다.
5. 트레이, 전역 단축키, 펫 창을 만든다.
6. 공유 `AppState`와 취소 토큰을 만든다.
7. 행동 스케줄러를 시작한다.
8. 집중 세션일 때만 전경 창 감지 루프와 비활동 감지 루프를 활성화한다.

### 7.2 행동 스케줄러

- 단일 scheduler만 다음 행동을 결정한다.
- 상태 변경은 `BehaviorRequest { kind, priority, source }`로 제출한다.
- 현재 행동보다 우선순위가 높거나 현재 행동이 종료되었을 때만 전환한다.
- 랜덤 행동은 빈도 설정에 따라 최소 간격을 두고 선택한다.
- Dragged 진입 시 이동, 경고와 발차기 토큰을 모두 취소한다.
- 긴급 중지 시 scheduler를 pause하고 모든 부가 창을 숨긴다.

초기 랜덤 간격 권장값:

| 빈도 | 다음 장난까지 범위 |
|---|---:|
| 낮음 | 120~240초 |
| 보통 | 60~150초 |
| 높음 | 30~90초 |

### 7.3 드래그와 던지기

1. pointer down 시 포인터 캡처 후 최근 5~8개 `(위치, 시간)` 표본을 수집한다.
2. 창 위치는 screen physical 좌표로 Rust에 전달한다.
3. pointer up 시 최근 80~120ms 이동량으로 속도를 계산한다.
4. 속력이 임계값 미만이면 해당 위치에서 Idle로 전환한다.
5. 이상이면 Thrown 상태로 전환하고 중력, 감속과 경계 반사를 적용한다.
6. 최대 3초, 낮은 속력 또는 바닥 접촉 중 먼저 충족하는 조건에서 종료한다.

초기 튜닝값:

- 던지기 임계 속력: 700 px/s
- 최대 속력: 2,500 px/s
- 중력: 2,200 px/s²
- 경계 반발 계수: 0.45
- 바닥 마찰 계수: 0.80
- 물리 tick: 16ms, 저전력 모드에서는 33ms

모든 값은 `PhysicsConfig`에 모아 코드 여러 곳에 상수로 흩어지지 않게 한다.

### 7.4 방해 창 개입

```text
Focus 실행 중
  → 1초마다 전경 창 snapshot
  → 보호 대상 검사
  → 활성 규칙과 일치
  → 동일 hwnd + 동일 rule이 grace 기간 유지
  → intervention ID 생성
  → 펫 이동 및 경고
  → 사용자 취소/드래그/Focus 종료 여부 확인
  → 전경 hwnd, 규칙, 보호 여부 재검사
  → Kick 충돌 프레임
  → ShowWindow(SW_MINIMIZE) 1회
  → rule/hwnd 쿨다운
```

다음 경우 즉시 취소한다.

- 전경 창이 바뀜
- 창이 닫히거나 이미 최소화됨
- 집중 세션이 멈춤, 휴식, 일시정지로 바뀜
- 사용자가 펫을 드래그함
- 긴급 중지가 실행됨
- 설정에서 개입 또는 규칙을 끔
- 보호 여부를 확정할 수 없음

### 7.5 보호 창 판정

24시간 MVP에서는 다음 순서로 보수적으로 판정한다.

1. 앱 자신의 PID 또는 알려진 window label이면 보호
2. 창이 없거나 보이지 않거나 이미 최소화되었으면 제외
3. 프로세스명/제목을 읽지 못하면 보호
4. `explorer.exe`, `taskmgr.exe`, 보안/UAC 관련 시스템 프로세스는 보호
5. 파일 열기/저장 대화상자로 알려진 class이면 보호
6. 설정의 사용자 보호 목록과 일치하면 보호
7. 전체 화면에 가까운 창은 MVP에서 보호
8. 원격 데스크톱/화상회의 기본 목록과 일치하면 보호

보호 목록은 `protected_window.rs`의 코드 기본값과 사용자 추가 목록을 합치며, 코드 기본값은 UI에서 해제할 수 없게 한다.

### 7.6 긴급 중지

긴급 중지는 하나의 atomic flag와 취소 토큰으로 처리한다.

1. `emergency_stopped = true`
2. 진행 중인 개입, 경고, 카드와 랜덤 행동 취소
3. 펫/카드/말풍선 창 숨김
4. 실행 중 타이머 일시정지
5. 전경 창 및 비활동 polling 중단
6. 트레이는 유지하고 `펫 다시 시작` 활성화

목표 반영 시간은 300ms 이내다. 단축키 등록 실패는 조용히 무시하지 않고 시작 직후 설정 창에 오류를 표시한다.

## 8. 창과 렌더링 설정

### 8.1 펫 창

- 크기: 기본 256×256 logical px, scale 설정 반영
- `decorations: false`, `transparent: true`, `alwaysOnTop: true`
- 작업표시줄에는 표시하지 않음
- 전체 화면 투명 창을 사용하지 않음
- 드래그 중이 아닐 때 투명 영역 click-through가 필요함
- Windows마다 투명 창/그림자 동작이 달라 개발 PC에서 가장 먼저 검증

투명 픽셀 단위 hit-test가 하루 안에 안정적으로 되지 않으면 MVP에서는 스프라이트 외곽의 작은 사각 창을 사용한다. 그 대신 여백을 8~12px로 줄여 아래 창 클릭 방해를 최소화한다.

### 8.2 좌표 규칙

- Rust 내부 기준 좌표는 Windows 가상 데스크톱의 physical px로 통일한다.
- UI 크기와 pointer 이벤트는 scale factor를 사용해 physical px로 변환한다.
- 각 모니터의 `work_area`를 저장하고 작업표시줄 영역을 목적지에서 제외한다.
- 펫의 최소 24px가 work area 안에 남도록 clamp한다.
- 모니터 구성이 바뀌면 가장 가까운 유효 work area로 좌표를 복구한다.

### 8.3 캐릭터 manifest

```json
{
  "schemaVersion": 1,
  "id": "placeholder",
  "canvas": { "width": 256, "height": 256 },
  "anchors": {
    "feet": { "x": 128, "y": 236 },
    "speech": { "x": 128, "y": 20 },
    "accessory": { "x": 128, "y": 128 }
  },
  "animations": {
    "idle": { "frames": ["idle/0.webp"], "frameMs": 500, "loop": true },
    "walk": { "frames": ["walk/0.webp", "walk/1.webp"], "frameMs": 140, "loop": true },
    "dragged": { "frames": ["dragged/0.webp"], "frameMs": 200, "loop": true },
    "thrown": { "frames": ["thrown/0.webp"], "frameMs": 100, "loop": true },
    "kick": { "frames": ["kick/0.webp", "kick/1.webp"], "frameMs": 100, "loop": false, "impactFrame": 1 },
    "speak": { "frames": ["speak/0.webp"], "frameMs": 200, "loop": true }
  }
}
```

필수 animation이 누락되면 placeholder pack의 같은 상태를 사용한다. 권장 animation은 idle 또는 walk로 대체한다.

## 9. 설정 저장과 마이그레이션

파일 위치는 Tauri app data directory 아래로 정한다.

```text
<app-data>/desktop-pet/
├─ settings.json
├─ settings.corrupt-<timestamp>.json
├─ session.json                 # 자동 재개하지 않을 복구 안내용
└─ logs/
   └─ app.log                   # 제목/규칙 문자열 미포함, 크기 제한
```

저장 순서:

1. 메모리 설정을 검증·정규화한다.
2. 같은 디렉터리의 `settings.tmp`에 JSON을 쓴다.
3. flush 후 기존 파일을 교체한다.
4. 실패 시 메모리 설정은 유지하고 UI에 저장 실패를 알린다.

schema migration은 `v1 → v2`처럼 작은 순수 함수로 분리하고 단위 테스트한다. 손상 파일의 내용을 로그에 복사하지 않는다.

## 10. 24시간 구현 일정

각 구간 끝에 실행 가능한 상태를 남긴다. 한 구간이 지연되면 그 구간의 P2 항목을 빼고 다음 핵심 경로로 이동한다.

| 시간 | 작업 | 구간 완료 조건 |
|---:|---|---|
| 0:00~1:00 | 기존 프로젝트 빌드, 브랜치/백업, 개발 실행 확인, scope 고정 | 깨끗한 상태에서 앱 1회 실행 |
| 1:00~2:30 | Tauri 창 label, 트레이, 투명 펫 창, 긴급 단축키 골격 | 트레이로 펫 표시/숨김 가능 |
| 2:30~4:30 | 설정 모델, 기본값, JSON load/save, AppState, command/event 계약 | 설정 수정 후 재실행 복원 |
| 4:30~7:00 | placeholder 스프라이트, Idle/Walk, 모니터 work area, 창 이동 | 펫이 화면 안에서 걷고 멈춤 |
| 7:00~9:00 | 클릭, drag 표본 수집, throw 물리, 경계 clamp | 잡고 던진 뒤 3초 안에 착지 |
| 9:00~11:00 | 뽀모도로 상태 머신과 timer UI | 시작/일시정지/재개/중지 동작 |
| 11:00~13:00 | Win32 전경 창 adapter, 규칙 matcher, 설정 UI | 테스트 창에서 규칙 일치 표시 |
| 13:00~15:30 | grace/cooldown, 보호 규칙, intervention cancellation | 잘못된 창은 최소화하지 않음 |
| 15:30~17:00 | 펫 경고→Kick impact→최소화 연결 | 전체 핵심 세로 단면 데모 성공 |
| 17:00~18:00 | 긴급 중지 전체 연결, 기본 안전값 확인 | 300ms 수준으로 개입 중단 |
| 18:00~19:30 | 대사, 춤, 클릭 이스터에그 | P1 펫 반응 동작 |
| 19:30~20:30 | 비활동 판정 또는 카드 중 하나 구현 | 더 가치 높은 P1/P2 하나 완료 |
| 20:30~22:00 | 단위/통합 테스트, 오류 경로 수정 | 핵심 Rust 테스트 통과 |
| 22:00~23:00 | DPI, 다중 모니터, Chrome/Edge, 관리자 창 수동 검사 | 체크리스트 기록 |
| 23:00~24:00 | release 빌드, README, 알려진 제한과 데모 순서 작성 | 재현 가능한 빌드 산출물 |

### 10.1 일정 중단선

- 7시간 시점에 펫 창 이동이 안 되면 hit-test 고도화를 중단하고 사각 입력 영역을 사용한다.
- 13시간 시점에 전경 창 정보 획득이 안 되면 프로세스명 규칙만 우선 완성한다.
- 17시간 시점에 최소화 세로 단면이 안 되면 모든 랜덤 장난과 카드 작업을 중단한다.
- 20시간 이후에는 새 기능을 추가하지 않고 안전성, 테스트와 release 빌드만 수행한다.

## 11. 구현 순서별 상세 체크리스트

### Phase A — 기반

- [ ] 개발/릴리스 빌드가 실행된다.
- [ ] `pet`, `settings`, `timer` window label을 고정한다.
- [ ] 트레이 메뉴: 펫 표시/숨김, 타이머, 설정, 긴급 중지, 종료.
- [ ] AppState 공유 락 범위를 짧게 유지한다.
- [ ] 종료 시 background task cancellation과 listener 해제를 수행한다.

### Phase B — 설정과 계약

- [ ] 안전한 기본값으로 `Default` 구현.
- [ ] save 전에 범위와 문자열 검증.
- [ ] capability를 창별로 분리.
- [ ] Rust DTO와 TypeScript type 필드를 동일하게 유지.
- [ ] 손상 JSON 복구 단위 테스트.

### Phase C — 펫

- [ ] manifest validation과 fallback.
- [ ] Idle/Walk 상태 전환.
- [ ] 방향에 따른 CSS 좌우 반전.
- [ ] 모니터 work area 목적지 선택.
- [ ] drag 중 자동 행동 중단.
- [ ] throw 속도 clamp와 최대 3초 종료.
- [ ] 7회/10초 클릭과 30분 쿨다운.

### Phase D — 타이머

- [ ] Focus/ShortBreak/LongBreak 순환.
- [ ] 네 번째 Focus 뒤 LongBreak.
- [ ] pause/resume/skip/stop.
- [ ] 집중이 아니면 모든 감지 중단.
- [ ] 절전 후 wall-clock과 monotonic 결과가 이상하면 음수 대신 0 처리.

### Phase E — 방해 감지와 개입

- [ ] polling interval 기본 1초.
- [ ] process/title matcher 단위 테스트.
- [ ] 동일 창 연속 감지로 grace 계산.
- [ ] warning 직전과 minimize 직전 이중 검증.
- [ ] 보호 대상/실패는 조용한 skip + 비민감 로그.
- [ ] 동일 규칙/창 cooldown.
- [ ] Kick impact frame에서 1회만 minimize.

### Phase F — 안전과 마감

- [ ] 개입 기본 off, 빈 규칙 목록.
- [ ] 긴급 중지에서 모든 cancellation path 검증.
- [ ] 앱 자체 창을 절대 최소화하지 않음.
- [ ] 실제 창 제목과 규칙 문자열을 로그하지 않음.
- [ ] release build와 새 PC 또는 별도 Windows 사용자 계정 smoke test.

## 12. 테스트 계획

### 12.1 필수 단위 테스트

```text
domain::pomodoro
  focus_to_short_break
  fourth_focus_to_long_break
  pause_does_not_consume_time
  skip_moves_to_correct_phase

domain::distraction
  process_name_is_case_insensitive
  title_contains_matches_case_insensitively
  disabled_rule_never_matches
  empty_rule_is_invalid

domain::behavior
  dragged_interrupts_walk
  kick_cannot_interrupt_dragged
  focus_end_cancels_warning

domain::physics
  speed_is_clamped
  pet_stays_inside_work_area
  thrown_finishes_within_three_seconds

application::intervention
  foreground_change_cancels_minimize
  cooldown_prevents_repeat
  emergency_stop_cancels_minimize
  protected_window_is_never_minimized
```

### 12.2 통합 테스트용 fake

다음 trait과 fake 구현을 둔다.

```rust
trait ForegroundWindowSource { fn snapshot(&self) -> Result<Option<WindowSnapshot>>; }
trait WindowMinimizer { fn minimize(&self, window: &WindowSnapshot) -> Result<()>; }
trait IdleTimeSource { fn idle_for(&self) -> Result<Duration>; }
trait Clock { fn now(&self) -> Instant; }
trait RandomSource { fn range(&self, min: u64, max: u64) -> u64; }
```

fake snapshot을 순서대로 반환해 `감지 → grace → warning → 재검증 → minimize`와 모든 취소 경로를 실제 창 조작 없이 검증한다.

### 12.3 Windows 수동 검사표

| 항목 | 기대 결과 | 결과 |
|---|---|---|
| 100%, 150%, 200% 배율 | 펫과 포인터 위치가 어긋나지 않음 | □ |
| 듀얼 모니터와 음수 X 좌표 | 화면 밖으로 사라지지 않음 | □ |
| 작업표시줄 위치/자동 숨김 | work area 안에 착지 | □ |
| 투명 영역 클릭 | 가능한 범위에서 아래 창 입력 전달 | □ |
| Chrome/Edge 제목 규칙 | grace 후 해당 창만 최소화 | □ |
| 경고 중 Alt+Tab | 새 전경 창을 최소화하지 않음 | □ |
| 관리자 권한 앱 | 실패해도 앱이 종료되지 않음 | □ |
| 설정/타이머 창 | 자체 창은 절대 최소화하지 않음 | □ |
| Focus 종료/휴식 | 감지와 개입 즉시 중단 | □ |
| 긴급 단축키 | 펫 숨김, 개입 취소, 타이머 일시정지 | □ |
| 설정 파일 손상 | 백업 후 기본 설정으로 시작 | □ |
| 30분 연속 실행 | 눈에 띄는 CPU/메모리 지속 증가 없음 | □ |

24시간 안정성 시험은 개발 24시간 안에 결과를 얻기 어렵다. MVP 당일에는 30분 soak test와 작업 관리자 관찰을 수행하고, release candidate를 만든 직후 별도로 24시간 soak test를 예약한다.

## 13. 성능 기준과 구현 주의점

- 전경 창 감지는 Focus 실행 중에만 1초 간격으로 수행한다.
- Idle/Walk 창 이동은 30fps로도 충분하다. Thrown만 최대 60fps를 사용한다.
- React state를 매 frame 갱신하지 말고, 가능하면 Rust가 창 위치를 직접 변경한다.
- animation frame 이미지는 시작 시 preload한다.
- 하나의 task가 AppState write lock을 잡은 채 await하지 않는다.
- tracing log는 rolling 또는 크기 제한을 적용하고 debug build에서만 상세화한다.
- 카드 창은 최대 하나, 말풍선도 최대 하나로 제한한다.
- 집중하지 않을 때 Win32 polling과 idle polling task를 중지한다.

## 14. 오류 처리 정책

| 상황 | 처리 |
|---|---|
| 설정 파일 없음 | 기본값 생성 |
| 설정 파일 손상 | 원본 보관, 기본값 시작, 사용자 알림 |
| 단축키 충돌 | 기능 상태 표시, 설정에서 변경 유도 |
| 창 제목/프로세스 접근 실패 | 보호 대상으로 취급하고 skip |
| 대상 HWND 소멸 | intervention 취소 |
| minimize 실패 | 한 번 기록 후 cooldown, 반복 호출 금지 |
| 캐릭터 프레임 누락 | placeholder 상태로 fallback |
| 펫 renderer 응답 없음 | 모든 pending intervention 취소 |
| 모니터 연결 해제 | 가장 가까운 주 모니터 work area로 복구 |
| Tauri event listener 실패 | 해당 기능 비활성화와 사용자에게 재시작 안내 |

사용자에게 보여주는 메시지는 해결 방법을 포함한다. 예: `긴급 중지 단축키를 등록하지 못했습니다. 설정에서 다른 키 조합을 선택하세요.`

## 15. 보안·개인정보 체크

- [ ] 네트워크 요청과 분석 SDK 없음.
- [ ] 키 입력 내용, 스크린샷, URL, 방문 기록을 수집하지 않음.
- [ ] 전경 창 정보는 메모리 판정 후 폐기.
- [ ] 로그에 실제 제목, 경로, 사용자 규칙 문자열을 남기지 않음.
- [ ] 외부 실행, 파일 이동/삭제, 강제 종료 기능 없음.
- [ ] 최소화는 opt-in이며 실행 직전 재검증.
- [ ] Tauri capability는 창별 최소 권한.
- [ ] `unsafe` 코드는 Windows adapter 안에 국한하고 호출 전후 조건을 주석으로 기록.
- [ ] 번들 이미지와 폰트의 재배포 권리 확인.

## 16. 개발 명령

기존 프로젝트의 패키지 매니저가 있다면 그대로 사용한다. 신규 기준 예시는 다음과 같다.

```powershell
npm install
npm run tauri dev
npm run lint
npm run test
cargo fmt --all --check --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --all-features --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

권장 `package.json` script:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "lint": "eslint .",
    "test": "vitest run",
    "tauri": "tauri"
  }
}
```

## 17. 24시간 완료 정의

### 기능

- [ ] 트레이와 펫 오버레이로 시작한다.
- [ ] Idle, Walk, Dragged, Thrown, Kick이 placeholder 자산으로 동작한다.
- [ ] 펫이 다중 모니터 작업 영역 안에 머문다.
- [ ] 뽀모도로 시작/일시정지/재개/중지와 휴식 전환이 된다.
- [ ] process/title 규칙을 추가, 수정, 비활성화할 수 있다.
- [ ] 집중 중 동일 대상이 grace를 만족할 때만 경고한다.
- [ ] 실행 직전에도 대상이 같을 때만 1회 최소화한다.
- [ ] Focus 종료, Dragged와 긴급 중지에서 개입이 취소된다.
- [ ] 설정이 앱 재시작 후 복원된다.

### 안전

- [ ] 개입 기능은 기본 꺼짐이다.
- [ ] 앱 자체 창과 알 수 없는 높은 권한 창을 최소화하지 않는다.
- [ ] 단축키가 충돌하면 사용자에게 알린다.
- [ ] 실제 창 정보가 저장 또는 로그되지 않는다.
- [ ] 모든 Win32 실패는 앱 충돌 없이 해당 작업만 건너뛴다.

### 품질과 인수인계

- [ ] Rust 핵심 단위/통합 테스트가 통과한다.
- [ ] TypeScript typecheck와 lint가 통과한다.
- [ ] Windows 수동 검사표 결과가 기록되어 있다.
- [ ] release build가 성공한다.
- [ ] README에 실행 방법, 설정 위치, 긴급 중지와 알려진 제한이 있다.
- [ ] 완성 자산을 manifest와 이미지 교체만으로 넣을 수 있다.

## 18. 24시간 이후 즉시 할 일

1. release candidate 24시간 soak test와 CPU/RSS 기록
2. Windows 11 기기 2대 이상에서 DPI/멀티 모니터 회귀 검사
3. 픽셀 단위 hit-test와 접근성 개선
4. Windows 알림, 자동 시작과 알림음 완성
5. 카드 전용 창과 액세서리 선택 UI 완성
6. 보호 대상 목록을 실제 테스트 결과로 보강
7. 코드 서명, 설치 프로그램과 업데이트 전략 수립
8. 최종 캐릭터 팩의 프레임, anchor, impact frame 검수

## 19. 최종 권고

24시간 개발에서 가장 중요한 것은 “많은 펫 행동”보다 `집중 시작 → 방해 창 감지 → 경고 → 대상 재검증 → 안전한 최소화 → 긴급 중지`의 세로 단면이다. 이 경로를 17시간 안에 완성하고 이후 시간을 테스트와 안전장치에 사용한다. 캐릭터 카드, 액세서리와 시각적 완성도는 핵심 경로가 안정된 뒤 추가한다.
