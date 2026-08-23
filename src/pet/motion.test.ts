import { describe, expect, it } from "vitest";
import {
  advanceToward,
  clampPosition,
  getPositionBounds,
  getVisiblePositionBounds,
  pickHorizontalTarget,
} from "./motion";

describe("pet motion", () => {
  it("keeps a normal pet window fully inside a work area", () => {
    const bounds = getPositionBounds(
      { x: -1920, y: 0, width: 1920, height: 1040 },
      { width: 256, height: 256 },
    );

    expect(bounds).toEqual({ minX: -1920, maxX: -256, minY: 0, maxY: 784 });
    expect(clampPosition({ x: -2500, y: 900 }, bounds)).toEqual({
      x: -1920,
      y: 784,
    });
  });

  it("keeps at least 24 pixels visible when a window is larger than the work area", () => {
    const bounds = getPositionBounds(
      { x: 0, y: 0, width: 100, height: 80 },
      { width: 200, height: 120 },
    );

    expect(bounds).toEqual({ minX: -176, maxX: 76, minY: -96, maxY: 56 });
  });

  it("allows dragging to an edge while preserving a 24 pixel recovery area", () => {
    const bounds = getVisiblePositionBounds(
      { x: 100, y: 50, width: 800, height: 600 },
      { width: 128, height: 128 },
    );

    expect(bounds).toEqual({ minX: -4, maxX: 876, minY: -54, maxY: 626 });
  });

  it("uses the fully visible floor line for walking, bouncing, and impact", () => {
    const bounds = getPositionBounds(
      { x: 0, y: 0, width: 1920, height: 1040 },
      { width: 128, height: 128 },
    );

    expect(bounds.maxY).toBe(912);
  });

  it("selects a far edge when a random target would barely move", () => {
    const target = pickHorizontalTarget(
      500,
      { minX: 0, maxX: 1000, minY: 0, maxY: 500 },
      0.52,
      96,
    );

    expect(target).toBe(1000);
  });

  it("does not overshoot a walking destination", () => {
    expect(advanceToward(10, 20, 100, 0.2)).toBe(20);
    expect(advanceToward(20, 0, 50, 0.1)).toBe(15);
  });
});
