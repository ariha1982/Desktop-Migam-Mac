import { describe, expect, it } from "vitest";
import { isGamchaRarity, rarityLabel, rouletteDelay } from "./gamcha-model";

describe("GAMCHA view model", () => {
  it("labels every drawable rarity", () => {
    expect(rarityLabel("common")).toBe("COMMON");
    expect(rarityLabel("special")).toBe("SPECIAL");
    expect(isGamchaRarity("default")).toBe(false);
  });

  it("slows the roulette toward the reveal", () => {
    expect(rouletteDelay(0)).toBe(45);
    expect(rouletteDelay(12)).toBeGreaterThan(rouletteDelay(3));
    expect(rouletteDelay(100)).toBe(190);
  });
});
