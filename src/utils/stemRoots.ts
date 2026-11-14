import { BRANCH_WUXING, STEM_WUXING } from "../constants/wuxing";

// 天干通根：地支藏干裡有和這個天干同五行的字，就是它的根。
// 依藏干位置分本氣根（最有力）、中氣根、餘氣根；坐下（同柱）的根最近、最有力。

/** 地支藏干，依本氣、中氣、餘氣排列。 */
export const BRANCH_HIDDEN_ORDERED: Record<string, string[]> = {
  子: ["癸"],
  丑: ["己", "癸", "辛"],
  寅: ["甲", "丙", "戊"],
  卯: ["乙"],
  辰: ["戊", "乙", "癸"],
  巳: ["丙", "庚", "戊"],
  午: ["丁", "己"],
  未: ["己", "丁", "乙"],
  申: ["庚", "壬", "戊"],
  酉: ["辛"],
  戌: ["戊", "辛", "丁"],
  亥: ["壬", "甲"],
};

export type RootLevel = "本氣" | "中氣" | "餘氣";
const LEVELS: RootLevel[] = ["本氣", "中氣", "餘氣"];

/** 天干在這個地支有沒有根；有的話回傳最強的一層。 */
export function rootLevel(stem: string, branch: string): RootLevel | null {
  const element = STEM_WUXING[stem];
  const hidden = BRANCH_HIDDEN_ORDERED[branch];
  if (!element || !hidden) return null;
  const index = hidden.findIndex((char) => STEM_WUXING[char] === element);
  return index < 0 ? null : LEVELS[index];
}

export interface StemRoot {
  /** 地支所在的欄（和四柱卡片同一個順序） */
  col: number;
  branch: string;
  level: RootLevel;
  /** 坐下：就在同一柱 */
  seated: boolean;
}

/** 一個天干在各柱地支的根，強的在前（本氣 → 中氣 → 餘氣，同層坐下優先）。 */
export function stemRoots(stem: string, stemCol: number, branches: Array<{ col: number; branch: string }>): StemRoot[] {
  return branches
    .flatMap(({ col, branch }) => {
      const level = rootLevel(stem, branch);
      return level ? [{ col, branch, level, seated: col === stemCol }] : [];
    })
    .sort((a, b) => LEVELS.indexOf(a.level) - LEVELS.indexOf(b.level) || Number(b.seated) - Number(a.seated));
}

/** 地支本氣和藏干表一致（防呆用）。 */
export function mainQiMatches(branch: string) {
  const hidden = BRANCH_HIDDEN_ORDERED[branch];
  return !!hidden && STEM_WUXING[hidden[0]] === BRANCH_WUXING[branch];
}
