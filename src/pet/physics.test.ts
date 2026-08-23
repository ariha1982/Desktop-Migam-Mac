import { describe, expect, it } from "vitest";
import {
  estimateThrowVelocity,
  isHardFloorImpact,
  stepThrow,
  type ThrowState,
} from "./physics";

const bounds = { minX: 0, maxX: 1_000, minY: 0, maxY: 700 };

describe("pet throw physics", () => {
  it("does not throw when the recent pointer speed is below the threshold", () => {
    expect(
      estimateThrowVelocity([
        { x: 0, y: 0, timeMilliseconds: 0 },
        { x: 50, y: 0, timeMilliseconds: 100 },
      ]),
    ).toBeNull();
  });

  it("uses recent samples and clamps excessive release speed", () => {
    const velocity = estimateThrowVelocity([
      { x: 0, y: 0, timeMilliseconds: 0 },
      { x: 100, y: 0, timeMilliseconds: 100 },
      { x: 500, y: 0, timeMilliseconds: 200 },
    ]);

    expect(velocity).not.toBeNull();
    expect(Math.hypot(velocity!.x, velocity!.y)).toBeCloseTo(2_500);
  });

  it("applies gravity and bounces from the floor", () => {
    const initial: ThrowState = {
      position: { x: 100, y: 690 },
      velocity: { x: 200, y: 300 },
      elapsedSeconds: 0,
    };
    const result = stepThrow(initial, 0.05, bounds);

    expect(result.state.position.y).toBe(700);
    expect(result.state.velocity.y).toBeLessThan(0);
    expect(result.state.velocity.x).toBe(160);
    expect(result.floorImpactSpeed).toBe(410);
  });

  it("bounces from a side boundary without leaving it", () => {
    const result = stepThrow(
      {
        position: { x: 995, y: 100 },
        velocity: { x: 500, y: 0 },
        elapsedSeconds: 0,
      },
      0.05,
      bounds,
    );

    expect(result.state.position.x).toBe(1_000);
    expect(result.state.velocity.x).toBe(-225);
  });

  it("always ends by three seconds", () => {
    const result = stepThrow(
      {
        position: { x: 500, y: 100 },
        velocity: { x: 100, y: -100 },
        elapsedSeconds: 2.95,
      },
      0.1,
      bounds,
    );

    expect(result.complete).toBe(true);
  });

  it("reports the downward speed that caused a floor impact", () => {
    const result = stepThrow(
      {
        position: { x: 500, y: 690 },
        velocity: { x: 0, y: 1_500 },
        elapsedSeconds: 0,
      },
      0.05,
      bounds,
    );

    expect(result.floorImpactSpeed).toBe(1_610);
    expect(isHardFloorImpact(result.floorImpactSpeed)).toBe(true);
    expect(isHardFloorImpact(1_399)).toBe(false);
  });
});
