import { describe, expect, it } from "vitest";
import {
  resourceIdleAnimation,
  resourceMovementAnimation,
  resourceSpeedMultiplier,
  selectedResourceLoad,
  shouldRunContinuously,
} from "./resource-response";

describe("resource-responsive pet behavior", () => {
  it("keeps CPU and memory selection independent", () => {
    expect(selectedResourceLoad({ cpuPercent: 82, memoryPercent: 25, mode: "cpu" })).toBe(82);
    expect(selectedResourceLoad({ cpuPercent: 82, memoryPercent: 25, mode: "memory" })).toBe(25);
    expect(selectedResourceLoad({ cpuPercent: 42, memoryPercent: 76, mode: "combined" })).toBe(76);
  });

  it("accelerates at high load and slows down at low load", () => {
    expect(resourceSpeedMultiplier({ cpuPercent: 19, memoryPercent: 20, mode: "cpu" })).toBe(1);
    expect(resourceSpeedMultiplier({ cpuPercent: 85, memoryPercent: 20, mode: "cpu" })).toBe(2.1);
  });

  it("selects a four-frame running stage for each twenty-percent band", () => {
    expect(resourceMovementAnimation({ cpuPercent: 19, memoryPercent: 0, mode: "cpu" }, "right")).toBe("running-right");
    expect(resourceMovementAnimation({ cpuPercent: 20, memoryPercent: 0, mode: "cpu" }, "right")).toBe("load-alert-right");
    expect(resourceMovementAnimation({ cpuPercent: 40, memoryPercent: 0, mode: "cpu" }, "left")).toBe("load-medium-left");
    expect(resourceMovementAnimation({ cpuPercent: 60, memoryPercent: 0, mode: "cpu" }, "right")).toBe("load-fast-right");
    expect(resourceMovementAnimation({ cpuPercent: 80, memoryPercent: 0, mode: "cpu" }, "left")).toBe("load-extreme-left");
  });

  it("runs continuously whenever a resource response mode is selected", () => {
    expect(shouldRunContinuously({ cpuPercent: 0, memoryPercent: 0, mode: "cpu" })).toBe(true);
    expect(shouldRunContinuously({ cpuPercent: 0, memoryPercent: 0, mode: "memory" })).toBe(true);
    expect(shouldRunContinuously({ cpuPercent: 0, memoryPercent: 0, mode: "combined" })).toBe(true);
    expect(shouldRunContinuously({ cpuPercent: 0, memoryPercent: 0, mode: "off" })).toBe(false);
  });

  it("uses the failure animation only for critical watched memory", () => {
    expect(resourceIdleAnimation({ cpuPercent: 95, memoryPercent: 20, mode: "cpu" })).toBe("busy");
    expect(resourceIdleAnimation({ cpuPercent: 20, memoryPercent: 92, mode: "memory" })).toBe("failed");
  });
});
