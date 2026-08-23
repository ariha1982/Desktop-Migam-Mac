# 02. 기술 스택

## 고정 기술

| 영역 | 선택 | 이유 |
|---|---|---|
| 데스크톱 | Tauri 2 | 기존 기준 유지, 트레이와 다중 창 지원 |
| 백엔드 | Rust stable | 상태 머신과 Windows API를 안전하게 분리 |
| 비동기 | Tokio | 취소 가능한 polling과 timer task |
| UI | TypeScript + DOM API | MVP 화면 규모에 맞춘 최소 의존성 UI 구현 |
| 빌드 | Vite | 빠른 개발 반복 |
| 직렬화 | Serde + serde_json | 설정과 command/event DTO |
| 오류 | thiserror + anyhow | 계층별 오류와 상위 전달 |
| 로깅 | tracing | 비민감 진단 로그 |
| Windows API | `windows` crate | 필요한 Win32 함수만 feature 활성화 |
| ID | uuid | 방해 규칙과 개입 ID |

패치 버전은 설치 시 호환되는 버전을 lockfile로 고정한다. 24시간 작업 도중 이유 없는 일괄 업그레이드를 하지 않는다.

## Tauri 기능

- 내장 tray API
- global-shortcut 플러그인
- notification 플러그인: P1
- autostart 플러그인: P2
- path API
- 창별 capability 파일

## 프런트엔드 정책

- 화면별 TypeScript 모듈과 작은 순수 상태 함수로 시작한다.
- 별도 라우터 없이 window label로 화면 루트를 선택한다.
- 고빈도 위치 업데이트는 DOM transform으로 직접 반영한다.
- 기능별 CSS class를 사용한다.
- UI 라이브러리는 기존 프로젝트에 있을 때만 유지한다.
- React는 화면 복잡도가 실제로 커지기 전에는 추가하지 않는다.

## Rust 정책

- 도메인은 Tauri와 Win32에 의존하지 않는다.
- `unsafe`는 `infrastructure/windows` 안에 제한한다.
- 공유 lock을 잡은 채 `await`하지 않는다.
- polling task는 cancellation token으로 종료한다.
- 시간 경과 판정은 `Instant`, 저장 날짜는 wall-clock을 사용한다.

## 추가하지 않을 기술

- 데이터베이스: JSON 설정이면 충분
- Redux/Zustand: 화면과 상태 규모에 비해 불필요
- 웹 서버: 로컬 앱에 필요 없음
- 클라우드/분석 SDK: 개인정보 원칙 위반
- 브라우저 자동화: URL 감시는 MVP 제외
- 게임 엔진: 2D 스프라이트와 간단한 물리에 과도함

## 품질 명령

```powershell
npm run lint
npm run test
cargo fmt --all --check --manifest-path src-tauri/Cargo.toml
cargo clippy --all-targets --all-features --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## 성능 목표

- 대기 평균 CPU 1% 미만
- 대기 RSS 120MB 미만
- 전경 창 확인 1초 간격
- 드래그 반응 50ms 이내
- 긴급 중지 300ms 이내
- Idle/Walk 30fps, Thrown만 필요 시 60fps
