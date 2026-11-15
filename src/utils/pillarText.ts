// 把書上抄來的四柱文字（例如「壬午 己酉 甲申 甲子」）拆成年、月、日、時四柱。
// 書的寫法有兩種順序：年月日時（一般）、日時月年（有些書先寫日柱）。
// 「自動」用五虎遁（年干定月干）、五鼠遁（日干定時干）判斷哪一種順序說得通。

export type PillarOrder = "auto" | "ymdh" | "dhmy";
export type PillarSet = { year: string; month: string; day: string; hour: string };
export type PillarTextResult =
  | { ok: true; pillars: PillarSet; order: Exclude<PillarOrder, "auto">; note: string }
  | { ok: false; error: string };

export const STEMS = "甲乙丙丁戊己庚辛壬癸";
export const BRANCHES = "子丑寅卯辰巳午未申酉戌亥";
export const ORDER_LABEL: Record<PillarOrder, string> = { auto: "自動判斷", ymdh: "年 月 日 時", dhmy: "日 時 月 年" };

/** 五虎遁：這個年干的這個月支，月干該是什麼。 */
export function expectedMonthStem(yearStem: string, monthBranch: string): string | null {
  const y = STEMS.indexOf(yearStem);
  const b = BRANCHES.indexOf(monthBranch);
  if (y < 0 || b < 0) return null;
  const tigerStart = [2, 4, 6, 8, 0][y % 5]; // 甲己→丙寅、乙庚→戊寅、丙辛→庚寅、丁壬→壬寅、戊癸→甲寅
  return STEMS[(tigerStart + ((b - 2 + 12) % 12)) % 10];
}

/** 五鼠遁：這個日干的這個時支，時干該是什麼。 */
export function expectedHourStem(dayStem: string, hourBranch: string): string | null {
  const d = STEMS.indexOf(dayStem);
  const b = BRANCHES.indexOf(hourBranch);
  if (d < 0 || b < 0) return null;
  const ratStart = [0, 2, 4, 6, 8][d % 5]; // 甲己→甲子、乙庚→丙子、丙辛→戊子、丁壬→庚子、戊癸→壬子
  return STEMS[(ratStart + b) % 10];
}

/** 干支是不是合法的六十甲子（陽干配陽支、陰干配陰支）。 */
const validPair = (pair: string) => {
  const s = STEMS.indexOf(pair[0]);
  const b = BRANCHES.indexOf(pair[1]);
  return s >= 0 && b >= 0 && s % 2 === b % 2;
};

/** 這組四柱在五虎遁、五鼠遁下說不說得通（0～2 分）。 */
export function pillarFit(p: PillarSet): number {
  return Number(expectedMonthStem(p.year[0], p.month[1]) === p.month[0]) + Number(expectedHourStem(p.day[0], p.hour[1]) === p.hour[0]);
}

const arrange = (list: string[], order: Exclude<PillarOrder, "auto">): PillarSet =>
  order === "ymdh" ? { year: list[0], month: list[1], day: list[2], hour: list[3] } : { day: list[0], hour: list[1], month: list[2], year: list[3] };

export function parsePillarText(text: string, order: PillarOrder = "auto"): PillarTextResult {
  const pairs = [...text.matchAll(new RegExp(`[${STEMS}][${BRANCHES}]`, "g"))].map((m) => m[0]);
  if (pairs.length !== 4) return { ok: false, error: `要四組干支（例如：壬午 己酉 甲申 甲子），現在找到 ${pairs.length} 組。` };
  const bad = pairs.filter((pair) => !validPair(pair));
  if (bad.length) return { ok: false, error: `${bad.join("、")} 不是六十甲子裡的干支（陽干要配陽支、陰干配陰支）。` };
  if (order !== "auto") {
    const pillars = arrange(pairs, order);
    const fit = pillarFit(pillars);
    return { ok: true, pillars, order, note: fit === 2 ? "" : "⚠ 照這個順序，月干或時干和五虎遁、五鼠遁對不上，請確認順序或書上的四柱。" };
  }
  const ymdh = arrange(pairs, "ymdh");
  const dhmy = arrange(pairs, "dhmy");
  const a = pillarFit(ymdh);
  const b = pillarFit(dhmy);
  if (a > b) return { ok: true, pillars: ymdh, order: "ymdh", note: `判斷為「${ORDER_LABEL.ymdh}」${a < 2 ? "，但月干或時干仍對不上，請確認" : ""}。` };
  if (b > a) return { ok: true, pillars: dhmy, order: "dhmy", note: `判斷為「${ORDER_LABEL.dhmy}」${b < 2 ? "，但月干或時干仍對不上，請確認" : ""}。` };
  return {
    ok: true,
    pillars: ymdh,
    order: "ymdh",
    note: a === 2 ? "兩種順序都說得通，先當「年 月 日 時」；如果書是先寫日柱，請改選「日 時 月 年」。" : "⚠ 兩種順序的月干、時干都對不上，先當「年 月 日 時」，請確認書上的四柱。",
  };
}
