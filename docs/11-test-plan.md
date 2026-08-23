# 11. 테스트 계획

## 테스트 원칙

- 도메인 계산은 OS 없이 테스트한다.
- Windows 호출은 trait 뒤에 두고 fake로 흐름을 검증한다.
- 실제 창 조작은 명시적인 수동 테스트 앱에만 수행한다.
- 테스트 로그에도 실제 창 제목을 남기지 않는다.

## 필수 단위 테스트

### Pomodoro

- Focus → ShortBreak
- 네 번째 Focus → LongBreak
- pause 동안 경과 시간 증가하지 않음
- resume 후 남은 시간 유지
- skip의 다음 phase
- 음수 remaining 방지

### DistractionRule

- process 이름 대소문자 무시
- title 포함 대소문자 무시
- disabled rule 불일치
- 빈 rule 검증 실패
- grace와 cooldown 경계값

### Behavior

- Dragged가 Walk 중단
- Kick이 Dragged를 중단하지 못함
- Focus 종료가 Warning/Kick 취소
- 한 번에 자동 행동 하나
- 7회 클릭/10초와 30분 cooldown

### Physics

- 최대 속력 clamp
- 중력과 감속
- 경계 반사
- work area 유지
- 3초 안에 종료
- 비정상 pointer time 처리

### Settings

- 기본값
- 범위 clamp
- 알 수 없는 field 무시
- 이전 schema migration
- 손상 JSON backup과 복구

## 통합 테스트

fake adapter:

```rust
trait ForegroundWindowSource {
    fn snapshot(&self) -> Result<Option<WindowSnapshot>>;
}

trait WindowMinimizer {
    fn minimize(&self, window: &WindowSnapshot) -> Result<()>;
}

trait IdleTimeSource {
    fn idle_for(&self) -> Result<Duration>;
}

trait Clock {
    fn now(&self) -> Instant;
}
```

필수 시나리오:

- 감지 → grace → warning → 재검증 → 최소화 1회
- warning 중 전경 변경 → 0회
- Focus 종료 → pending 개입 취소
- emergency stop → pending 개입 취소
- protected window → 0회
- 권한 오류 → 앱 유지, 0회
- cooldown 중 재감지 → 0회
- renderer 실패 → 개입 취소

## Windows 수동 검사

| 항목 | 기대 결과 | 결과/메모 |
|---|---|---|
| 100% DPI | 위치 정상 | [ ] |
| 150% DPI | pointer와 창 위치 일치 | [ ] |
| 200% DPI | scale과 경계 정상 | [ ] |
| 듀얼 모니터 | 화면 밖 유실 없음 | [ ] |
| 음수 가상 좌표 | clamp 정상 | [ ] |
| 작업표시줄 자동 숨김 | work area 정상 | [ ] |
| 펫 투명 영역 | 아래 창 클릭 가능 | [ ] |
| Chrome/Edge 제목 규칙 | 대상만 최소화 | [ ] |
| warning 중 Alt+Tab | 아무 새 창도 최소화 안 함 | [ ] |
| 관리자 앱 | 안전하게 skip | [ ] |
| 설정/타이머 창 | 자체 창 보호 | [ ] |
| 전체 화면 앱 | 보호 | [ ] |
| Focus 종료/휴식 | 감지 즉시 중지 | [ ] |
| 긴급 hotkey | 개입 취소와 숨김 | [ ] |
| 설정 손상 | backup 후 기본값 | [ ] |
| 30분 soak | 지속 메모리 증가 없음 | [ ] |

## 회귀 테스트 순서

1. Rust domain tests
2. Rust integration tests
3. TypeScript tests
4. lint/typecheck/clippy
5. dev smoke test
6. release build
7. Windows 수동 검사

## 버그 보고 형식

```text
제목:
환경: Windows build / DPI / 모니터 구성
사전 조건:
재현 단계:
기대 결과:
실제 결과:
재현 빈도:
안전 영향:
비민감 로그:
```
