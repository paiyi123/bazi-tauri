/**
 * 課堂（許羽賢老師 2026-11-04，講義第 24 章第五節）對大運、流年的提醒：
 *
 * 1. 例八「印格身旺，金運要帶土」：身旺又印重時，官殺運會先去生印、再生日主，制身無力；
 *    運裡要同時見財（剋印）才有用。乙木見金運要帶土，就是這一條。
 * 2. 例一「土厚埋金」（母慈滅子）：印是病，原局又沒有能用的財時，財運進來會「印旺財折」，
 *    走財運也沒用；只能走比劫運洩印幫身。
 * 3. 例三「五行中和，行五運皆可，惟歲運逢沖較忌」：判成可能中和的命，運歲沖原局才是看運重點。
 *
 * 這些只是提示，不改課堂喜忌與量化分數。
 */
import type { LessonThreeStepAnalysis } from "../types/bazi";
import type { LuckEvent } from "./luckEvents";

const STEM_ELEMENT: Record<string, string> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
const BRANCH_HIDDEN: Record<string, string[]> = {
  子: ["癸"], 丑: ["己", "癸", "辛"], 寅: ["甲", "丙", "戊"], 卯: ["乙"], 辰: ["戊", "乙", "癸"], 巳: ["丙", "戊", "庚"],
  午: ["丁", "己"], 未: ["己", "丁", "乙"], 申: ["庚", "壬", "戊"], 酉: ["辛"], 戌: ["戊", "辛", "丁"], 亥: ["壬", "甲"],
};
const ORDER = ["木", "火", "土", "金", "水"];
const CLASH_PAIRS = ["子午", "丑未", "寅申", "卯酉", "辰戌", "巳亥"];
const PILLAR_NAMES = ["年", "月", "日", "時"];

const shift = (element: string, step: number) => ORDER[(ORDER.indexOf(element) + step + 5) % 5];
/** 日主五行換算各家族的五行。 */
function familyElements(day: string) {
  return { 比劫: day, 食傷: shift(day, 1), 財: shift(day, 2), 官殺: shift(day, 3), 印: shift(day, 4) };
}

/** 一柱（干＋支藏干全部）裡出現的五行。 */
function pillarElements(ganZhi: string): Set<string> {
  const set = new Set<string>();
  const stem = STEM_ELEMENT[ganZhi.charAt(0)];
  if (stem) set.add(stem);
  (BRANCH_HIDDEN[ganZhi.charAt(1)] ?? []).forEach((hidden) => set.add(STEM_ELEMENT[hidden]));
  return set;
}

/** 一柱的天干五行與地支本氣五行。 */
function pillarSurface(ganZhi: string): string[] {
  return [STEM_ELEMENT[ganZhi.charAt(0)], STEM_ELEMENT[(BRANCH_HIDDEN[ganZhi.charAt(1)] ?? [])[0]]].filter(Boolean);
}

export interface ClassroomLuckOptions {
  /** 原局四柱，依年、月、日、時。 */
  natal: string[];
  analysis?: LessonThreeStepAnalysis | null;
  /** 流年時傳所屬大運，用來看「歲運逢沖」。 */
  luck?: string;
}

export function classroomLuckEvents(incoming: string, who: "運" | "歲", options: ClassroomLuckOptions): LuckEvent[] {
  const { natal, analysis } = options;
  if (!analysis || natal.length !== 4 || incoming.length < 2) return [];
  const day = STEM_ELEMENT[natal[2].charAt(0)];
  if (!day) return [];
  const family = familyElements(day);
  const strength = analysis.strength;
  const strong = ["強候選", "旺候選", "旺極候選"].includes(strength.classification);
  const events: LuckEvent[] = [];
  const whoName = who === "運" ? "大運" : "流年";
  const surface = pillarSurface(incoming);
  const all = pillarElements(incoming);

  // 1. 身旺印重：官殺運要帶財（講義例八「金運要帶土」）。只看大運，流年力量小。
  const monthIsSeal = analysis.facts?.monthCommand?.element === family.印;
  const natalSurface = natal.flatMap(pillarSurface);
  const sealCount = natalSurface.filter((element) => element === family.印).length;
  if (who === "運" && strong && (monthIsSeal || sealCount >= 3) && surface.includes(family.官殺)) {
    const withWealth = all.has(family.財);
    events.push({
      kind: withWealth ? "classroom-officer-with-wealth" : "classroom-officer-feeds-seal",
      label: withWealth ? `${family.官殺}運帶${family.財}` : `${family.官殺}運生印`,
      level: "info",
      detail: withWealth
        ? `身旺又印（${family.印}）重：這步${whoName}的官殺（${family.官殺}）帶著財（${family.財}），財能剋印，官殺才能制身（講義例八「金運要帶土」，11/04）。`
        : `身旺又印（${family.印}）重：官殺（${family.官殺}）會先去生印、印再生日主，制身無力；這步${whoName}沒有財（${family.財}）可以剋印，用處不大（講義例八「金運要帶土」，11/04）。`,
    });
  }

  // 2. 母慈滅子：印是病，財運印旺財折、比劫運洩印幫身（講義例一）。
  if (strength.motherOverload) {
    const natalWealth = natalSurface.includes(family.財);
    if (surface.includes(family.財) && !natalWealth) {
      events.push({
        kind: "classroom-seal-breaks-wealth",
        label: "印旺財折",
        level: "info",
        detail: `母慈滅子：原局印（${family.印}）太旺、又沒有能用的財，${family.財}的${whoName}進來會被印反侮（土旺木折一類），用處不大（講義例一，11/04）。`,
      });
    }
    if (surface.includes(family.比劫)) {
      events.push({
        kind: "classroom-peer-drains-seal",
        label: "比劫洩印",
        level: "info",
        detail: `母慈滅子：比劫（${family.比劫}）${whoName}能洩印幫身，是這種命唯一的好運（例一「只能走金運，在土山裡挖金礦」，11/04）。`,
      });
    }
  }

  // 3. 中和：看運重點在沖（講義例三「行五運皆可，惟歲運逢沖較忌」）。
  if (strength.balanceHint) {
    const branch = incoming.charAt(1);
    const hits = natal
      .map((pillar, index) => (CLASH_PAIRS.some((pair) => pair === branch + pillar.charAt(1) || pair === pillar.charAt(1) + branch) ? PILLAR_NAMES[index] : ""))
      .filter(Boolean);
    const luckClash = who === "歲" && options.luck && CLASH_PAIRS.some((pair) => pair === branch + options.luck!.charAt(1) || pair === options.luck!.charAt(1) + branch);
    if (hits.length || luckClash) {
      events.push({
        kind: "classroom-balance-clash",
        label: "中和逢沖",
        level: "warn",
        detail: `這張命可能偏中和，平常「行五運皆可」、起落不大，最怕的是沖：這${who === "運" ? "步大運" : "年"}${hits.length ? `沖原局${hits.join("、")}支` : ""}${hits.length && luckClash ? "，又" : ""}${luckClash ? "和大運相沖" : ""}，變化會比較大（講義例三，11/04）。`,
      });
    }
  }
  return events;
}
