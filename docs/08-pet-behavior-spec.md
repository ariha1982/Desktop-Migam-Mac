# 08. 펫 행동 명세

## 상태와 우선순위

| 상태 | 우선순위 | 진입 | 종료 |
|---|---:|---|---|
| Idle | 10 | 다른 행동 없음 | 다음 행동 선택 |
| Walk | 20 | 목적지 선택 | 도착/중단 |
| ChaseCursor | 30 | 확률 이벤트 | 제한 시간/입력 |
| Speak | 40 | 대사 이벤트 | 표시 종료 |
| Dance | 40 | 춤 이벤트 | 1회 완료 |
| CarryCard | 50 | 카드 이벤트 | 카드 배치 |
| FocusWarning | 70 | 방해/비활동 | 취소/Kick |
| Kick | 80 | 경고 완료 | impact와 애니메이션 완료 |
| Thrown | 90 | 빠른 놓기 | 착지/3초 |
| Dragged | 100 | pointer down | pointer up |

## 전이 규칙

- 한 번에 자동 행동 하나만 실행한다.
- 높은 우선순위만 낮은 행동을 중단할 수 있다.
- Dragged는 현재 행동과 pending intervention을 취소한다.
- Dragged/Thrown 중 Kick은 실행하지 않는다.
- Focus 종료와 긴급 중지는 모든 경고/Kick 예약보다 우선한다.
- 행동 종료 후 유효 상태가 없으면 Idle로 간다.

## 이동

- 내부 좌표: 가상 데스크톱 physical px
- 목적지: monitor work area 내부
- 작업표시줄 영역 제외
- 최소 24px가 화면 안에 남도록 clamp
- 연결되지 않은 모니터로 순간 이동 금지
- Idle/Walk update: 30fps
- Thrown update: 최대 60fps

## 던지기 초기값

| 값 | 기본값 |
|---|---:|
| 임계 속력 | 700 px/s |
| 최대 속력 | 2,500 px/s |
| 중력 | 2,200 px/s² |
| 반발 계수 | 0.45 |
| 바닥 마찰 | 0.80 |
| 최대 시간 | 3초 |

속도는 최근 80~120ms의 pointer 표본으로 계산한다. 표본 시간이 0이거나 비정상적이면 던지지 않고 놓은 위치에서 Idle로 간다.

## 랜덤 행동 간격

| 빈도 | 다음 장난까지 |
|---|---:|
| 낮음 | 120~240초 |
| 보통 | 60~150초 |
| 높음 | 30~90초 |

동일 대사는 기본 10분 쿨다운을 가진다. 동시에 말풍선 하나, 카드 하나만 허용한다.

## 클릭

- 단일 클릭: 짧은 반응과 session click 기록
- 10초 안에 7회: 이스터에그
- 실행 뒤 30분 쿨다운
- 시스템이나 파일 상태를 변경하지 않음
- Drag 시작으로 판정된 pointer는 일반 클릭에서 제외 가능

## 대사

- 일반 랜덤: `데프콘 가자`
- 집중 중 비활동: `데프콘 안 갈 거야?`
- 아무 말: 짧은 문장 → 0.5~1.5초 → `미안하다`
- 표시 시간: 2~4초
- 움직임 줄이기 설정에서는 말풍선 또는 Windows 알림을 우선

## 행동 요청 모델

```rust
struct BehaviorRequest {
    kind: PetBehavior,
    source: BehaviorSource,
    requested_at: Instant,
    cancellation_id: Option<Uuid>,
}
```

스케줄러는 UI 이미지 객체를 저장하지 않는다. 상태/animation ID만 이벤트로 내보낸다.

## 필수 테스트

- Dragged가 Walk를 중단함
- Dragged 중 Kick 거부
- Thrown이 3초 이내 종료
- 모니터 경계 밖 좌표 clamp
- 7회 클릭 시간 창과 쿨다운
- Focus 종료가 경고 취소
- 랜덤 행동이 동시에 두 개 실행되지 않음
