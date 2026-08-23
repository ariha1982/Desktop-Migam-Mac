# 15. 문제 해결 가이드

## 투명 창이 검게 보임

확인 순서:

1. Tauri 창의 `transparent` 설정
2. body/html 배경이 실제 transparent인지
3. Windows 그래픽 드라이버와 원격 세션 여부
4. decorations/shadow 설정
5. dev와 release 차이

우회: 완전 투명 효과보다 기능 검증을 우선하고 작은 배경 없는 사각 창을 유지한다.

## 아래 창을 클릭할 수 없음

- pet 창 크기가 실제 스프라이트보다 지나치게 큰지 확인
- drag 중에만 입력을 받고 평상시 click-through 전환이 가능한지 검토
- 픽셀 hit-test가 지연되면 hitbox가 작은 사각 창으로 MVP 단순화
- 전체 화면 오버레이는 사용하지 않음

## 드래그 위치가 DPI에서 어긋남

- logical/physical 좌표를 섞지 않았는지 확인
- monitor scale factor를 한 번만 적용했는지 확인
- 내부 좌표를 physical px로 통일
- 100%, 150%, 200% 각각 기록

## 펫이 모니터 밖으로 사라짐

- 가상 데스크톱의 음수 좌표 처리 확인
- screen bounds가 아닌 work area 사용 확인
- 펫 크기를 고려한 clamp 확인
- 모니터 연결 해제 이벤트에서 주 모니터로 복구
- 시작 시 저장 좌표가 현재 구성에 유효한지 검사

## 다른 창이 최소화됨

즉시 개입 기능을 끄고 S0/S1 버그로 처리한다.

확인:

1. warning 전 snapshot과 실행 전 snapshot이 같은 hwnd인지
2. fresh foreground snapshot을 실제로 다시 읽는지
3. stale intervention task가 남았는지
4. emergency/Focus/Dragged cancellation token이 전파되는지
5. 한 intervention에서 minimize가 한 번만 호출되는지

안전 회귀 테스트를 추가하기 전까지 기능을 다시 켜지 않는다.

## 관리자 앱에서 감지 실패

정상적인 안전 실패일 수 있다. 앱을 자동 승격하지 말고 해당 창을 보호한다. 로그에는 `access_denied` 범주만 남긴다.

## 전역 단축키 등록 실패

- 다른 앱이 조합을 사용 중인지 확인
- 등록 결과를 UI에 표시
- 사용자가 대체 조합을 저장할 수 있게 함
- tray의 긴급 중지 메뉴는 항상 유지

## 설정 파일이 자꾸 손상됨

- 같은 디렉터리 tmp write인지 확인
- write/flush 성공 뒤 교체하는지 확인
- 동시에 두 save가 실행되지 않도록 직렬화
- 종료 중 save task가 중단되는지 확인
- 손상 파일을 덮어쓰지 말고 timestamp backup

## CPU 사용량이 높음

- Focus가 아닌데 polling하는 task 확인
- Idle/Walk가 60fps인지 확인
- 위치를 전체 화면 상태로 매 frame 갱신하는지 확인
- 중복 event listener와 interval 확인
- 카드/말풍선의 숨은 animation 확인
- tracing level과 로그 write 빈도 확인

## 종료 후 프로세스가 남음

- cancellation token 전달
- tray와 hotkey 해제
- loop가 cancellation을 확인하는지
- listener unlisten
- lock을 잡은 task가 await 상태인지

## 테스트는 통과하지만 실제 창에서 실패

- fake snapshot에 class/visibility/minimized 조건이 빠졌는지 확인
- Unicode title과 프로세스 경로 처리 확인
- 관리자 권한 차이 확인
- release build의 capability/권한 차이 확인
- 실제 검사를 추가하되 제목은 로그하지 않음

## Tauri dev 중 Vite `EBUSY` 오류

증상:

- `watch '...src-tauri\\target\\...exe'` 경로와 함께 `EBUSY: resource busy or locked`가 표시된다.

대응:

- `vite.config.ts`의 `server.watch.ignored`에 `**/src-tauri/target/**`와 `**/.tools/**`가 있는지 확인한다.
- 실행 중인 dev 프로세스를 종료한 뒤 `npm run tauri -- dev`를 다시 실행한다.
