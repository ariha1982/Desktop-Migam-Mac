# 03. 아키텍처

## 전체 구조

```text
TypeScript 창
  pet / timer / settings / card
        ↕ command · event
Tauri presentation
        ↓
application services
        ↓
domain state machines
        ↕ trait
infrastructure/windows · filesystem · clock
```

## 의존 방향

```text
presentation → application → domain
                         ↘ infrastructure의 trait
```

- `domain`: 순수 상태, 규칙과 계산
- `application`: 사용 사례 순서, 취소와 서비스 조정
- `infrastructure`: 파일, 시계, 난수와 Windows API
- `presentation`: Tauri command/event, tray, window와 hotkey
- TypeScript UI: DOM 표시와 포인터 입력 수집

## 주요 서비스

### BehaviorScheduler

- 행동 요청의 우선순위를 비교한다.
- 한 번에 하나의 자동 행동만 허용한다.
- Dragged와 EmergencyStopped를 즉시 반영한다.
- 랜덤 행동 간 최소 간격을 보장한다.

### MotionService

- work area 안의 목적지를 고른다.
- Walk와 Thrown 위치를 계산한다.
- 모니터 구성 변경 후 유효 위치로 복구한다.

### PomodoroService

- Focus, ShortBreak, LongBreak 상태를 관리한다.
- pause/resume/skip/stop을 처리한다.
- Focus가 아닐 때 감지 task를 중지한다.

### DistractionMonitor

- Focus 중 1초마다 전경 창 snapshot을 읽는다.
- 규칙 일치, 유예 시간과 쿨다운을 관리한다.
- 실제 창 정보는 저장하지 않는다.

### InterventionService

- 펫 이동 → 경고 → 재검증 → Kick → 최소화 순서를 담당한다.
- 대상 변경, Dragged, Focus 종료와 긴급 중지에서 취소한다.

### SettingsService

- 기본값, validation, migration과 원자 저장을 담당한다.
- 손상 파일을 보관하고 안전한 설정으로 복구한다.

## 공유 상태

```text
AppState
├─ settings: RwLock<AppSettings>
├─ pomodoro: Mutex<PomodoroState>
├─ behavior: Mutex<PetBehaviorState>
├─ emergency_stopped: AtomicBool
├─ service cancellation tokens
└─ adapters: Arc<dyn ...>
```

하나의 명령에서 여러 lock이 필요하면 항상 같은 순서로 짧게 획득한다. 가능하면 값의 복사본을 만든 뒤 lock을 해제하고 I/O를 수행한다.

## 시작 순서

1. 앱 데이터 경로 결정
2. 설정 load/migrate/validate
3. 캐릭터 manifest 검증
4. AppState와 adapter 조립
5. 트레이와 전역 단축키 생성
6. 펫 창 생성
7. BehaviorScheduler 시작
8. Focus 상태일 때만 monitor 시작

## 종료 순서

1. 신규 명령 차단
2. 개입과 monitor 취소
3. 타이머 상태 저장
4. event listener와 task 종료
5. 창과 tray 정리

## 핵심 개입 시퀀스

```text
ForegroundWindowSource
  → DistractionMonitor: snapshot
  → rule match + grace
  → InterventionService: warning requested
  → BehaviorScheduler: FocusWarning
  → user cancellation window
  → fresh snapshot + protection check
  → BehaviorScheduler: Kick
  → impact frame
  → WindowMinimizer: one call
  → cooldown
```

## 실패 격리

- 펫 렌더러 실패: 개입 취소
- 전경 창 접근 실패: 해당 검사만 skip
- 최소화 실패: 1회 기록하고 cooldown
- 설정 저장 실패: 메모리 설정 유지, UI 알림
- 단축키 충돌: 설정 변경 안내, 앱은 계속 실행
