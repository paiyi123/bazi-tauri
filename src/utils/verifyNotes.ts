import type { BaziResponse } from "../types/bazi";
import type { WuXing } from "../constants/wuxing";
import { lessonPreferences, type Favor } from "./lessonPreference";
import { marksOf, overallOf, type FavorMark } from "./conclusion";

// 反驗筆記：把已經發生的事記在那一年，和那一年的大運、流年喜忌（▲▼）對照。
// 好事落在喜年、壞事落在忌年算「符合」，相反算「不符」；婚、財、健等只記錄，不判符不符。

export type NoteKind = "good" | "bad" | "marriage" | "wealth" | "health" | "other";
export type VerifyNote = { id: string; year: number; kind: NoteKind; text: string };

export const NOTE_KINDS: { value: NoteKind; label: string }[] = [
  { value: "good", label: "好事" },
  { value: "bad", label: "壞事" },
  { value: "marriage", label: "婚戀" },
  { value: "wealth", label: "財務" },
  { value: "health", label: "健康" },
  { value: "other", label: "其他" },
];

export type YearView = {
  year: number;
  age: number | null;
  luck: string;
  annual: string;
  luckMarks: FavorMark[];
  annualMarks: FavorMark[];
  /** 以流年為主，流年干支喜忌不同時看大運。 */
  verdict: Favor;
};

export type CheckedNote = VerifyNote & { view: YearView | null; match: "match" | "miss" | "none" };

export function chartKey(result: BaziResponse): string {
  return [result.baZi, result.gender ?? "", result.solarDateTime].join("|");
}

export function yearView(result: BaziResponse, year: number): YearView | null {
  const prefs = lessonPreferences(result.quantModel?.lessonContent?.threeStepAnalysis);
  if (!prefs) return null;
  const favorOf = (element: WuXing): Favor => prefs.elements[element].favor;
  for (const luck of result.daYun ?? []) {
    const annual = luck.liuNian?.find((item) => item.year === year);
    if (!annual) continue;
    const luckMarks = marksOf(luck.ganZhi, favorOf);
    const annualMarks = marksOf(annual.ganZhi, favorOf);
    const annualOverall = overallOf(annualMarks);
    const luckOverall = overallOf(luckMarks);
    const verdict = annualOverall === "good" || annualOverall === "bad" ? annualOverall : luckOverall;
    return { year, age: annual.age ?? null, luck: luck.ganZhi, annual: annual.ganZhi, luckMarks, annualMarks, verdict };
  }
  return null;
}

export function checkNotes(result: BaziResponse, notes: VerifyNote[]) {
  const checked: CheckedNote[] = [...notes]
    .sort((a, b) => a.year - b.year)
    .map((note) => {
      const view = yearView(result, note.year);
      let match: CheckedNote["match"] = "none";
      if (view && (note.kind === "good" || note.kind === "bad") && (view.verdict === "good" || view.verdict === "bad")) {
        match = view.verdict === note.kind ? "match" : "miss";
      }
      return { ...note, view, match };
    });
  const hits = checked.filter((note) => note.match === "match").length;
  const misses = checked.filter((note) => note.match === "miss").length;
  const judged = hits + misses;
  return {
    notes: checked,
    hits,
    misses,
    rate: judged ? Math.round((hits / judged) * 100) : null,
    // 至少三件能判斷的事、而且一半以上不符時，提醒回頭檢查強弱與用神。
    warn: judged >= 3 && hits * 2 < judged,
  };
}
