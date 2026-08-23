# 07. 데이터와 통신 계약

## 설정 JSON

```json
{
  "schemaVersion": 2,
  "pet": {
    "enabled": true,
    "alwaysOnTop": true,
    "scalePercent": 100,
    "prankFrequency": "low",
    "cursorChase": true,
    "dance": true,
    "speech": true,
    "cards": true,
    "accessoryId": null
  },
  "pomodoro": {
    "focusMinutes": 25,
    "shortBreakMinutes": 5,
    "longBreakMinutes": 15,
    "longBreakEvery": 4,
    "autoStartBreak": false,
    "autoStartFocus": false
  },
  "focusGuard": {
    "enabled": false,
    "idleWarningEnabled": false,
    "idleMinutes": 10,
    "warningSeconds": 3,
    "rules": []
  },
  "emergencyHotkey": "Ctrl+Shift+F12"
}
```

## 방해 규칙

```rust
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

검증:

- process/title 중 하나 이상 필요
- 빈 문자열은 `None`
- 프로세스명은 경로가 아닌 파일명만 저장
- grace: 5초~10분
- cooldown: 30초~60분
- 비교는 대소문자 무시

## Tauri commands

| 명령 | 입력 | 출력 |
|---|---|---|
| `get_bootstrap_state` | 없음 | settings, timer, pet 상태 |
| `save_settings` | settings DTO | 정규화된 settings |
| `start_pomodoro` | 선택 phase | timer DTO |
| `pause_pomodoro` | 없음 | timer DTO |
| `resume_pomodoro` | 없음 | timer DTO |
| `skip_pomodoro` | 없음 | timer DTO |
| `stop_pomodoro` | 없음 | timer DTO |
| `pet_clicked` | timestamp | 없음 |
| `pet_drag_started` | pointer point | 없음 |
| `pet_drag_moved` | screen point | 없음 |
| `pet_drag_ended` | velocity | 없음 |
| `cancel_intervention` | intervention ID | 없음 |
| `emergency_stop` | 없음 | 없음 |
| `resume_pet` | 없음 | 없음 |

## Rust → UI events

| 이벤트 | payload |
|---|---|
| `pet://behavior-changed` | behavior, direction, animation |
| `pet://speech` | text, durationMs |
| `pet://card` | visible, cardId |
| `timer://state-changed` | phase, runState, remainingSeconds |
| `focus://warning` | interventionId, seconds |
| `focus://cancelled` | interventionId |
| `app://emergency-stopped` | 없음 |
| `settings://changed` | 정규화된 settings |

## PetBehavior

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

## WindowSnapshot

```rust
struct WindowSnapshot {
    hwnd: isize,
    process_id: u32,
    process_name: Option<String>,
    title: Option<String>,
    class_name: Option<String>,
    is_visible: bool,
    is_minimized: bool,
}
```

이 구조는 메모리 전용이다. Serialize를 derive하지 않고 로그의 debug field로 넘기지 않는다.

## 호환성 규칙

- Rust DTO와 TypeScript type의 field name은 camelCase 직렬화를 명시한다.
- enum 문자열은 문서화된 값만 사용한다.
- 새 설정 필드는 기본값을 가져야 한다.
- 알 수 없는 JSON 필드는 무시한다.
- breaking change에는 `schemaVersion`을 올리고 migration test를 추가한다.
