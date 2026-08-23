# Windows Desktop Pet MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current cross-platform CPU tray monitor with a Windows 11-only Tauri 2 desktop pet MVP that roams the desktop, supports drag/throw interactions, runs Pomodoro focus sessions, and minimizes user-configured distraction windows after a kick animation.

**Architecture:** A Rust domain/application core owns deterministic behavior, physics, Pomodoro, rules, and settings. Thin Windows adapters use Win32 APIs for foreground-window inspection, idle time, monitor work areas, and safe minimization. Small Tauri windows render the pet, cards, speech, timer, and settings without a full-screen click-blocking overlay.

**Tech Stack:** Rust 2021, Tauri 2, `windows-sys`, Serde/serde_json, thiserror, TypeScript 5, Vite 7, Vitest, HTML/CSS Canvas-free DOM animation.

**Spec:** `docs/superpowers/specs/2026-08-23-windows-desktop-pet-mvp-design.md`

## Global Constraints

- Target Windows 11 64-bit only.
- Preserve the former remote `main` at `archive/pre-desktop-pet-mvp-2026-08-23`.
- Do not copy former character/reference assets into the new main branch.
- Do not launch WinDbg, Hyper-V, or any other program from image cards.
- Window intervention defaults off and requires at least one user rule.
- Never persist window titles, browser history, keystrokes, or detected app history.
- Revalidate the foreground window immediately before minimizing it.
- Never minimize protected, unknown, elevated, full-screen, meeting, remote-desktop, file-dialog, shell, or application-owned windows.
- Emergency stop must cancel interventions and hide overlays within 300 ms.
- Missing final character art must fall back to bundled neutral placeholder assets through the same character-pack interface.
- New direct dependencies are limited to `windows-sys`, `serde_json`, `thiserror`, Vite, TypeScript, and Vitest unless the user approves more.

---

### Task 1: Replace the repository with a Windows-only Tauri skeleton

**Files:**
- Delete: previous `assets/**`, `docs/PRODUCT_SPEC.md`, `src/**`, `src-tauri/**`, root Cargo files, package files, README, license guidance, and former CI
- Preserve: `docs/superpowers/specs/2026-08-23-windows-desktop-pet-mvp-design.md`
- Preserve: `docs/superpowers/plans/2026-08-23-windows-desktop-pet-mvp.md`
- Create: `.gitignore`
- Create: `README.md`
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `vite.config.ts`
- Create: `index.html`
- Create: `src/main.ts`
- Create: `src/styles.css`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `.github/workflows/check.yml`

**Interfaces:**
- Produces: a buildable Tauri package named `desktop-pet-mvp` with window labels `pet`, `card`, `timer`, and `settings`.
- Produces: npm scripts `dev`, `build`, `test`, `typecheck`, and `tauri`.

- [ ] **Step 1: Remove tracked legacy content on the isolated implementation branch**

Run `git rm -r` only against the tracked paths listed above. Confirm the approved spec and this plan remain present with `git status --short`.

- [ ] **Step 2: Write the minimal package and Tauri manifests**

Use this package shape:

```json
{
  "name": "desktop-pet-mvp",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "test": "vitest run",
    "typecheck": "tsc --noEmit",
    "tauri": "tauri"
  },
  "devDependencies": {
    "@tauri-apps/api": "^2.8.0",
    "@tauri-apps/cli": "^2.8.0",
    "typescript": "^5.9.0",
    "vite": "^7.1.0",
    "vitest": "^3.2.0"
  }
}
```

Use Rust dependencies `tauri`, `serde`, `serde_json`, `thiserror`, and target-specific `windows-sys` with only the Win32 features used by Task 5.

- [ ] **Step 3: Configure non-blocking windows**

Define `pet` as transparent, undecorated, always-on-top, skip-taskbar, initially 256×256. Define `card`, `timer`, and `settings` as initially hidden. Do not create a desktop-sized transparent window.

- [ ] **Step 4: Add a neutral placeholder icon and character pack**

Bundle a simple original neutral geometric icon and six placeholder states. Mark them as temporary in `assets/character/manifest.json`; do not reuse former repository art.

- [ ] **Step 5: Install and verify the skeleton**

Run:

```powershell
npm install
npm run typecheck
npm test
cargo check --workspace
```

Expected: all commands exit 0 and `package-lock.json` is created.

- [ ] **Step 6: Commit**

```powershell
git add -- .gitignore README.md package.json package-lock.json tsconfig.json vite.config.ts index.html src src-tauri assets .github docs
git commit -m "build: replace legacy monitor with Windows pet skeleton"
```

### Task 2: Implement deterministic settings, Pomodoro, rules, and behavior domains

**Files:**
- Create: `src-tauri/src/domain/mod.rs`
- Create: `src-tauri/src/domain/settings.rs`
- Create: `src-tauri/src/domain/pomodoro.rs`
- Create: `src-tauri/src/domain/distraction.rs`
- Create: `src-tauri/src/domain/behavior.rs`
- Test: inline Rust unit tests in each module

**Interfaces:**
- Produces: `Settings::default() -> Settings`, `Settings::validate(self) -> Result<Settings, ValidationError>`.
- Produces: `PomodoroMachine::reduce(&mut self, PomodoroEvent, Instant) -> Vec<PomodoroEffect>`.
- Produces: `DistractionRule::matches(&self, process_name: &str, window_title: &str) -> bool`.
- Produces: `BehaviorMachine::reduce(&mut self, BehaviorEvent) -> Vec<BehaviorEffect>`.

- [ ] **Step 1: Write failing settings tests**

Cover default intervention disabled, empty rule list, 25/5/15 durations, 4-session long-break cadence, 1–120 focus-minute validation, and migration from schema version 1.

- [ ] **Step 2: Implement settings types and validation**

Use serializable structs `Settings`, `PetSettings`, `PomodoroSettings`, `FocusGuardSettings`, and `DistractionRule`. Reject invalid durations and clamp visual scale to 50–200.

- [ ] **Step 3: Write failing Pomodoro transition tests**

Cover `Stopped → Focus → ShortBreak`, fourth focus to `LongBreak`, pause/resume preserving remaining duration, skip, stop, and completion after sleep using a stored deadline.

- [ ] **Step 4: Implement Pomodoro state reduction**

Keep time access outside the domain by passing `Instant`/elapsed values into the reducer. Emit effects for phase notification, persistence, and focus-guard enablement.

- [ ] **Step 5: Write and implement rule tests**

Match process names case-insensitively, title substrings case-insensitively, require all populated conditions, and reject rules with neither condition.

- [ ] **Step 6: Write and implement behavior priority tests**

Assert `Dragged > Thrown > Kick > FocusWarning > CarryCard > Speak/Dance > Chase > Walk > Idle`. Assert dragging cancels a scheduled kick and focus stop clears warning/kick effects.

- [ ] **Step 7: Run and commit**

Run `cargo test --workspace domain`; expect all domain tests to pass.

```powershell
git add -- src-tauri/src/domain src-tauri/src/lib.rs
git commit -m "feat: add pet and focus domain state machines"
```

### Task 3: Implement pet physics and interaction commands

**Files:**
- Create: `src-tauri/src/domain/physics.rs`
- Create: `src-tauri/src/application/interaction_controller.rs`
- Create: `src-tauri/src/application/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: unit tests in `physics.rs` and `interaction_controller.rs`

**Interfaces:**
- Consumes: `BehaviorEvent::{DragStarted, DragEnded, Tick}`.
- Produces: `PhysicsBody::release(velocity: Vec2)`, `PhysicsBody::step(dt: Duration, bounds: Rect) -> PhysicsStep`.
- Produces Tauri commands: `pet_click`, `pet_drag_start`, `pet_drag_move`, `pet_drag_end`.

- [ ] **Step 1: Write failing physics tests**

Test release below threshold lands in place, release above threshold enters thrown motion, velocity clamps, gravity applies, edge collision keeps at least 24px visible, and all motion settles within three seconds.

- [ ] **Step 2: Implement pure physics**

Use `Vec2 { x, y }`, `Rect { left, top, right, bottom }`, and `PhysicsBody`. Keep window movement and Win32 calls out of this module.

- [ ] **Step 3: Write failing interaction tests**

Test click counter increments, seven clicks in ten seconds emits `Special`, the special has a 30-minute cooldown, and drag end computes velocity from timestamped pointer samples.

- [ ] **Step 4: Implement interaction controller and Tauri commands**

Retain only the latest five pointer samples. Clamp release speed before passing it to physics. Emit serializable render events rather than image data.

- [ ] **Step 5: Run and commit**

Run `cargo test --workspace physics interaction` and `npm run typecheck`.

```powershell
git add -- src-tauri/src/domain/physics.rs src-tauri/src/application src-tauri/src/lib.rs
git commit -m "feat: add draggable throwable pet interactions"
```

### Task 4: Build the overlay renderer and character-pack adapter

**Files:**
- Create: `src/bridge.ts`
- Create: `src/pet/pet-view.ts`
- Create: `src/pet/character-pack.ts`
- Create: `src/pet/character-pack.test.ts`
- Create: `src/pet/speech-view.ts`
- Create: `src/card/card-view.ts`
- Modify: `src/main.ts`
- Modify: `src/styles.css`
- Create: `assets/character/manifest.json`
- Create: `assets/cards/windbg.svg`
- Create: `assets/cards/hyper-v.svg`

**Interfaces:**
- Consumes Tauri event `pet://render` with `{ state, frame, facing, x, y, speech? }`.
- Consumes Tauri event `card://show` with `{ kind, expiresAt }`.
- Produces commands from Task 3 and `dismiss_card`.
- Produces: `parseCharacterManifest(input: unknown): CharacterManifest`.

- [ ] **Step 1: Write failing manifest tests**

Test required Idle/Walk/Dragged/Thrown/Kick/Speak states, fallback for recommended states, numeric anchors, transparent asset paths, and rejection of absolute/`..` paths.

- [ ] **Step 2: Implement the manifest adapter**

Map missing Chase/Dance/CarryCard/ClickReact/Special states to Idle or Walk without changing Rust behavior code.

- [ ] **Step 3: Implement pet rendering and input**

Render only the active frame. Use pointer capture for drag. Keep transparent regions click-through where Tauri/Windows hit testing allows; keep the pet window bounded to the pet sprite.

- [ ] **Step 4: Implement speech and safe cards**

Show speech for 2–4 seconds. Display one card at a time for 15 seconds. Cards expose only dismiss/drag behavior and contain no links or Tauri invoke calls that launch programs.

- [ ] **Step 5: Run and commit**

Run `npm test`, `npm run typecheck`, and `npm run build`.

```powershell
git add -- src assets
git commit -m "feat: render character packs speech and safe cards"
```

### Task 5: Add Windows adapters and focus-guard safety

**Files:**
- Create: `src-tauri/src/infrastructure/mod.rs`
- Create: `src-tauri/src/infrastructure/windows/mod.rs`
- Create: `src-tauri/src/infrastructure/windows/foreground_window.rs`
- Create: `src-tauri/src/infrastructure/windows/window_minimizer.rs`
- Create: `src-tauri/src/infrastructure/windows/idle_time.rs`
- Create: `src-tauri/src/infrastructure/windows/monitor_layout.rs`
- Create: `src-tauri/src/application/distraction_monitor.rs`
- Test: adapter-independent tests with fake traits in `distraction_monitor.rs`

**Interfaces:**
- Produces trait `ForegroundWindowSource::current() -> Result<Option<WindowSnapshot>, PlatformError>`.
- Produces trait `WindowMinimizer::minimize(window: WindowId) -> Result<(), PlatformError>`.
- Produces trait `IdleTimeSource::idle_for() -> Result<Duration, PlatformError>`.
- Produces trait `MonitorLayoutSource::work_areas() -> Result<Vec<Rect>, PlatformError>`.
- Produces `DistractionMonitor::tick(now) -> Vec<InterventionEffect>`.

- [ ] **Step 1: Write failing monitor safety tests**

Use fakes to cover grace period, cooldown, focus-only operation, target change cancellation, application-owned window exclusion, protected process exclusion, unknown/elevated window failure, and revalidation immediately before minimize.

- [ ] **Step 2: Implement the monitor against traits**

Poll once per second only during Focus. Store transient `WindowId` and timestamps in memory. Never persist titles or process names observed at runtime.

- [ ] **Step 3: Implement Windows foreground and idle adapters**

Use `GetForegroundWindow`, `GetWindowThreadProcessId`, `GetWindowTextW`, `QueryFullProcessImageNameW`, and `GetLastInputInfo`. Convert errors to `PlatformError` without logging captured titles.

- [ ] **Step 4: Implement protection and minimization**

Reject application-owned HWNDs, shell/security/task-manager/file-dialog/full-screen/meeting/remote-desktop targets and windows that cannot be inspected. Call `ShowWindow(hwnd, SW_MINIMIZE)` only after final identity revalidation.

- [ ] **Step 5: Implement monitor work areas**

Use Windows monitor enumeration and work-area rectangles. Normalize pet coordinates after monitor disconnect and per-monitor DPI changes.

- [ ] **Step 6: Run and commit**

Run `cargo test --workspace distraction_monitor` and then `cargo test --workspace`.

```powershell
git add -- src-tauri/src/infrastructure src-tauri/src/application/distraction_monitor.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add safe Windows distraction intervention"
```

### Task 6: Connect Pomodoro, behavior scheduling, persistence, and emergency stop

**Files:**
- Create: `src-tauri/src/application/pomodoro_service.rs`
- Create: `src-tauri/src/application/behavior_scheduler.rs`
- Create: `src-tauri/src/application/settings_service.rs`
- Create: `src-tauri/src/infrastructure/settings_store.rs`
- Create: `src-tauri/src/infrastructure/windows/hotkey.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: service unit/integration tests using temporary directories and fake clocks

**Interfaces:**
- Produces commands `start_focus`, `pause_timer`, `resume_timer`, `skip_phase`, `stop_timer`, `load_settings`, `save_settings`, `cancel_intervention`, `emergency_stop`, and `resume_pet`.
- Produces events `timer://state`, `pet://render`, `focus://warning`, and `app://emergency-stopped`.

- [ ] **Step 1: Write failing persistence tests**

Test atomic temp-write/flush/rename, corrupt-file quarantine, schema migration, and omission of runtime window data.

- [ ] **Step 2: Implement settings storage**

Store only validated `Settings` in the Windows app-data directory. Debounce writes by 300 ms and flush on normal exit.

- [ ] **Step 3: Write failing scheduler tests**

Test same-speech 10-minute cooldown, random prank frequency caps, one-card limit, idle warning maximum twice per focus session, and no automatic action during Dragged/Thrown.

- [ ] **Step 4: Implement services and event wiring**

Run a single reducer loop for state changes. Drop late animation frames rather than queueing them. Cancel warning/kick effects on break, stop, target change, overlay failure, or emergency stop.

- [ ] **Step 5: Implement emergency hotkey**

Register `Ctrl+Shift+F12` with Win32 `RegisterHotKey`. On activation, hide pet/card/speech, pause the timer, cancel intervention, and keep the tray available. Surface conflicts in settings without terminating the app.

- [ ] **Step 6: Run and commit**

Run `cargo test --workspace` and `npm test`.

```powershell
git add -- src-tauri/src
git commit -m "feat: connect timer settings scheduling and emergency stop"
```

### Task 7: Build timer, settings, tray, and onboarding UI

**Files:**
- Create: `src/timer/timer-view.ts`
- Create: `src/settings/settings-view.ts`
- Create: `src/settings/rule-editor.ts`
- Create: `src/settings/settings-view.test.ts`
- Create: `src/onboarding/onboarding-view.ts`
- Modify: `src/main.ts`
- Modify: `src/styles.css`
- Create: `src-tauri/src/presentation/mod.rs`
- Create: `src-tauri/src/presentation/tray.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes all commands/events from Task 6.
- Produces validated settings payload matching Rust `Settings`.
- Produces tray actions: show/hide pet, start focus, pause/resume, open timer, open settings, clear overlays, emergency stop/resume, quit.

- [ ] **Step 1: Write failing settings UI tests**

Test 1–120 focus minutes, 1–60 short break, 1–90 long break, one populated rule condition, intervention disabled when no rule exists, and explicit confirmation before first enabling minimization.

- [ ] **Step 2: Implement timer UI**

Display Stopped/Focus/ShortBreak/LongBreak/Paused, remaining time, start, pause/resume, skip, and stop. Use text plus color so state is not color-only.

- [ ] **Step 3: Implement settings and rule editor**

Expose behavior frequency, chase/dance/speech/cards toggles, size 50–200%, available accessory, Pomodoro values, idle warning, warning delay, process/title rules, and hotkey status.

- [ ] **Step 4: Implement onboarding and tray**

Explain drag, throw, focus guard, local-only matching, title-based YouTube limitation, and emergency stop. Add tray state/remaining time and all actions from the interface block.

- [ ] **Step 5: Run accessibility and build checks**

Run `npm test`, `npm run typecheck`, and `npm run build`. Manually tab through timer/settings and verify every control has a visible focus indicator and accessible label.

- [ ] **Step 6: Commit**

```powershell
git add -- src src-tauri/src/presentation src-tauri/src/lib.rs
git commit -m "feat: add focus timer settings onboarding and tray UI"
```

### Task 8: Integration verification, documentation, and main publication

**Files:**
- Modify: `README.md`
- Modify: `.github/workflows/check.yml`
- Create: `docs/WINDOWS_TEST_CHECKLIST.md`
- Modify: any files necessary to fix verification failures, limited to the failing behavior

**Interfaces:**
- Consumes: all prior tasks.
- Produces: a tested `agent/windows-pet-mvp-rebuild` commit ready to fast-forward `main`.

- [ ] **Step 1: Add CI**

On Windows, run `npm ci`, `npm test`, `npm run typecheck`, `npm run build`, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo check --workspace`.

- [ ] **Step 2: Write operator documentation**

README must state Windows 11-only support, current placeholder-art status, run/build commands, focus-guard opt-in, emergency hotkey, title-matching limitation, and archive branch location.

- [ ] **Step 3: Run automated verification**

Run every CI command locally. Record actual pass/fail output; do not report success for commands not run.

- [ ] **Step 4: Run Windows smoke tests**

Verify launch, transparent click behavior, drag/throw, Pomodoro transitions, Chrome/Edge title rule, warning cancellation, safe minimization, emergency stop, multi-monitor bounds, sleep/resume, and settings restoration. Record results in `docs/WINDOWS_TEST_CHECKLIST.md`.

- [ ] **Step 5: Inspect repository replacement**

Run `git status --short`, `git diff --stat origin/main...HEAD`, and `git ls-tree -r --name-only HEAD`. Confirm no former art/reference files remain and only intended new files are tracked.

- [ ] **Step 6: Commit final verification changes**

```powershell
git add -- README.md .github/workflows/check.yml docs/WINDOWS_TEST_CHECKLIST.md
git commit -m "docs: finalize Windows desktop pet MVP"
```

- [ ] **Step 7: Publish implementation branch and update main**

Push `agent/windows-pet-mvp-rebuild` first. Verify `archive/pre-desktop-pet-mvp-2026-08-23` still points to `302bc7aa8cc128bf96072133b1d306972f20b819`. Update local `main` by fast-forwarding it to the verified implementation head, then push `main` without force.

- [ ] **Step 8: Verify GitHub state**

Confirm remote `main` points to the verified MVP commit, archive branch still exists, and Actions has started. Report commit hashes and CI URL. Do not delete the implementation or archive branches.
