import { clampPosition, type Point, type PositionBounds } from "./motion";

export interface Velocity extends Point {}

export interface PointerSample extends Point {
  timeMilliseconds: number;
}

export interface ThrowState {
  position: Point;
  velocity: Velocity;
  elapsedSeconds: number;
}

export interface ThrowStep {
  state: ThrowState;
  complete: boolean;
  floorImpactSpeed: number | null;
}

export const THROW_SPEED_THRESHOLD = 700;
export const MAX_THROW_SPEED = 2_500;
export const MAX_THROW_DURATION_SECONDS = 3;
export const HARD_FLOOR_IMPACT_SPEED = 1_400;

export function isHardFloorImpact(
  floorImpactSpeed: number | null,
  threshold = HARD_FLOOR_IMPACT_SPEED,
): boolean {
  return floorImpactSpeed !== null && floorImpactSpeed >= threshold;
}

export function estimateThrowVelocity(
  samples: PointerSample[],
  sampleWindowMilliseconds = 110,
  speedThreshold = THROW_SPEED_THRESHOLD,
  maximumSpeed = MAX_THROW_SPEED,
): Velocity | null {
  if (samples.length < 2) return null;

  const latest = samples[samples.length - 1];
  const earliest =
    samples.find(
      (sample) => latest.timeMilliseconds - sample.timeMilliseconds <= sampleWindowMilliseconds,
    ) ?? latest;
  const deltaSeconds = (latest.timeMilliseconds - earliest.timeMilliseconds) / 1_000;
  if (deltaSeconds <= 0) return null;

  const velocity = {
    x: (latest.x - earliest.x) / deltaSeconds,
    y: (latest.y - earliest.y) / deltaSeconds,
  };
  const speed = Math.hypot(velocity.x, velocity.y);
  if (!Number.isFinite(speed) || speed < speedThreshold) return null;

  const scale = Math.min(1, maximumSpeed / speed);
  return { x: velocity.x * scale, y: velocity.y * scale };
}

export function stepThrow(
  state: ThrowState,
  deltaSeconds: number,
  bounds: PositionBounds,
  gravityPixelsPerSecondSquared = 2_200,
  bounceCoefficient = 0.45,
  floorFriction = 0.8,
): ThrowStep {
  const delta = Math.max(0, Math.min(deltaSeconds, 0.1));
  let velocityX = state.velocity.x;
  let velocityY = state.velocity.y + gravityPixelsPerSecondSquared * delta;
  let position = {
    x: state.position.x + velocityX * delta,
    y: state.position.y + velocityY * delta,
  };
  let touchedFloor = false;
  let floorImpactSpeed: number | null = null;

  if (position.x < bounds.minX || position.x > bounds.maxX) {
    position = clampPosition(position, bounds);
    velocityX = -velocityX * bounceCoefficient;
  }

  if (position.y < bounds.minY) {
    position = clampPosition(position, bounds);
    velocityY = Math.abs(velocityY) * bounceCoefficient;
  } else if (position.y > bounds.maxY) {
    floorImpactSpeed = Math.max(0, velocityY);
    position = clampPosition(position, bounds);
    velocityY = -Math.abs(velocityY) * bounceCoefficient;
    velocityX *= floorFriction;
    touchedFloor = true;
  }

  const elapsedSeconds = state.elapsedSeconds + delta;
  const settled =
    touchedFloor && Math.abs(velocityX) < 25 && Math.abs(velocityY) < 70;

  return {
    state: {
      position,
      velocity: settled ? { x: 0, y: 0 } : { x: velocityX, y: velocityY },
      elapsedSeconds,
    },
    complete: settled || elapsedSeconds >= MAX_THROW_DURATION_SECONDS,
    floorImpactSpeed,
  };
}
