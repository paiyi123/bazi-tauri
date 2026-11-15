import type { BaziResponse, DaYun } from "../types/bazi";

// 「今年」「目前大運」不能用西曆年份判斷：
// - 八字的年以立春為界，1/1 到立春前（約 2/4）還是上一個干支年。
// - 大運以交運那一刻為界，每步大運列出的 11 個流年會和下一步重疊（交運那年兩邊都有），
//   不能用「流年列表裡有沒有今年」來判斷目前大運。

const DEG = Math.PI / 180;

/** 太陽視黃經（度），Meeus《Astronomical Algorithms》第 25 章低精度公式，誤差約 0.01°（約 15 分鐘）。 */
function solarLongitude(jd: number): number {
  const t = (jd - 2451545.0) / 36525;
  const l0 = 280.46646 + 36000.76983 * t + 0.0003032 * t * t;
  const m = (357.52911 + 35999.05029 * t - 0.0001537 * t * t) * DEG;
  const c =
    (1.914602 - 0.004817 * t - 0.000014 * t * t) * Math.sin(m) +
    (0.019993 - 0.000101 * t) * Math.sin(2 * m) +
    0.000289 * Math.sin(3 * m);
  const omega = (125.04 - 1934.136 * t) * DEG;
  const lambda = l0 + c - 0.00569 - 0.00478 * Math.sin(omega);
  return ((lambda % 360) + 360) % 360;
}

const toJd = (date: Date) => date.getTime() / 86400000 + 2440587.5;
const fromJd = (jd: number) => new Date((jd - 2440587.5) * 86400000);

/** 某年立春（太陽到黃經 315°）的時刻。 */
export function liChunTime(year: number): Date {
  let jd = toJd(new Date(Date.UTC(year, 1, 4)));
  for (let i = 0; i < 8; i += 1) {
    let diff = 315 - solarLongitude(jd);
    if (diff > 180) diff -= 360;
    if (diff < -180) diff += 360;
    jd += diff / 0.9856;
    if (Math.abs(diff) < 1e-6) break;
  }
  return fromJd(jd);
}

/** 這個時刻屬於哪個干支年（以立春為界，回傳西曆年份，App 的流年就用這個年份標示）。 */
export function baziYearAt(now: Date = new Date()): number {
  const year = now.getFullYear();
  return now.getTime() < liChunTime(year).getTime() ? year - 1 : year;
}

/** 解析 Rust 回傳的「YYYY-MM-DD HH:mm:ss」（台灣／北京時間 UTC+8）。 */
function parseSolar(text?: string | null): Date | null {
  const match = text?.match(/^(\d{1,4})-(\d{1,2})-(\d{1,2})(?:[ T](\d{1,2}):(\d{1,2})(?::(\d{1,2}))?)?/);
  if (!match) return null;
  const [, y, mo, d, h = "0", mi = "0", s = "0"] = match;
  return new Date(Date.UTC(Number(y), Number(mo) - 1, Number(d), Number(h) - 8, Number(mi), Number(s)));
}

/** 每步大運的交運時刻：第一步依起運時刻，之後每十年同一天（2/29 改 2/28）。 */
export function luckStartTimes(result: BaziResponse): Map<number, Date> {
  const out = new Map<number, Date>();
  const first = parseSolar(result.luckStart?.startSolar);
  const rows = (result.daYun ?? []).filter((row) => row.ganZhi?.length === 2);
  if (!first) return out;
  const ordered = [...rows].sort((a, b) => (a.startYear ?? a.index) - (b.startYear ?? b.index));
  ordered.forEach((row, order) => {
    const date = new Date(first.getTime());
    const month = date.getUTCMonth();
    date.setUTCFullYear(date.getUTCFullYear() + order * 10);
    if (date.getUTCMonth() !== month) date.setUTCDate(0);
    out.set(row.index, date);
  });
  return out;
}

/** 目前所在的大運（交運之後才算進下一步）；還沒起運或沒有日期（直接輸入四柱）時回傳 null。 */
export function currentLuckRow(result: BaziResponse, now: Date = new Date()): DaYun | null {
  const starts = luckStartTimes(result);
  const rows = (result.daYun ?? []).filter((row) => row.ganZhi?.length === 2);
  if (starts.size) {
    let current: DaYun | null = null;
    for (const row of rows) {
      const start = starts.get(row.index);
      if (start && start.getTime() <= now.getTime() && (!current || start.getTime() >= (starts.get(current.index)?.getTime() ?? 0))) {
        current = row;
      }
    }
    return current;
  }
  // 沒有起運時刻時退回年份判斷：取起運年份不晚於今年（干支年）的最後一步。
  const year = baziYearAt(now);
  const started = rows.filter((row) => row.startYear != null && row.startYear <= year);
  return started.length ? started.reduce((a, b) => ((a.startYear ?? 0) >= (b.startYear ?? 0) ? a : b)) : null;
}

/** 目前大運的 index；還沒起運時回傳 null。 */
export function currentLuckIndex(result: BaziResponse, now: Date = new Date()): number | null {
  return currentLuckRow(result, now)?.index ?? null;
}
