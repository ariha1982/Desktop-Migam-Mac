import { describe, expect, it } from "vitest";

import { formatRemaining, timerControls } from "./timer-view";

describe("timer view helpers", () => {
  it("formats remaining seconds as a stable clock", () => {
    expect(formatRemaining(1_500)).toBe("25:00");
    expect(formatRemaining(65)).toBe("01:05");
    expect(formatRemaining(-1)).toBe("00:00");
  });

  it("offers start only while stopped", () => {
    expect(timerControls("stopped")).toEqual({
      start: true,
      pause: false,
      resume: false,
      skip: false,
      stop: false,
    });
  });

  it("offers resume and stop while paused", () => {
    expect(timerControls("paused")).toEqual({
      start: false,
      pause: false,
      resume: true,
      skip: false,
      stop: true,
    });
  });

  it("offers three evenly usable controls while a phase is running", () => {
    expect(timerControls("focus")).toEqual({
      start: false,
      pause: true,
      resume: false,
      skip: true,
      stop: true,
    });
  });
});
