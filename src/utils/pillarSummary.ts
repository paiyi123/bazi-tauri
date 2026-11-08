import type { LessonThreeStepAnalysis } from "../types/bazi";
import type { WuXing } from "../constants/wuxing";
import { ELEMENT_NAMES, lessonPreferences, type Favor } from "./lessonPreference";

// 手機版「四柱」頁的精簡摘要：只重排課堂三步法已算好的結果（天干生剋合化、地支沖合會刑害、
// 作用變化後強弱複核、喜忌方案），不做新的推論。細節留在「分析」頁的課堂卡片。

export type SummaryLine = {
  key: string;
  label: string;
  where: string;
  detail: string;
  verdict: Favor;
  muted: boolean;
};

export type SummaryChip = {
  key: string;
  text: string;
  note: string;
  muted: boolean;
};

export type SummaryElement = { element: WuXing; name: string };

export type PillarSummary = {
  strength: { label: string; detail: string; review: string | null };
  useGods: string[];
  planCount: number;
  noPlanReason: string | null;
  favorable: SummaryElement[];
  unfavorable: SummaryElement[];
  undecided: SummaryElement[];
  stemLines: SummaryLine[];
  stemRelations: SummaryChip[];
  branchLines: SummaryLine[];
};

const ORDER = Object.keys(ELEMENT_NAMES) as WuXing[];

function toFavor(value: string): Favor {
  return value === "good" || value === "bad" || value === "mixed" ? value : "neutral";
}

// 「年支戌（土，食傷）」「月干癸（傷官）」→ 年、月
function pillarsOf(positions: string[]): string {
  return positions.map((position) => position.charAt(0)).join("");
}

// 「月干癸（傷官）」→ 癸
function charOf(position: string): string {
  return position.match(/[干支](.)/)?.[1] ?? position.charAt(0);
}

function pillarOf(position: string): string {
  return position.charAt(0);
}

export function pillarSummary(analysis?: LessonThreeStepAnalysis | null): PillarSummary | null {
  if (!analysis) return null;
  const strength = analysis.strength;
  const checks = (strength.checks ?? [])
    .map((check) => `${check.item.replace(/（.*?）/, "")}${check.hit ? "✓" : "✗"}`)
    .join(" ");
  const party = `同黨 ${strength.samePartyCount}：異黨 ${strength.opposingPartyCount}`;
  const review = analysis.interactionStrength;
  const reviewText =
    review?.changed
      ? `作用變化後同黨 ${review.samePartyCount}：異黨 ${review.opposingPartyCount}，可能變為「${review.classification}」，請複核`
      : null;

  const plans = analysis.preferences?.plans ?? [];
  const useGods = [
    ...new Set(plans.flatMap((plan) => plan.useDirections.map((d) => `${d.family}（${d.element}）`))),
  ];
  const prefs = lessonPreferences(analysis);
  const pick = (favor: Favor) =>
    prefs
      ? ORDER.filter((element) => prefs.elements[element].favor === favor).map((element) => ({
          element,
          name: ELEMENT_NAMES[element],
        }))
      : [];

  const stems = analysis.stemInteractions;
  const stemLines: SummaryLine[] = (stems?.combinations ?? []).map((combo) => ({
    key: combo.id,
    label: `${combo.stems}合${combo.element}・${combo.kindLabel}${combo.direction ? `・${combo.direction}` : ""}`,
    where: pillarsOf(combo.positions),
    detail: `${combo.scope}；${combo.verdictLabel}${combo.needsConfirmation ? "；待確認" : ""}`,
    verdict: toFavor(combo.verdict),
    muted: combo.kind === "blocked",
  }));
  const stemRelations: SummaryChip[] = (stems?.relations ?? []).map((item) => ({
    key: item.id,
    text: `${pillarOf(item.from)}${charOf(item.from)}${item.relation}${pillarOf(item.to)}${charOf(item.to)}`,
    note: item.status === "active" ? item.distance : item.statusLabel,
    muted: item.status === "ignored",
  }));

  const branches = analysis.branchInteractions;
  const branchLines: SummaryLine[] = [
    ...(branches?.clashes ?? []).map((item) => ({
      key: item.id,
      label: `${item.branches}沖・${item.outcomeLabel}`,
      where: pillarsOf(item.positions),
      detail: `${item.distance}；${item.verdictLabel}`,
      verdict: toFavor(item.verdict),
      muted: false,
    })),
    ...(branches?.combinations ?? []).map((item) => ({
      key: item.id,
      label: `${item.branches}六合・${item.kindLabel}`,
      where: pillarsOf(item.positions),
      detail: `${item.adjacent ? "相鄰" : "隔位，僅供參考"}；${item.verdictLabel}${item.clashEffect ? `；${item.clashEffect}` : ""}`,
      verdict: toFavor(item.verdict),
      muted: !item.adjacent,
    })),
    ...(branches?.gatherings ?? []).map((item) => ({
      key: item.id,
      label: item.kindLabel,
      where: pillarsOf(item.positions),
      detail: item.verdictLabel,
      verdict: toFavor(item.verdict),
      muted: !item.formed && item.verdict === "neutral",
    })),
    ...(branches?.punishments ?? []).map((item) => ({
      key: item.id,
      label: item.kindLabel,
      where: pillarsOf(item.positions),
      detail: `${item.relation}；${item.verdictLabel}`,
      verdict: toFavor(item.verdict),
      muted: false,
    })),
  ];

  return {
    strength: {
      label: strength.classification,
      detail: [checks, party].filter(Boolean).join("；"),
      review: reviewText,
    },
    useGods,
    planCount: plans.length,
    noPlanReason: plans.length ? null : analysis.preferences?.noPlanReasons?.join("；") || "依據不足",
    favorable: pick("good"),
    unfavorable: pick("bad"),
    undecided: pick("mixed"),
    stemLines,
    stemRelations,
    branchLines,
  };
}
