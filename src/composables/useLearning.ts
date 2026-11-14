import { ref, watch } from "vue";

// 學習設定：課程進度（還沒上到的步驟先收起）與練習模式（先自己判斷再看答案）。存在本機 localStorage。
export type CourseStage = "all" | "pattern" | "strength" | "yongshen" | "special" | "stem" | "branch";

export const COURSE_STAGES: { value: CourseStage; label: string; dates: string }[] = [
  { value: "all", label: "全部顯示", dates: "" },
  { value: "pattern", label: "格局", dates: "07-01～07-15" },
  { value: "strength", label: "日主強弱", dates: "07-22～07-29" },
  { value: "yongshen", label: "取用神與喜忌", dates: "08-05～08-26" },
  { value: "special", label: "特別格局用神", dates: "09-02～09-09" },
  { value: "stem", label: "天干生剋合化", dates: "10-07～10-14" },
  { value: "branch", label: "地支沖合會刑害", dates: "10-21～11-04" },
];

const PROGRESS_KEY = "bazi:course-progress:v1";
const PRACTICE_KEY = "bazi:practice-mode:v1";

function read(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string) {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // 無法儲存時只影響下次開啟的預設值
  }
}

const storedStage = typeof window === "undefined" ? null : read(PROGRESS_KEY);
const courseStage = ref<CourseStage>(COURSE_STAGES.some((stage) => stage.value === storedStage) ? (storedStage as CourseStage) : "all");
const practiceMode = ref(typeof window !== "undefined" && read(PRACTICE_KEY) === "1");

watch(courseStage, (value) => write(PROGRESS_KEY, value));
watch(practiceMode, (value) => write(PRACTICE_KEY, value ? "1" : "0"));

/** 這個主題在目前的課程進度下是否已經上過。stage 為 all 時全部算學過。 */
export function isLearned(progress: CourseStage, needs: CourseStage | null): boolean {
  if (!needs || progress === "all") return true;
  const order = COURSE_STAGES.map((stage) => stage.value);
  return order.indexOf(needs) <= order.indexOf(progress);
}

export function useLearning() {
  return { courseStage, practiceMode };
}
