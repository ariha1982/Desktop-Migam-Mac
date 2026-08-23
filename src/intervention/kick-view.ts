import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { PhysicalPosition, getCurrentWindow } from "@tauri-apps/api/window";
import kickImage from "../../images/characters/gamjabot/extra/frames/kick/00.png";
import type { InterventionRequest } from "../contracts";

const FLIGHT_MS = 760;

function nextFrame(): Promise<number> {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}

function easeOutCubic(value: number): number {
  return 1 - (1 - value) ** 3;
}

export async function mountKick(root: HTMLElement): Promise<() => void> {
  const shell = document.createElement("main");
  shell.className = "kick-shell";
  const image = document.createElement("img");
  image.className = "kick-character";
  image.src = kickImage;
  image.alt = "방해 창을 차는 네모 캐릭터";
  shell.append(image);
  root.replaceChildren(shell);

  const effectWindow = getCurrentWindow();
  let activeId: number | undefined;
  let disposed = false;

  const run = async (request: InterventionRequest): Promise<void> => {
    activeId = request.interventionId;
    image.classList.remove("impact");
    image.classList.add("flying");
    await effectWindow.setPosition(new PhysicalPosition(request.startX, request.y));
    const startedAt = performance.now();
    let lastMoveAt = 0;
    while (!disposed && activeId === request.interventionId) {
      const timestamp = await nextFrame();
      const progress = Math.min(1, (timestamp - startedAt) / FLIGHT_MS);
      if (timestamp - lastMoveAt >= 24 || progress === 1) {
        const x = Math.round(
          request.startX + (request.impactX - request.startX) * easeOutCubic(progress),
        );
        await effectWindow.setPosition(new PhysicalPosition(x, request.y));
        lastMoveAt = timestamp;
      }
      if (progress >= 0.82) image.classList.add("impact");
      if (progress >= 1) break;
    }
    if (disposed || activeId !== request.interventionId) return;
    image.classList.remove("flying");
    try {
      await invoke<boolean>("complete_intervention", {
        interventionId: request.interventionId,
      });
    } catch {
      await invoke("cancel_intervention", {
        interventionId: request.interventionId,
      }).catch(() => undefined);
    }
    if (activeId === request.interventionId) activeId = undefined;
  };

  const unlistenStart = await listen<InterventionRequest>(
    "focus://intervention-start",
    (event) => void run(event.payload),
  );
  const unlistenCancel = await listen<number>("focus://intervention-cancel", (event) => {
    if (activeId === event.payload) activeId = undefined;
    image.classList.remove("flying", "impact");
  });

  return () => {
    disposed = true;
    activeId = undefined;
    unlistenStart();
    unlistenCancel();
  };
}
