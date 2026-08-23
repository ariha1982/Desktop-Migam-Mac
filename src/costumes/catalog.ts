import manifest from "../../pack/manifest.json";
import { isGamchaRarity, type GamchaRarity } from "../gamcha/gamcha-model";

interface ManifestCostume {
  id: string;
  name: string;
  rarity: string;
  file: string;
}

export interface Costume {
  id: string;
  name: string;
  rarity: GamchaRarity;
  file: string;
  url: string;
  slot: CostumeSlot;
}

export type CostumeSlot = "head" | "face" | "neck" | "body" | "full";

function costumeSlot(name: string): CostumeSlot {
  if (/(안경|선글라스|안대|모노클|바이저|가면)/.test(name)) return "face";
  if (/(넥타이|나비넥타이|목도리|스카프|넥워머|목걸이|칼라)/.test(name)) return "neck";
  if (/(앞치마|멜빵|조끼|카디건|벨트|백팩|가방|망토|케이프)/.test(name)) return "body";
  if (/(세트|갑주)/.test(name)) return "full";
  return "head";
}

const assetUrls = import.meta.glob<string>([
  "../../pack/common/*.png",
  "../../pack/rare/*.png",
  "../../pack/epic/*.png",
  "../../pack/legendary/*.png",
  "../../pack/special/*.png",
], {
  eager: true,
  query: "?url",
  import: "default",
});

export const costumes = (manifest.costumes as ManifestCostume[])
  .filter((costume) => isGamchaRarity(costume.rarity))
  .map((costume): Costume => ({
    ...costume,
    rarity: costume.rarity as GamchaRarity,
    url: assetUrls[`../../pack/${costume.file}`],
    slot: costumeSlot(costume.name),
  }))
  .filter((costume) => Boolean(costume.url));

export const costumeById = new Map(costumes.map((costume) => [costume.id, costume]));

export function costumeUrl(costumeId: string | null | undefined): string | null {
  return costumeId ? (costumeById.get(costumeId)?.url ?? null) : null;
}
