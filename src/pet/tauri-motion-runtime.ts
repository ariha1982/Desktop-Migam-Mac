import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  availableMonitors,
  currentMonitor,
  cursorPosition,
  getCurrentWindow,
  monitorFromPoint,
  PhysicalPosition,
  primaryMonitor,
  type Monitor,
  type PhysicalSize,
} from "@tauri-apps/api/window";
import {
  advanceToward,
  clampPosition,
  getPositionBounds,
  getVisiblePositionBounds,
  pickHorizontalTarget,
  type Point,
  type PositionBounds,
  type WorkArea,
} from "./motion";
import {
  estimateThrowVelocity,
  isHardFloorImpact,
  stepThrow,
  type PointerSample,
  type ThrowState,
} from "./physics";
import type { PetAnimation, PetSprite } from "./sprite";
import type { SystemMetricsState } from "../contracts";
import {
  resourceIdleAnimation,
  resourceMovementAnimation,
  resourceSpeedMultiplier as speedForMetrics,
  shouldRunContinuously,
} from "./resource-response";

const MOVEMENT_FPS = 30;
const WALK_SPEED_PIXELS_PER_SECOND = 92;
const ARRIVAL_TOLERANCE_PIXELS = 1;
const IDLE_MINIMUM_MILLISECONDS = 1_800;
const IDLE_RANGE_MILLISECONDS = 2_800;
const POINTER_SAMPLE_RETENTION_MILLISECONDS = 140;
const LANDING_ANIMATION_MILLISECONDS = 480;
const HARD_IMPACT_ANIMATION_MILLISECONDS = 1_000;

interface IdleMode {
  kind: "idle";
  untilMilliseconds: number;
}

interface WalkingMode {
  kind: "walking";
  x: number;
  targetX: number;
  groundY: number;
  bounds: PositionBounds;
}

interface DraggedMode {
  kind: "dragged";
  pointerId: number;
  interactionId: number;
  anchorCursor: Point | null;
  anchorWindow: Point | null;
  samples: PointerSample[];
}

interface ThrownMode {
  kind: "thrown";
  throwState: ThrowState;
  bounds: PositionBounds;
}

interface RecoveryMode {
  kind: "landing" | "hard-impact";
  untilMilliseconds: number;
}

interface TimerMode {
  kind: "timer";
  phase: "focus" | "shortBreak" | "longBreak" | "paused";
}

interface TimerSnapshot {
  phase: "stopped" | TimerMode["phase"];
}

type RuntimeMode = IdleMode | WalkingMode | DraggedMode | ThrownMode | RecoveryMode | TimerMode;

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}

function idleMode(nowMilliseconds = performance.now()): IdleMode {
  return {
    kind: "idle",
    untilMilliseconds:
      nowMilliseconds +
      IDLE_MINIMUM_MILLISECONDS +
      Math.random() * IDLE_RANGE_MILLISECONDS,
  };
}

function toWorkArea(monitor: Monitor): WorkArea {
  return {
    x: monitor.workArea.position.x,
    y: monitor.workArea.position.y,
    width: monitor.workArea.size.width,
    height: monitor.workArea.size.height,
  };
}

async function resolveMonitor(): Promise<Monitor | null> {
  const activeMonitor = await currentMonitor();
  if (activeMonitor) return activeMonitor;

  const preferredMonitor = await primaryMonitor();
  if (preferredMonitor) return preferredMonitor;

  return (await availableMonitors())[0] ?? null;
}

function appendPointerSample(
  samples: PointerSample[],
  point: Point,
  timeMilliseconds: number,
): void {
  samples.push({ ...point, timeMilliseconds });
  const oldestAllowedTime = timeMilliseconds - POINTER_SAMPLE_RETENTION_MILLISECONDS;
  while (samples.length > 2 && samples[0].timeMilliseconds < oldestAllowedTime) {
    samples.shift();
  }
}

export function startPetMotion(sprite: PetSprite): () => void {
  const petWindow = getCurrentWindow();
  let active = true;
  let mode: RuntimeMode = idleMode();
  let interactionId = 0;
  let timerActive = false;
  let unlistenTimer: (() => void) | null = null;
  let unlistenTodo: (() => void) | null = null;
  let celebrationTimer: number | undefined;
  let resourceSpeedMultiplier = 1;
  let latestSystemMetrics: SystemMetricsState = {
    cpuPercent: 0,
    memoryPercent: 0,
    mode: "off",
  };
  let lastAnimationFrameAt = performance.now();

  const animationTimer = window.setInterval(() => {
    const now = performance.now();
    if (now - lastAnimationFrameAt >= 120 / resourceSpeedMultiplier) {
      sprite.advanceFrame();
      lastAnimationFrameAt = now;
    }
  }, 30);

  const setMode = (nextMode: RuntimeMode, animation: PetAnimation): void => {
    mode = nextMode;
    sprite.element.dataset.interaction = nextMode.kind;
    sprite.setAnimation(animation);
  };

  const beginDrag = async (event: PointerEvent): Promise<void> => {
    if (event.button !== 0 || timerActive) return;
    event.preventDefault();
    sprite.element.setPointerCapture(event.pointerId);

    const nextInteractionId = ++interactionId;
    const draggedMode: DraggedMode = {
      kind: "dragged",
      pointerId: event.pointerId,
      interactionId: nextInteractionId,
      anchorCursor: null,
      anchorWindow: null,
      samples: [],
    };
    setMode(draggedMode, "dragged");

    const [cursor, windowPosition] = await Promise.all([
      cursorPosition(),
      petWindow.outerPosition(),
    ]);
    if (mode !== draggedMode || draggedMode.interactionId !== interactionId) return;

    draggedMode.anchorCursor = cursor;
    draggedMode.anchorWindow = windowPosition;
    appendPointerSample(draggedMode.samples, cursor, performance.now());
  };

  const finishDrag = async (event: PointerEvent): Promise<void> => {
    if (mode.kind !== "dragged" || event.pointerId !== mode.pointerId) return;
    event.preventDefault();
    if (sprite.element.hasPointerCapture(event.pointerId)) {
      sprite.element.releasePointerCapture(event.pointerId);
    }

    const draggedMode = mode;
    const finishingInteractionId = draggedMode.interactionId;
    const cursor = await cursorPosition();
    appendPointerSample(draggedMode.samples, cursor, performance.now());
    if (
      mode !== draggedMode ||
      finishingInteractionId !== interactionId ||
      !active
    ) {
      return;
    }

    const velocity = estimateThrowVelocity(draggedMode.samples);
    if (!velocity) {
      setMode(idleMode(), "idle");
      return;
    }

    const [windowPosition, windowSize, monitor] = await Promise.all([
      petWindow.outerPosition(),
      petWindow.outerSize(),
      monitorFromPoint(cursor.x, cursor.y),
    ]);
    if (mode !== draggedMode || finishingInteractionId !== interactionId) return;

    const fallbackMonitor = monitor ?? (await resolveMonitor());
    if (!fallbackMonitor) {
      setMode(idleMode(), "idle");
      return;
    }

    setMode(
      {
        kind: "thrown",
        throwState: {
          position: windowPosition,
          velocity,
          elapsedSeconds: 0,
        },
        bounds: getPositionBounds(toWorkArea(fallbackMonitor), windowSize),
      },
      "thrown",
    );
  };

  const updateIdle = async (idle: IdleMode, nowMilliseconds: number): Promise<void> => {
    if (nowMilliseconds < idle.untilMilliseconds) return;

    const monitor = await resolveMonitor();
    if (!monitor || mode !== idle) return;

    const [windowSize, windowPosition] = await Promise.all([
      petWindow.outerSize(),
      petWindow.outerPosition(),
    ]);
    if (mode !== idle) return;

    const bounds = getPositionBounds(toWorkArea(monitor), windowSize);
    const safePosition = clampPosition(windowPosition, bounds);
    const groundY = bounds.maxY;
    const targetX = pickHorizontalTarget(safePosition.x, bounds, Math.random());
    await petWindow.setPosition(
      new PhysicalPosition(Math.round(safePosition.x), Math.round(groundY)),
    );
    if (mode !== idle) return;

    const direction = targetX < safePosition.x ? "left" : "right";
    setMode(
      {
        kind: "walking",
        x: safePosition.x,
        targetX,
        groundY,
        bounds,
      },
      resourceMovementAnimation(latestSystemMetrics, direction),
    );
  };

  const updateWalking = async (
    walking: WalkingMode,
    deltaSeconds: number,
  ): Promise<void> => {
    walking.x = advanceToward(
      walking.x,
      walking.targetX,
      WALK_SPEED_PIXELS_PER_SECOND,
      deltaSeconds * resourceSpeedMultiplier,
    );
    if (mode !== walking) return;

    await petWindow.setPosition(
      new PhysicalPosition(Math.round(walking.x), Math.round(walking.groundY)),
    );
    if (mode !== walking) return;

    if (Math.abs(walking.targetX - walking.x) <= ARRIVAL_TOLERANCE_PIXELS) {
      if (shouldRunContinuously(latestSystemMetrics)) {
        walking.targetX = pickHorizontalTarget(
          walking.x,
          walking.bounds,
          Math.random(),
        );
        const direction = walking.targetX < walking.x ? "left" : "right";
        sprite.setAnimation(resourceMovementAnimation(latestSystemMetrics, direction));
      } else {
        setMode(idleMode(), "idle");
      }
    }
  };

  const updateDragged = async (dragged: DraggedMode): Promise<void> => {
    if (!dragged.anchorCursor || !dragged.anchorWindow) return;

    const cursor = await cursorPosition();
    if (mode !== dragged) return;
    appendPointerSample(dragged.samples, cursor, performance.now());

    const monitor = (await monitorFromPoint(cursor.x, cursor.y)) ?? (await resolveMonitor());
    if (!monitor || mode !== dragged) return;

    const windowSize: PhysicalSize = await petWindow.outerSize();
    if (mode !== dragged) return;

    const desiredPosition = {
      x: dragged.anchorWindow.x + cursor.x - dragged.anchorCursor.x,
      y: dragged.anchorWindow.y + cursor.y - dragged.anchorCursor.y,
    };
    const safePosition = clampPosition(
      desiredPosition,
      getVisiblePositionBounds(toWorkArea(monitor), windowSize),
    );
    await petWindow.setPosition(
      new PhysicalPosition(Math.round(safePosition.x), Math.round(safePosition.y)),
    );
  };

  const updateThrown = async (thrown: ThrownMode, deltaSeconds: number): Promise<void> => {
    const result = stepThrow(thrown.throwState, deltaSeconds, thrown.bounds);
    thrown.throwState = result.state;
    if (mode !== thrown) return;

    await petWindow.setPosition(
      new PhysicalPosition(
        Math.round(result.state.position.x),
        Math.round(result.state.position.y),
      ),
    );
    if (mode !== thrown) return;

    if (isHardFloorImpact(result.floorImpactSpeed)) {
      setMode(
        {
          kind: "hard-impact",
          untilMilliseconds: performance.now() + HARD_IMPACT_ANIMATION_MILLISECONDS,
        },
        "hard-impact",
      );
      return;
    }

    if (result.complete) {
      setMode(
        {
          kind: "landing",
          untilMilliseconds: performance.now() + LANDING_ANIMATION_MILLISECONDS,
        },
        "landing",
      );
    }
  };

  const updateRecovery = (recovery: RecoveryMode, nowMilliseconds: number): void => {
    if (nowMilliseconds >= recovery.untilMilliseconds) {
      setMode(idleMode(nowMilliseconds), "idle");
    }
  };

  const run = async (): Promise<void> => {
    let previousTime = performance.now();

    while (active) {
      const frameStartedAt = performance.now();
      const deltaSeconds = Math.min((frameStartedAt - previousTime) / 1_000, 0.1);
      previousTime = frameStartedAt;
      const currentMode = mode;

      if (currentMode.kind === "idle") {
        await updateIdle(currentMode, frameStartedAt);
      } else if (currentMode.kind === "walking") {
        await updateWalking(currentMode, deltaSeconds);
      } else if (currentMode.kind === "dragged") {
        await updateDragged(currentMode);
      } else if (currentMode.kind === "thrown") {
        await updateThrown(currentMode, deltaSeconds);
      } else if (currentMode.kind !== "timer") {
        updateRecovery(currentMode, frameStartedAt);
      }

      const frameBudget = 1_000 / MOVEMENT_FPS;
      await delay(Math.max(0, frameBudget - (performance.now() - frameStartedAt)));
    }
  };

  const cancelDrag = (event: PointerEvent): void => {
    if (mode.kind === "dragged" && event.pointerId === mode.pointerId) {
      interactionId += 1;
      setMode(idleMode(), "idle");
    }
  };

  sprite.element.addEventListener("pointerdown", (event) => void beginDrag(event));
  sprite.element.addEventListener("pointerup", (event) => void finishDrag(event));
  sprite.element.addEventListener("pointercancel", cancelDrag);

  const applyTimerState = (state: TimerSnapshot): void => {
    const active = state.phase !== "stopped";
    timerActive = active;
    if (active) {
      interactionId += 1;
      const phase = state.phase as TimerMode["phase"];
      setMode({ kind: "timer", phase }, phase === "focus" ? "focused" : "idle");
    } else if (mode.kind === "timer") {
      setMode(idleMode(), "idle");
    }
  };

  const applySystemMetrics = (metrics: SystemMetricsState): void => {
    latestSystemMetrics = metrics;
    resourceSpeedMultiplier = speedForMetrics(metrics);

    if (timerActive) return;
    if (mode.kind === "walking") {
      const direction = mode.targetX < mode.x ? "left" : "right";
      sprite.setAnimation(resourceMovementAnimation(metrics, direction));
    } else if (mode.kind === "idle") {
      if (shouldRunContinuously(metrics)) {
        mode.untilMilliseconds = performance.now();
      }
      sprite.setAnimation(resourceIdleAnimation(metrics));
    }
  };

  const pollSystemMetrics = (): void => {
    void invoke<SystemMetricsState>("get_system_metrics")
      .then(applySystemMetrics)
      .catch(() => {
        resourceSpeedMultiplier = 1;
      });
  };

  void invoke<TimerSnapshot>("get_timer_state").then(applyTimerState).catch(() => undefined);
  void listen<TimerSnapshot>("timer://state", ({ payload }) => applyTimerState(payload)).then(
    (unlisten) => {
      if (active) unlistenTimer = unlisten;
      else unlisten();
    },
  );
  void listen("todo://all-completed", () => {
    interactionId += 1;
    sprite.element.classList.add("todo-celebrating");
    setMode({ kind: "timer", phase: "shortBreak" }, "jumping");
    window.clearTimeout(celebrationTimer);
    celebrationTimer = window.setTimeout(() => {
      sprite.element.classList.remove("todo-celebrating");
      void invoke<TimerSnapshot>("get_timer_state").then(applyTimerState).catch(() => {
        setMode(idleMode(), "idle");
      });
    }, 4_500);
  }).then((unlisten) => {
    if (active) unlistenTodo = unlisten;
    else unlisten();
  });
  pollSystemMetrics();
  const systemMetricsTimer = window.setInterval(pollSystemMetrics, 1_000);

  void run().catch((error: unknown) => {
    setMode(idleMode(), "idle");
    console.warn("Pet motion stopped because the window could not be moved.", error);
  });

  return () => {
    active = false;
    interactionId += 1;
    window.clearInterval(animationTimer);
    window.clearInterval(systemMetricsTimer);
    window.clearTimeout(celebrationTimer);
    unlistenTimer?.();
    unlistenTodo?.();
  };
}
