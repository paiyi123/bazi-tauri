import { ref } from "vue";
import type { PillarSet } from "../utils/pillarText";

// 從別的地方（例如「書上相似命例」）要求直接用某組四柱排盤；App.vue 收到後切到「直接輸入四柱」並開始分析。
export interface PillarImportRequest {
  pillars: PillarSet;
  /** 顯示用，例如「《命理新論》p.215」 */
  source?: string;
  nonce: number;
}

const request = ref<PillarImportRequest | null>(null);

export function usePillarImport() {
  function importPillars(pillars: PillarSet, source?: string) {
    request.value = { pillars, source, nonce: Date.now() };
  }
  return { request, importPillars };
}
