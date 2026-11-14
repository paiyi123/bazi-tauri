import type { BaziResponse } from "../types/bazi";
import type { CourseStage } from "../composables/useLearning";
import { conclusionOf } from "./conclusion";
import { topicLeads } from "./topicLeads";
import { yongShenGuide } from "./yongShen";

// 解盤 10 步：照《八字諸書融會貫通指南》第十章「統一解盤流程」排列，每一步只重排已算好的結果。
// topic 是「看詳細」要跳去的分析分區；needs 是課程進度要上到哪裡才算學過。

export type StepStatus = "ok" | "warn" | "info" | "none";
export type ReadingStep = {
  no: number;
  title: string;
  status: StepStatus;
  summary: string;
  why: string[];
  topic: string | null;
  needs: CourseStage | null;
};

const STEMS = "甲乙丙丁戊己庚辛壬癸";
const BRANCHES = "子丑寅卯辰巳午未申酉戌亥";

/** 五虎遁：由年干推月干。月支寅月起算。 */
export function expectedMonthStem(yearStem: string, monthBranch: string): string | null {
  const y = STEMS.indexOf(yearStem);
  const m = BRANCHES.indexOf(monthBranch);
  if (y < 0 || m < 0) return null;
  const first = [2, 4, 6, 8, 0][y % 5]; // 甲己丙寅、乙庚戊寅、丙辛庚寅、丁壬壬寅、戊癸甲寅
  const offset = (m - 2 + 12) % 12;
  return STEMS[(first + offset) % 10];
}

/** 五鼠遁：由日干推時干。子時起算。 */
export function expectedHourStem(dayStem: string, hourBranch: string): string | null {
  const d = STEMS.indexOf(dayStem);
  const h = BRANCHES.indexOf(hourBranch);
  if (d < 0 || h < 0) return null;
  const first = [0, 2, 4, 6, 8][d % 5]; // 甲己甲子、乙庚丙子、丙辛戊子、丁壬庚子、戊癸壬子
  return STEMS[(first + h) % 10];
}

function checkChart(result: BaziResponse): Pick<ReadingStep, "status" | "summary" | "why"> {
  const why: string[] = [];
  const month = expectedMonthStem(result.yearPillar?.stem, result.monthPillar?.branch);
  const hour = expectedHourStem(result.dayPillar?.stem, result.hourPillar?.branch);
  if (month && month !== result.monthPillar.stem) {
    why.push(`五虎遁：${result.yearPillar.stem}年的${result.monthPillar.branch}月應該是${month}${result.monthPillar.branch}，這裡是${result.monthPillar.ganZhi}，請核對月柱。`);
  }
  if (hour && hour !== result.hourPillar.stem) {
    why.push(`五鼠遁：${result.dayPillar.stem}日的${result.hourPillar.branch}時應該是${hour}${result.hourPillar.branch}，這裡是${result.hourPillar.ganZhi}，請核對時柱。`);
  }
  const time = result.solarDateTime?.match(/\s(\d{1,2}):(\d{2})/);
  if (time) {
    const minutes = Number(time[1]) * 60 + Number(time[2]);
    if (minutes >= 22 * 60 + 40 || minutes <= 60 + 20) {
      why.push("出生時間接近子時：早子時、晚子時的換日方式，以及真太陽時，都可能讓日柱或時柱不同。");
    }
  }
  if (!why.length) {
    return { status: "ok", summary: "月柱、時柱符合五虎遁、五鼠遁，出生時間不在子時交界。", why: ["五虎遁由年干推月干，五鼠遁由日干推時干；書上命例也常有錯，所以先核對。"] };
  }
  return { status: "warn", summary: why[0], why };
}

/**
 * 11/04：月令被刑或被沖之後只剩本氣，取格要用本氣（例一：戌月被刑，時干透丁也取本氣戊的偏印格）。
 * App 的取格還沒照這條改，這裡只提醒對照。
 */
function monthBranchDisturbed(result: BaziResponse): string | null {
  const branches = result.quantModel?.lessonContent?.threeStepAnalysis?.branchInteractions;
  const touchesMonth = (positions: string[]) => positions.some((position) => position.startsWith("月支"));
  const hits = [
    ...(branches?.clashes ?? []).filter((item) => touchesMonth(item.positions)).map((item) => `${item.branches}沖`),
    ...(branches?.punishments ?? [])
      .filter((item) => item.kind !== "harm" && touchesMonth(item.positions))
      .map((item) => item.kindLabel),
  ];
  if (!hits.length) return null;
  const main = result.monthHiddenStems?.[0];
  const mainGod = result.monthBranchTenGods?.[0];
  const mainText = main && mainGod ? `月令${result.monthPillar.branch}的本氣是${main}（${mainGod}）` : "請看月令本氣";
  return `月令被刑沖（${hits.join("、")}）：老師 11/04 說刑沖之後月令只剩本氣，取格要用本氣。${mainText}，請和上面的格局對照。`;
}

export function readingSteps(result: BaziResponse): ReadingStep[] {
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  const data = conclusionOf(result);
  const guide = yongShenGuide(analysis);
  const lead = (topic: Parameters<typeof topicLeads>[1]) => topicLeads(result, topic).map((line) => line.text);
  const names = (items: { name: string }[] | undefined) => items?.map((item) => item.name).join("、") || "—";
  const steps: ReadingStep[] = [];

  steps.push({ no: 1, title: "核對命盤", topic: null, needs: null, ...checkChart(result) });

  const monthHit = monthBranchDisturbed(result);
  steps.push({
    no: 2,
    title: "定格局",
    status: !analysis ? "none" : monthHit ? "warn" : "ok",
    summary: analysis?.pattern?.ordinaryResult
      ? `${analysis.pattern.ordinaryResult}${monthHit ? "（月令被刑沖，要對照本氣）" : ""}`
      : "沒有課堂分析",
    why: [
      analysis?.pattern?.conclusion ?? "",
      ...(analysis?.pattern?.ordinaryCandidates ?? []).map((c) => `候選：${c.pattern}（${c.selection}）`),
      monthHit ?? "",
    ].filter(Boolean),
    topic: "pattern",
    needs: "pattern",
  });

  const interaction = lead("interaction");
  steps.push({
    no: 3,
    title: "看干支作用（合沖刑害）",
    status: analysis ? "info" : "none",
    summary: interaction[0] ?? "沒有課堂分析",
    why: [
      ...interaction.slice(1),
      analysis?.interactionStrength?.conclusion ?? "",
      (analysis?.branchInteractions?.punishments ?? []).length
        ? "刑、害、自刑都讀成地支本氣的五行生剋，再看喜忌；刑在月令是加強，在日時是鬆動（11/04）。害屬神煞，只當參考。"
        : "",
    ].filter(Boolean),
    topic: "interaction",
    needs: "stem",
  });

  const review = analysis?.interactionStrength;
  steps.push({
    no: 4,
    title: "判強弱（得令、得勢、得地）",
    status: !data ? "none" : review?.changed || data.agreement.status === "split" ? "warn" : "ok",
    summary: data ? `${data.strength.label}：${data.strength.checks}；${data.strength.party}` : "沒有課堂分析",
    why: [
      data?.strength.reason ?? "",
      data?.agreement.status === "split" ? `${data.agreement.text}。${data.agreement.detail}` : "",
      review?.changed ? `作用變化後同黨 ${review.samePartyCount}：異黨 ${review.opposingPartyCount}，可能變為「${review.classification}」，請複核。` : "",
    ].filter(Boolean),
    topic: "strength",
    needs: "strength",
  });

  const special = result.quantModel?.specialPattern?.primaryPattern;
  steps.push({
    no: 5,
    title: "排除特別格局",
    status: special ? "warn" : "ok",
    summary: special ? `可能是${special}，要用嚴格條件確認` : "不是特別格局，照一般格局取用神",
    why: lead("pattern").slice(-1),
    topic: "pattern",
    needs: "special",
  });

  steps.push({
    no: 6,
    title: "取用神",
    status: !guide ? "none" : guide.status === "decided" ? "ok" : "warn",
    summary: guide?.headline ?? "沒有課堂分析",
    why: guide?.steps.map((step) => `${step.label}：${step.text}`) ?? [],
    topic: "strength",
    needs: "yongshen",
  });

  steps.push({
    no: 7,
    title: "明喜忌",
    status: data ? "ok" : "none",
    summary: data ? `喜 ${names(data.favorable)}；忌 ${names(data.unfavorable)}${data.undecided.length ? `；未定 ${names(data.undecided)}` : ""}` : "沒有課堂分析",
    why: ["喜神生或幫用神；忌神剋用神；仇神生忌神；閒神影響不大。", "七殺、傷官不是壞字，正官、正財也不是好字，好壞全看喜忌。"],
    topic: "strength",
    needs: "yongshen",
  });

  const family = lead("family");
  const health = lead("health");
  const career = lead("career");
  steps.push({
    no: 8,
    title: "看人生各項",
    status: data ? "info" : "none",
    summary: family[0] ?? career[0] ?? "沒有課堂分析",
    why: [...family, ...health.slice(0, 1), ...career.slice(0, 1)],
    topic: "family",
    needs: "yongshen",
  });

  steps.push({
    no: 9,
    title: "看大運流年",
    status: data?.luck ? "info" : "none",
    summary: data?.luck
      ? `目前大運 ${data.luck.ganZhi}（${data.luck.range}）${({ good: "喜", bad: "忌", mixed: "干支不同", neutral: "未列喜忌" })[data.luck.overall]}${data.annual ? `；今年 ${data.annual.ganZhi}${({ good: "喜", bad: "忌", mixed: "干支不同", neutral: "未列喜忌" })[data.annual.overall]}` : ""}`
      : "沒有大運資料",
    why: [
      "看一年的步驟：大運喜忌（地支為主）→ 運和原局的合沖 → 加上流年 → 運喜歲喜最好、運忌歲忌最差 → 看動到哪一宮、哪一個十神。",
      "大運要干支一起看：原局有病無藥時，運的天干地支同一個五行（干支同氣）才真的有用（11/04）。",
      "大運沖原局不一定大運贏：原局沒有那個五行、被沖的字又在月令得勢時，反而激怒旺神（11/04）。",
      ...(data?.highlights.map((item) => `${item.label}：${item.detail}`) ?? []),
    ],
    topic: "luck",
    needs: "yongshen",
  });

  steps.push({
    no: 10,
    title: "用已發生的事反驗",
    status: "info",
    summary: "把過去幾年發生的事記在「學習」分區的反驗筆記，看和 ▲▼ 符不符合。",
    why: ["結論和過去不符時，回頭檢查第 4 步（強弱）和第 6 步（用神），這是最容易判錯的兩步，不要硬套標籤。"],
    topic: "learn",
    needs: null,
  });

  return steps;
}
