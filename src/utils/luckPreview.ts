import type { BaziResponse, DaYun, LiuNian, LuckPreviewPillar } from "../types/bazi";
import { baziYearAt, currentLuckIndex } from "./baziNow";

// 大運加進四柱當預覽柱：四柱下方的大運列和四柱上的「換大運」清單共用同一套做法。

export interface LuckRowView extends DaYun {
  /** 歲數寫法和大運面板相同（結束歲數 +1） */
  ages: string;
  years: string;
  isCurrent: boolean;
}

const span = (a: number | null, b: number | null) => (a == null ? "" : b == null ? `${a}` : `${a}-${b}`);

/** 命盤的大運列表（原本順序：最早的在前）。目前大運看交運時刻（不是西曆年份）。 */
export function luckRows(result: BaziResponse, now: Date = new Date()): LuckRowView[] {
  const current = currentLuckIndex(result, now);
  return (result.daYun ?? [])
    .filter((row) => row.ganZhi?.length === 2)
    .map((row) => ({
      ...row,
      ages: span(row.startAge, row.endAge == null ? null : row.endAge + 1),
      years: span(row.startYear, row.endYear),
      isCurrent: row.index === current,
    }));
}

/** 把一步大運變成四柱的預覽柱。 */
export function luckPreviewPillar(row: LuckRowView): LuckPreviewPillar {
  return {
    key: `luck-${row.index}`,
    label: "大運",
    ganZhi: row.ganZhi,
    stemTenGod: row.stemTenGod,
    branchHiddenStems: row.branchHiddenStems ?? [],
    branchTenGods: row.branchTenGods ?? [],
    xun: row.xun,
    xunKong: row.xunKong,
    isCurrentYear: row.isCurrent,
  };
}

export interface AnnualRowView extends LiuNian {
  luckIndex: number;
  isCurrent: boolean;
}

/** 一步大運裡的流年（原本順序：最早的在前）。 */
/** year：今年的干支年（以立春分年）。 */
export function annualRows(row: DaYun | null | undefined, year = baziYearAt()): AnnualRowView[] {
  return (row?.liuNian ?? [])
    .filter((item) => item.ganZhi?.length === 2)
    .map((item) => ({ ...item, luckIndex: row!.index, isCurrent: item.year === year }));
}

/** 流年預覽柱的 key：和大運流年面板相同（annual-大運序號-流年序號）。 */
export const annualKey = (luckIndex: number, annualIndex: number) => `annual-${luckIndex}-${annualIndex}`;

/** 把一個流年變成四柱的預覽柱（接在大運柱後面）。 */
export function annualPreviewPillar(row: AnnualRowView): LuckPreviewPillar {
  return {
    key: annualKey(row.luckIndex, row.index),
    label: "流年",
    ganZhi: row.ganZhi,
    stemTenGod: row.stemTenGod,
    branchHiddenStems: row.branchHiddenStems ?? [],
    branchTenGods: row.branchTenGods ?? [],
    xun: row.xun,
    xunKong: row.xunKong,
    isCurrentYear: row.isCurrent,
  };
}

/** 從預覽柱的 key 找回它是哪一個流年。 */
export function annualFromKey(result: BaziResponse, key: string | null | undefined, year = baziYearAt()): AnnualRowView | null {
  const match = key?.match(/^annual-(\d+)-(\d+)$/);
  if (!match) return null;
  const luck = result.daYun?.find((row) => row.index === Number(match[1]));
  return annualRows(luck, year).find((item) => item.index === Number(match[2])) ?? null;
}

/** 十神縮成一兩個字放在小格子裡（例：正印→正印、比肩→比肩；沒有就空白）。 */
export const tenGodShort = (value?: string | null) => (value && value !== "-" ? value : "");
