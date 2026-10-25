import type { LessonDirection, LessonThreeStepAnalysis } from "../types/bazi";
import type { WuXing } from "../constants/wuxing";

// 喜忌 from the classroom three-step analysis (第三步：用神與喜忌), used to judge luck relations.
// The analysis keeps several conditional plans and never merges conflicting roles, so an element is
// 喜 or 忌 here only when no plan says the opposite; otherwise it is 「依方案而定」.

export type Favor = "good" | "bad" | "mixed" | "neutral";

export type ElementPreference = {
  element: WuXing;
  name: string;
  favor: Favor;
  label: string;
  notes: string[];
};

export type LessonPreferences = {
  elements: Record<WuXing, ElementPreference>;
  planTitles: string[];
};

export type RelationKind = "combine" | "clash" | "punish" | "break" | "echo" | "hidden";

type Verdict = "good" | "bad" | "neutral";

export type RelationJudgement = {
  favor: Verdict;
  reason: string;
};

export const ELEMENT_NAMES: Record<WuXing, string> = {
  wood: "木",
  fire: "火",
  earth: "土",
  metal: "金",
  water: "水",
};

const ELEMENT_BY_NAME = Object.fromEntries(
  Object.entries(ELEMENT_NAMES).map(([element, name]) => [name, element as WuXing]),
) as Record<string, WuXing>;

const FAVOR_LABELS: Record<Favor, string> = {
  good: "喜用",
  bad: "忌",
  mixed: "依方案而定",
  neutral: "未列喜忌",
};

const ROLE_LABELS: Record<string, string> = {
  use: "用神",
  favorable: "喜神",
  unfavorable: "忌神",
  conditional: "待比較",
};

export function elementFromName(name: string): WuXing | undefined {
  return ELEMENT_BY_NAME[name];
}

export function lessonPreferences(analysis?: LessonThreeStepAnalysis | null): LessonPreferences | null {
  const plans = analysis?.preferences?.plans ?? [];
  if (!plans.length) {
    return null;
  }
  const elements = {} as Record<WuXing, ElementPreference>;
  (Object.keys(ELEMENT_NAMES) as WuXing[]).forEach((element) => {
    const name = ELEMENT_NAMES[element];
    let good = false;
    let bad = false;
    const notes: string[] = [];
    plans.forEach((plan) => {
      const directions: Array<[string, LessonDirection[]]> = [
        ["use", plan.useDirections],
        ["favorable", plan.favorableDirections],
        ["unfavorable", plan.unfavorableDirections],
        ["conditional", plan.conditionalDirections],
      ];
      directions.forEach(([role, list]) => {
        list
          .filter((direction) => direction.element === name)
          .forEach((direction) => {
            good ||= role === "use" || role === "favorable";
            bad ||= role === "unfavorable";
            notes.push(`「${plan.title}」：${direction.family}（${name}）為${ROLE_LABELS[role] ?? role}`);
          });
      });
    });
    const favor: Favor = good && bad ? "mixed" : good ? "good" : bad ? "bad" : "neutral";
    elements[element] = { element, name, favor, label: FAVOR_LABELS[favor], notes };
  });
  return { elements, planTitles: plans.map((plan) => plan.title) };
}

export function relationKind(text: string): RelationKind {
  if (text.includes("伏吟")) return "echo";
  if (text.includes("沖")) return "clash";
  if (text.includes("刑")) return "punish";
  if (text.includes("破") || text.includes("害")) return "break";
  if (text.includes("暗合") || text.includes("暗會")) return "hidden";
  return "combine";
}

// 「甲己合土」「寅午半合火」→ 土、火
export function combinationElement(text: string): WuXing | undefined {
  const match = text.match(/合([木火土金水])$/);
  return match ? elementFromName(match[1]) : undefined;
}

// seat：大運干的坐支（只在天干關係用），10/07 用來判斷大運天干五合化不化。
export type RelationSide = { char: string; element: WuXing; seat?: string };

const BRANCH_MAIN: Record<string, WuXing> = {
  子: "water", 丑: "earth", 寅: "wood", 卯: "wood", 辰: "earth", 巳: "fire",
  午: "fire", 未: "earth", 申: "metal", 酉: "metal", 戌: "earth", 亥: "water",
};

// 地支藏干的五行（本氣、中氣、餘氣）。
const BRANCH_HIDDEN: Record<string, WuXing[]> = {
  子: ["water"], 丑: ["earth", "water", "metal"], 寅: ["wood", "fire", "earth"], 卯: ["wood"],
  辰: ["earth", "wood", "water"], 巳: ["fire", "metal", "earth"], 午: ["fire", "earth"],
  未: ["earth", "fire", "wood"], 申: ["metal", "water", "earth"], 酉: ["metal"],
  戌: ["earth", "metal", "fire"], 亥: ["water", "wood"],
};

/**
 * 2026-10-07：大運天干與原局五合，化不化看大運所坐的地支（丁壬合、大運坐卯 → 化木）。
 * 與原局年月合化同一條件（10/14「化神通根乘旺」）：坐支本氣是化神或生化神，且藏有化神。
 */
export function seatTransforms(seat: string, combined: WuXing): boolean {
  const main = BRANCH_MAIN[seat];
  if (!main) return false;
  const generator = (Object.keys(GENERATES) as WuXing[]).find((element) => GENERATES[element] === combined);
  return (main === combined || main === generator) && (BRANCH_HIDDEN[seat] ?? []).includes(combined);
}

const GENERATES: Record<WuXing, WuXing> = { wood: "fire", fire: "earth", earth: "metal", metal: "water", water: "wood" };
const CONTROLS: Record<WuXing, WuXing> = { wood: "earth", earth: "water", water: "fire", fire: "metal", metal: "wood" };

function strengthened(favor: Favor): Verdict {
  return favor === "good" ? "good" : favor === "bad" ? "bad" : "neutral";
}

function weakened(favor: Favor): Verdict {
  return favor === "good" ? "bad" : favor === "bad" ? "good" : "neutral";
}

function agree(verdicts: Verdict[]): Verdict {
  const decided = new Set(verdicts.filter((verdict) => verdict !== "neutral"));
  return decided.size === 1 ? [...decided][0] : "neutral";
}

/**
 * 2026-09-09：地支有合無化。子丑合看作土剋水、寅亥合看作水生木，照五行生剋判斷：
 * 被生的一方變強、被剋或被洩的一方變弱，再看那一方是喜用還是忌。
 */
function judgeBranchCombination(luck: RelationSide, natal: RelationSide, preferences: LessonPreferences): RelationJudgement {
  const luckPref = preferences.elements[luck.element];
  const natalPref = preferences.elements[natal.element];
  const luckLabel = `大運${luck.char}${luckPref.name}（${luckPref.label}）`;
  const natalLabel = `原局${natal.char}${natalPref.name}（${natalPref.label}）`;
  const prefix = "地支有合無化，";
  if (GENERATES[luck.element] === natal.element) {
    return { favor: strengthened(natalPref.favor), reason: `${prefix}看作${luckLabel}生${natalLabel}` };
  }
  if (CONTROLS[luck.element] === natal.element) {
    return { favor: weakened(natalPref.favor), reason: `${prefix}看作${luckLabel}剋${natalLabel}` };
  }
  if (GENERATES[natal.element] === luck.element) {
    return {
      favor: agree([weakened(natalPref.favor), strengthened(luckPref.favor)]),
      reason: `${prefix}看作${natalLabel}生${luckLabel}`,
    };
  }
  if (CONTROLS[natal.element] === luck.element) {
    return { favor: weakened(luckPref.favor), reason: `${prefix}看作${natalLabel}剋${luckLabel}` };
  }
  return { favor: strengthened(natalPref.favor), reason: `${prefix}${luckLabel}與${natalLabel}同五行` };
}

/**
 * Judges one luck-cycle relation against the natal pillar it touches.
 * - 沖: clashing away a 忌 is 有利, clashing a 喜用 is 不利 (2026-09-02、09-23).
 * - 天干五合: 化不化看大運坐支 (2026-10-07). 化 → judged by the combined element;
 *   不化 → 合絆 (2026-10-14): binding a natal 喜用 is 減吉, binding a 忌 is 減凶.
 * - 地支六合、半合: 有合無化 (2026-09-09), judged as plain 生剋 between the two branches.
 * - 伏吟: the repeated element's role.
 * - 害 is 神煞 and not used in class (2026-09-23); 刑、破、暗合、暗會 get no verdict yet.
 */
export function judgeLuckRelation(
  text: string,
  scope: "stem" | "branch",
  luck: RelationSide,
  natalSide: RelationSide,
  preferences: LessonPreferences,
): RelationJudgement {
  const natal = preferences.elements[natalSide.element];
  const natalLabel = `原局${natalSide.char}${natal.name}（${natal.label}）`;
  const kind = relationKind(text);

  if (kind === "clash") {
    if (natal.favor === "good") return { favor: "bad", reason: `沖動${natalLabel}` };
    if (natal.favor === "bad") return { favor: "good", reason: `沖去${natalLabel}` };
    return { favor: "neutral", reason: `沖${natalLabel}` };
  }

  if (kind === "combine" && scope === "branch") {
    return judgeBranchCombination(luck, natalSide, preferences);
  }

  if (kind === "combine") {
    // Two effects: the natal side is bound (合絆), and the pair may turn into the combined element (合化).
    const bound = weakened(natal.favor);
    const boundText = natal.favor === "good" ? `合絆${natalLabel}` : natal.favor === "bad" ? `合去${natalLabel}` : `合${natalLabel}`;
    const combined = combinationElement(text);
    if (!combined) {
      return { favor: bound, reason: boundText };
    }
    const result = preferences.elements[combined];
    const transformed = strengthened(result.favor);
    if (luck.seat) {
      // 10/07：由大運坐支決定化或不化，不再兩可。
      if (seatTransforms(luck.seat, combined)) {
        return {
          favor: transformed,
          reason: `大運坐${luck.seat}，${text.replace(/合[木火土金水]$/, "")}合化為${result.name}（化不化看大運坐支），這步運以${result.name}論，${result.name}為${result.label}`,
        };
      }
      const effect = natal.favor === "good" ? "減吉" : natal.favor === "bad" ? "減凶" : "";
      return {
        favor: bound,
        reason: `${boundText}；大運坐${luck.seat}不是${result.name}的旺地，合而不化（剋合）${effect ? `，${effect}` : ""}`,
      };
    }
    const reason = `${boundText}；若合化為${result.name}，${result.name}為${result.label}`;
    if (bound === "neutral" || transformed === "neutral" || bound === transformed) {
      return { favor: bound !== "neutral" ? bound : transformed, reason };
    }
    // e.g. binding a 忌 but turning into a 忌: depends on whether it really 化.
    return { favor: "neutral", reason: `${reason}；吉凶看是否成化` };
  }

  if (kind === "echo") {
    if (natal.favor === "good") return { favor: "good", reason: `${natalLabel}再現，加重喜用` };
    if (natal.favor === "bad") return { favor: "bad", reason: `${natalLabel}再現，加重忌神` };
    return { favor: "neutral", reason: `${natalLabel}再現` };
  }

  if (text.includes("害")) {
    return { favor: "neutral", reason: `涉及${natalLabel}；害屬神煞，課堂不採用` };
  }

  return { favor: "neutral", reason: `涉及${natalLabel}；刑、破、暗合、暗會暫不判吉凶` };
}

// Several natal pillars can share one tag (e.g. 午 twice); a single 有利 and 不利 cancel to 中性.
export function combineJudgements(judgements: RelationJudgement[]): RelationJudgement {
  const favors = new Set(judgements.map((judgement) => judgement.favor).filter((favor) => favor !== "neutral"));
  const reason = [...new Set(judgements.map((judgement) => judgement.reason))].join("\n");
  if (favors.size === 1) {
    return { favor: [...favors][0], reason };
  }
  return { favor: "neutral", reason };
}
