import type { BaziResponse, DaYun, LiuNian } from "../types/bazi";
import { BRANCH_WUXING, STEM_WUXING, type WuXing } from "../constants/wuxing";
import { ELEMENT_NAMES, lessonPreferences, type Favor } from "./lessonPreference";
import { yongShenGuide } from "./yongShen";
import { annualEvents, majorLuckEvents } from "./luckEvents";

// 「本命結論」卡：給新手一眼看懂的答案。主答案一律用課堂三步法（許老師），
// 量化五行力與陸致極只拿來看「是否一致」，不另外給答案。這裡不做新的推論，只重排已算好的結果。

export type ConclusionElement = { element: WuXing; name: string };

export type Agreement = {
  /** agree：三法同一邊；split：有方法判在另一邊（邊界盤）；unknown：資料不足 */
  status: "agree" | "split" | "unknown";
  text: string;
  detail: string;
};

export type FavorMark = { char: string; element: WuXing; name: string; favor: Favor };

export type ConclusionLuck = {
  label: string;
  ganZhi: string;
  range: string;
  marks: FavorMark[];
  /** 干支兩字合起來的傾向：good／bad／mixed／neutral */
  overall: Favor;
};

export type Conclusion = {
  dayMaster: string;
  pattern: string;
  strength: { label: string; checks: string; party: string; reason: string };
  useElements: ConclusionElement[];
  favorable: ConclusionElement[];
  unfavorable: ConclusionElement[];
  undecided: ConclusionElement[];
  noPlanReason: string | null;
  agreement: Agreement;
  luck: ConclusionLuck | null;
  annual: ConclusionLuck | null;
  highlights: { label: string; detail: string }[];
};

const ORDER = Object.keys(ELEMENT_NAMES) as WuXing[];
const ELEMENT_BY_NAME = Object.fromEntries(ORDER.map((key) => [ELEMENT_NAMES[key], key])) as Record<string, WuXing>;

function side(value: string | undefined): "強" | "弱" | null {
  if (!value) return null;
  if (value.includes("強") || value.includes("旺")) return "強";
  if (value.includes("弱") || value.includes("衰")) return "弱";
  return null;
}

function agreementOf(result: BaziResponse, lessonLabel: string | undefined): Agreement {
  const quant = result.quantModel;
  const views = [
    { name: "課堂", value: lessonLabel },
    { name: "算分法（量化五行力）", value: quant?.strengthLabel },
    { name: "看根法（陸致極）", value: quant?.luStrength?.verdict },
  ].map((view) => ({ ...view, side: side(view.value) }));
  const lesson = views[0].side;
  const known = views.filter((view) => view.side);
  const detail = views.map((view) => `${view.name}：${view.value || "—"}`).join("；");
  if (!lesson || known.length < 2) {
    return { status: "unknown", text: "資料不足，無法比對各方法", detail };
  }
  const others = known.slice(1).filter((view) => view.side !== lesson);
  if (!others.length) {
    return { status: "agree", text: "各方法一致", detail };
  }
  return {
    status: "split",
    text: `邊界盤：${others.map((view) => view.name.replace(/（.*）/, "")).join("、")}判為身${others[0].side}`,
    detail: `${detail}。強弱接近中間時各書最容易判不一樣，建議用過去大運發生的事反驗。`,
  };
}

export function marksOf(ganZhi: string, favorOf: (element: WuXing) => Favor): FavorMark[] {
  const marks: FavorMark[] = [];
  const stem = STEM_WUXING[ganZhi.charAt(0)];
  const branch = BRANCH_WUXING[ganZhi.charAt(1)];
  if (stem) marks.push({ char: ganZhi.charAt(0), element: stem, name: ELEMENT_NAMES[stem], favor: favorOf(stem) });
  if (branch) marks.push({ char: ganZhi.charAt(1), element: branch, name: ELEMENT_NAMES[branch], favor: favorOf(branch) });
  return marks;
}

export function overallOf(marks: FavorMark[]): Favor {
  // 大運以地支為主（各書多數意見），所以干支不同時以地支為準，但標成 mixed 讓畫面寫「干支不同」。
  const favors = new Set(marks.map((mark) => mark.favor));
  if (favors.size === 1) return marks[0]?.favor ?? "neutral";
  if (favors.has("good") && favors.has("bad")) return "mixed";
  if (favors.has("good")) return "good";
  if (favors.has("bad")) return "bad";
  return "mixed";
}

export function findCurrentLuck(daYun: DaYun[], year: number): { luck: DaYun | null; annual: LiuNian | null } {
  const luck = daYun.find((row) => row.liuNian?.some((item) => item.year === year)) ?? null;
  const annual = luck?.liuNian?.find((item) => item.year === year) ?? null;
  return { luck, annual };
}

export function conclusionOf(result: BaziResponse, year = new Date().getFullYear()): Conclusion | null {
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  if (!analysis) return null;
  const strength = analysis.strength;
  const prefs = lessonPreferences(analysis);
  const guide = yongShenGuide(analysis);
  const pick = (favor: Favor): ConclusionElement[] =>
    prefs ? ORDER.filter((key) => prefs.elements[key].favor === favor).map((key) => ({ element: key, name: ELEMENT_NAMES[key] })) : [];
  const favorOf = (element: WuXing): Favor => prefs?.elements[element].favor ?? "neutral";
  const plans = analysis.preferences?.plans ?? [];

  const natal = [result.yearPillar, result.monthPillar, result.dayPillar, result.hourPillar].map(
    (pillar) => `${pillar?.stem ?? ""}${pillar?.branch ?? ""}`,
  );
  const { luck, annual } = findCurrentLuck(result.daYun ?? [], year);
  const luckView = luck?.ganZhi
    ? (() => {
        const marks = marksOf(luck.ganZhi, favorOf);
        return {
          label: "目前大運",
          ganZhi: luck.ganZhi,
          range: luck.startYear && luck.endYear ? `${luck.startYear}–${luck.endYear}` : "",
          marks,
          overall: overallOf(marks),
        };
      })()
    : null;
  const annualView = annual?.ganZhi
    ? (() => {
        const marks = marksOf(annual.ganZhi, favorOf);
        return { label: "今年", ganZhi: annual.ganZhi, range: String(annual.year), marks, overall: overallOf(marks) };
      })()
    : null;

  // 重點只取陸致極「較嚴重」（warn）的運歲事件，最多三條；其他標籤留在大運表。
  const lu = result.quantModel?.luStrength;
  const ctx = { natal, strength: lu?.category, rootless: lu ? lu.roots.length === 0 : undefined, verdict: lu?.verdict };
  const highlights =
    natal.every((item) => item.length === 2) && luck?.ganZhi
      ? [
          ...(annual?.ganZhi ? annualEvents(annual.ganZhi, luck.ganZhi, ctx).map((event) => ({ ...event, at: `今年` })) : []),
          ...majorLuckEvents(luck.ganZhi, ctx).map((event) => ({ ...event, at: "大運" })),
        ]
          .filter((event) => event.level === "warn")
          .filter((event, index, list) => list.findIndex((other) => other.label === event.label) === index)
          .slice(0, 3)
          .map((event) => ({ label: `${event.at}：${event.label}`, detail: event.detail }))
      : [];

  const useNames = guide?.useElements ?? [];
  return {
    dayMaster: result.dayMaster,
    pattern: analysis.pattern?.ordinaryResult || analysis.pattern?.conclusion || "",
    strength: {
      label: strength.classification,
      checks: (strength.checks ?? []).map((check) => `${check.item.replace(/（.*?）/, "")}${check.hit ? "✓" : "✗"}`).join(" "),
      party: `同黨 ${strength.samePartyCount}：異黨 ${strength.opposingPartyCount}`,
      reason: strength.verdictReason ?? "",
    },
    useElements: useNames
      .map((name) => ELEMENT_BY_NAME[name])
      .filter((key): key is WuXing => !!key)
      .map((key) => ({ element: key, name: ELEMENT_NAMES[key] })),
    favorable: pick("good"),
    unfavorable: pick("bad"),
    undecided: pick("mixed"),
    noPlanReason: plans.length ? null : analysis.preferences?.noPlanReasons?.join("；") || "依據不足",
    agreement: agreementOf(result, strength.classification),
    luck: luckView,
    annual: annualView,
    highlights,
  };
}
