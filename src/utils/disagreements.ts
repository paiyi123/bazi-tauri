import type { BaziResponse } from "../types/bazi";
import { conclusionOf } from "./conclusion";

// 這張盤碰到哪些「各書說法不同」的節點。內容對應《八字諸書融會貫通指南》附錄的 20 項分歧表，
// 只挑程式能偵測到的情況；建議做法照指南（以課堂為準）。

export type Disagreement = {
  no: number;
  title: string;
  why: string;
  classroom: string;
  others: string;
  advice: string;
  askTeacher: boolean;
};

const side = (value?: string | null): "強" | "弱" | null => {
  if (!value) return null;
  if (value.includes("強") || value.includes("旺")) return "強";
  if (value.includes("弱") || value.includes("衰")) return "弱";
  return null;
};

const WINTER = "亥子丑";
const SUMMER = "巳午未";

export function disagreementsOf(result: BaziResponse): Disagreement[] {
  const quant = result.quantModel;
  const analysis = quant?.lessonContent?.threeStepAnalysis;
  if (!analysis) return [];
  const list: Disagreement[] = [];
  const lesson = side(analysis.strength.classification);
  const quantSide = side(quant?.strengthLabel);
  const lu = quant?.luStrength;
  const luSide = side(lu?.verdict);
  const rootless = lu ? lu.roots.length === 0 : false;

  if (lesson === "強" && luSide === "弱" && rootless) {
    list.push({
      no: 2,
      title: "印算不算根",
      why: "日主在地支沒有根，但印多，所以課堂判身強，陸致極判身弱。",
      classroom: "印算同黨，數人頭判強弱。",
      others: "陸致極：印不能代替根，無根必弱。",
      advice: "無根時不要輕易判強；用過去大運反驗。",
      askTeacher: false,
    });
  } else if (lesson && ((quantSide && quantSide !== lesson) || (luSide && luSide !== lesson))) {
    list.push({
      no: 1,
      title: "得勢怎麼數",
      why: `課堂判身${lesson}，${quantSide && quantSide !== lesson ? `量化五行力判身${quantSide}` : `陸致極判身${luSide}`}。`,
      classroom: "八個字分同黨、異黨數人頭，平手看月令。",
      others: "量化五行力算分數；陸致極分九級並看根。",
      advice: "以課堂為主，其他當第二意見；分數接近 0 的盤最要小心。",
      askTeacher: false,
    });
  }

  const month = result.monthPillar?.branch ?? "";
  const season = WINTER.includes(month) ? "冬" : SUMMER.includes(month) ? "夏" : "";
  if (season) {
    const want = season === "冬" ? "火" : "水";
    const data = conclusionOf(result);
    const uses = data?.useElements.map((item) => item.name) ?? [];
    if (uses.length && !uses.includes(want)) {
      list.push({
        no: 4,
        title: "調候和扶抑誰優先",
        why: `${season}天出生（月令${month}），調候要${want}，但課堂用神是${uses.join("、")}。`,
        classroom: "扶抑優先，調候只在不衝突時配合。",
        others: "陸致極、妙算、蘇民峰：調候優先。",
        advice: `扶抑定方向；${want}當配合，並留意寒暖對健康的影響。`,
        askTeacher: false,
      });
    }
  }

  const stemCombos = analysis.stemInteractions?.combinations ?? [];
  if (stemCombos.length) {
    const pending = stemCombos.some((combo) => combo.needsConfirmation);
    list.push({
      no: 6,
      title: "天干合了會不會化",
      why: `原局有${stemCombos.map((combo) => `${combo.stems}合`).join("、")}${pending ? "，其中有需要再確認的" : ""}。`,
      classroom: "分五種情形（合而不合、合而不化、合來合去、爭合妒合、合而能化）；年月合化要時支見辰。",
      others: "妙算：日干合而不化；八字心訣：要五個條件。",
      advice: "用課堂標準，沒把握就當合而不化。",
      askTeacher: false,
    });
  }

  if ((analysis.branchInteractions?.combinations ?? []).length) {
    list.push({
      no: 7,
      title: "地支六合會不會化",
      why: "原局有地支六合。",
      classroom: "地支有合無化，但合住的字作用減弱。",
      others: "妙算：透干才化；陸致極：合去根；量化：用係數打折。",
      advice: "不化，但要算「合去根」對強弱的影響。",
      askTeacher: false,
    });
  }

  if (analysis.interactionStrength?.changed) {
    list.push({
      no: 9,
      title: "合沖刑會不會改變強弱",
      why: "干支作用變化後，同黨、異黨人數改變，強弱可能翻轉。",
      classroom: "69 年次女命：卯未半合解未戌刑，判弱。",
      others: "量化、陸致極與三步法初判多半判強。",
      advice: "兩種結果都記下，用大運反驗；值得在課堂上問老師。",
      askTeacher: true,
    });
  }

  const special = quant?.specialPattern?.primaryPattern ?? "";
  if (special.includes("從") || analysis.pattern?.specialReview?.required) {
    list.push({
      no: 12,
      title: "真從、假從",
      why: special ? `可能是${special}。` : "課堂三步法提示要檢查特別格局。",
      classroom: "07-01 說從格不分真假；中級二課本說有一點根就不從，兩份課堂資料說法不同。",
      others: "蘇民峰：條件從嚴。",
      advice: "用嚴標準，再用大運驗證；值得在課堂上問老師。",
      askTeacher: true,
    });
  }

  return list;
}
