# 미친감자 Desktop Pet MVP

Windows 11 전용 Tauri 2 + Rust + 순수 TypeScript 투명 오버레이 데스크톱 펫 프로젝트입니다. 감자봇이 바탕화면을 돌아다니고, 사용자가 드래그하거나 던질 수 있으며, 뽀모도로 집중 세션과 방해 창 개입 기능을 제공하는 것을 목표로 합니다.

현재 앱 코드는 MVP 골격을 구현 중이며, 캐릭터 원화·애니메이션 결과물과 개발 명세를 함께 보관합니다.

## 저장소 구조

```text
.
├─ src/                         # TypeScript 오버레이 UI
├─ src-tauri/                   # Tauri 2 + Rust 애플리케이션
├─ images/
│  ├─ app/                      # 앱 아이콘과 임시 실행용 캐릭터
│  ├─ characters/
│  │  ├─ gamjabot/              # 감자봇 원본·프레임·아틀라스·QA 자료
│  │  └─ nemo/                  # 네모 원본·프레임·아틀라스·QA 자료
│  └─ references/               # Stitch 캐릭터 디자인 참고 이미지
└─ docs/                        # 제품 명세, 구현 계획, 안전·테스트 문서
```

## MVP 핵심 범위

- 투명 오버레이 펫의 걷기, 드래그, 던지기
- 말하기, 춤, 클릭 이스터에그 등 랜덤 반응
- 사용자 설정 가능한 뽀모도로 타이머
- 집중 중 방해 앱 감지, 발차기 애니메이션 후 대상 창 최소화
- 긴급 중지 단축키와 보수적인 Windows 창 보호 규칙
- 교체 가능한 캐릭터 팩과 후속 `네모` 이스터에그 확장 지점

상세 제품·기술 기준은 [개발 문서 색인](docs/README.md)과 [Windows 데스크톱 펫 MVP 설계](docs/superpowers/specs/2026-08-23-windows-desktop-pet-mvp-design.md)를 참고하세요.

## 개발

```powershell
npm install
. .\scripts\use-project-rust.ps1
npm run tauri -- dev
```

## 검증

```powershell
npm run typecheck
npm test
Set-Location src-tauri
cargo check --workspace
```

집중 방해 창 최소화는 사용자가 명시적으로 활성화해야 하며, 긴급 중지 단축키는 `Ctrl+Shift+F12`를 기준으로 합니다.

## 현재 상태

- 설정·뽀모도로·방해 규칙 도메인 모델: 구현 중
- Windows 오버레이와 창 개입: 후속 구현 필요
- 감자봇·네모 v2 아틀라스: 정적 검증 완료, 최종 시각 QA 필요
