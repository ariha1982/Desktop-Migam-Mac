export type GamchaRarity = "common" | "rare" | "epic" | "legendary" | "special";

export const GAMCHA_RARITIES: readonly GamchaRarity[] = [
  "common",
  "rare",
  "epic",
  "legendary",
  "special",
];

const labels: Record<GamchaRarity, string> = {
  common: "COMMON",
  rare: "RARE",
  epic: "EPIC",
  legendary: "LEGENDARY",
  special: "SPECIAL",
};

export function rarityLabel(rarity: GamchaRarity): string {
  return labels[rarity];
}

export function isGamchaRarity(value: string): value is GamchaRarity {
  return GAMCHA_RARITIES.includes(value as GamchaRarity);
}

export function rouletteDelay(frame: number): number {
  return Math.min(190, 45 + Math.floor(Math.max(0, frame) ** 1.45));
}
