import type { SystemMetricsState } from "../contracts";
import type { PetAnimation } from "./sprite";

export function selectedResourceLoad(metrics: SystemMetricsState): number | null {
  if (metrics.mode === "cpu") return metrics.cpuPercent;
  if (metrics.mode === "memory") return metrics.memoryPercent;
  if (metrics.mode === "combined") return Math.max(metrics.cpuPercent, metrics.memoryPercent);
  return null;
}

export function resourceSpeedMultiplier(metrics: SystemMetricsState): number {
  const load = selectedResourceLoad(metrics);
  if (load === null) return 1;
  if (load < 20) return 1;
  if (load < 40) return 1.15;
  if (load < 60) return 1.35;
  if (load < 80) return 1.65;
  return 2.1;
}

export function resourceMovementAnimation(
  metrics: SystemMetricsState,
  direction: "left" | "right",
): PetAnimation {
  const load = selectedResourceLoad(metrics);
  if (load === null || load < 20) {
    return direction === "left" ? "running-left" : "running-right";
  }
  if (load < 40) {
    return direction === "left" ? "load-alert-left" : "load-alert-right";
  }
  if (load < 60) {
    return direction === "left" ? "load-medium-left" : "load-medium-right";
  }
  if (load < 80) {
    return direction === "left" ? "load-fast-left" : "load-fast-right";
  }
  return direction === "left" ? "load-extreme-left" : "load-extreme-right";
}

export function shouldRunContinuously(metrics: SystemMetricsState): boolean {
  return selectedResourceLoad(metrics) !== null;
}

export function resourceIdleAnimation(metrics: SystemMetricsState): PetAnimation {
  const load = selectedResourceLoad(metrics);
  if (load === null) return "idle";
  if ((metrics.mode === "memory" || metrics.mode === "combined") && metrics.memoryPercent >= 90) {
    return "failed";
  }
  if (load >= 70) return "busy";
  if (load < 30) return "waiting";
  return "idle";
}
