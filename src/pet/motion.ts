export interface Point {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface WorkArea extends Point, Size {}

export interface PositionBounds {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
}

export function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

export function getPositionBounds(
  workArea: WorkArea,
  windowSize: Size,
  minimumVisiblePixels = 24,
): PositionBounds {
  const fitsHorizontally = windowSize.width <= workArea.width;
  const fitsVertically = windowSize.height <= workArea.height;

  return {
    minX: fitsHorizontally
      ? workArea.x
      : workArea.x - windowSize.width + minimumVisiblePixels,
    maxX: fitsHorizontally
      ? workArea.x + workArea.width - windowSize.width
      : workArea.x + workArea.width - minimumVisiblePixels,
    minY: fitsVertically
      ? workArea.y
      : workArea.y - windowSize.height + minimumVisiblePixels,
    maxY: fitsVertically
      ? workArea.y + workArea.height - windowSize.height
      : workArea.y + workArea.height - minimumVisiblePixels,
  };
}

export function getVisiblePositionBounds(
  workArea: WorkArea,
  windowSize: Size,
  minimumVisiblePixels = 24,
): PositionBounds {
  return {
    minX: workArea.x - windowSize.width + minimumVisiblePixels,
    maxX: workArea.x + workArea.width - minimumVisiblePixels,
    minY: workArea.y - windowSize.height + minimumVisiblePixels,
    maxY: workArea.y + workArea.height - minimumVisiblePixels,
  };
}

export function clampPosition(position: Point, bounds: PositionBounds): Point {
  return {
    x: clamp(position.x, bounds.minX, bounds.maxX),
    y: clamp(position.y, bounds.minY, bounds.maxY),
  };
}

export function pickHorizontalTarget(
  currentX: number,
  bounds: PositionBounds,
  randomValue: number,
  minimumTravelPixels = 96,
): number {
  const normalizedRandom = clamp(randomValue, 0, 1);
  const candidate = bounds.minX + (bounds.maxX - bounds.minX) * normalizedRandom;

  if (Math.abs(candidate - currentX) >= minimumTravelPixels) {
    return candidate;
  }

  const distanceToLeft = Math.abs(currentX - bounds.minX);
  const distanceToRight = Math.abs(bounds.maxX - currentX);
  return distanceToLeft > distanceToRight ? bounds.minX : bounds.maxX;
}

export function advanceToward(
  current: number,
  target: number,
  speedPixelsPerSecond: number,
  deltaSeconds: number,
): number {
  const distance = target - current;
  const maximumStep = Math.max(0, speedPixelsPerSecond * deltaSeconds);

  if (Math.abs(distance) <= maximumStep) {
    return target;
  }

  return current + Math.sign(distance) * maximumStep;
}
