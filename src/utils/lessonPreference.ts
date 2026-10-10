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

/**
 * Judges one luck-cycle relation against the natal pillar it touches.
 * - 沖: clashing away a 忌 is 有利, clashing a 喜用 is 不利.
 * - 合: binding a natal 喜用 is 不利, binding a 忌 is 有利; if the natal side has no role, the
 *   combined element decides, worded as conditional because 合 does not always 化.
 * - 伏吟: the repeated element's role.
 * - 刑、破、害、暗合、暗會: no verdict yet, only which roles are involved.
 */
export function judgeLuckRelation(
  text: string,
  natalChar: string,
  natalElement: WuXing,
  preferences: LessonPreferences,
): RelationJudgement {
  const natal = preferences.elements[natalElement];
  const natalLabel = `原局${natalChar}${natal.name}（${natal.label}）`;
  const kind = relationKind(text);

  if (kind === "clash") {
    if (natal.favor === "good") return { favor: "bad", reason: `沖動${natalLabel}` };
    if (natal.favor === "bad") return { favor: "good", reason: `沖去${natalLabel}` };
    return { favor: "neutral", reason: `沖${natalLabel}` };
  }

  if (kind === "combine") {
    // Two effects: the natal side is bound (合絆), and the pair may turn into the combined element (合化).
    const bound: Verdict = natal.favor === "good" ? "bad" : natal.favor === "bad" ? "good" : "neutral";
    const boundText = natal.favor === "good" ? `合絆${natalLabel}` : natal.favor === "bad" ? `合去${natalLabel}` : `合${natalLabel}`;
    const combined = combinationElement(text);
    if (!combined) {
      return { favor: bound, reason: boundText };
    }
    const result = preferences.elements[combined];
    const transformed: Verdict = result.favor === "good" || result.favor === "bad" ? result.favor : "neutral";
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

  return { favor: "neutral", reason: `涉及${natalLabel}；刑破害、暗合暗會暫不判吉凶` };
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
