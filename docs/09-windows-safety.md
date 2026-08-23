# 09. Windows 연동과 안전 정책

## 사용하는 API

| 기능 | Win32 API |
|---|---|
| 전경 창 | `GetForegroundWindow` |
| PID | `GetWindowThreadProcessId` |
| 실행 파일 | `QueryFullProcessImageNameW` |
| 제목 | `GetWindowTextLengthW`, `GetWindowTextW` |
| 최소화 | `ShowWindow(hwnd, SW_MINIMIZE)` |
| 비활동 | `GetLastInputInfo` |
| 유효성 | `IsWindow`, `IsWindowVisible`, `IsIconic` |
| 클래스 | `GetClassNameW` |
| 모니터 | `EnumDisplayMonitors`, `GetMonitorInfoW` |

## 절대 규칙

- 창 최소화 기능은 기본 off다.
- 활성 규칙이 하나 이상 있어야 개입한다.
- `HWND`를 디스크에 저장하지 않는다.
- 실행 직전에 fresh snapshot을 다시 얻는다.
- 읽을 수 없거나 보호 여부를 확정할 수 없는 창은 보호한다.
- `ShowWindow`는 한 intervention에서 최대 1회 호출한다.
- 실패 후 즉시 반복하지 않고 cooldown을 적용한다.
- 실제 제목, 실행 경로와 규칙 문자열을 로그하지 않는다.

## 최소화 절차

1. Focus가 Running인지 확인
2. 개입 enabled 확인
3. fresh foreground snapshot 획득
4. 창 유효성/가시성 검사
5. 보호 대상 검사
6. 활성 규칙 일치 검사
7. 동일 창이 grace를 만족하는지 확인
8. warning 표시
9. 취소 가능 시간 대기
10. Focus, emergency, Dragged 상태 재검사
11. fresh snapshot 획득
12. hwnd, 규칙과 보호 여부 재검사
13. Kick impact frame 확인
14. `ShowWindow(SW_MINIMIZE)` 1회
15. cooldown 기록

## 기본 보호 대상

- 앱 자신의 PID와 모든 window label
- Windows 보안/UAC/잠금 화면
- 작업 관리자와 핵심 shell 창
- 파일 열기/저장 대화상자
- 제목/프로세스/권한을 확인할 수 없는 창
- 전체 화면에 가까운 창
- 화상회의와 원격 데스크톱 기본 목록
- 사용자가 추가한 보호 프로세스

코드 기본 보호 목록은 UI에서 해제할 수 없다.

## 취소 조건

- 전경 창 변경
- 창 종료 또는 이미 최소화
- Focus 종료, 휴식, pause 또는 stop
- 펫 Dragged/Thrown
- emergency stop
- 개입 설정 또는 규칙 비활성화
- renderer 응답 실패
- 보호 여부 변경

## 권한 차이

관리자 권한 프로세스에 접근하지 못해도 앱을 관리자 권한으로 다시 실행하도록 자동 유도하지 않는다. 해당 창을 보호 대상으로 처리하고 사용자가 이해할 수 있는 비민감 오류 범주만 표시한다.

## 로그 허용 예시

```text
INFO intervention cancelled reason=foreground_changed rule_id=<uuid>
WARN window inspection skipped reason=access_denied
INFO minimize requested result=failed category=permission
```

## 로그 금지 예시

```text
title="사용자의 실제 창 제목"
process_path="C:\\Users\\..."
rule_text="사용자가 입력한 문자열"
```

## 긴급 중지 불변 조건

`emergency_stopped == true`인 동안 다음이 참이어야 한다.

- monitor가 새 intervention을 만들지 않음
- pending warning/Kick이 취소됨
- `WindowMinimizer`가 호출되지 않음
- 펫/카드/말풍선이 숨겨짐
- 실행 중 timer가 paused
- tray의 재시작 기능은 남음
