# 미친감자 Desktop Migam Mac

macOS용 Tauri 2 + Rust + 순수 TypeScript 데스크톱 펫입니다. 감자봇이 작업 영역을 돌아다니고, 드래그·던지기·뽀모도로·할 일·GAMCHA·사진 배달·시스템 부하 반응을 제공합니다.

## 주요 기능

- 투명 always-on-top 펫 창과 macOS 메뉴 막대 제어
- Idle, Walk, Dragged, Thrown, Landing, Hard Impact 애니메이션
- 뽀모도로와 할 일 연동, 코스튬 수집·착용
- CPU·메모리 사용률에 따른 움직임과 메뉴 막대 표시
- 집중 중 전면 앱/창 규칙 감지와 사용자 승인 기반 최소화
- `Cmd+Shift+F12` 긴급 중지

창 개입은 기본적으로 꺼져 있습니다. 프로세스 이름만 사용하는 규칙은 별도 권한 없이 동작하지만, 다른 앱의 창 제목을 읽으려면 화면 기록 권한이 필요할 수 있고 창을 최소화하려면 시스템 설정의 개인정보 보호 및 보안 > 손쉬운 사용에서 앱 권한을 허용해야 합니다. 권한이나 창 정보가 불확실하면 앱은 해당 창을 조작하지 않습니다.

## 개발

요구 사항은 macOS, Xcode Command Line Tools, Node.js, Rust stable입니다. Homebrew의 `rustup`을 사용한다면 현재 셸에서 다음 경로를 먼저 추가합니다.

```sh
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
npm install
npm run tauri -- dev
```

## 검증과 빌드

```sh
npm test
npm run typecheck
npm run build

cd src-tauri
cargo test
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cd ..

npm run tauri -- build
```

Apple Silicon 산출물은 `src-tauri/target/release/bundle/macos/Desktop Migam Mac.app`과 `src-tauri/target/release/bundle/dmg/Desktop Migam Mac_0.1.0_aarch64.dmg`에 생성됩니다. 현재 개발 빌드는 서명·공증되지 않았습니다.

프로젝트 구조와 상세 기록은 [개발 문서 색인](docs/README.md), [진행 현황판](docs/13-progress-board.md), [세션 인수인계](docs/17-session-handoff.md)를 참고하세요. 기존 Windows 설계 문서는 원본 구현의 역사 자료로 유지합니다.
