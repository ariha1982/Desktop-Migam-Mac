import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { invokeWhenReady } from "../tauri/invoke-when-ready";

export type TimerPhase = "stopped" | "focus" | "shortBreak" | "longBreak" | "paused";

export interface TimerState {
  phase: TimerPhase;
  remainingSeconds: number;
  completedFocusSessions: number;
}

const phaseLabels: Record<TimerPhase, string> = {
  stopped: "대기", focus: "집중", shortBreak: "짧은 휴식", longBreak: "긴 휴식", paused: "일시정지",
};

export function formatRemaining(totalSeconds: number): string {
  const safeSeconds = Math.max(0, Math.floor(totalSeconds));
  const minutes = Math.floor(safeSeconds / 60);
  const seconds = safeSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function timerControls(phase: TimerPhase): {
  start: boolean; pause: boolean; resume: boolean; skip: boolean; stop: boolean;
} {
  const running = phase === "focus" || phase === "shortBreak" || phase === "longBreak";
  return { start: phase === "stopped", pause: running, resume: phase === "paused", skip: running, stop: phase !== "stopped" };
}

export async function mountTimer(container: HTMLElement): Promise<() => void> {
  container.innerHTML = `
    <main class="timer-panel" aria-label="뽀모도로 타이머">
      <section class="timer-bubble">
        <button class="timer-bubble-close" type="button" aria-label="타이머 닫기">×</button>
        <div class="timer-readout"><p id="timer-phase" class="timer-phase">불러오는 중</p><p id="timer-remaining" class="timer-remaining">--:--</p></div>
        <span id="timer-error" class="timer-error" role="alert"></span>
      </section>
    </main>`;
  const phase = container.querySelector<HTMLElement>("#timer-phase")!;
  const remaining = container.querySelector<HTMLElement>("#timer-remaining")!;
  const error = container.querySelector<HTMLElement>("#timer-error")!;
  let disposed = false;
  const render = (state: TimerState): void => {
    if (!disposed) { phase.textContent = phaseLabels[state.phase]; remaining.textContent = formatRemaining(state.remainingSeconds); }
  };
  container.querySelector(".timer-bubble-close")?.addEventListener("click", () => void invoke("hide_utility_window", { label: "timer" }));
  try { render(await invokeWhenReady<TimerState>("get_timer_state")); }
  catch { error.textContent = "재연결 중"; }
  const unlisten = await listen<TimerState>("timer://state", ({ payload }) => { error.textContent = ""; render(payload); }).catch(() => () => undefined);
  const poll = window.setInterval(() => {
    void invoke<TimerState>("get_timer_state").then(render).catch(() => undefined);
    void invoke("position_timer_bubble").catch(() => undefined);
  }, 500);
  return () => { disposed = true; window.clearInterval(poll); unlisten(); };
}
