import { computed, ref, watch } from "vue";

// 新手／研究模式。新手只看課堂三步法的主答案與各主題重點；研究模式顯示全部書的原始卡片與計分。
export type ViewMode = "beginner" | "research";

const STORAGE_KEY = "bazi:view-mode:v1";

function load(): ViewMode {
  try {
    return window.localStorage.getItem(STORAGE_KEY) === "research" ? "research" : "beginner";
  } catch {
    return "beginner";
  }
}

const mode = ref<ViewMode>(typeof window === "undefined" ? "beginner" : load());

watch(mode, (value) => {
  try {
    window.localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // 無法儲存時只影響下次開啟的預設值
  }
});

export function useViewMode() {
  return {
    mode,
    isResearch: computed(() => mode.value === "research"),
    isBeginner: computed(() => mode.value === "beginner"),
  };
}
