# 12. 릴리스 체크리스트

## 기능 동결

- [ ] 20시간 이후 새 기능을 추가하지 않았다.
- [ ] P0 범위가 모두 구현되었다.
- [ ] 미완성 P1/P2는 코드가 아닌 알려진 제한으로 기록했다.
- [ ] 임시 debug UI와 테스트 단축키를 제거했다.

## 품질 게이트

- [ ] TypeScript typecheck 통과
- [ ] ESLint 통과
- [ ] UI 테스트 통과
- [ ] rustfmt check 통과
- [ ] Clippy warning 0
- [ ] Rust 단위·통합 테스트 통과
- [ ] release build 성공
- [ ] 개발 PC에서 release 실행 확인

## 안전 게이트

- [ ] 창 개입 기본 off
- [ ] 빈 규칙 목록
- [ ] 자체 창 보호
- [ ] fresh snapshot 재검증
- [ ] emergency stop 통합 테스트
- [ ] 실제 창 제목/경로 로그 없음
- [ ] 관리자/불명확 창 skip
- [ ] 외부 프로그램 실행 기능 없음

## 자산

- [ ] 필수 캐릭터 상태 존재 또는 placeholder fallback
- [ ] manifest validation 통과
- [ ] tray icon 포함
- [ ] 카드/폰트/음원의 재배포 권리 확인
- [ ] 개발 경로를 가리키는 절대 asset path 없음

## 문서

- [ ] README 실행 방법
- [ ] 설정 위치
- [ ] 긴급 중지 단축키
- [ ] 개인정보 원칙
- [ ] 알려진 제한
- [ ] Windows 수동 검사 결과
- [ ] release artifact 위치와 checksum, 필요 시 기록

## 데모 순서

1. 앱 실행과 tray 확인
2. 펫 Idle/Walk
3. 클릭/드래그/던지기
4. settings에서 방해 규칙 등록
5. 개입 opt-in 활성화
6. 축약 Focus 시작
7. 테스트 대상 창으로 감지/경고/Kick/최소화
8. warning 중 Alt+Tab 취소 시연
9. 긴급 단축키 시연
10. 재실행 후 설정 복원 확인

## 알려진 제한 템플릿

```text
- 사이트 감지는 URL이 아니라 브라우저 창 제목에 의존한다.
- 관리자 권한 창은 접근 권한 차이로 개입하지 않을 수 있다.
- 일부 DPI/그래픽 구성에서 투명 영역 입력 범위가 사각형일 수 있다.
- 자동 업데이트와 코드 서명은 이번 MVP에 포함되지 않는다.
- 24시간 안정성 시험 결과는 release candidate 생성 후 추가한다.
```

## 릴리스 판정

다음 중 하나라도 참이면 외부 배포하지 않고 내부 데모로만 표시한다.

- 잘못된 창 최소화 재현
- emergency stop 누락
- 손상 설정으로 앱 시작 불가
- release build 실패
- 라이선스가 불명확한 자산 포함
- 민감한 창 정보가 로그에 남음
