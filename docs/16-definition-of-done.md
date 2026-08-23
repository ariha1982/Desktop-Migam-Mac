# 16. 완료 정의

## P0 기능

- [ ] Windows 11에서 tray와 투명 pet 창으로 시작한다.
- [ ] pet이 work area 안에서 Idle/Walk한다.
- [ ] 다중 모니터와 음수 좌표에서 사라지지 않는다.
- [ ] 클릭, 드래그와 던지기가 동작한다.
- [ ] Thrown이 최대 3초 안에 종료한다.
- [ ] Focus/ShortBreak/LongBreak가 전환된다.
- [ ] pause/resume/skip/stop이 동작한다.
- [ ] 프로세스명/창 제목 규칙을 편집할 수 있다.
- [ ] Focus 중에만 규칙을 감지한다.
- [ ] grace와 cooldown이 적용된다.
- [ ] warning 뒤 fresh snapshot을 재검증한다.
- [ ] 대상이 같을 때만 해당 창을 1회 최소화한다.
- [ ] 설정이 재실행 후 복원된다.

## 안전

- [ ] 개입 기본값이 off다.
- [ ] 기본 규칙 목록이 비어 있다.
- [ ] 앱 자체 창은 보호된다.
- [ ] 알 수 없거나 높은 권한 창은 보호된다.
- [ ] warning 중 Alt+Tab하면 최소화하지 않는다.
- [ ] Focus 종료/휴식/pause에서 개입하지 않는다.
- [ ] Dragged/Thrown 중 개입하지 않는다.
- [ ] 긴급 중지가 pending 개입을 취소한다.
- [ ] 최소화 실패가 앱을 종료시키지 않는다.
- [ ] 실제 창 제목과 사용 이력을 저장하지 않는다.
- [ ] 로그에 제목, 경로, 규칙 문자열이 없다.

## 자산

- [ ] 필수 character 상태가 있다.
- [ ] 누락 상태에 placeholder fallback이 있다.
- [ ] 완성 자산 교체에 Rust 로직 수정이 필요 없다.
- [ ] Kick impact frame이 manifest에 있다.
- [ ] 사용한 외부 자산의 재배포 권리를 확인했다.

## 테스트

- [ ] Rust domain tests 통과
- [ ] intervention integration tests 통과
- [ ] emergency stop integration test 통과
- [ ] settings recovery test 통과
- [ ] TypeScript tests 통과
- [ ] typecheck/lint/rustfmt/clippy 통과
- [ ] Windows 수동 검사표 작성
- [ ] 30분 soak test 통과
- [ ] release candidate 이후 24시간 soak 예약

## 인수인계

- [ ] release build 성공
- [ ] README에 실행/빌드 방법이 있다.
- [ ] 설정 파일 위치가 문서화되어 있다.
- [ ] 긴급 중지와 복구 방법이 문서화되어 있다.
- [ ] 알려진 제한이 문서화되어 있다.
- [ ] 진행판과 마지막 인수인계가 최신이다.

## 최종 판정

다음 조건을 모두 만족하면 `24시간 MVP 완료`로 표시한다.

1. P0 기능 항목 모두 통과
2. 안전 항목 모두 통과
3. 핵심 자동 테스트 통과
4. release build 실행 성공
5. S0/S1 미해결 버그 없음

P1/P2 미완성은 MVP 실패가 아니다. 단, 미완성 코드가 P0 안정성을 해치면 제거하고 알려진 제한으로 남긴다.
