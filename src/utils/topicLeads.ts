import type { BaziResponse } from "../types/bazi";
import { BRANCH_WUXING, STEM_WUXING, type WuXing } from "../constants/wuxing";
import { ELEMENT_NAMES, lessonPreferences, type Favor } from "./lessonPreference";
import { conclusionOf } from "./conclusion";
import { pillarSummary } from "./pillarSummary";
import { patternGrade } from "./patternGrade";

// 分析頁每個主題最上面的「一句白話重點」。主答案依課堂三步法的喜忌；
// 六親、健康、財富用各書共同的簡單原則（見《八字諸書融會貫通指南》第六～八章），不新增推論規則。

export type LeadTone = "good" | "bad" | "mixed" | "info";
export type LeadLine = { text: string; tone: LeadTone };
export type TopicKey = "strength" | "pattern" | "interaction" | "family" | "health" | "career";

const CONTROLS: Record<WuXing, WuXing> = { wood: "earth", earth: "water", water: "fire", fire: "metal", metal: "wood" };
const CONTROLLED_BY = Object.fromEntries(Object.entries(CONTROLS).map(([a, b]) => [b, a])) as Record<WuXing, WuXing>;
const ORGANS: Record<WuXing, string> = {
  wood: "肝、膽、筋骨",
  fire: "心、小腸、血液循環",
  earth: "脾、胃、消化",
  metal: "肺、大腸、呼吸道",
  water: "腎、膀胱、泌尿",
};
const FAVOR_WORD: Record<Favor, string> = { good: "喜", bad: "忌", mixed: "依方案而定", neutral: "未列喜忌" };

const name = (element: WuXing) => ELEMENT_NAMES[element];
const toneOf = (favor: Favor): LeadTone => (favor === "good" ? "good" : favor === "bad" ? "bad" : "mixed");

function favorReader(result: BaziResponse) {
  const prefs = lessonPreferences(result.quantModel?.lessonContent?.threeStepAnalysis);
  return prefs ? (element: WuXing): Favor => prefs.elements[element].favor : null;
}

function strengthLeads(result: BaziResponse): LeadLine[] {
  const data = conclusionOf(result);
  if (!data) return [{ text: "這張命盤沒有課堂三步分析，無法判斷強弱與喜忌。", tone: "info" }];
  const list = (items: { name: string }[]) => items.map((item) => item.name).join("、") || "—";
  return [
    {
      text: `${data.strength.label}（${data.strength.checks}；${data.strength.party}）。用神 ${list(data.useElements)}；喜 ${list(data.favorable)}；忌 ${list(data.unfavorable)}。`,
      tone: "info",
    },
    {
      text:
        data.agreement.status === "split"
          ? `${data.agreement.text}。各書在強弱接近中間時最容易判不一樣，下方「各書細節」有比較。`
          : data.agreement.text,
      tone: data.agreement.status === "split" ? "mixed" : "info",
    },
  ];
}

function patternLeads(result: BaziResponse): LeadLine[] {
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  const special = result.quantModel?.specialPattern?.primaryPattern;
  const lines: LeadLine[] = [];
  const grade = patternGrade(result);
  if (grade) {
    lines.push({
      text: `格局高低總評：${grade.level}（合計 ${grade.total > 0 ? "+" : ""}${grade.total}，滿分 ±${grade.max}）${grade.headline ? `。${grade.headline}` : ""}。只是提示，每一條加減分的理由與出處見下方「格局高低總評」卡。`,
      tone: grade.level === "上" || grade.level === "中上" ? "good" : grade.level === "中" ? "mixed" : "bad",
    });
  }
  if (analysis?.pattern?.ordinaryResult) {
    lines.push({ text: `課堂取格：${analysis.pattern.ordinaryResult}（看月令藏干透出哪一個十神）。格局是這張盤的主題，用神才是答案。`, tone: "info" });
  }
  lines.push(
    special
      ? { text: `可能是特別格局：${special}。特別格局要非常嚴格才成立，請看下方蘇民峰的判斷過程。`, tone: "mixed" }
      : { text: "不是特別格局，照一般格局用扶抑取用神。", tone: "info" },
  );
  return lines;
}

function interactionLeads(result: BaziResponse): LeadLine[] {
  const summary = pillarSummary(result.quantModel?.lessonContent?.threeStepAnalysis);
  if (!summary) return [];
  const lines = [...summary.stemLines, ...summary.branchLines].filter((line) => !line.muted);
  if (!lines.length) return [{ text: "原局天干沒有五合，地支沒有沖、合、會、刑、害，結構單純。", tone: "info" }];
  const good = lines.filter((line) => line.verdict === "good").length;
  const bad = lines.filter((line) => line.verdict === "bad").length;
  return [
    {
      text: `原局有 ${lines.length} 組作用：${lines.map((line) => `${line.label}（${line.where}）`).join("、")}。`,
      tone: "info",
    },
    {
      text: `依喜忌看：▲有利 ${good} 組、▼不利 ${bad} 組。合沖本身沒有好壞，要看它動到喜神還是忌神。`,
      tone: good > bad ? "good" : bad > good ? "bad" : "mixed",
    },
  ];
}

function familyLeads(result: BaziResponse): LeadLine[] {
  const favorOf = favorReader(result);
  const day = STEM_WUXING[result.dayPillar?.stem ?? ""];
  const palace = BRANCH_WUXING[result.dayPillar?.branch ?? ""];
  if (!favorOf || !day || !palace) return [];
  const lines: LeadLine[] = [];
  const palaceFavor = favorOf(palace);
  if (result.gender) {
    const star = result.gender === "MALE" ? CONTROLS[day] : CONTROLLED_BY[day];
    const starName = result.gender === "MALE" ? "妻星（財）" : "夫星（官殺）";
    const starFavor = favorOf(star);
    lines.push({
      text: `${starName}是${name(star)}，為${FAVOR_WORD[starFavor]}；配偶宮（日支${result.dayPillar.branch}，${name(palace)}）為${FAVOR_WORD[palaceFavor]}。`,
      tone: starFavor === palaceFavor ? toneOf(starFavor) : "mixed",
    });
    const both = starFavor === "good" && palaceFavor === "good";
    const neither = starFavor === "bad" && palaceFavor === "bad";
    lines.push({
      text: both
        ? "星與宮都是喜，配偶與婚姻關係的助力較多。"
        : neither
          ? "星與宮都是忌，感情要多經營，婚期也較常受大運流年影響。"
          : "星看人、宮看關係，兩者一好一壞時，看大運流年哪一邊被引動。",
      tone: both ? "good" : neither ? "bad" : "mixed",
    });
  } else {
    lines.push({ text: `配偶宮（日支${result.dayPillar.branch}，${name(palace)}）為${FAVOR_WORD[palaceFavor]}。填寫性別後可以看配偶星。`, tone: toneOf(palaceFavor) });
  }
  return lines;
}

function healthLeads(result: BaziResponse): LeadLine[] {
  const pillars = [result.yearPillar, result.monthPillar, result.dayPillar, result.hourPillar];
  const counts = Object.fromEntries((Object.keys(ELEMENT_NAMES) as WuXing[]).map((key) => [key, 0])) as Record<WuXing, number>;
  pillars.forEach((pillar) => {
    const stem = STEM_WUXING[pillar?.stem ?? ""];
    const branch = BRANCH_WUXING[pillar?.branch ?? ""];
    if (stem) counts[stem] += 1;
    if (branch) counts[branch] += 1;
  });
  const keys = Object.keys(counts) as WuXing[];
  const max = Math.max(...keys.map((key) => counts[key]));
  const min = Math.min(...keys.map((key) => counts[key]));
  const most = keys.filter((key) => counts[key] === max);
  const least = keys.filter((key) => counts[key] === min);
  return [
    {
      text: `八個字裡最多的是${most.map(name).join("、")}（${max} 個，${most.map((key) => ORGANS[key]).join("；")}），最少的是${least.map(name).join("、")}（${min} 個，${least.map((key) => ORGANS[key]).join("；")}）。`,
      tone: "info",
    },
    { text: "各書共識：太過、不及、互相交戰的五行，對應的身體部位要多留意。這只是保養方向，不是診斷。", tone: "info" },
  ];
}

function careerLeads(result: BaziResponse): LeadLine[] {
  const favorOf = favorReader(result);
  const day = STEM_WUXING[result.dayPillar?.stem ?? ""];
  const data = conclusionOf(result);
  if (!favorOf || !day || !data) return [];
  const wealth = CONTROLS[day];
  const favor = favorOf(wealth);
  const strong = data.strength.label.includes("強") || data.strength.label.includes("旺");
  return [
    {
      text: `財星是${name(wealth)}，為${FAVOR_WORD[favor]}。${
        strong
          ? "日主偏強，「身能任財」，有能力承擔財務與事業壓力。"
          : "日主偏弱，財多反而負擔重，宜先有印、比劫幫身（學歷、證照、團隊）再求財。"
      }`,
      tone: favor === "good" ? "good" : favor === "bad" ? "bad" : "mixed",
    },
    {
      text: `選方向先看用神（${data.useElements.map((item) => item.name).join("、") || "未定"}）：和用神五行相關的行業、或用神那個十神的做事方式，通常比較順手。細節見下方各書。`,
      tone: "info",
    },
  ];
}

export function topicLeads(result: BaziResponse, topic: TopicKey): LeadLine[] {
  switch (topic) {
    case "strength":
      return strengthLeads(result);
    case "pattern":
      return patternLeads(result);
    case "interaction":
      return interactionLeads(result);
    case "family":
      return familyLeads(result);
    case "health":
      return healthLeads(result);
    case "career":
      return careerLeads(result);
  }
}
