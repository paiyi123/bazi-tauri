import type { LessonThreeStepAnalysis } from "../types/bazi";
import { BRANCH_WUXING, STEM_WUXING, type WuXing } from "../constants/wuxing";
import {
  elementFromName,
  judgeLuckRelation,
  type Favor,
  type LessonPreferences,
  type RelationContext,
} from "./lessonPreference";

// 四柱括號線上的箭頭：這個合沖刑對日主有利（▲）還是不利（▼）。
// 原局的關係直接用課堂三步法已經判好的結果；大運、流年加進來的關係用和大運面板相同的規則判。

export type RelationVerdict = { favor: Favor; reason: string };
export type RelationFamily = "clash" | "combine" | "punish";
type NatalVerdict = RelationVerdict & { scope: "stem" | "branch"; family: RelationFamily; pillars: string };

const PILLAR_ORDER = "年月日時";
const toFavor = (value: string): Favor => (value === "good" || value === "bad" || value === "mixed" ? value : "neutral");
/** 「年支戌（土，食傷）」「月干癸（傷官）」→ 排序後的柱名，例如「年月」。 */
const pillarsOf = (positions: string[]) =>
  [...new Set(positions.map((position) => position.charAt(0)))]
    .filter((char) => PILLAR_ORDER.includes(char))
    .sort((a, b) => PILLAR_ORDER.indexOf(a) - PILLAR_ORDER.indexOf(b))
    .join("");
const join = (...parts: Array<string | null | undefined>) => parts.filter(Boolean).join("；");

export function relationFamily(type: string): RelationFamily | null {
  if (type.includes("沖")) return "clash";
  if (type.includes("刑")) return "punish";
  if (type.includes("合") || type.includes("會")) return "combine";
  return null;
}

/** 11/04：刑（沖也一樣）在月令是加強那個五行，在日時是鬆動減弱。 */
function punishPositionNote(pillars: string) {
  if (pillars.includes("月")) return "刑在月令，那個五行加強（11/04）";
  if (pillars === "日時") return "刑在日時，鬆動減弱（11/04）";
  return "";
}

/** 課堂三步法已經判過的原局關係。 */
export function natalRelationVerdicts(analysis?: LessonThreeStepAnalysis | null): NatalVerdict[] {
  if (!analysis) return [];
  const list: NatalVerdict[] = [];
  for (const combo of analysis.stemInteractions?.combinations ?? []) {
    list.push({
      scope: "stem",
      family: "combine",
      pillars: pillarsOf(combo.positions),
      favor: toFavor(combo.verdict),
      reason: join(combo.kindLabel, combo.verdictLabel, combo.needsConfirmation ? "待確認" : ""),
    });
  }
  const branches = analysis.branchInteractions;
  for (const item of branches?.clashes ?? []) {
    list.push({ scope: "branch", family: "clash", pillars: pillarsOf(item.positions), favor: toFavor(item.verdict), reason: join(item.outcomeLabel, item.verdictLabel) });
  }
  for (const item of branches?.combinations ?? []) {
    list.push({
      scope: "branch",
      family: "combine",
      pillars: pillarsOf(item.positions),
      favor: toFavor(item.verdict),
      reason: join(item.kindLabel, item.adjacent ? "" : "隔位，僅供參考", item.verdictLabel),
    });
  }
  for (const item of branches?.gatherings ?? []) {
    list.push({ scope: "branch", family: "combine", pillars: pillarsOf(item.positions), favor: toFavor(item.verdict), reason: join(item.kindLabel, item.verdictLabel) });
  }
  for (const item of branches?.punishments ?? []) {
    if (item.kind === "harm") continue; // 害屬神煞，圖上不畫
    const pillars = pillarsOf(item.positions);
    list.push({ scope: "branch", family: "punish", pillars, favor: toFavor(item.verdict), reason: join(item.relation, item.verdictLabel, punishPositionNote(pillars)) });
  }
  return list;
}

export function findNatalVerdict(
  list: NatalVerdict[],
  scope: "stem" | "branch",
  type: string,
  pillarLabels: string[],
): RelationVerdict | null {
  const family = relationFamily(type);
  if (!family) return null;
  const pillars = pillarsOf(pillarLabels);
  const hit = list.find((item) => item.scope === scope && item.family === family && item.pillars === pillars);
  return hit ? { favor: hit.favor, reason: hit.reason } : null;
}

export type PreviewRelation = {
  scope: "stem" | "branch";
  /** 合、半合、三合、三會、沖、刑、破 */
  type: string;
  /** 合局的五行名（木火土金水），沒有就空白。 */
  elementName?: string;
  /** 大運或流年這一邊的字；天干要附坐支。 */
  luck: { char: string; seat?: string } | null;
  /** 原局這一邊的字（可以不只一個，例如三合）。 */
  natal: Array<{ char: string; isMonth: boolean }>;
  /** 原局八個字的五行（地支取本氣）。 */
  natalElements: Set<WuXing>;
};

/** 大運、流年和原局之間的關係，判斷規則同大運面板的標籤。 */
export function previewRelationVerdict(relation: PreviewRelation, preferences: LessonPreferences | null): RelationVerdict {
  if (!preferences) return { favor: "neutral", reason: "沒有課堂喜忌，未判" };
  const table = relation.scope === "stem" ? STEM_WUXING : BRANCH_WUXING;
  const combined = relation.elementName ? elementFromName(relation.elementName) : undefined;
  if (relation.type === "三合" || relation.type === "三會") {
    if (!combined) return { favor: "neutral", reason: "未判" };
    const pref = preferences.elements[combined];
    const favor: Favor = pref.favor === "good" ? "good" : pref.favor === "bad" ? "bad" : "neutral";
    return { favor, reason: `${relation.type}成${pref.name}局，${pref.name}為${pref.label}` };
  }
  const luckElement = relation.luck ? table[relation.luck.char] : undefined;
  if (!relation.luck || !luckElement || relation.natal.length !== 1) {
    return { favor: "neutral", reason: "大運和流年之間的關係，不判喜忌" };
  }
  const natal = relation.natal[0];
  const natalElement = table[natal.char];
  if (!natalElement) return { favor: "neutral", reason: "未判" };
  const text =
    relation.type === "合" || relation.type === "半合" ? `合${relation.elementName ?? ""}` : relation.type;
  const context: RelationContext =
    relation.scope === "branch"
      ? { natalIsMonthBranch: natal.isMonth, natalHasLuckElement: relation.natalElements.has(luckElement) }
      : {};
  return judgeLuckRelation(
    text,
    relation.scope,
    { char: relation.luck.char, element: luckElement, ...(relation.scope === "stem" ? { seat: relation.luck.seat } : {}) },
    { char: natal.char, element: natalElement },
    preferences,
    context,
  );
}
