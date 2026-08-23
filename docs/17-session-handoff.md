# 세션 인수인계

최종 갱신: 2026-08-24

이 문서는 다음 작업 세션이 가장 먼저 확인하는 단일 인수인계 기록이다. 작업을 끝낼 때마다 오래된 내용을 남기지 말고 현재 상태로 갱신한다.

## 현재 목표

macOS 전용 별도 레포 `Desktop-Migam-Mac`에서 기존 Desktop Pet MVP를 macOS용으로 마이그레이션한다. 네이티브 어댑터와 Apple Silicon `.app`·DMG 빌드까지 완료했고 일반 사용자 화면의 동작·권한 수동 검증이 남았다.

## 최근 macOS 마이그레이션 작업 요약

- Windows 원본 레포에서 `src`, `src-tauri`, `images`, `pack`, `running`, `costumes`, `draw-picture`, `scripts`와 Vite/TypeScript/package 파일을 `mac/` 레포로 복사했다.
- 앱 이름을 `Desktop Migam Mac`, npm 패키지명을 `desktop-migam-mac`, Rust crate를 `desktop-migam-mac`/`desktop_migam_mac_lib`, 번들 ID를 `com.migam.desktop.mac`으로 바꿨다.
- `src-tauri/src/infrastructure/mod.rs`가 플랫폼별 구현을 re-export하고, macOS `foreground_window`는 AppKit·CoreGraphics로 전면 앱과 최상위 창을 읽는다.
- 최소화는 fresh window ID를 다시 검증한 뒤 macOS Accessibility의 focused window에 `AXMinimized`를 설정한다. 손쉬운 사용 권한이 없으면 `AccessDenied`로 안전하게 건너뛴다.
- `system_metrics`는 sysinfo로 macOS 전체 CPU·메모리를 읽고 기존 750ms cache와 CPU smoothing을 유지한다.
- 전역 긴급 중지를 macOS `Cmd+Shift+F12`로 등록하고 UI 안내를 macOS 용어로 변경했다.
- 감자봇 원본에서 `.icns`를 생성해 번들 설정에 연결했다.
- macOS에서는 `transparent: true`만으로 WKWebView 배경이 투명해지지 않아 흰 사각형이 표시됐다. `app.macOSPrivateApi: true`와 투명 창별 `backgroundColor: #00000000`을 추가했다.
- 사진 배달 감자펫이 macOS WebKit에서 보이지 않아 런타임 `blob:` CSS 배경을 제거하고 투명화 결과를 600×200 canvas에 직접 그려 4프레임을 재생하도록 변경했다.
- 집중 보호 설정에 화면 기록·손쉬운 사용 권한 상태와 요청 버튼을 추가했다. `NAVER Whale`/`Whale` 및 Windows/macOS Chrome·Edge 실행 이름은 제한된 알려진 별칭으로 정규화한다.
- 개입 직전 전경 window ID·PID·규칙 일치 재검증을 유지한다. Kick 창이 전면 앱을 점유할 때는 그 아래 최상위 외부 창을 재검증한다.
- 후속 사용자 검증에서 Whale의 CoreGraphics frame과 Accessibility frame 차이 때문에 정확한 창을 찾지 못하는 문제가 확인됐다. `AXFocusedWindow`와 `AXWindows`의 각 AX 창에서 `_AXUIElementGetWindow`로 실제 `CGWindowID`를 얻어 대상 ID와 직접 대조하며, 해당 API가 실패하는 환경에서만 8px geometry 비교를 보조 경로로 사용한다.
- Safari 제목 규칙이 계속 감지되지 않아 전경 감지도 `AXFocusedWindow`에서 제목과 `CGWindowID`를 한 쌍으로 읽도록 변경했다. CoreGraphics 목록의 첫 창이나 비어 있는 `kCGWindowName`에 의존하지 않고 실제 포커스 창을 선택하며, AX 조회 실패 시에만 기존 CoreGraphics 제목을 사용한다.
- 설정의 감지 상태는 집중 비활성, 전경 창 조회 실패, 보호 창, 제목 조회 실패, 규칙 불일치, 일치 성공을 구분해 실제 제목이나 경로를 노출하지 않고 표시한다.
- 설정창 자체가 전경이 되면 앱 자체 보호 판정으로 Safari 후보와 유예가 취소되던 문제를 수정했다. 평상시 polling도 앱 PID를 제외하고 그 뒤의 외부 전경창을 읽으며, 전체 화면 보호는 일반 시스템/자체 창 보호와 별도 상태로 표시한다.
- Kick 웹뷰의 위치 이동이나 이벤트 처리가 실패해도 Rust가 1.2초 뒤 pending 개입을 다시 fresh 검증하고 완료한다. 설정 화면은 `minimized`, 권한 거부, 대상 변경, 검사 실패 결과를 비민감 문구로 표시한다.
- 집중 보호 설정 저장은 권한 프롬프트를 다시 호출하지 않고 현재 권한 상태만 조회한다. 권한 요청은 사용자가 `권한 설정` 버튼을 누를 때만 실행한다.
- `npm install`은 샌드박스 DNS 차단 뒤 네트워크 승인으로 성공했다.
- `npm test`: 프런트 25개 테스트 통과.
- `npm run typecheck`: 통과.
- `npm run build`: Vite production build 통과.
- Rust 1.98 stable을 설치했고 현재 `cargo test` 51개, rustfmt, Clippy `-D warnings`가 통과했다.
- `npm run tauri -- build --no-bundle`과 전체 bundle build가 통과했다.
- `Desktop Migam Mac.app`과 `Desktop Migam Mac_0.1.0_aarch64.dmg`를 생성했고 번들 앱 프로세스 실행을 확인했다.

## 최근 투두리스트·뽀모도로 작업 요약

- Rust `TodoState`에 CRUD·단일 집중 선택·전체 완료 1회 잠금과 재발동 규칙을 구현했고 단위 테스트로 고정했다.
- 앱 데이터의 `todo.json`을 원자 저장하며 손상 파일은 별도 보존한 뒤 빈 목록으로 안전하게 복구한다.
- Focus 시작 시 선택 ID·제목을 snapshot으로 고정한다. 정상 종료에서만 `완료하기`·`계속하기`·`다음 집중`을 표시하며 Skip·Stop은 항목을 변경하지 않는다.
- `timer`는 펫 위의 작은 시간 말풍선으로 되돌렸고, 펫 우클릭 메뉴의 `투두리스트`로 460×640 독립 `todo` 창을 연다.
- 독립 창은 Todoist·Microsoft To Do·투두메이트·Notion의 빠른 입력과 오늘 목록 구조만 참고한 자체 UI이며 추가·수정·삭제·완료·집중 선택과 완료 항목 숨기기를 제공한다.
- 독립 투두 창의 둥근 카드형 외형은 제거했다. 설정창·펫 우클릭 메뉴와 같은 WinDbg형 상단 메뉴와 리본, `0:000>` 명령줄, 각진 필드·목록, 파란 상태바로 디자인을 통일했다.
- 완료 직후 체크 결과를 볼 수 있도록 500ms 뒤 미완료/완료 그룹을 재정렬한다.
- 마지막 미완료 항목을 직접 완료할 때 `todo://all-completed`가 한 번 발생하며 감자펫은 4.5초 점프·광원 축하를 실행한다.
- 2026-08-23 기준 Rust 43개·프런트 25개 테스트, rustfmt, Clippy, TypeScript 검사와 production build가 통과했다.

## 최근 앱 아이콘 작업 요약

- 배포 빌드를 막던 미추적 `images/characters/gamjabot/final/spritesheet-extended.webp` 참조를 Git에 포함된 `references/base-spritesheet-extended.png`로 교체했다. 2026-08-23 NSIS 빌드가 통과했고 `src-tauri/target/release/bundle/nsis/Desktop Pet MVP_0.1.0_x64-setup.exe`가 생성됐다.
- `bundle.windows.nsis.installerIcon`과 `uninstallerIcon`을 `images/app/icon.ico`로 지정해 NSIS 기본 다운로드 아이콘 대신 청록색 감자봇 얼굴이 표시되도록 했다.
- release `main.rs`에 Windows GUI subsystem 속성을 적용했다. 새 release EXE의 PE subsystem 값 2를 확인했으며 설치판 실행 시 검은 터미널 창이 더 이상 생성되지 않는다.
- 독립 투두 창의 WinDbg 분위기는 명령줄·각진 패널·파란 상태바로 유지하되, 클릭할 수 없는 File/Home/Focus/View 메뉴와 장식용 리본 및 내부 가짜 닫기 표시는 제거했다.
- 사용자 제공 청록색 감자봇 얼굴에서 좌측 위 숫자와 바깥 흰 여백을 제외하고 투명 512×512 원본 `images/app/icon-source.png`를 만들었다.
- Tauri 아이콘 생성기로 다중 해상도 Windows `images/app/icon.ico`를 다시 만들었고 release 실행 파일에서 새 아이콘을 추출해 확인했다. 제목 표시줄과 작업표시줄 아이콘은 앱을 완전히 종료하고 재실행해야 바뀐다.
- 기존 `tauri dev`가 오후 7:45의 구형 debug 실행 파일을 다시 사용해 보라색 아이콘과 최신 프런트/구형 설정 계약 불일치가 함께 나타났다. 실행 중 프로세스를 종료하고 해당 패키지의 debug 산출물을 정리한 뒤 완전 재빌드했으며, 새 debug 실행 파일에서 감자봇 아이콘을 직접 추출해 확인했다.
- 개발 중 Rust 재컴파일과 WebView 재로드가 겹치면 `state not managed ... get_bootstrap_state`가 간헐적으로 발생했다. `src/tauri/invoke-when-ready.ts`가 이 오류에만 100ms 간격 최대 30회 재시도하며 설정·펫 코스튬·타이머·GAMCHA·우클릭 메뉴의 최초 상태 조회에 적용된다.

## 최근 GAMCHA 인벤토리 작업 요약

- 전체화면 하단을 가리던 `OUTFIT` 선택·조절 바를 제거하고 기존 GAMCHA 창 안에 `GAMCHA`와 `INVENTORY` 탭을 분리했다.
- 인벤토리는 실제 보유 코스튬만 카드로 표시하며 등급 색, 현재 착용 배지, 선택 상태, 상세 이미지와 착용·기본 모습 전환을 제공한다.
- GAMCHA 창은 펫이 있는 모니터의 전체 해상도가 아니라 Windows 작업 영역에 맞춘다. 화면 가장자리 연출은 유지하면서 작업표시줄은 덮지 않는다.
- 뽑기 버튼은 아래쪽 코스튬 궤도와 겹치지 않도록 작업 영역 하단에서 최소 128px 위에 두고 별도 UI 층으로 올렸다. 한글 버튼은 Segoe UI Variable Display·Segoe UI·맑은 고딕 순으로 표시한다.
- 화면 외곽을 돌던 코스튬 32개는 DOM 생성부터 제거했다. 추첨 중에는 중앙 코스튬만 빠르게 바뀌며 80~94% 범위로 살짝 축소·복귀하는 펄스를 반복한다.
- 로고·결과 스테이지·뽑기 버튼은 화면 중앙의 최대 500px 세로 묶음 안에 배치한다. 스테이지를 고정 좌표에서 상대 배치로 바꿔 당첨 reveal의 scale이 중앙 정렬 transform을 덮어쓰며 아이템이 대각선으로 이동하던 문제도 제거했다.
- 모서리에 흩어져 있던 티켓 수·GAMCHA/INVENTORY 탭·닫기 버튼은 중앙 결과 바로 위의 단일 툴바로 합쳤다. 결과 스테이지 상단은 등급을 왼쪽, `NEW/COLLECTION`을 오른쪽에 작은 글씨로 고정해 서로 겹치지 않는다.
- 코스튬별 X·Y·크기와 위치 초기화는 선택한 카드의 상세 패널 안에서만 표시되며 기존 영구 저장 명령을 그대로 사용한다.
- 오른쪽 상세 미리보기는 아이템 단독 이미지가 아니라 기존 `createPetSprite()`의 Idle 감자펫과 코스튬 오버레이를 그대로 합성한다. 슬라이더 입력 즉시 미리보기를 갱신하고 120ms debounce 뒤 기존 저장 명령으로 영구 반영한다.

## 최근 사진 배달 작업 요약

- 루트 `draw-picture/`의 `191227743` 계열 4장은 실제로 배달할 사진이며, `images/characters/gamjabot/extra/photo-delivery/photos/`에 용도별 이름으로 복사했다. `191554191` 계열 4장은 당기기 포즈 참고 원본으로만 남겼다.
- built-in imagegen으로 기존 감자봇 선화와 맞춘 가로 4프레임 당기기 스트립을 생성해 `images/characters/gamjabot/extra/photo-delivery/gamjabot-pull-strip.png`에 포함했다.
- 생성기가 투명 알파 대신 흰 배경을 출력했으므로 `src/pet/photo-delivery-view.ts`가 바깥 테두리에서 연결된 밝은 픽셀만 Canvas flood-fill로 투명화한다. 검은 외곽선 안쪽의 흰 얼굴·몸통은 보존한다.
- 배달 중 `photo-delivery`는 현재 펫 모니터의 작업 영역만 덮는 투명·비포커스·마우스 입력 통과 창이다. 목적지는 중앙을 제외하고 좌·우 화면 가장자리 중 하나를 고른 뒤 32~122px 안쪽과 세로 전체 범위에서 무작위로 뽑는다.
- 왼쪽 가장자리에 놓을 때는 왼쪽 화면 밖에서, 오른쪽 가장자리에 놓을 때는 오른쪽 화면 밖에서 등장한다. 왼쪽 진입은 사진·펫 표시 순서와 감자펫 방향을 반전해 사진을 화면 안쪽으로 끌고 들어온다.
- 사진·펫 순서는 flex-direction 반전에 의존하지 않는다. 왼쪽 출발은 `사진(order 0)–펫(order 1)`, 오른쪽 출발은 `펫(order 0)–사진(order 1)`으로 명시해 펫이 항상 이동 방향 앞쪽에서 끌도록 고정한다.
- 목적지의 세로 위치는 출발 전에 한 번만 정하며 시작과 도착에 같은 Y 좌표를 사용한다. 전진·밀림 구간은 X축에만 적용되어 사진과 펫이 이동 중 내려가거나 올라가지 않는다.
- 평상시 2~4분마다 사진 4장 중 하나를 무작위 배달한다. Focus뿐 아니라 짧은 휴식·긴 휴식·일시정지 중에도 시작하지 않으며 긴급 중지 시 즉시 숨긴다.
- 펫 우클릭 WinDbg 메뉴에 `사진 배달 테스트`가 있다. 타이머가 `READY`일 때 선택하면 즉시 확인할 수 있다.
- Desktop Goose처럼 힘들어 보이도록 18초 동안 전진·조금 밀림·버팀을 반복한다. 당기는 4자세를 900ms로 재생하고 몸 기울기와 사진 흔들림을 함께 사용한다.
- 사진은 원본 비율을 유지하며 약 300~520px 범위로 키운다. 외곽 제목 표시줄·테두리·여백과 보이는 `X`가 없으며, 사진 우측 상단 46×46px의 투명 영역을 누르면 닫힌다.
- 도착 후 사진은 무작위 목적지에 남고 감자펫만 3.5초 동안 왼쪽으로 퇴장한다. 감자펫 퇴장 뒤 기존 128×128 펫을 복원하고 배달 창은 사진의 실제 위치·크기로 축소한다.
- 사진은 자동으로 사라지지 않으며 사용자가 사진 위의 `X`를 눌러야 닫힌다.
- 자동 검사: 프런트 25개, Rust 38개, TypeScript, Vite production build, rustfmt, Clippy `-D warnings` 통과. 실제 WebView2 투명도·DPI·다중 모니터는 수동 확인이 필요하다.

## 최근 시스템 반응 작업 요약

- Windows 전체 CPU와 메모리 사용률을 1초 단위로 읽고 설정에서 `사용 안 함`, `CPU`, `메모리`, `CPU와 메모리 중 높은 값`을 선택한다.
- 알림 영역에는 CPU 파랑·MEM 빨강 감자봇 아이콘 두 개가 표시된다. 각 아이콘의 굵은 하단 막대가 사용률에 비례하고 툴팁에 정확한 퍼센트와 상태가 나온다.
- 선택된 사용률 0~19%는 기존 걷기, 20~39%는 `running/alert`, 40~59%는 `running/medium`, 60~79%는 `running/fast`, 80~100%는 `running/extreme`의 각 4프레임을 사용한다.
- 새 달리기 원본은 오른쪽 방향이며 왼쪽 이동 시 캐릭터와 코스튬을 함께 반전한다. 단계별 이동·프레임 속도는 1.0/1.15/1.35/1.65/2.1배다.
- 시스템 반응을 켜면 시작 대기와 목적지 사이 휴식 없이 계속 달린다. `사용 안 함`에서만 기존 Idle과 랜덤 휴식을 사용한다.
- 설정의 `시스템 반응 기준` select는 다른 입력과 동일한 각진 WinDbg 테두리, 높이, Consolas 글꼴과 키보드 포커스 표시를 사용한다.

## 현재 상태

- Tauri 2 + Rust + Vite + 순수 TypeScript 기반 구축이 완료되어 있다.
- `pet`, `card`, `timer`, `todo`, `settings`, `gamcha` 창과 트레이, 설정 저장·복구, 전역 긴급 중지가 구현되어 있다.
- 집중 시간이 자연스럽게 끝날 때만 GAMCHA 티켓 1장이 지급된다. Skip과 Stop은 보상하지 않는다.
- GAMCHA 티켓·누적 추첨·보유 코스튬은 앱 데이터 디렉터리의 `gamcha.json`에 저장되고 손상 파일은 별도로 보존한다.
- `pack/manifest.json` 159종 중 `default` 3종을 제외한 156종이 실제 추첨 후보이다.
- 확률은 Common 60%, Rare 25%, Epic 10%, Legendary 4%, Special 1%이다.
- 선택된 등급 안에서는 그 등급의 모든 코스튬을 모을 때까지 중복이 나오지 않는다.
- 자연 집중 완료 시에는 작은 `gamcha-notice` 보상 말풍선만 펫 위에 나타난다.
- 보상 말풍선을 클릭하면 펫이 위치한 모니터 크기의 투명 `gamcha` 오버레이가 열리며, 기존 바탕 화면과 앱 창은 그대로 보인다.
- 추첨은 32개 실제 코스튬이 모니터의 위·오른쪽·아래·왼쪽 네 변을 따라 약 0.9초마다 양방향 회전하고, 결과 카드는 화면 정중앙에 표시된다. 56개 색종이, 중앙 24프레임 감속 셔플과 등급별 공개 효과도 유지한다.
- 중앙 결과는 사각 프레임 없이 경계가 사라지는 원형 등급 광원 위에 아이템·이름만 표시한다. 뽑기 버튼은 작은 반투명 단색 버튼과 상단 3px 무지개 선을 사용한다.
- 타이머 UI polling과 백그라운드 ticker 중 어느 쪽이 먼저 자연 완료를 처리해도 공통 Tick 경로에서 티켓을 지급한다.
- GAMCHA 왼쪽 아래 `OUTFIT` 옷장에서 보유 코스튬 또는 `기본 모습`을 선택해 착용·해제할 수 있다.
- 착용 ID는 `gamcha.json`에 저장되고 `gamcha://equipped` 이벤트로 펫에 즉시 반영되며 재시작 후 복원된다. 보유하지 않은 ID는 Rust에서 거부한다.
- 256×256 코스튬은 128×128로 표시하고 96×104 펫 셀에 `left -16px`, `top -12px`로 중심 정렬한다. 걷기 중 2px 바운스를 적용하고 Hard Impact에서는 숨긴다.
- `pack`의 개별 자산별 runtime alignment 메타데이터는 아직 없으므로 모자·안경·몸 장식별 Windows 육안 확인 후 종류별 미세 보정이 다음 단계다.
- 보라색 placeholder 대신 `images/characters/gamjabot/final/spritesheet-extended.webp`의 실제 감자봇을 표시한다.
- 감자봇 atlas는 1536×2288, 192×208 셀, 8열×11행이며 deterministic validation과 chroma despill이 통과했다.
- 펫 창은 128×128이며, HTML/root/body/app/shell 배경을 모두 투명하게 두어 감자봇 외 픽셀을 그리지 않는다.
- 원본 192×208 셀을 96×104로 표시해 초기 구현보다 감자봇 크기를 정확히 절반으로 줄였다.
- Idle 6프레임, running-right 8프레임, running-left 8프레임을 실제 atlas에서 재생한다.
- 펫은 1.8~4.6초 Idle 후 현재 모니터 작업 영역의 바닥선을 따라 좌우 Walk한다.
- 현재 모니터의 physical pixel work area를 사용하며 작업표시줄을 제외하고, 일반 창은 완전히 안쪽에 유지한다.
- 창이 작업 영역보다 큰 예외 상황에는 최소 24px가 보이도록 경계를 계산한다.
- 창 이동 권한은 `pet` 창에만 별도 capability로 허용한다.
- 감자봇을 왼쪽 버튼으로 누르면 Dragged가 자동 Idle/Walk를 즉시 중단한다.
- 드래그는 Tauri physical cursor 좌표와 pointer capture를 사용하므로 DPI와 창 이동 중에도 같은 잡기 지점을 유지한다.
- 최근 110ms 포인터 표본이 700px/s 이상이면 Thrown으로 전환하고 최대 속력은 2,500px/s로 제한한다.
- Thrown은 중력 2,200px/s², 반동 계수 0.45, 바닥 마찰 0.80을 사용하고 최대 3초 안에 종료한다.
- 드래그와 던지기 중에도 현재 모니터 work area 기준 최소 24px가 화면에 남는다.
- 던지는 동안 atlas의 jumping 5프레임을 재생한다.
- `images/characters/gamjabot/extra/frames`에 Dragged 4장, Thrown 6장, Landing 4장의 전용 투명 PNG가 준비되어 있다.
- 추가 프레임은 모두 192×208 RGBA이며 `extra/manifest.json`에 재생 순서와 권장 프레임 시간이 기록되어 있다.
- `extra/qa/contact-sheet-normalized.png`와 상태별 GIF로 투명 배경, 캐릭터 단독 표시, chroma despill을 육안 검수했다.
- 사용자 제공 ‘땅에 박힌 감자봇’ 그림은 네모 캐릭터와 삽을 제거해 감자봇과 연결된 흙더미만 남겼으며, `extra/frames/hard-impact/00.png`의 192×208 투명 Hard Impact 프레임으로 정리했다.
- Hard Impact의 감자봇 머리 크기는 일반 착지 프레임과 비슷하게 맞추고 흙더미 바닥선을 셀 하단에 정렬했다.
- Hard Impact 프레임의 흙더미 불투명 픽셀은 208px 셀 최하단까지 닿아 작업표시줄과 투명 틈이 생기지 않는다.
- 드래그 중에는 Dragged 4프레임, 비행 중에는 Thrown 6프레임, 일반 종료에는 Landing 4프레임을 재생한다.
- 바닥 충돌 직전 하강 속도가 1,400px/s 이상이면 반동을 멈추고 Hard Impact를 1초 표시한 뒤 Idle로 돌아간다.
- 걷기·던지기 바닥 반동·Hard Impact는 모두 창 전체가 보이는 동일한 작업 영역 최하단을 사용한다. 드래그 중에만 복구를 위해 최소 24px 표시 경계를 허용한다.
- Rust `PomodoroMachine`은 `PomodoroService`를 통해 앱 전역 상태로 연결되어 있다.
- 타이머는 Stopped, Focus, ShortBreak, LongBreak, Paused 상태와 절대 종료 시각 기반 남은 시간을 사용한다.
- 앱 내부 1초 ticker가 타이머 창 표시 여부와 무관하게 단계 완료를 처리한다.
- `get_timer_state`, `start_focus`, `pause_timer`, `resume_timer`, `skip_phase`, `stop_timer` Tauri 명령이 연결되어 있다.
- 타이머 창은 상태 문구, MM:SS 남은 시간, 완료한 집중 횟수와 모든 제어 버튼을 표시한다.
- 타이머 창은 360×280 안에 맞고 문서 스크롤이 없다. 대기에는 시작만, 실행 중에는 일시정지·건너뛰기·중지, Paused에는 재개·중지만 균등 표시한다.
- 설정 변경은 Stopped 타이머에 즉시 적용되며 실행 중 변경은 다음 Stop 이후부터 적용된다.
- 전역 긴급 중지는 실행 중인 타이머를 Paused로 전환한다.
- 설정 창에서 방해 규칙 이름, 사용 여부, 프로세스 파일명, 창 제목 포함 문자열, 유예 시간과 재감지 대기를 추가·삭제·편집할 수 있다.
- 프로세스명/창 제목 중 하나 이상이 필요하고 프로세스 경로 입력은 거부한다. 유예는 5~600초, 재감지 대기는 30~3,600초다.
- 방해 감지는 기본 off이며 활성 규칙이 있어야 사용자가 명시적으로 켤 수 있다.
- `ForegroundWindowSource` trait과 fake, Win32 `GetForegroundWindow`/PID/프로세스 파일명/제목 snapshot 구현이 있다.
- 전경 창 source는 Focus 실행 중이면서 감지가 켜진 경우에만 1초마다 호출된다.
- 감지 결과 이벤트에는 실제 창 제목·프로세스 경로를 넣지 않고 일치 여부와 사용자가 만든 규칙 ID만 넣는다.
- 설정 창은 감지 결과를 `일치 감지됨 · 규칙 이름` 또는 `일치하는 전경 창 없음`으로 표시한다.
- 사용자가 실제 Chrome/YouTube에서 규칙 일치 표시를 확인해 작업 6 감지 게이트를 통과했다.
- 사용자 제공 그림에서 네모 캐릭터만 분리·복원한 투명 Kick 자산을 `extra/frames/kick/00.png`에 추가했다.
- Kick 전용 투명 `card` 창은 220×180이며 대상 모니터의 왼쪽 화면 밖에서 대상 창 중앙까지 760ms 동안 날아온다.
- 동일 hwnd와 rule이 설정된 grace 동안 유지되어야 Kick이 시작된다.
- 충돌 순간 Focus, 긴급 중지, 개입 설정, fresh foreground hwnd와 규칙 일치를 다시 확인하고 `ShowWindow(SW_MINIMIZE)`를 한 번만 호출한다.
- 대상 변경, Focus 종료와 긴급 중지는 pending Kick을 취소한다.
- 앱 자체 PID, 읽을 수 없는 창, 작업 관리자·Explorer 등 핵심 프로세스와 전체 화면 창은 보호한다.
- 개입 성공/실패 뒤 동일 hwnd·rule에 설정된 cooldown을 적용한다.
- 타이머와 설정 창은 WinDbg를 참고한 밝은 회색 도구 UI다. 파란 활성 탭·상태바, 리본형 상단 영역, 얇은 도킹 패널 경계와 Consolas 상태 표시를 사용한다.
- 타이머의 360×280 무스크롤 조건과 상태별 실제 제어 버튼 수는 WinDbg 스타일 변경 뒤에도 유지된다.

## 이번 세션 변경 파일

- `src/gamcha/gamcha-model.ts`
- `src/gamcha/gamcha-model.test.ts`
- `src/gamcha/gamcha-view.ts`
- `src-tauri/src/domain/gamcha.rs`
- `src-tauri/src/application/gamcha_service.rs`
- `src-tauri/src/presentation/tray.rs`
- `src-tauri/capabilities/default.json`

- `src/contracts.ts`
- `src/main.ts`
- `src/intervention/kick-view.ts`
- `src/styles.css`
- `src/pet/motion.ts`
- `src/pet/motion.test.ts`
- `src/pet/physics.ts`
- `src/pet/physics.test.ts`
- `src/pet/sprite.ts`
- `src/pet/tauri-motion-runtime.ts`
- `src/timer/timer-view.ts`
- `src/timer/timer-view.test.ts`
- `src-tauri/src/application/pomodoro_service.rs`
- `src-tauri/src/application/foreground_monitor.rs`
- `src-tauri/src/domain/pomodoro.rs`
- `src-tauri/src/domain/distraction.rs`
- `src-tauri/src/domain/foreground.rs`
- `src-tauri/src/domain/settings.rs`
- `src-tauri/src/infrastructure/mod.rs`
- `src-tauri/src/infrastructure/windows/mod.rs`
- `src-tauri/src/infrastructure/windows/foreground_window.rs`
- `src-tauri/src/app_state.rs`
- `src-tauri/src/presentation/commands.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/capabilities/pet-motion.json`
- `src-tauri/tauri.conf.json`
- `docs/13-progress-board.md`
- `docs/17-session-handoff.md`
- `docs/18-todo-pomodoro-spec.md`
- `images/characters/gamjabot/extra/manifest.json`
- `images/characters/gamjabot/extra/frames/dragged/*.png`
- `images/characters/gamjabot/extra/frames/thrown/*.png`
- `images/characters/gamjabot/extra/frames/landing/*.png`
- `images/characters/gamjabot/extra/frames/hard-impact/00.png`
- `images/characters/gamjabot/extra/frames/kick/00.png`
- `images/characters/gamjabot/extra/qa/contact-sheet-normalized.png`
- `images/characters/gamjabot/extra/qa/previews/*.gif`

## 검증 상태

- GAMCHA 구현 후 `npm test`: 프런트 18개 테스트 통과.
- GAMCHA 구현 후 `cargo test`: Rust 32개 테스트 통과.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`: 통과.
- `npm run tauri -- build --no-bundle`: 통과, release 실행 파일 생성.
- 일반 사용자 Windows에서 자연 완료 보상, 말풍선 위치, 등급별 연출과 재시작 복원은 수동 확인이 필요하다.
- 사용자 확인에서 발견된 `휴식 전환 후 TICKET 0` 경쟁 조건과 추첨 전 깨진 빈 이미지는 수정했고 전체 검사를 재통과했다.
- 타이머 초기 연결 실패 시 `타이머를 불러오지 못했습니다`가 작은 말풍선에서 크게 잘리던 UI를 제거했다. 이제 `재연결 --:--` 상태에서 500ms polling으로 자동 복구한다.
- GAMCHA 2단계 전체화면 변경 후 프런트 18개·Rust 32개·Clippy와 release build를 재통과했다.
- 코스튬 착용 구현 후 프런트 18개·Rust 33개, Clippy와 release build를 통과했다.
- 코스튬을 이름 기준 `head`·`face`·`neck`·`body`·`full` 슬롯으로 분류하고 슬롯별 크기와 기준점을 적용했다. 사용자에게 보인 `rare_025` 연금술사 보석모는 머리 위에 작게 정렬된다.
- 백그라운드 Tick이 일시적으로 실패해도 스레드를 종료하지 않고 다음 주기에 복구한다. Tick 상태 이벤트 전달도 best-effort로 바꿔 집중 중 전경 감시와 Kick이 함께 영구 중단되는 경로를 제거했다.
- 위 수정 후 `npm test` 프런트 18개, TypeScript 검사, 별도 임시 경로 Vite production build, `cargo fmt --check`, Rust 33개 테스트를 통과했다. 실행 중 앱이 기본 `dist`와 `target`을 잠근 상태라 검증 출력만 임시 경로를 사용했다.
- GAMCHA 옷장에 코스튬별 X(-80~80), Y(-80~80), 크기(48~180) 슬라이더와 `위치 초기화`를 추가했다. 선택한 코스튬을 먼저 적용한 뒤 조절하면 펫에 약 120ms 단위로 반영되고 `gamcha.json`에 코스튬 ID별로 저장된다.
- 저장되지 않은 코스튬은 기존 슬롯 기본값을 사용하며, 과거 `gamcha.json`은 새 `costumeAlignments` 필드가 없어도 자동으로 빈 값으로 읽는다.
- 개별 보정 기반 구현 후 프런트 20개, Rust 34개 테스트, TypeScript, Vite production build와 Clippy `-D warnings`를 통과했다.
- Windows `GetSystemTimes`와 `GlobalMemoryStatusEx`로 시스템 전체 CPU·메모리 사용률을 읽는다. CPU는 지수 이동 평균을 적용하고 750ms 이내 중복 요청은 캐시해 설정 창과 펫 창의 동시 조회로 값이 흔들리지 않게 했다.
- 설정의 펫 섹션에서 시스템 반응 기준을 `사용 안 함`, `CPU`, `메모리`, `CPU와 메모리 중 높은 값` 중 고를 수 있고 현재 두 수치를 1초마다 함께 표시한다. 기존 schema 2 설정은 새 필드가 없으면 안전하게 `사용 안 함`으로 읽는다.
- 선택 부하 30% 미만은 이동·프레임 0.7배와 waiting, 60~79%는 1.45배, 80% 이상은 2배, 70% 이상 제자리에서는 busy를 사용한다. 메모리를 감시하는 모드에서 메모리 90% 이상이면 failed 동작을 사용한다. 집중 타이머 동작이 항상 우선한다.
- 시스템 반응 구현 후 프런트 23개, Rust 36개 테스트, TypeScript, Vite production build와 Clippy `-D warnings`를 통과했다.
- 펫 우클릭의 Windows 네이티브 메뉴를 292×430 무장식 `pet-menu` 웹뷰로 교체했다. WinDbg식 제목·탭·명령줄·파란 상태바와 각진 명령 행을 사용하며 타이머 상태/남은 시간, CPU·메모리, GAMCHA 티켓을 표시한다.
- 집중 시작/일시정지/재개/중지 버튼은 현재 타이머 단계에 맞춰 활성화된다. GAMCHA·타이머·설정·긴급 중지·펫 재개·종료도 기존 명령을 그대로 사용한다. Escape, 닫기, 포커스 이탈 시 메뉴가 숨겨진다.
- 우클릭 좌표를 기준으로 팝업을 열되 현재 모니터 작업 영역을 벗어날 경우 좌·상 방향으로 뒤집고 마지막으로 작업 영역에 clamp한다.
- WinDbg 우클릭 메뉴 구현 후 프런트 23개·Rust 36개 테스트, TypeScript와 별도 임시 경로 Vite production build를 통과했다.
- 기존 앱 트레이를 파란 CPU 감자봇 아이콘으로 바꾸고 빨간 MEM 감자봇 아이콘을 추가했다. 두 아이콘 모두 1초마다 갱신되며 10개의 방사형 칸으로 사용률을, 얼굴로 `0~29 여유`, `30~59 보통`, `60~79 바쁨`, `80~100 과부하`를 표현한다.
- 두 아이콘에 마우스를 올리면 `CPU 37% · 보통`, `MEM 82% · 과부하` 형식으로 정확한 수치가 표시된다. 어느 아이콘을 눌러도 기존 펫·타이머·GAMCHA·설정·긴급 중지 메뉴를 사용할 수 있다.
- CPU·MEM 트레이 구현 후 프런트 테스트 23개, Rust 테스트 38개, TypeScript, Vite production build, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`를 통과했다. 실제 Windows 알림 영역 표시와 DPI 축소 결과는 재시작 후 수동 확인이 필요하다.
- Windows 알림 영역에서 방사형 선이 너무 작게 보인다는 사용자 확인에 따라 얼굴 폭을 약 40% 키워 64px 캔버스를 거의 채우고, 게이지를 아이콘 전폭의 굵은 하단 막대로 교체했다. CPU는 파랑, MEM은 빨강이며 막대 길이가 실제 사용률에 비례한다. 변경 후 Rust 38개 테스트와 Clippy를 통과했다.
- 루트 `running/`의 alert·medium·fast·extreme 각 4장, 총 16장(256×256 RGBA, validation `ok: true`)을 실제 이동 애니메이션에 연결했다. 선택된 CPU/MEM/통합 부하가 0~19이면 기존 걷기, 20~39 alert, 40~59 medium, 60~79 fast, 80~100 extreme을 사용한다.
- 새 자산은 오른쪽 방향 원본을 쓰고 왼쪽 이동 시 감자봇과 코스튬을 함께 좌우 반전한다. 이동 중 부하 구간이 바뀌어도 1초 metrics polling에서 즉시 애니메이션을 교체한다. 이동 속도와 프레임 속도도 단계별 1.0/1.15/1.35/1.65/2.1배로 맞췄다.
- 구간별 달리기 연결 후 프런트 테스트 24개, TypeScript 검사와 별도 임시 경로 Vite production build를 통과했으며 16개 PNG가 production asset에 포함됨을 확인했다.
- 사용자 확인에서 목적지 사이마다 기존 Idle 자세로 멈추는 장면이 보였으므로 CPU/MEM/통합 모드에서는 도착 즉시 새 목적지를 뽑아 연속 달리도록 변경했다. 시스템 반응 `사용 안 함`에서만 기존 랜덤 휴식을 유지하며, 설정을 켠 순간 Idle 중이면 바로 이동을 시작한다. 변경 후 프런트 테스트 25개, TypeScript와 production build를 통과했다.

- 감자봇 `final/validation-extended.json`: `ok: true`, 1536×2288, 8×11, 오류·경고 없음.
- 감자봇 `qa/chroma-despill-extended.json`: `ok: true`.
- `npm test`: 펫 경계·음수 모니터 좌표·최소 24px·목적지·이동 테스트 4개 통과.
- `npm run typecheck`: 통과.
- `npm run build`: 통과. 감자봇 WebP가 production asset에 포함됨.
- 50% 크기 변경 후 `npm test`와 `npm run build` 재통과.
- Drag/Throw 구현 후 `npm test`: 이동 5개와 던지기 물리 5개, 총 10개 통과.
- Drag/Throw 구현 후 `npm run typecheck`, `npm run build`: 통과.
- Drag/Throw 구현 후 `npm run tauri -- build --no-bundle`: 통과, release exe 생성.
- `cargo test`: 기존 Rust 테스트 16개 통과.
- `cargo clippy --all-targets -- -D warnings`: 통과.
- `npm run tauri -- build --no-bundle`: 통과, release exe 생성.
- 전체 `tauri build`는 release exe 생성 후 WiX 다운로드가 샌드박스 네트워크 정책으로 차단되어 MSI만 만들지 못했다.
- Codex 샌드박스에서는 Windows GUI 동작을 직접 볼 수 없어 투명 창과 실제 이동은 일반 사용자 세션 확인이 필요하다.
- 크기 변경 뒤 debug Tauri build는 실행 중인 기존 dev 앱이 `target/debug/desktop-pet-mvp.exe`를 잠가 교체하지 못했다. 앱 종료 후 다시 실행하면 된다.
- 추가 동작 이미지 14장은 192×208 RGBA로 정규화되었고, 프레임 검사에서 오류·경고가 없으며 최종 chroma despill을 통과했다.
- Hard Impact 연결 후 `npm test` 11개, `npm run typecheck`, `npm run build`가 모두 통과했다.
- 바닥 경계 정렬 수정 후 자동 테스트는 12개이며 Hard Impact가 화면 아래로 잘리던 원인을 제거했다.
- Pomodoro 구현 후 `cargo test`: Rust 20개 테스트 통과.
- Pomodoro 구현 후 `npm test`: 프런트 16개 테스트 통과.
- Pomodoro 구현 후 `npm run build`, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`: 통과.
- Pomodoro 구현 후 `npm run tauri -- build --no-bundle`: 통과, release 실행 파일 생성.
- 전경 창·규칙 구현 후 `cargo test`: Rust 24개 테스트 통과.
- 전경 창·규칙 구현 후 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`: 통과.
- 전경 창·규칙 구현 후 `npm test`: 프런트 16개 테스트 통과, `npm run build`: 통과.
- 안전 개입 구현 후 `cargo test`: Rust 24개 테스트 통과, grace 후 시작·fresh 재검증·긴급 중지 취소 포함.
- 안전 개입 구현 후 `cargo clippy --all-targets -- -D warnings`: 통과.
- 안전 개입 구현 후 `npm test`: 프런트 16개 테스트 통과, `npm run build`: Kick PNG 포함 production build 통과.
- WinDbg UI 적용 후 `npm test`: 프런트 16개 테스트 통과, `npm run build`: 통과.
- WinDbg raw UI 조정 후 `npm test`: 프런트 16개 테스트 통과, `npm run build`: 통과. 타이머와 설정창을 고전 Win32 디버거의 조밀하고 거친 형태로 조정했다.
- 펫 우클릭에 Windows 네이티브 메뉴를 연결해 타이머·설정·긴급 중지·재시작·종료를 실행할 수 있다.
- 타이머 창은 156×76 투명 무장식 말풍선이며 `상태 · 남은 시간`만 표시하고, 열린 동안 250ms 간격으로 펫 위치를 따라간다. 제어는 펫 우클릭·트레이 메뉴로 이동했다.
- 우클릭 메뉴·말풍선 연결 후 `npm test` 16개, `cargo test` 24개, `cargo clippy --all-targets -- -D warnings`, `npm run build`가 통과했다.
- 집중 타이머가 활성화되면 펫의 자동 이동과 드래그가 중단된다. Focus에서는 컴퓨터 작업 전용 이미지 `images/characters/gamjabot/extra/frames/focused/00.png`를 표시하고, 휴식·일시정지에도 제자리에서 대기한다.
- 집중 전용 이미지는 built-in imagegen으로 생성·배경 추출했으며 1186×1327 RGBA, corner alpha 0을 확인했다.
- Chrome 프로세스 경로 조회가 실패하면 ToolHelp 프로세스 목록에서 파일명을 재확인한다. Kick 창은 `focusable: false`로 대상 창의 전경 상태를 빼앗지 않으며 감지 주기는 250ms다.

## 차단 요소와 위험

- 새 사진 배달 canvas와 권한 허용 후 `CGWindowID` 직접 대조를 적용한 Whale/YouTube 최소화는 일반 사용자 화면에서 최종 확인해야 한다.
- 화면 기록 또는 손쉬운 사용 권한을 새로 허용한 뒤 macOS가 요구하면 앱을 완전히 종료하고 다시 실행해야 한다.
- macOS 화면 기록 권한이 없으면 다른 앱의 창 제목이 비어 제목 기반 규칙이 매치되지 않을 수 있다. 프로세스 이름 규칙은 계속 사용할 수 있다.
- macOS 손쉬운 사용 권한이 없으면 방해 창 최소화는 안전하게 실패한다. 개입은 기본 off다.
- 번들 앱은 Apple Developer ID 서명·공증을 아직 하지 않은 개발 산출물이다.
- 현재 ad-hoc 서명은 지정 요구사항이 빌드별 `CDHash`라 앱을 새로 빌드하면 macOS 손쉬운 사용 권한을 한 번 다시 허용해야 한다. 설정 저장만으로는 더 이상 권한 요청을 띄우지 않는다.
- 투명 WKWebView를 위해 Tauri `macOSPrivateApi`를 사용하므로 Mac App Store에는 배포할 수 없다. GitHub DMG 직접 배포 경로를 유지한다.
- 자동 화면 캡처는 화면 기록 권한 부재로 실패했다. 투명 펫 창, always-on-top, 메뉴 막대 아이콘과 각 유틸리티 창은 사용자 화면에서 확인해야 한다.
- 일반 사용자 데스크톱에서 감자봇 외 배경이 완전히 투명한지 확인해야 한다.
- 단일 모니터에서 작업표시줄을 침범하지 않는지, 가능한 경우 보조 모니터/다른 DPI에서도 확인해야 한다.
- 설정의 `visualScalePercent`는 아직 실제 펫 창/스프라이트 크기에 적용되지 않는다.
- v2 자산의 16방향 blind visual QA는 원래 생성 환경의 ACL 문제로 완료되지 않았지만, 이번 단계에서 사용하는 행 0~2는 deterministic validation과 접촉 시트 육안 확인을 마쳤다.
- pointer capture가 움직이는 투명 Tauri 창에서도 release까지 유지되는지 Windows 실제 입력으로 확인해야 한다.
- Windows 실제 타이머 창에서 키보드 포커스, 버튼 활성 상태, 창 숨김 중 시간 진행과 긴급 중지 일시정지를 확인해야 한다.
- 단계 전환 Windows 알림과 트레이의 현재 상태·제어 메뉴는 아직 연결하지 않았다.
- Win32 전경 창 판독과 설정 창의 감지 상태는 일반 사용자 데스크톱에서 실제 메모장/브라우저로 확인해야 한다.
- 투명 Kick 창의 실제 비행 위치, 포커스를 빼앗지 않는지와 최소화 충돌 타이밍은 Windows 사용자 세션에서 확인해야 한다.
- 펫 Dragged/Thrown 상태를 Rust 개입 서비스에 알리는 연결은 아직 없어 해당 두 상태의 즉시 취소를 보강해야 한다.
- 사진 배달 스프라이트는 런타임 외곽 배경 제거 결과를 일반 사용자 WebView2에서 확인해야 한다. 잔상이 있으면 `isLightBackground` 임계값만 조정한다.
- 사진 배달 중 원래 펫의 이동 루프는 숨은 상태에서도 진행된다. 복원 위치가 어색하면 연출 시작·종료 이벤트로 모션 일시정지를 추가한다.

## 다음 작업

1. 펫 우클릭 `사진 배달 테스트`에서 사진과 canvas 감자펫이 함께 진입하는지 확인한다.
2. 설정의 `권한 설정`에서 화면 기록·손쉬운 사용을 허용하고 필요하면 앱을 재시작한다.
3. 현재 `NAVER Whale` + `YouTube` 규칙으로 Focus를 시작해 감지 표시, 5초 유예, Kick, `방해 창 최소화 완료` 상태와 실제 최소화를 확인한다.
4. 대상 변경과 `Cmd+Shift+F12`에서 pending 개입이 취소되는지 확인한다.
5. 단일 Retina와 가능하면 보조 모니터에서 걷기·드래그·던지기·작업 영역 경계를 확인한다.

## 작업 10 완료 게이트

- 투두 CRUD·선택·완료 상태가 재시작 후 복원된다.
- 미완료 우선 정렬과 완료 직후 500ms 지연 이동이 동작한다.
- Focus 시작 시 선택 ID가 고정되고 정상 완료에서만 사용자에게 완료 여부를 묻는다.
- Skip·Stop·삭제로는 항목을 자동 완료하거나 전체 완료 축하를 발생시키지 않는다.
- 마지막 미완료 항목을 사용자가 완료할 때만 축하가 한 번 발생하고 재발동·재시작 규칙이 지켜진다.
- 긴급 중지가 축하를 즉시 취소하고 움직임 줄이기와 효과음 설정이 적용된다.
- 키보드 조작, Rust·TypeScript 테스트, rustfmt, Clippy와 production build가 통과한다.

## 작업 5 완료 게이트

- Stopped→Focus→ShortBreak/LongBreak→Focus 전환이 동작한다.
- Pause/Resume가 남은 시간을 보존하고 Skip/Stop이 올바르게 동작한다.
- 타이머 창을 숨겨도 종료 시각 기준으로 시간이 진행된다.
- 긴급 중지가 실행 중 타이머를 일시정지한다.
- 상태를 색상뿐 아니라 텍스트로도 표시하고 모든 제어를 키보드로 사용할 수 있다.
- Rust·TypeScript 테스트, Clippy와 production build가 통과한다.

## 작업 3 완료 게이트

- 실제 감자봇만 보이고 창 배경이 완전히 투명하다.
- Idle과 좌·우 Walk가 실제 감자봇 프레임으로 재생된다.
- 단일/다중 모니터 작업 영역을 벗어나 영구 유실되지 않는다.
- 작업표시줄 영역을 침범하지 않는다.
- 자동 테스트, TypeScript 검사와 release exe 빌드가 통과한다.

## 작업 4 완료 게이트

- Dragged가 Idle/Walk를 즉시 중단한다.
- 천천히 놓기와 빠르게 던지기가 구분된다.
- 던진 감자봇이 중력과 화면 경계 반동을 적용받는다.
- 어떤 던지기도 3초 이내 종료되고 화면 밖으로 영구 유실되지 않는다.
- 포인터 속도와 물리 자동 테스트가 통과한다.
