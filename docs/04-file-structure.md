# 04. 파일 구조

## 목표 구조

```text
desktop-pet/
├─ package.json
├─ vite.config.ts
├─ tsconfig.json
├─ index.html
├─ README.md
├─ docs/
│  ├─ architecture.md
│  ├─ manual-test-checklist.md
│  └─ privacy.md
├─ public/icons/tray.ico
├─ src/
│  ├─ main.ts
│  ├─ contracts.ts
│  ├─ shared/
│  │  ├─ api/commands.ts
│  │  ├─ api/events.ts
│  │  ├─ api/contracts.ts
│  │  └─ styles/tokens.css
│  ├─ pet/
│  │  ├─ pet-app.ts
│  │  ├─ pet-sprite.ts
│  │  ├─ speech-bubble.ts
│  │  ├─ pet-interaction.ts
│  │  ├─ spriteResolver.ts
│  │  └─ pet.css
│  ├─ timer/
│  │  ├─ timer-app.ts
│  │  ├─ timer-controls.ts
│  │  └─ timer.css
│  ├─ settings/
│  │  ├─ settings-app.ts
│  │  ├─ pet-settings-section.ts
│  │  ├─ pomodoro-settings-section.ts
│  │  ├─ focus-guard-section.ts
│  │  ├─ distraction-rule-editor.ts
│  │  ├─ safety-section.ts
│  │  └─ settingsReducer.ts
│  └─ card/card-app.ts
└─ src-tauri/
   ├─ Cargo.toml
   ├─ tauri.conf.json
   ├─ capabilities/
   │  ├─ pet.json
   │  ├─ timer.json
   │  └─ settings.json
   ├─ resources/
   │  ├─ characters/placeholder/
   │  │  ├─ manifest.json
   │  │  ├─ idle/
   │  │  ├─ walk/
   │  │  ├─ dragged/
   │  │  ├─ thrown/
   │  │  ├─ kick/
   │  │  └─ speak/
   │  └─ cards/
   └─ src/
      ├─ main.rs
      ├─ lib.rs
      ├─ app_state.rs
      ├─ error.rs
      ├─ domain/
      │  ├─ behavior.rs
      │  ├─ physics.rs
      │  ├─ pomodoro.rs
      │  ├─ distraction.rs
      │  ├─ settings.rs
      │  ├─ character.rs
      │  └─ events.rs
      ├─ application/
      │  ├─ behavior_scheduler.rs
      │  ├─ motion_service.rs
      │  ├─ pomodoro_service.rs
      │  ├─ distraction_monitor.rs
      │  ├─ intervention_service.rs
      │  ├─ idle_monitor.rs
      │  └─ settings_service.rs
      ├─ infrastructure/
      │  ├─ clock.rs
      │  ├─ filesystem.rs
      │  ├─ random.rs
      │  └─ windows/
      │     ├─ foreground_window.rs
      │     ├─ window_minimizer.rs
      │     ├─ protected_window.rs
      │     ├─ idle_time.rs
      │     └─ monitor_layout.rs
      ├─ presentation/
      │  ├─ commands.rs
      │  ├─ pet_commands.rs
      │  ├─ timer_commands.rs
      │  ├─ settings_commands.rs
      │  ├─ tray.rs
      │  ├─ hotkey.rs
      │  ├─ windows.rs
      │  └─ emitter.rs
      └─ tests/
         ├─ distraction_flow.rs
         ├─ emergency_stop.rs
         └─ settings_recovery.rs
```

## 처음 생성할 최소 파일

첫 4시간에는 아래만 만든다.

1. `src-tauri/src/lib.rs`
2. `src-tauri/src/app_state.rs`
3. `src-tauri/src/domain/settings.rs`
4. `src-tauri/src/application/settings_service.rs`
5. `src-tauri/src/presentation/windows.rs`
6. `src-tauri/src/presentation/tray.rs`
7. `src-tauri/src/presentation/hotkey.rs`
8. `src/main.ts`
9. `src/pet/pet-app.ts`
10. `src/settings/settings-app.ts`

나머지는 해당 기능을 구현할 때 생성한다. 빈 골격 파일을 한꺼번에 만들지 않는다.

## 파일 책임 원칙

- 한 파일에 command, Win32 호출과 도메인 판정을 섞지 않는다.
- `commands.rs`는 등록만 하고 실제 로직은 service로 보낸다.
- `contracts.ts`는 Rust DTO의 프런트 표현만 담는다.
- CSS에 행동 규칙을 넣지 않는다.
- manifest path를 코드에 여러 번 하드코딩하지 않는다.
- 테스트 fake는 운영 adapter와 같은 trait을 구현한다.
