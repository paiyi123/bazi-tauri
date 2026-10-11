/**
 * 婚姻、財運、健康：依陸致極《八字命理學進階教程》第六、七、十一、十三章整理。
 *
 * 原局部分列出書中的判讀要點（配偶星與配偶宮、財氣與財庫、弱臟強臟與寒暖燥濕）；
 * 運歲部分只標出書中說容易應事的時機，不預言具體事件。
 */

import type { LuckEvent } from "./luckEvents";

export type Gender = "MALE" | "FEMALE";
export type Element = "木" | "火" | "土" | "金" | "水";

export interface TopicItem {
  label: string;
  detail: string;
  tone: "good" | "bad" | "note";
}

export interface TopicSection {
  key: "marriage" | "wealth" | "health";
  title: string;
  summary: string;
  items: TopicItem[];
}

export interface TopicContext {
  natal: string[];
  gender?: Gender;
  /** 陸致極強弱檢核的結論：強／弱。 */
  strength?: string;
}

const PILLARS = ["年", "月", "日", "時"];
const STEMS = "甲乙丙丁戊己庚辛壬癸";
const STEM_ELEMENT: Record<string, Element> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
const HIDDEN: Record<string, string[]> = {
  子: ["癸"], 丑: ["己", "癸", "辛"], 寅: ["甲", "丙", "戊"], 卯: ["乙"], 辰: ["戊", "乙", "癸"], 巳: ["丙", "戊", "庚"],
  午: ["丁", "己"], 未: ["己", "丁", "乙"], 申: ["庚", "壬", "戊"], 酉: ["辛"], 戌: ["戊", "辛", "丁"], 亥: ["壬", "甲"],
};
const GENERATES: Record<Element, Element> = { 木: "火", 火: "土", 土: "金", 金: "水", 水: "木" };
const CONTROLS: Record<Element, Element> = { 木: "土", 土: "水", 水: "火", 火: "金", 金: "木" };
const CONTROLLED_BY: Record<Element, Element> = { 土: "木", 水: "土", 火: "水", 金: "火", 木: "金" };
const CLASH_PAIRS = ["子午", "丑未", "寅申", "卯酉", "辰戌", "巳亥"];
const SIX_COMBINE = ["子丑", "寅亥", "卯戌", "辰酉", "巳申", "午未"];
const TRIADS = ["申子辰", "亥卯未", "寅午戌", "巳酉丑"];
const HARMS = ["子未", "丑午", "寅巳", "卯辰", "申亥", "酉戌"];
const STORAGE = "辰戌丑未";
/** 五行之庫（財庫用）：木庫未、火庫戌、金庫丑、水庫辰；土隨水庫在辰，也有說在戌。 */
const STORE_OF: Record<Element, string[]> = { 木: ["未"], 火: ["戌"], 金: ["丑"], 水: ["辰"], 土: ["辰", "戌"] };
const ORGAN: Record<Element, string> = { 木: "肝膽", 火: "心、小腸", 土: "脾胃", 金: "肺、大腸", 水: "腎、膀胱" };
const STEM_ORGAN: Record<string, string> = {
  甲: "膽", 乙: "肝", 丙: "小腸", 丁: "心", 戊: "胃", 己: "脾", 庚: "大腸", 辛: "肺", 壬: "膀胱、三焦", 癸: "腎",
};
/** 表 13.3：干支在年、月、日、時柱對應的部位（節錄，空白者不列）。 */
const BODY_PART: Record<string, [string, string, string, string]> = {
  甲: ["頭、頭髮", "上肢、膽", "", "下肢、腿腳筋骨"],
  乙: ["頭髮、頸部神經", "上肢、肝", "", "腿腳筋骨"],
  丙: ["腦神經、眼目", "肩胸", "小腸、闌尾", "下體"],
  丁: ["腦神經、眼目", "心臟、血液系統", "", "下肢、腿足"],
  戊: ["鼻、面", "兩肋、胸肌", "胃部、消化系統", "腿部肌肉"],
  己: ["頭部肌肉、脾", "胸肌、臂肌", "腹、腹肌", "腿肌"],
  庚: ["頭骨、牙齒", "胸骨、上臂骨骼", "臍輪、大腸", "尾骨、下肢骨骼"],
  辛: ["牙齒、喉部、呼吸道", "肺、胸腔、淋巴", "股部", "足、趾"],
  壬: ["口、舌、耳道", "脊骨", "三焦、膀胱、泌尿系統", "脛部"],
  癸: ["腦供血系統、耳", "脊髓", "腎", "足"],
  子: ["耳道", "", "膀胱、尿道、生殖系統", "足"],
  丑: ["面、頭頸表皮", "脾臟", "胞肚", "足底、左腿"],
  寅: ["頭、頭髮、腦神經", "膽、手臂", "大腿", "左腿"],
  卯: ["頭髮、血脈", "肝、手臂和十指", "生殖器官", "腿腳"],
  辰: ["頸、面部皮膚", "肩、胸、肋", "胃、消化系統", "下肢皮膚"],
  巳: ["面門、眼目、齒、咽喉", "心臟、血液循環", "肛門", "足"],
  午: ["頭、眼目、精神系統", "心臟、血液循環", "血液循環", "下肢"],
  未: ["面部肌膚", "脊梁、胸肌", "胃脘、腹肌", "下肢肌膚"],
  申: ["頭骨、呼吸道", "肺、胸腔、經絡", "大腸", "下肢骨骼"],
  酉: ["頭骨、呼吸道", "肺", "小腸、精血", "下肢踝骨"],
  戌: ["面部肌膚", "胸腔", "命門、胃、腹肌", "右腿、踝足"],
  亥: ["頭、血液", "血液", "小腹、腎囊", "泌尿、生殖、右腳"],
};
/** 表 1.1–1.3：寒暖燥濕計分（《基礎教程》，進階 p19–20）。 */
const STEM_CLIMATE: Record<string, number> = { 甲: 3, 乙: 1, 丙: 6, 丁: 4, 戊: 5, 己: -4, 庚: -1, 辛: -3, 壬: -5, 癸: -6 };
const BRANCH_CLIMATE: Record<string, number> = {
  寅: 3, 卯: 1, 辰: -4, 巳: 5, 午: 6, 未: 3, 申: -2, 酉: -3, 戌: 4, 亥: -5, 子: -6, 丑: -4,
};
const MONTH_CLIMATE: Record<string, number> = {
  寅: 0, 卯: 1, 辰: 2, 巳: 3, 午: 4, 未: 3, 申: 0, 酉: -1, 戌: -2, 亥: -3, 子: -4, 丑: -4,
};
/** 表 1.4：《窮通寶鑑》十干調候用神（梁湘潤整理）。月序寅、卯……丑。 */
const TIAO_HOU: Record<string, string[]> = {
  甲: ["丙癸", "庚丙戊丁己", "庚丁壬", "癸丁庚", "癸丁庚", "癸丁庚", "庚丁壬", "庚丁丙", "庚甲壬丁癸", "庚丁戊丙", "丁庚丙", "丁庚丙"],
  乙: ["丙癸", "丙癸", "癸丙戊", "癸", "癸丙", "癸丙", "丙癸己", "癸丙丁", "癸辛", "丙戊", "丙", "丙"],
  丙: ["壬庚", "壬己", "壬甲", "壬庚癸", "壬庚", "壬庚", "壬戊", "壬癸", "甲壬", "甲戊庚壬", "壬戊己", "壬甲"],
  丁: ["甲庚", "庚甲", "甲庚", "甲庚", "壬庚癸", "甲壬庚", "甲庚丙戊", "甲庚丙戊", "甲庚戊", "甲庚", "甲庚", "甲庚"],
  戊: ["丙甲癸", "丙甲癸", "甲丙癸", "甲丙癸", "壬甲丙", "癸丙甲", "丙癸甲", "丙癸", "甲丙癸", "甲丙", "丙甲", "丙甲"],
  己: ["丙庚甲", "甲癸丙", "丙癸甲", "癸丙", "癸丙", "癸丙", "丙癸", "丙癸", "甲丙癸", "丙甲戊", "丙甲戊", "丙甲戊"],
  庚: ["戊甲丙壬丁", "丁甲丙庚", "甲丁壬癸", "壬戊丙丁", "壬癸", "丁甲", "丁甲", "丁甲丙", "甲壬", "丁丙", "丁甲丙", "丙丁甲"],
  辛: ["己壬庚", "壬甲", "壬甲", "壬甲癸", "壬己癸", "壬庚甲", "壬甲戊", "壬甲", "壬甲", "壬丙", "丙戊壬甲", "丙壬戊己"],
  壬: ["庚丙戊", "戊辛庚", "甲庚", "壬辛庚癸", "癸庚辛", "辛甲", "戊丁", "甲庚", "甲丙", "戊丙庚", "戊丙", "丙丁甲"],
  癸: ["辛丙", "庚辛", "丙辛甲", "辛", "庚壬癸", "庚辛壬癸", "丁", "辛丙", "辛甲壬癸", "庚辛戊丁", "丙辛", "丙丁"],
};
const MONTH_ORDER = "寅卯辰巳午未申酉戌亥子丑";

const stemOf = (ganZhi: string) => ganZhi.charAt(0);
const branchOf = (ganZhi: string) => ganZhi.charAt(1);
const isYang = (stem: string) => STEMS.indexOf(stem) % 2 === 0;
const pairIn = (list: string[], a: string, b: string) => list.some((pair) => pair === a + b || pair === b + a);
export const isClash = (a: string, b: string) => pairIn(CLASH_PAIRS, a, b);
const isSixCombine = (a: string, b: string) => pairIn(SIX_COMBINE, a, b);
const isHarm = (a: string, b: string) => pairIn(HARMS, a, b);
/** 半合：三合中的中神加另一字。 */
function isHalfCombine(a: string, b: string): boolean {
  return TRIADS.some((triad) => a !== b && triad.includes(a) && triad.includes(b) && (a === triad[1] || b === triad[1]));
}
const mainElement = (branch: string): Element | undefined => {
  const stem = HIDDEN[branch]?.[0];
  return stem ? STEM_ELEMENT[stem] : undefined;
};

/** 十神名稱。 */
export function tenGod(dayStem: string, stem: string): string {
  const day = STEM_ELEMENT[dayStem];
  const other = STEM_ELEMENT[stem];
  const same = isYang(dayStem) === isYang(stem);
  if (day === other) return same ? "比肩" : "劫財";
  if (GENERATES[other] === day) return same ? "偏印" : "正印";
  if (GENERATES[day] === other) return same ? "食神" : "傷官";
  if (CONTROLS[day] === other) return same ? "偏財" : "正財";
  return same ? "七殺" : "正官";
}

function family(dayStem: string, stem: string): string {
  const name = tenGod(dayStem, stem);
  if (name === "比肩" || name === "劫財") return "比劫";
  if (name.endsWith("印")) return "印";
  if (name === "食神" || name === "傷官") return "食傷";
  if (name.endsWith("財")) return "財";
  return "官殺";
}

interface Placement {
  pillar: number;
  stem: string;
  visible: boolean;
  branch?: string;
}

/** 所有天干（不含日干）與地支藏干的位置。 */
function placements(natal: string[]): Placement[] {
  const result: Placement[] = [];
  natal.forEach((pillar, index) => {
    if (index !== 2) result.push({ pillar: index, stem: stemOf(pillar), visible: true });
    (HIDDEN[branchOf(pillar)] ?? []).forEach((stem) =>
      result.push({ pillar: index, stem, visible: false, branch: branchOf(pillar) }),
    );
  });
  return result;
}

function spouseStars(ctx: TopicContext): string[] {
  return ctx.gender === "FEMALE" ? ["正官", "七殺"] : ["正財", "偏財"];
}

/** 配偶星：先找正星，沒有正星才以偏星代之（男以偏財、女以七殺）。 */
function spouseInfo(ctx: TopicContext) {
  const dayStem = stemOf(ctx.natal[2]);
  const [primary, secondary] = spouseStars(ctx);
  const all = placements(ctx.natal);
  const primaryHits = all.filter((item) => tenGod(dayStem, item.stem) === primary);
  const hits = primaryHits.length ? primaryHits : all.filter((item) => tenGod(dayStem, item.stem) === secondary);
  const name = primaryHits.length ? primary : hits.length ? secondary : primary;
  const visible = hits.filter((item) => item.visible);
  const onlyInStorage = hits.length > 0 && visible.length === 0 && hits.every((item) => STORAGE.includes(item.branch ?? ""));
  return { name, hits, visible, onlyInStorage, storageBranches: [...new Set(hits.map((item) => item.branch ?? ""))] };
}

export function natalMarriage(ctx: TopicContext): TopicSection {
  const natal = ctx.natal;
  const dayStem = stemOf(natal[2]);
  const palace = branchOf(natal[2]);
  const female = ctx.gender === "FEMALE";
  const starLabel = female ? "夫星" : "妻星";
  const items: TopicItem[] = [];
  const info = spouseInfo(ctx);

  if (!info.hits.length) {
    items.push({
      label: `不見${starLabel}`,
      detail: `原局天干、藏干都沒有${female ? "官殺" : "財星"}。書中列為晚婚或婚姻緣薄的信號，要等運歲帶來${starLabel}（進階 第六、七章）。`,
      tone: "bad",
    });
  } else {
    const where = info.hits
      .map((item) => `${PILLARS[item.pillar]}${item.visible ? "干" : `支${item.branch}中`}${item.stem}`)
      .join("、");
    const earliest = Math.min(...info.hits.map((item) => item.pillar));
    const timing = ["戀愛較早", "戀愛時間正常", "戀愛較晚", "戀愛、婚姻較晚"][earliest];
    items.push({
      label: `${starLabel}：${info.name}`,
      detail: `${info.name}在${where}。${starLabel}所在柱看感情早晚：年柱早、月柱正常、日時柱晚，本命${timing}（進階 p145、p167）。`,
      tone: "note",
    });
    if (info.onlyInStorage) {
      items.push({
        label: `${starLabel}入庫`,
        detail: `${starLabel}只藏在${info.storageBranches.join("、")}庫中、沒有透出：主晚婚。${
          female ? "作者觀察：唯一的官星入墓，第一次被沖開是結婚時機，再被沖則往往離異（命例 98）。" : "庫被沖開才有機會（命例 85）。"
        }`,
        tone: "bad",
      });
    }
  }

  const palaceGod = tenGod(dayStem, HIDDEN[palace][0]);
  const relations: string[] = [];
  natal.forEach((pillar, index) => {
    if (index === 2) return;
    const other = branchOf(pillar);
    if (isClash(palace, other)) relations.push(`被${PILLARS[index]}支${other}沖（宮位不穩、易爭執）`);
    if (isSixCombine(palace, other)) relations.push(`與${PILLARS[index]}支${other}合（宮位被合住）`);
    if (isHarm(palace, other)) relations.push(`與${PILLARS[index]}支${other}穿（害，宮位有損）`);
  });
  items.push({
    label: `配偶宮：日支${palace}`,
    detail: `日支本氣為${palaceGod}。${relations.length ? relations.join("；") + "。" : "原局沒有沖合穿，宮位較靜。"}婚期多在運歲改變配偶宮狀態時：合入、由沖變合、原有的合被沖開（進階 p152–155）。`,
    tone: relations.some((text) => text.includes("沖") || text.includes("穿")) ? "bad" : "note",
  });
  if (female && palaceGod === "傷官") {
    items.push({ label: "夫宮傷官", detail: "日支本氣為傷官，傷官剋官，書中視為不太利婚姻（命例 98）。", tone: "bad" });
  }

  const visibleGods = natal
    .map((pillar, index) => (index === 2 ? "" : tenGod(dayStem, stemOf(pillar))))
    .filter(Boolean);
  if (!female && visibleGods.includes("正財") && visibleGods.includes("偏財")) {
    items.push({
      label: "偏正財雜透",
      detail:
        ctx.strength === "弱"
          ? "正財、偏財都透出而日主偏弱：書中說「身弱而偏正財雜透，貧而多淫」，感情易複雜（進階 p149）。"
          : "正財、偏財都透出：書中視為「雙妻」之象，感情易有第三者（進階 p149）。",
      tone: "bad",
    });
  }
  const peerCount = visibleGods.filter((god) => god === "比肩" || god === "劫財").length;
  if (!female && peerCount >= 2) {
    items.push({ label: "比劫爭財", detail: "天干比劫多，男命比劫與妻星對立（劫財奪妻），需要官殺制比劫或食傷化比劫（進階 p146）。", tone: "bad" });
  }
  if (female && visibleGods.includes("正官") && visibleGods.includes("七殺")) {
    items.push({ label: "官殺混雜", detail: "正官、七殺都透出，女命八法列為「濁」，感情易複雜（進階 p166）。", tone: "bad" });
  }
  if (female && visibleGods.includes("傷官") && visibleGods.includes("正官")) {
    items.push({ label: "傷官見官", detail: "天干傷官與正官並見，傷官剋夫星，書中視為婚姻不利的信號（進階 第七章）。", tone: "bad" });
  }
  const outputCount = placements(natal).filter((item) => (item.visible || HIDDEN[item.branch ?? ""]?.[0] === item.stem) && family(dayStem, item.stem) === "食傷").length;
  if (female && outputCount >= 3) {
    items.push({ label: "食傷旺", detail: "食傷多而剋官殺，書中列為晚婚信號，需印星護夫（進階 第七章）。", tone: "bad" });
  }

  const summary = info.hits.length
    ? `${starLabel}為${info.name}${info.onlyInStorage ? "，藏庫未透" : ""}；配偶宮日支${palace}${relations.length ? "與原局有沖合" : "較靜"}。`
    : `原局不見${starLabel}；配偶宮日支${palace}。`;
  return { key: "marriage", title: "婚姻", summary, items };
}

export function natalWealth(ctx: TopicContext): TopicSection {
  const natal = ctx.natal;
  const dayStem = stemOf(natal[2]);
  const day = STEM_ELEMENT[dayStem];
  const wealth = CONTROLS[day];
  const items: TopicItem[] = [];
  const all = placements(natal);
  const wealthHits = all.filter((item) => STEM_ELEMENT[item.stem] === wealth);
  const wealthVisible = wealthHits.filter((item) => item.visible);
  const visibleFamilies = natal.map((pillar, index) => (index === 2 ? "" : family(dayStem, stemOf(pillar))));
  const mainFamilies = natal.map((pillar) => family(dayStem, HIDDEN[branchOf(pillar)][0]));
  const has = (name: string) => visibleFamilies.includes(name) || mainFamilies.includes(name);

  if (!wealthHits.length) {
    items.push({
      label: "命中無財",
      detail: `原局天干、藏干都沒有${wealth}（財）。作者提醒比劫不是財，要靠運歲帶財（進階 p271–272）。`,
      tone: "bad",
    });
  }
  const monthBranch = branchOf(natal[1]);
  const monthHidden = HIDDEN[monthBranch];
  if (STEM_ELEMENT[monthHidden[0]] === wealth) {
    items.push({ label: "財氣通門戶", detail: `月令${monthBranch}本氣是財：財星由「門戶」（月令）而出，是富命的重要條件（進階 p259）。`, tone: "good" });
  } else if (monthHidden.some((stem) => STEM_ELEMENT[stem] === wealth)) {
    const transparent = wealthVisible.some((item) => monthHidden.includes(item.stem));
    items.push({
      label: transparent ? "財由月令透出" : "月令藏財",
      detail: `月令${monthBranch}藏有財星${transparent ? "，並透出天干" : "，未透出"}（財氣通門戶看月令，進階 p259–261）。`,
      tone: transparent ? "good" : "note",
    });
  }
  if (wealthHits.length) {
    if (ctx.strength === "強") {
      items.push({ label: "身能任財", detail: "日主偏強，「身強才能任財」（進階 p260）。", tone: "good" });
    } else if (ctx.strength === "弱") {
      const count = wealthVisible.length + mainFamilies.filter((name) => name === "財").length;
      items.push({
        label: count >= 3 ? "身弱財旺" : "身弱",
        detail:
          count >= 3
            ? "日主偏弱而財多：書中稱「富屋貧人」，財多身弱難以任財，喜印比運補身（進階 p268–270）。"
            : "日主偏弱，任財能力有限，走印比運較能掌握財（進階 p260、p268）。",
        tone: count >= 3 ? "bad" : "note",
      });
    }
  }
  if (has("食傷") && wealthHits.length) {
    items.push({ label: "食傷生財", detail: "有食傷（財的原神）生財：傷官靠創新競爭，食神靠穩健經營（進階 p260、p265）。", tone: "good" });
  }
  const peerVisible = visibleFamilies.includes("比劫");
  if (peerVisible && has("官殺")) {
    items.push({ label: "官殺護財", detail: "天干有比劫，局中也有官殺制比劫，官殺是財的保護神（進階 p260）。", tone: "good" });
  }
  if (wealthVisible.length && peerVisible) {
    items.push({ label: "財露逢比劫", detail: "財星露在天干又見比劫：財要藏不宜露，身強時再走比劫運難免破財（進階 p266–267，命例 148）。", tone: "bad" });
  }
  const stores = STORE_OF[wealth];
  natal.forEach((pillar, index) => {
    const branch = branchOf(pillar);
    if (stores.includes(branch)) {
      const opened = natal.some((other, j) => j !== index && isClash(branch, branchOf(other)));
      items.push({
        label: `${PILLARS[index]}支${branch}為財庫`,
        detail: `${branch}是${wealth}的庫${index === 2 ? "，日主自坐財庫（命例 150 豆腐例）" : ""}${index === 1 ? "，月令財庫規模較大" : ""}${
          index === 2 ? "；日支庫的規模比月令庫小" : ""
        }。${opened ? "原局已有沖刑把庫打開。" : "庫要被沖開才能用，留意沖開財庫的運歲。"}（進階 p266–268）`,
        tone: opened ? "good" : "note",
      });
    }
  });
  const summary = wealthHits.length
    ? `財為${wealth}；${items.filter((item) => item.tone === "good").map((item) => item.label).join("、") || "沒有明顯的富命條件"}。`
    : `原局無財（${wealth}）。`;
  return { key: "wealth", title: "財運", summary, items };
}

export interface ClimateScore {
  total: number;
  level: "暖燥" | "中和" | "寒濕";
  parts: { label: string; score: number }[];
}

export function climateScore(natal: string[]): ClimateScore {
  const parts: { label: string; score: number }[] = [];
  natal.forEach((pillar, index) => {
    parts.push({ label: `${PILLARS[index]}干${stemOf(pillar)}`, score: STEM_CLIMATE[stemOf(pillar)] ?? 0 });
    parts.push({ label: `${PILLARS[index]}支${branchOf(pillar)}`, score: BRANCH_CLIMATE[branchOf(pillar)] ?? 0 });
  });
  parts.push({ label: `月令${branchOf(natal[1])}加分`, score: MONTH_CLIMATE[branchOf(natal[1])] ?? 0 });
  const total = parts.reduce((sum, part) => sum + part.score, 0);
  return { total, level: total > 6 ? "暖燥" : total < -6 ? "寒濕" : "中和", parts };
}

interface OrganState {
  element: Element;
  kind: "weak" | "strong" | "hurt";
  reason: string;
  parts: string[];
}

/** 弱臟、強臟（進階 p305–315）。 */
export function organStates(natal: string[]): OrganState[] {
  const states: OrganState[] = [];
  const stems = natal.map(stemOf);
  const branches = natal.map(branchOf);
  const count = (element: Element) =>
    stems.filter((stem) => STEM_ELEMENT[stem] === element).length + branches.filter((branch) => mainElement(branch) === element).length;
  const disturbed = (index: number) =>
    branches.some((other, j) => j !== index && (isClash(branches[index], other) || isSixCombine(branches[index], other) || isHalfCombine(branches[index], other)));
  const partsOf = (element: Element) => {
    const result: string[] = [];
    natal.forEach((pillar, index) => {
      const stem = stemOf(pillar);
      if (STEM_ELEMENT[stem] === element) {
        const rawPart = BODY_PART[stem]?.[index];
        const part = rawPart && rawPart !== STEM_ORGAN[stem] ? rawPart : "";
        result.push(`${PILLARS[index]}干${stem}（${STEM_ORGAN[stem]}${part ? `；${part}` : ""}）`);
      }
    });
    return result;
  };
  (["木", "火", "土", "金", "水"] as Element[]).forEach((element) => {
    const controller = CONTROLLED_BY[element];
    const visibleIdx = stems.map((stem, index) => (STEM_ELEMENT[stem] === element ? index : -1)).filter((index) => index >= 0);
    const rootIdx = branches
      .map((branch, index) => ((HIDDEN[branch] ?? []).some((stem) => STEM_ELEMENT[stem] === element) ? index : -1))
      .filter((index) => index >= 0);
    const liveRoots = rootIdx.filter((index) => !disturbed(index));
    const tightControl = visibleIdx.some((index) =>
      [index - 1, index + 1].some((j) => j >= 0 && j < 4 && STEM_ELEMENT[stems[j]] === controller),
    );
    const heavyControl = count(controller) >= 3;
    const rootlessAndHit = rootIdx.length === 0 && count(controller) >= 1;
    if (visibleIdx.length && liveRoots.length === 0 && (rootlessAndHit || tightControl || count(controller) >= 2)) {
      states.push({
        element,
        kind: "weak",
        reason: `${element}孤露天干、地支${rootIdx.length ? "的根被沖合" : "無根"}，又受${controller}${tightControl ? "緊貼" : "重重"}剋制`,
        parts: partsOf(element),
      });
    } else if (!visibleIdx.length && rootIdx.length && rootIdx.every((index) => disturbed(index) || heavyControl) && count(element) <= 1) {
      states.push({
        element,
        kind: "weak",
        reason: `${element}只藏在地支（${rootIdx.map((index) => PILLARS[index] + "支" + branches[index]).join("、")}），又被沖合或受${controller}重剋`,
        parts: [],
      });
    }
    const strongTriad = TRIADS.some((triad) => {
      const element3 = mainElement(triad[1]);
      return element3 === element && triad.split("").every((member) => branches.includes(member));
    });
    if (count(element) >= 4 || strongTriad) {
      states.push({
        element,
        kind: "strong",
        reason: `${element}在天干與地支本氣共 ${count(element)} 個${strongTriad ? "，並成三合局" : ""}，五行太過`,
        parts: partsOf(element),
      });
      const victim = CONTROLS[element];
      if (count(victim) > 0) {
        states.push({ element: victim, kind: "hurt", reason: `${victim}受太過的${element}所剋`, parts: partsOf(victim) });
      }
    }
  });
  return states;
}

export function natalHealth(ctx: TopicContext): TopicSection {
  const natal = ctx.natal;
  const items: TopicItem[] = [];
  const climate = climateScore(natal);
  items.push({
    label: `寒暖燥濕 ${climate.total > 0 ? "+" : ""}${climate.total}（${climate.level}）`,
    detail: `${climate.parts.map((part) => `${part.label}${part.score > 0 ? "+" : ""}${part.score}`).join("、")}。−6 到 +6 為中和，超出則偏頗（進階 p19–20）。`,
    tone: climate.level === "中和" ? "good" : "bad",
  });
  const dayStem = stemOf(natal[2]);
  const monthIndex = MONTH_ORDER.indexOf(branchOf(natal[1]));
  const needs = monthIndex >= 0 ? TIAO_HOU[dayStem]?.[monthIndex] ?? "" : "";
  if (needs) {
    const chars = natal.flatMap((pillar) => [stemOf(pillar), ...(HIDDEN[branchOf(pillar)] ?? [])]);
    const present = needs.split("").filter((stem) => chars.includes(stem));
    items.push({
      label: `調候用神：${needs.split("").join("、")}`,
      detail: `《窮通寶鑑》${dayStem}生${branchOf(natal[1])}月取${needs.split("").join("、")}；原局${present.length ? `有${present.join("、")}` : "都沒有"}（進階 表 1.4）。`,
      tone: present.length ? "good" : "bad",
    });
  }
  const states = organStates(natal);
  states.forEach((state) => {
    const label =
      state.kind === "weak" ? `弱臟：${ORGAN[state.element]}（${state.element}）` : state.kind === "strong" ? `強臟：${ORGAN[state.element]}（${state.element}）` : `受剋：${ORGAN[state.element]}（${state.element}）`;
    items.push({
      label,
      detail: `${state.reason}。${state.parts.length ? `對應：${state.parts.join("、")}。` : ""}${
        state.kind === "weak" ? "書中歸納弱臟所對應的臟腑往往是先天疾病因子，運歲再剋時多為發病期（進階 p310）。" : state.kind === "strong" ? "五行太過的強臟本身也可能致病（進階 p310–314）。" : "強臟所剋之臟也容易得病（進階 p314）。"
      }`,
      tone: "bad",
    });
  });
  const branches = natal.map(branchOf);
  const hasEarth = natal.some((pillar) => STEM_ELEMENT[stemOf(pillar)] === "土" || mainElement(branchOf(pillar)) === "土");
  if (hasEarth && climate.total < -6) {
    items.push({ label: "水多土虛", detail: "寒濕而土虛，土主皮膚，水氣淤積易有皮膚病（曾國藩，進階 p315–316）。", tone: "bad" });
  } else if (hasEarth && climate.total > 6) {
    items.push({ label: "火盛土焦", detail: "暖燥而土焦，「燥則皮癢」，易有皮膚過敏（命例 185，進階 p316）。", tone: "bad" });
  }
  if ("申酉戌".includes(branches[1]) && natal.filter((pillar) => STEM_ELEMENT[stemOf(pillar)] === "金" || mainElement(branchOf(pillar)) === "金").length >= 3) {
    items.push({ label: "金氣盛", detail: "秋令金旺氣燥，書中統計常見皮膚過敏、蕁麻疹（命例 186–187，進階 p317）。", tone: "bad" });
  }
  const weak = states.filter((state) => state.kind === "weak").map((state) => ORGAN[state.element]);
  const summary = `寒暖燥濕 ${climate.total}（${climate.level}）${weak.length ? `；弱臟：${weak.join("、")}` : "；未見明顯弱臟"}。`;
  return { key: "health", title: "健康", summary, items };
}

/** 運歲（大運或流年）帶來的婚姻、財運、健康提示。pair 是同時作用的另一干支（流年時為大運）。 */
export function topicEvents(incoming: string, who: "運" | "歲", ctx: TopicContext, pair = ""): LuckEvent[] {
  const natal = ctx.natal;
  if (natal.length !== 4 || incoming.length < 2) return [];
  const events: LuckEvent[] = [];
  const whoName = who === "運" ? "大運" : "流年";
  const dayStem = stemOf(natal[2]);
  const day = STEM_ELEMENT[dayStem];
  const stem = stemOf(incoming);
  const branch = branchOf(incoming);
  const palace = branchOf(natal[2]);
  const female = ctx.gender === "FEMALE";
  const starLabel = female ? "夫星" : "妻星";

  // 婚姻
  if (isSixCombine(branch, palace) || isHalfCombine(branch, palace)) {
    events.push({
      kind: "marriage-combine-palace",
      label: "合配偶宮",
      level: "info",
      topic: "marriage",
      detail: `${whoName}${branch}與日支${palace}相合，配偶宮由靜變動，書中多為婚期或感情機緣（進階 p152–155）。`,
    });
  }
  if (isClash(branch, palace)) {
    events.push({
      kind: "marriage-clash-palace",
      label: "沖配偶宮",
      level: "warn",
      topic: "marriage",
      detail: `${whoName}${branch}沖日支${palace}：宮位動搖，可能是婚期，也可能是爭執、分離（進階 第六、七章）。`,
    });
  }
  natal.forEach((pillar, index) => {
    if (index === 2) return;
    const other = branchOf(pillar);
    if (isSixCombine(palace, other) && isClash(branch, other)) {
      events.push({
        kind: "marriage-release",
        label: "解宮合",
        level: "info",
        topic: "marriage",
        detail: `${whoName}${branch}沖開原局${PILLARS[index]}支${other}與日支的合，配偶宮由合住變為可動，書中列為婚期信號（進階 p152）。`,
      });
    }
  });
  const stars = spouseStars(ctx);
  const incomingGod = tenGod(dayStem, stem);
  if (stars.includes(incomingGod)) {
    events.push({
      kind: "marriage-star",
      label: `${starLabel}透`,
      level: "info",
      topic: "marriage",
      detail: `${whoName}天干${stem}為${incomingGod}（${starLabel}）出現，感情事容易被引動（進階 命例 98：丁運七殺透出，婚姻因子出現）。`,
    });
  }
  if (pair) {
    const pairGod = tenGod(dayStem, stemOf(pair));
    const sealAndStar =
      (incomingGod.endsWith("印") && stars.includes(pairGod)) || (pairGod.endsWith("印") && stars.includes(incomingGod));
    if (sealAndStar) {
      events.push({
        kind: "marriage-certificate",
        label: female ? "印官同現" : "印財同現",
        level: "info",
        topic: "marriage",
        detail: "運歲天干印星與配偶星同時出現：曲煒認為印是文書，常是領結婚證之年（進階 第六章）。",
      });
    }
  }

  // 財運
  const wealth = CONTROLS[day];
  natal.forEach((pillar, index) => {
    const natalBranch = branchOf(pillar);
    if (STORE_OF[wealth].includes(natalBranch) && isClash(branch, natalBranch)) {
      events.push({
        kind: `wealth-open-store-${index}`,
        label: "沖開財庫",
        level: "info",
        topic: "wealth",
        detail: `${whoName}${branch}沖開${PILLARS[index]}支${natalBranch}財庫，書中多為進財時機（命例 150），但沖太過也會破庫（進階 p101–103）。`,
      });
    }
  });
  if (STORE_OF[wealth].includes(branch) && (isSixCombine(branch, palace) || isHalfCombine(branch, palace))) {
    events.push({
      kind: "wealth-store-into-day",
      label: "財庫合入日支",
      level: "info",
      topic: "wealth",
      detail: `${whoName}${branch}是財庫，又與日支${palace}相合，把財合進自身。馬雲丑運酉丑合入日支而創業（進階 p262）。`,
    });
  }
  const wealthVisible = natal.some((pillar, index) => index !== 2 && STEM_ELEMENT[stemOf(pillar)] === wealth);
  if (wealthVisible && STEM_ELEMENT[stem] === day) {
    events.push({
      kind: "wealth-robbed",
      label: "比劫奪財",
      level: "warn",
      topic: "wealth",
      detail: `原局財星露在天干，${whoName}${stem}比劫出現：財露最怕比劫，書中多有此時資金周轉出問題的例子（命例 148）。`,
    });
  }
  if (ctx.strength === "弱" && STEM_ELEMENT[stem] === wealth && mainElement(branch) === wealth) {
    events.push({
      kind: "wealth-heavy",
      label: "財多身弱",
      level: "info",
      topic: "wealth",
      detail: `日主偏弱，${whoName}${incoming}干支皆財，財多身弱難任，書中說喜印比運補身（進階 p268–270）。`,
    });
  }

  // 健康：運歲再剋弱臟（進階 p310）。
  organStates(natal)
    .filter((state) => state.kind === "weak")
    .forEach((state) => {
      const controller = CONTROLLED_BY[state.element];
      const stemHits = STEM_ELEMENT[stem] === controller;
      const branchHits = mainElement(branch) === controller;
      if (stemHits || branchHits) {
        events.push({
          kind: `health-weak-${state.element}`,
          label: `剋弱臟(${state.element})`,
          level: stemHits && branchHits ? "warn" : "info",
          topic: "health",
          detail: `原局${ORGAN[state.element]}（${state.element}）為弱臟，${whoName}${incoming}帶來${controller}再剋。書中說弱臟遇運歲引發，往往是發病期（進階 p310）。`,
        });
      }
    });
  return events;
}

/**
 * 配偶星只藏在庫中時，依時間順序找出沖開該庫的流年（只算 minAge 歲以後）：第一次為婚期候選，之後為婚變留意（命例 98）。
 */
export function spouseStorageYears(
  ctx: TopicContext,
  years: { year: number; ganZhi: string; age?: number }[],
  minAge = 20,
): Map<number, "first" | "again"> {
  const result = new Map<number, "first" | "again">();
  if (ctx.natal.length !== 4) return result;
  const info = spouseInfo(ctx);
  if (!info.onlyInStorage) return result;
  const sorted = [...years].sort((a, b) => a.year - b.year);
  let seen = false;
  for (const row of sorted) {
    if (row.age !== undefined && row.age < minAge) continue;
    const branch = branchOf(row.ganZhi);
    if (info.storageBranches.some((store) => isClash(branch, store))) {
      result.set(row.year, seen ? "again" : "first");
      seen = true;
    }
  }
  return result;
}

export function lifeTopicSections(ctx: TopicContext): TopicSection[] {
  if (ctx.natal.length !== 4) return [];
  return [natalMarriage(ctx), natalWealth(ctx), natalHealth(ctx)];
}
