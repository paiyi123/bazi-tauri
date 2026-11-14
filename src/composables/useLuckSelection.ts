import { ref } from "vue";
import type { BaziResponse } from "../types/bazi";

// 大運／流年／流月選到哪一格，整個 App 共用：四柱下方的大運列和正式的「大運流年」面板同步。
// 只有換了一張命盤才回到預設（目前大運），切換手機分頁、面板重新出現時保留選擇。

const luckIndex = ref<number | null>(null);
const annualYear = ref<number | null>(null);
const monthIndex = ref<number | null>(null);
let initializedFor: object | null = null;

export function useLuckSelection() {
  /** 這張命盤第一次出現時設成預設（目前大運、今年），並回傳 true；之後都回傳 false。 */
  function ensureFor(result: BaziResponse) {
    if (initializedFor === result) return false;
    initializedFor = result;
    const year = new Date().getFullYear();
    const current = result.daYun?.find((row) => row.liuNian?.some((item) => item.year === year)) ?? result.daYun?.[0];
    luckIndex.value = current?.index ?? null;
    annualYear.value = current?.liuNian?.some((item) => item.year === year) ? year : null;
    monthIndex.value = null;
    return true;
  }
  return { luckIndex, annualYear, monthIndex, ensureFor };
}
