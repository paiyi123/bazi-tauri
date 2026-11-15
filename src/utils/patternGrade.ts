import type { BaziResponse, DaYun } from "../types/bazi";
import { STEM_WUXING, BRANCH_WUXING } from "../constants/wuxing";
import { basicStructure } from "./basicStructure";
import { tenGodOverview } from "./tenGodCombos";
import { specialPatternReport, type PatternCandidate } from "./specialPatterns";
import { lessonPreferences, type Favor } from "./lessonPreference";
import { yongShenGuide } from "./yongShen";
import { elementFlow } from "./elementFlow";
import { climateReport, flowReport, useQuality } from "./fourBooks";
import { tenGod } from "./lifeTopics";

/**
 * 「格局高低總評」：把已經算好的各家結果收在一起，給一個上／中上／中／中下／下的提示。
 *
 * 架構照吳俊民《命理新論》下冊 p.936–937 看富貴的三個步驟：
 *   一看格局是否成立，二看喜用是否得力，三看行運向背。
 * 再補上蘇民峰《八字論命》第 12 章「格局高低六項」（真假、虛實、清濁、有力、有情、團結）、
 * 陸致極《基礎教程》p127–130「格局看單純與有力、用神看有力與有情」，
 * 以及許老師 2026-10-07「喜用的力量與命格高低（命有 A、B、C 等級）」。
 *
 * 這裡不做新的命理推論，只把各卡片已經得到的結果換成加分、扣分，每一條都寫出處。
 * 強弱本身不分好壞，只看平衡；極強極弱時先看是不是特別格局。
 * 結論只是提示：老師說「中和只是好命，好命不等於富命，富命也不等於貴命」（2026-07-29）。
 */

export type GradeLevel = "上" | "中上" | "中" | "中下" | "下";

export interface GradeReason {
  text: string;
  /** 加減分；0 是只提醒、不計分。 */
  delta: number;
  source: string;
}

export interface GradeDimension {
  key: "formation" | "clarity" | "balance" | "useGod" | "flow" | "luck";
  title: string;
  /** 這一項在問什麼（白話）。 */
  question: string;
  /** 夾在 -2～+2 之間的分數。 */
  score: number;
  /** 各條理由加起來的原始分（夾之前）。 */
  raw: number;
  reasons: GradeReason[];
  /** 這一項不計分的原因（例如成特別格局時大運照普通格局喜忌算不準）。 */
  skipped?: string;
}

export interface PatternGrade {
  level: GradeLevel;
  levelText: string;
  total: number;
  /** 實際有計分的項目數 × 2。 */
  max: number;
  headline: string;
  dimensions: GradeDimension[];
  /** 成立（或大致成立）的特別格局名稱。 */
  special: string | null;
  caveats: string[];
}

const DIM_CAP = 2;

const SRC = {
  threeSteps: "吳俊民《命理新論》下冊 p.936–937（看富貴三步驟）",
  lesson1007: "許老師 2026-10-07 課（喜用的力量與命格高低）",
  lesson0729: "許老師 2026-07-29 課（中和不等於富貴）",
  lesson0722: "許老師 2026-07-22 課（中和：五行不過多也不過少）",
  lesson0819: "許老師 2026-08-19 課（中和也要寒暖適中）",
  lesson0902: "許老師 2026-09-02 課（同黨、異黨數法）",
  lesson1104: "許老師 2026-11-04 課（刑沖後只剩本氣取格、講義例三中和）",
  lesson1007mother: "許老師 2026-10-07 課（母慈滅子）",
  basicTable: "陸致極《基礎教程》表 7.2、7.3 與 p148–157（格局成敗）",
  basicGrade: "陸致極《基礎教程》p127–130（格局看單純與有力）",
  tenGods: "潘樂德《十神洩天機》",
  suSpecial: "蘇民峰《八字論命》第 14 章（特別格局）",
  suGrade: "蘇民峰《八字論命》第 12 章 p220–236（格局高低六項）",
  miaosuan: "林信銘《妙算人生八字》第六篇第五節 p244–246（用神四品質）",
  xinjue: "張惠能《八字心訣》第 92、99 訣（寒暖、順逆、神枯）",
  biasDry: "吳俊民《命理新論》下冊 p.938（五行偏枯、氣勢不能流通）",
  luck1007: "許老師 2026-10-07 課（原局喜用不足，要行運引出才能借運生發）",
} as const;

const clamp = (value: number) => Math.max(-DIM_CAP, Math.min(DIM_CAP, value));
const round = (value: number) => Math.round(value * 10) / 10;

function dimension(
  key: GradeDimension["key"],
  title: string,
  question: string,
  reasons: GradeReason[],
  skipped?: string,
): GradeDimension {
  const raw = round(reasons.reduce((sum, reason) => sum + reason.delta, 0));
  return { key, title, question, raw, score: skipped ? 0 : clamp(raw), reasons, skipped };
}

/** 從「……（p146、p196）」這類說明裡取最後一個括號頁碼。 */
function pageOf(detail: string): string {
  const matches = [...detail.matchAll(/（([^（）]*p\d[^（）]*)）/g)];
  return matches.length ? matches[matches.length - 1][1] : "";
}

function natalOf(result: BaziResponse): string[] {
  return [result.yearPillar, result.monthPillar, result.dayPillar, result.hourPillar].map(
    (pillar) => `${pillar?.stem ?? ""}${pillar?.branch ?? ""}`,
  );
}

const STRONG = ["強候選", "旺候選", "旺極候選"];
const WEAK = ["弱候選", "衰極候選"];

// ---------------------------------------------------------------------------
// 一、成格或破格

function formationReasons(natal: string[], classification: string, result: BaziResponse, special: PatternCandidate[]): GradeReason[] {
  const reasons: GradeReason[] = [];
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  const candidates = analysis?.pattern?.ordinaryCandidates ?? [];
  const knocked = candidates.filter((item) => item.selection === "knocked_candidate");
  const kept = candidates.filter((item) => item.selection !== "knocked_candidate");
  if (knocked.length) {
    reasons.push({
      text: `月令被刑沖，原本的${knocked.map((item) => item.pattern).join("、")}打掉了，只能看本氣取${kept.map((item) => item.pattern).join("、") || "格"}：格的根基受損。`,
      delta: -1,
      source: SRC.lesson1104,
    });
  }

  special.forEach((item) => {
    if (item.verdict === "formed") {
      reasons.push({
        text: `${item.name}的條件都符合：全局氣勢純粹，書以特別格局論，順勢行運反而好。`,
        delta: 2,
        source: `${SRC.suSpecial} ${item.page}`,
      });
    } else if (item.verdict === "partial") {
      reasons.push({
        text: `${item.name}大致符合但有瑕疵（${item.verdictLabel}）：書說假從、假化遇真運也能發，運過就退，比真的低一級。`,
        delta: 0.5,
        source: `${SRC.suSpecial} ${item.page}`,
      });
    }
  });

  const basic = basicStructure(natal, STRONG.includes(classification) ? "強" : WEAK.includes(classification) ? "弱" : undefined);
  basic.checks.forEach((check) => {
    if (check.tone === "note") {
      reasons.push({ text: `${check.label}：${check.detail}`, delta: 0, source: SRC.basicTable });
      return;
    }
    reasons.push({
      text: `${basic.pattern.name}${check.tone === "good" ? "成格" : "破格"}——${check.label}：${check.detail}`,
      delta: check.tone === "good" ? 1 : -1,
      source: SRC.basicTable,
    });
  });
  if (!basic.checks.length) {
    reasons.push({
      text: `${basic.pattern.name}：書上表 7.2、7.3 沒有列這個格的成敗條件，原局也沒有官殺混雜、重見，不加不減。`,
      delta: 0,
      source: SRC.basicTable,
    });
  }
  return reasons;
}

// ---------------------------------------------------------------------------
// 二、清或濁

function clarityReasons(natal: string[], result: BaziResponse): GradeReason[] {
  const reasons: GradeReason[] = [];
  const dayStem = natal[2].charAt(0);
  const shown = natal.filter((_, index) => index !== 2).map((pillar) => tenGod(dayStem, pillar.charAt(0)));
  // 基礎教程只在官、殺格裡查混雜；這裡不分格局，看天干是否正官、七殺同透（已在成格那一項扣過的不重複）。
  const basic = basicStructure(natal);
  const counted = basic.checks.some((check) => check.label === "官殺混雜");
  if (!counted && shown.includes("正官") && shown.includes("七殺")) {
    reasons.push({
      text: "正官、七殺同時透在天干（官殺混雜），主導勢力不單純，算「濁」；有合官或合殺可解。",
      delta: -1,
      source: SRC.basicGrade,
    });
  }

  const prefs = lessonPreferences(result.quantModel?.lessonContent?.threeStepAnalysis);
  const overview = tenGodOverview(natal, prefs);
  overview?.items.forEach((item) => {
    if (item.tone === "note") return;
    const page = pageOf(item.detail);
    // 書把喜忌當成已知；這裡用課堂喜忌打折：好組合裡要有喜用的十神才加分，壞組合傷到的若都是忌神就減半。
    const favorNote = item.favorNote?.replace(/^課堂喜忌：/, "") ?? "";
    const touchesUse = favorNote.includes("喜用");
    const delta = item.tone === "good" ? (touchesUse ? 0.5 : 0) : touchesUse ? -1 : -0.5;
    const why =
      item.tone === "good"
        ? touchesUse
          ? ""
          : "組合裡沒有課堂判為喜用的十神，書上的好處打折，不加分。"
        : touchesUse
          ? ""
          : "牽涉的十神都不是喜用，傷害較輕，減半。";
    reasons.push({
      text: `${item.label}：${item.detail.replace(/（[^（）]*p\d[^（）]*）。?$/, "。")}${favorNote ? `（課堂喜忌：${favorNote}）` : ""}${why}`,
      delta,
      source: `${SRC.tenGods}${page ? ` ${page}` : ""}`,
    });
  });
  if (!reasons.length) {
    reasons.push({ text: "原局沒有官殺混雜，也沒有書上列的好壞十神組合：不清不濁，不加不減。", delta: 0, source: `${SRC.basicGrade}；${SRC.tenGods}` });
  }
  return reasons;
}

// ---------------------------------------------------------------------------
// 三、中和或偏枯

function balanceReasons(natal: string[], result: BaziResponse, special: PatternCandidate[]): GradeReason[] {
  const reasons: GradeReason[] = [];
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  const strength = analysis?.strength;
  const formedSpecial = special.find((item) => item.verdict === "formed");
  if (strength) {
    const same = strength.samePartyCount;
    const opp = strength.opposingPartyCount;
    const gap = Math.abs(same - opp);
    const extreme = ["旺極候選", "衰極候選"].includes(strength.classification);
    const base = `同黨（印、比劫）${same} 個字：異黨（食傷、財、官殺）${opp} 個字，課堂判「${strength.classification}」。`;
    if (formedSpecial && (extreme || gap >= 4)) {
      reasons.push({
        text: `${base}一邊倒，但已符合${formedSpecial.name}，順著旺勢就好，不當偏枯扣分。`,
        delta: 0,
        source: `${SRC.lesson0902}；${SRC.suSpecial}`,
      });
    } else if (extreme || gap >= 6) {
      reasons.push({ text: `${base}一邊倒又沒有成特別格局：偏枯，起落大。`, delta: -2, source: `${SRC.lesson0902}；${SRC.biasDry}` });
    } else if (gap >= 4) {
      reasons.push({ text: `${base}兩邊差 ${gap} 個字，偏向一邊。`, delta: -1, source: `${SRC.lesson0902}；${SRC.lesson0722}` });
    } else if (gap === 0) {
      reasons.push({ text: `${base}兩黨一樣多，強弱接近中間（強弱本身不分好壞，平衡就好）。`, delta: 1, source: `${SRC.lesson0902}；${SRC.lesson0722}` });
    } else {
      reasons.push({ text: `${base}兩邊只差 ${gap} 個字，還算平均。`, delta: 0, source: SRC.lesson0902 });
    }
    if (strength.balanceHint) {
      reasons.push({ text: `中和提示：${strength.balanceHint}`, delta: 1, source: SRC.lesson1104 });
    }
    if (strength.motherOverload) {
      reasons.push({ text: `母慈滅子：${strength.motherOverload}`, delta: -1, source: SRC.lesson1007mother });
    }
  }

  const climate = climateReport(natal);
  if (climate.check.status === "fail") {
    reasons.push({ text: `寒暖：${climate.check.detail}`, delta: -1, source: `${SRC.lesson0819}；${SRC.xinjue}` });
  } else if (climate.check.status === "partial") {
    reasons.push({ text: `寒暖：${climate.check.detail}`, delta: 0, source: SRC.xinjue });
  } else if (climate.side !== "中和") {
    reasons.push({ text: `寒暖：${climate.check.detail}`, delta: 0, source: SRC.xinjue });
  }
  return reasons;
}

// ---------------------------------------------------------------------------
// 四、用神有沒有力

function useGodReasons(natal: string[], result: BaziResponse): { reasons: GradeReason[]; skipped?: string } {
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  const guide = yongShenGuide(analysis);
  if (!guide || !guide.useElements.length) {
    return { reasons: [], skipped: guide?.headline ? `用神未定（${guide.headline}），這一項不計分。` : "沒有課堂三步法的資料，這一項不計分。" };
  }
  const reasons: GradeReason[] = [];
  const verdict = guide.strengthVerdict;
  const use = guide.useElements.join("、");
  const where = guide.positions.map((item) => `${item.pillar.replace("柱", "")}${item.place === "天干" ? "干" : item.place === "地支本氣" ? "支" : item.place}${item.char}`).join("、");
  const delta = verdict.startsWith("透干又通根")
    ? 1
    : verdict.startsWith("透干但")
      ? 0
      : verdict.startsWith("地支本氣")
        ? 0
        : verdict.startsWith("只藏")
          ? -1
          : verdict.includes("不見") || verdict.includes("等大運")
            ? -2
            : 0;
  reasons.push({
    text: `用神${use}${where ? `（在${where}）` : ""}：${verdict}。`,
    delta,
    source: `${SRC.lesson1007}；${SRC.basicGrade}`,
  });

  if (guide.useElements.length === 1) {
    const quality = useQuality(natal, guide.useElements[0]);
    const passes = quality.checks.filter((check) => check.status === "pass").map((check) => check.label);
    const fails = quality.checks.filter((check) => check.status === "fail").map((check) => check.label);
    reasons.push({
      text: `${quality.summary}：${quality.checks.map((check) => `${check.label}${check.status === "pass" ? "✓" : check.status === "fail" ? "✗" : "△"}`).join(" ")}。${
        fails.length ? `${fails.join("、")}不足。` : passes.length ? `${passes.join("、")}都好。` : ""
      }`,
      delta: passes.length >= 3 ? 1 : fails.length >= 3 ? -1 : 0,
      source: `${SRC.miaosuan}；${SRC.suGrade}`,
    });
  }

  const hurts = guide.interactions.filter((item) => item.verdict === "bad");
  const helps = guide.interactions.filter((item) => item.verdict === "good");
  hurts.slice(0, 2).forEach((item) =>
    reasons.push({ text: `原局作用傷到喜用：${item.label}（${item.reason}）`, delta: -1, source: "許老師 2026-10-14、10-21 課（天干、地支卡）" }),
  );
  helps.slice(0, 2).forEach((item) =>
    reasons.push({ text: `原局作用幫到喜用：${item.label}（${item.reason}）`, delta: 0.5, source: "許老師 2026-10-14、10-21 課（天干、地支卡）" }),
  );
  if (guide.status === "conflict") {
    reasons.push({ text: `用神候選${use}並列，方案還要比較，用神不夠專一。`, delta: 0, source: "課堂三步法（用神喜忌卡）" });
  }
  return { reasons };
}

// ---------------------------------------------------------------------------
// 五、五行流通

function flowReasons(natal: string[], result: BaziResponse): GradeReason[] {
  const reasons: GradeReason[] = [];
  const flow = elementFlow(result);
  const spread = flow?.points.find((point) => point.key === "spread");
  if (spread) {
    reasons.push({ text: `${spread.title}：${spread.text}`, delta: spread.tone === "good" ? 1 : -1, source: `${SRC.biasDry}；${SRC.lesson0722}` });
  }
  const clash = flow?.points.find((point) => point.key === "clash");
  if (clash && clash.tone === "bad") {
    reasons.push({ text: `${clash.title}：兩神對立又沒有通關。`, delta: spread?.tone === "bad" ? -0.5 : -1, source: SRC.biasDry });
  }

  // 順逆：身強要流出去（洩秀），身弱要流進來（生身）。強弱用課堂判定，不用心訣自己的三取二。
  const classification = result.quantModel?.lessonContent?.threeStepAnalysis?.strength?.classification ?? "";
  const report = flowReport(natal);
  if (STRONG.includes(classification)) {
    reasons.push(
      report.outflow
        ? { text: `身強，氣從日主順流出去：${report.outflow}。`, delta: 1, source: SRC.xinjue }
        : { text: `身強，日主的氣沒有出口（${report.inflow ? `只有流進來的 ${report.inflow}` : "和鄰字都不相生"}）。`, delta: -1, source: SRC.xinjue },
    );
  } else if (WEAK.includes(classification)) {
    reasons.push(
      report.inflow
        ? { text: `身弱，有相生鏈流進日主：${report.inflow}。`, delta: 1, source: SRC.xinjue }
        : { text: `身弱，沒有印比一路生進日主（${report.outflow ? `只有洩出去的 ${report.outflow}` : "和鄰字都不相生"}）。`, delta: -1, source: SRC.xinjue },
    );
  }
  return reasons;
}

// ---------------------------------------------------------------------------
// 六、大運順不順

const favorValue = (favor: Favor) => (favor === "good" ? 1 : favor === "bad" ? -1 : 0);

function luckReasons(result: BaziResponse): { reasons: GradeReason[]; skipped?: string; rows: string[] } {
  const prefs = lessonPreferences(result.quantModel?.lessonContent?.threeStepAnalysis);
  if (!prefs) return { reasons: [], skipped: "課堂三步法沒有喜忌方案，大運無法判順逆，這一項不計分。", rows: [] };
  const known = Object.values(prefs.elements).filter((item) => item.favor === "good" || item.favor === "bad").length;
  if (!known) return { reasons: [], skipped: "各方案喜忌互相衝突，大運無法判順逆，這一項不計分。", rows: [] };
  // 看前七步（約到七十歲）；大運以地支為重：干 1 份、支 2 份。
  const steps = (result.daYun ?? []).filter((row: DaYun) => row.ganZhi && row.ganZhi.length === 2).slice(0, 7);
  if (!steps.length) return { reasons: [], skipped: "沒有大運資料，這一項不計分。", rows: [] };
  const favorOf = (char: string) => {
    const element = STEM_WUXING[char] ?? BRANCH_WUXING[char];
    return element ? prefs.elements[element].favor : "neutral";
  };
  const values = steps.map((row) => {
    const stem = favorValue(favorOf(row.ganZhi.charAt(0)));
    const branch = favorValue(favorOf(row.ganZhi.charAt(1)));
    return (stem + branch * 2) / 3;
  });
  const mark = (value: number) => (value > 0.3 ? "▲" : value < -0.3 ? "▼" : "－");
  const rows = steps.map((row, index) => {
    const age = row.startAge != null ? `${row.startAge}歲` : row.startYear ? `${row.startYear}` : `第${index + 1}步`;
    return `${row.ganZhi}${mark(values[index])}（${age}）`;
  });
  const average = values.reduce((sum, value) => sum + value, 0) / values.length;
  // 青壯年（第二到第五步，約二十到六十歲）另外看一次。
  const prime = values.slice(1, 5);
  const primeAverage = prime.length ? prime.reduce((sum, value) => sum + value, 0) / prime.length : average;
  const good = values.filter((value) => value > 0.3).length;
  const bad = values.filter((value) => value < -0.3).length;
  const delta = average >= 0.35 ? 2 : average >= 0.1 ? 1 : average <= -0.35 ? -2 : average <= -0.1 ? -1 : 0;
  const reasons: GradeReason[] = [
    {
      text: `前 ${steps.length} 步大運：${good} 步偏喜用、${bad} 步偏忌（地支為重）。${rows.join("、")}。`,
      delta,
      source: `${SRC.threeSteps}；${SRC.luck1007}`,
    },
  ];
  const primeGap = primeAverage - average;
  if (Math.abs(primeGap) >= 0.3 && prime.length >= 3) {
    reasons.push({
      text: primeGap > 0 ? "青壯年（約第二到第五步）走得比整體好，最能用力的時候有運。" : "青壯年（約第二到第五步）走得比整體差，最能用力的時候運不配合。",
      delta: primeGap > 0 ? 0.5 : -0.5,
      source: SRC.threeSteps,
    });
  }
  const mixed = Object.values(prefs.elements).filter((item) => item.favor === "mixed").map((item) => item.name);
  if (mixed.length) {
    reasons.push({ text: `${mixed.join("、")}依方案而定，走到這些五行的大運不計順逆。`, delta: 0, source: "課堂三步法（喜忌合併規則）" });
  }
  return { reasons, rows };
}

// ---------------------------------------------------------------------------

function levelOf(total: number, max: number): GradeLevel {
  // 以實際計分項數的滿分換算比例：六項都計分時，≥5 上、≥2 中上、≥-1 中、≥-4 中下。
  const ratio = max ? total / max : 0;
  if (ratio >= 5 / 12) return "上";
  if (ratio >= 2 / 12) return "中上";
  if (ratio > -2 / 12) return "中";
  if (ratio > -5 / 12) return "中下";
  return "下";
}

const LEVEL_TEXT: Record<GradeLevel, string> = {
  上: "上等：各項加分明顯多於扣分",
  中上: "中上：加分多於扣分",
  中: "中等：有好有壞，看大運引出哪一邊",
  中下: "中下：扣分多於加分，要靠行運補救",
  下: "下等：扣分明顯多於加分",
};

export function patternGrade(result: BaziResponse): PatternGrade | null {
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  const natal = natalOf(result);
  if (!analysis || natal.some((pillar) => pillar.length !== 2)) return null;
  const classification = analysis.strength?.classification ?? "";
  const report = specialPatternReport(natal, { female: result.gender === "FEMALE" });
  const special = report
    ? [...report.follow, report.prosperous, report.transform, ...report.marks.filter((item) => item.key === "two-element")].filter(
        (item): item is PatternCandidate => !!item && item.verdict !== "none",
      )
    : [];
  const formed = special.find((item) => item.verdict === "formed") ?? null;

  const use = useGodReasons(natal, result);
  const luck = luckReasons(result);
  const specialSkip = formed ? `已成${formed.name}，喜忌要改看順勢；這一項照普通格局的喜忌算會不準，只列出來不計分。` : undefined;

  const dimensions: GradeDimension[] = [
    dimension("formation", "成格或破格", "月令取的格有沒有成？有沒有被破？", formationReasons(natal, classification, result, special)),
    dimension("clarity", "清或濁", "主導的十神單不單純？十神之間是互相成全還是互相破壞？", clarityReasons(natal, result)),
    dimension("balance", "中和或偏枯", "兩黨、寒暖平不平均？（強弱本身不分好壞）", balanceReasons(natal, result, special)),
    dimension("useGod", "用神有沒有力", "用神透不透干、有沒有根、有沒有被沖合剋傷？", use.reasons, specialSkip ?? use.skipped),
    dimension("flow", "五行流通", "五行接不接得起來？氣有沒有往日主需要的方向流？", flowReasons(natal, result)),
    dimension("luck", "大運配合", "一生大運多走喜用還是多走忌神？", luck.reasons, specialSkip ?? luck.skipped),
  ];

  const scored = dimensions.filter((item) => !item.skipped);
  const total = round(scored.reduce((sum, item) => sum + item.score, 0));
  const max = scored.length * DIM_CAP;
  const level = levelOf(total, max);
  const best = [...scored].filter((item) => item.score > 0).sort((a, b) => b.score - a.score).slice(0, 2);
  const worst = [...scored].filter((item) => item.score < 0).sort((a, b) => a.score - b.score).slice(0, 2);
  const headline = [
    best.length ? `長處在${best.map((item) => item.title).join("、")}` : "",
    worst.length ? `短處在${worst.map((item) => item.title).join("、")}` : "",
  ]
    .filter(Boolean)
    .join("；");

  const caveats = [
    "這是把各卡片的結果換成加減分的提示，不是命定的吉凶。老師說：中和只是好命，好命不等於富，富也不等於貴（2026-07-29）。",
    "命局定底子，大運定時機：好命走壞運也會起伏，普通命走好運也能有一段好成績（《命理新論》下冊 p.903）。",
  ];
  if (formed) caveats.push(`已成${formed.name}：用神、大運兩項照普通格局（扶抑）的喜忌算不準，所以不計分；請看「特別格局」卡的喜忌。`);
  else if (special.length) caveats.push(`${special.map((item) => item.name).join("、")}只是大致符合，仍照普通格局評；若老師判為特別格局，用神與大運兩項要重看。`);
  if (analysis.pattern?.specialReview?.required) caveats.push(`課堂特殊格審查：${analysis.pattern.specialReview.reasons.join("；")}`);

  return {
    level,
    levelText: LEVEL_TEXT[level],
    total,
    max,
    headline,
    dimensions,
    special: formed?.name ?? null,
    caveats,
  };
}

