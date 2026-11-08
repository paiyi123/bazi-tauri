/**
 * 四本書的「人生各項」判斷（2026-10-30 第二批）：
 * - 四柱與六親、姻緣婚嫁：妙算人生八字第七、八篇；八字心訣第 45、52、60、61 訣；子平正源第二章（轉盤論妻妾、六親受剋）。
 * - 健康、事業、財富、學業考運：妙算第九至十二篇；心訣第 57–59、68、74、75 訣。
 * - 大運流年要訣：妙算第十三篇「六神與行運」；心訣第 56–60 訣；子平正源「轉角運」「運剋歲為犯上」。
 *
 * 和第一批一樣只做提示：不改分數，喜忌、用神、強弱都用課堂三步法（沒有時才退回心訣三選二）。
 * 書中舊社會的斷語（妾命、剋夫、生男生女、娼命）不採用；只留結構上的意思。
 */

import type { LuckEvent } from "./luckEvents";
import { familyOf, tenGodOf, threeOfTwo, type Element, type Favor } from "./fourBooks";

export type Tone = "good" | "bad" | "note";

export interface BookItem {
  label: string;
  detail: string;
  tone: Tone;
  source: string;
}

export interface RelativeRow {
  relative: string;
  star: string;
  where: string;
  distance: "緊貼" | "遠離" | "不現";
  favor: Favor;
  palace: string;
  palaceFavor: Favor;
  vigor: string;
}

export interface LuckGodRow {
  family: string;
  element: Element;
  favor: Favor;
  domain: string;
  meaning: string;
  money: string;
  fame: string;
  career: string;
  love: string;
}

export interface BookTopicSection {
  key: "family" | "marriage" | "health" | "career" | "wealth" | "study" | "luck";
  title: string;
  items: BookItem[];
}

export interface BookTopicContext {
  natal: string[];
  gender: "MALE" | "FEMALE";
  /** 課堂三步法的強弱：true 身強、false 身弱；undefined 時用心訣三選二。 */
  strong?: boolean;
  favorOf?: (element: Element) => Favor;
  useElements?: Element[];
  /** 大運干支（依順序），用來看幼運、學歷運。 */
  luck?: string[];
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
const ELEMENTS: Element[] = ["木", "火", "土", "金", "水"];
const CLASH_PAIRS = ["子午", "丑未", "寅申", "卯酉", "辰戌", "巳亥"];
const SIX_COMBINE = ["子丑", "寅亥", "卯戌", "辰酉", "巳申", "午未"];
const STEM_COMBINE = ["甲己", "乙庚", "丙辛", "丁壬", "戊癸"];
const TRIADS = ["申子辰", "亥卯未", "寅午戌", "巳酉丑"];
const MEETINGS = ["寅卯辰", "巳午未", "申酉戌", "亥子丑"];
const STORAGE = "辰戌丑未";
const FAVOR_TEXT: Record<Favor, string> = { good: "喜", bad: "忌", mixed: "依方案", neutral: "平" };

const stemOf = (ganZhi: string) => ganZhi.charAt(0);
const branchOf = (ganZhi: string) => ganZhi.charAt(1);
const isYang = (stem: string) => STEMS.indexOf(stem) % 2 === 0;
const pairIn = (list: string[], a: string, b: string) => list.some((pair) => pair === a + b || pair === b + a);
const isClash = (a: string, b: string) => pairIn(CLASH_PAIRS, a, b);
const isSixCombine = (a: string, b: string) => pairIn(SIX_COMBINE, a, b);
const isStemCombine = (a: string, b: string) => pairIn(STEM_COMBINE, a, b);
const mainStem = (branch: string) => HIDDEN[branch]?.[0] ?? "";
const mainElement = (branch: string) => STEM_ELEMENT[mainStem(branch)];

function familyName(god: string): string {
  if (god === "比肩" || god === "劫財") return "比劫";
  if (god.endsWith("印")) return "印";
  if (god === "食神" || god === "傷官") return "食傷";
  if (god.endsWith("財")) return "財";
  return "官殺";
}

interface Spot {
  pillar: number;
  part: "干" | "支";
  stem: string;
  god: string;
  name: string;
  visible: boolean;
}

/** 七個位置（日干除外）：天干與地支本氣；地支餘氣另外標 visible=false。 */
function spots(natal: string[]): Spot[] {
  const day = stemOf(natal[2]);
  const list: Spot[] = [];
  natal.forEach((pillar, index) => {
    if (index !== 2) {
      const stem = stemOf(pillar);
      list.push({ pillar: index, part: "干", stem, god: tenGodOf(day, stem), name: `${PILLARS[index]}干${stem}`, visible: true });
    }
    const branch = branchOf(pillar);
    HIDDEN[branch].forEach((stem, order) =>
      list.push({
        pillar: index,
        part: "支",
        stem,
        god: tenGodOf(day, stem),
        name: `${PILLARS[index]}支${branch}${order ? `（藏${stem}）` : ""}`,
        visible: order === 0,
      }),
    );
  });
  return list;
}

/** 月干、時干、日支緊貼日主（妙算「緊貼日主」、心訣第 45 訣「距離」）。 */
const isClose = (spot: Spot) => (spot.part === "干" && (spot.pillar === 1 || spot.pillar === 3)) || (spot.part === "支" && spot.pillar === 2);

function counts(natal: string[]): Record<Element, number> {
  const result = { 木: 0, 火: 0, 土: 0, 金: 0, 水: 0 } as Record<Element, number>;
  natal.forEach((pillar, index) => {
    result[STEM_ELEMENT[stemOf(pillar)]] += 1;
    result[mainElement(branchOf(pillar))] += index === 1 ? 2 : 1;
  });
  return result;
}

function hiddenHas(natal: string[], element: Element) {
  return natal.some((pillar) => HIDDEN[branchOf(pillar)].some((stem) => STEM_ELEMENT[stem] === element));
}

function strongOf(ctx: BookTopicContext): boolean {
  return ctx.strong ?? threeOfTwo(ctx.natal, stemOf(ctx.natal[2])).strong;
}

function pillarFavor(ctx: BookTopicContext, index: number): Favor {
  if (!ctx.favorOf) return "neutral";
  const pillar = ctx.natal[index];
  const a = ctx.favorOf(STEM_ELEMENT[stemOf(pillar)]);
  const b = ctx.favorOf(mainElement(branchOf(pillar)));
  if (a === b) return a;
  if (a === "mixed" || b === "mixed") return "mixed";
  if ((a === "good" && b === "bad") || (a === "bad" && b === "good")) return "mixed";
  return a === "neutral" ? b : a;
}

// ---------------------------------------------------------------------------
// 一、四柱與六親。

export function relativeRows(ctx: BookTopicContext): RelativeRow[] {
  const { natal, gender } = ctx;
  const day = stemOf(natal[2]);
  const me = STEM_ELEMENT[day];
  const all = spots(natal);
  const female = gender === "FEMALE";
  const defs: { relative: string; gods: string[]; palace: number; palaceName: string }[] = [
    { relative: "父親", gods: ["偏財", "正財"], palace: 0, palaceName: "年柱" },
    { relative: "母親", gods: ["正印", "偏印"], palace: 0, palaceName: "年柱" },
    { relative: "兄弟姊妹", gods: ["比肩", "劫財"], palace: 1, palaceName: "月柱" },
    { relative: female ? "丈夫" : "妻子", gods: female ? ["正官", "七殺"] : ["正財", "偏財"], palace: 2, palaceName: "日支" },
    { relative: "子女", gods: female ? ["傷官", "食神"] : ["七殺", "正官"], palace: 3, palaceName: "時柱" },
  ];
  return defs.map((def) => {
    // 先找正星，沒有才用偏星（妙算：沒有偏財、正印時用正財、偏印權宜代替）。
    let hits = all.filter((spot) => spot.god === def.gods[0]);
    let star = def.gods[0];
    if (!hits.length) {
      hits = all.filter((spot) => spot.god === def.gods[1]);
      star = def.gods[1];
    }
    const starElement = ELEMENTS.find((item) => familyOf(me, item) === familyName(def.gods[0])) as Element;
    const close = hits.filter(isClose);
    const visible = hits.filter((spot) => spot.visible);
    // 那位親人自己的旺衰（心訣第 4 步）：有沒有本氣或透干、有沒有被它自己的官殺剋（子平正源：親人受剋＝遇到他的官殺）。
    const killer = ELEMENTS.find((item) => CONTROLS[item] === starElement) as Element;
    const killerCount = counts(natal)[killer] - (STEM_ELEMENT[day] === killer ? 1 : 0);
    const supportCount = counts(natal)[starElement] + counts(natal)[ELEMENTS.find((item) => GENERATES[item] === starElement) as Element];
    const inStorageOnly = hits.length > 0 && !visible.length && hits.every((spot) => STORAGE.includes(branchOf(natal[spot.pillar])));
    let vigor = "";
    if (!hits.length) vigor = "原局不見";
    else if (inStorageOnly) vigor = "只藏在庫中（心訣第 61 訣：運歲臨墓要留意）";
    else if (killerCount >= supportCount + 1) vigor = `${killer}（它的官殺）多，受剋偏弱`;
    else if (visible.length && supportCount >= 3) vigor = "有根有勢，自身強";
    else vigor = "一般";
    const palaceFavor = def.palace === 2 && ctx.favorOf ? ctx.favorOf(mainElement(branchOf(natal[2]))) : pillarFavor(ctx, def.palace);
    return {
      relative: def.relative,
      star: hits.length ? `${star}（${starElement}）` : `${def.gods.join("、")}不見`,
      where: hits.length ? hits.map((spot) => spot.name).join("、") : "—",
      distance: !hits.length ? "不現" : close.length ? "緊貼" : "遠離",
      favor: hits.length && ctx.favorOf ? ctx.favorOf(starElement) : "neutral",
      palace: def.palaceName,
      palaceFavor,
      vigor,
    };
  });
}

function familyItems(ctx: BookTopicContext): BookItem[] {
  const { natal } = ctx;
  const items: BookItem[] = [];
  const all = spots(natal);
  const stems = natal.map(stemOf);
  const day = stems[2];

  // 出身（妙算 p255）：年月柱喜忌 × 幼運。
  if (ctx.favorOf) {
    const year = pillarFavor(ctx, 0);
    const month = pillarFavor(ctx, 1);
    const early = ctx.luck?.[0];
    const earlyFavor: Favor | null = early
      ? (() => {
          const a = ctx.favorOf!(STEM_ELEMENT[stemOf(early)]);
          const b = ctx.favorOf!(mainElement(branchOf(early)));
          return a === b ? a : a === "neutral" ? b : b === "neutral" ? a : "mixed";
        })()
      : null;
    const natalGood = year === "good" || month === "good";
    const natalBad = (year === "bad" || year === "neutral") && month === "bad";
    let text = `年柱${FAVOR_TEXT[year]}、月柱${FAVOR_TEXT[month]}${early ? `，第一步大運${early}${FAVOR_TEXT[earlyFavor as Favor]}` : ""}`;
    let tone: Tone = "note";
    if (natalGood && earlyFavor === "good") {
      text += "：年月與幼運都喜，書說出身好、多得祖蔭父母照顧。";
      tone = "good";
    } else if (natalBad && earlyFavor === "bad") {
      text += "：年月與幼運都忌，書說出身寒微、多靠自己打拼。";
      tone = "bad";
    } else {
      text += "：喜忌夾雜，吉凶參半。";
    }
    items.push({ label: "出身", detail: text, tone, source: "妙算 p255" });
  }

  // 父母（妙算 p256–257）。
  const fatherStar = all.filter((spot) => spot.god === "偏財" || spot.god === "正財");
  const motherStar = all.filter((spot) => spot.god === "正印" || spot.god === "偏印");
  const visibleStem = (list: Spot[]) => list.filter((spot) => spot.part === "干");
  const fatherOut = visibleStem(fatherStar).length > 0;
  const motherOut = visibleStem(motherStar).length > 0;
  if (fatherOut !== motherOut) {
    items.push({
      label: fatherOut ? "父親當家" : "母親當家",
      detail: `${fatherOut ? "財星" : "印星"}透在天干：天干是公開的，書說${fatherOut ? "父親" : "母親"}是家裡說話算數的人。`,
      tone: "note",
      source: "妙算 p257 細則 5",
    });
  }
  const yearMonth = [0, 1];
  const fatherYM = fatherStar.some((spot) => yearMonth.includes(spot.pillar) && spot.visible);
  const motherYM = motherStar.some((spot) => yearMonth.includes(spot.pillar) && spot.visible);
  if (fatherYM && motherYM) {
    items.push({ label: "父母星同見年月", detail: "年月柱同時見財星和印星（財剋印＝父母星在打架）：書說童年時父母常爭吵。", tone: "bad", source: "妙算 p257" });
  }
  const robbers = all.filter((spot) => spot.visible && familyName(spot.god) === "比劫").length;
  if (robbers >= 3 && fatherStar.filter((spot) => spot.visible).length <= 1) {
    items.push({ label: "比劫重疊", detail: `比劫 ${robbers} 個而財星少：比劫是財（父親）的官殺，書說「比劫重疊會損父」，和父親緣分要多經營。`, tone: "bad", source: "妙算 p257；子平正源轉盤解釋" });
  }

  // 兄弟（妙算 p257–259）。
  const c = counts(natal);
  const me = STEM_ELEMENT[day];
  const wealthOfficer = c[ELEMENTS.find((item) => familyOf(me, item) === "財") as Element] + c[ELEMENTS.find((item) => familyOf(me, item) === "官殺") as Element];
  const siblings = c[me] - 1;
  if (siblings >= 3 && wealthOfficer <= 2) {
    items.push({ label: "比劫旺、財官弱", detail: "書說兄弟朋友多，但多是來分資源，幫助不大。", tone: "bad", source: "妙算 p258" });
  } else if (siblings <= 1 && wealthOfficer >= 4) {
    items.push({ label: "財官旺、比劫弱", detail: "書說兄弟少，但幫得上忙。", tone: "good", source: "妙算 p258" });
  }
  if (tenGodOf(day, stems[1]) === "七殺" || tenGodOf(day, mainStem(branchOf(natal[1]))) === "七殺") {
    items.push({ label: "月柱七殺", detail: "書說多是長子，或做事有老大作風、想當領導。", tone: "note", source: "妙算 p259 細則 5" });
  }

  // 子女（妙算 p263–265）：時干看女兒、時支看兒子（作者用法）。
  if (ctx.favorOf) {
    const hourStem = ctx.favorOf(STEM_ELEMENT[stemOf(natal[3])]);
    const hourBranch = ctx.favorOf(mainElement(branchOf(natal[3])));
    items.push({
      label: "時柱（子女宮）",
      detail: `時干${stemOf(natal[3])}${FAVOR_TEXT[hourStem]}、時支${branchOf(natal[3])}${FAVOR_TEXT[hourBranch]}。書說時干喜女兒貼心、忌女兒叛逆；時支喜兒子順從、忌兒子難管；晚運和時柱都好，晚年安享。`,
      tone: hourStem === "good" && hourBranch === "good" ? "good" : hourStem === "bad" && hourBranch === "bad" ? "bad" : "note",
      source: "妙算 p264 細則 3、8",
    });
  }
  return items;
}

// ---------------------------------------------------------------------------
// 二、姻緣婚嫁。

const MALE_STRONG_PALACE: Record<string, string> = {
  正官: "配偶端莊、賢內助，能約束幫助自己",
  七殺: "配偶個性剛強，夫妻容易吵",
  正印: "配偶善良穩重；但印多財弱時婆媳不和",
  偏印: "配偶善良穩重；但印多財弱時婆媳不和",
  正財: "得賢內助（身強喜財）",
  偏財: "得賢內助（身強喜財）；子平正源：日坐偏財多外緣",
  食神: "配偶寬厚、能持家（不見偏印剋食）",
  傷官: "配偶秀外慧中、當家；可因配偶的巧思創業",
  比肩: "配偶愛面子、交遊廣、不擅理財",
  劫財: "配偶強悍、耗財，易受配偶拖累",
};
const MALE_WEAK_PALACE: Record<string, string> = {
  正財: "身弱財旺：配偶不太照顧自己，也主怕配偶",
  偏財: "身弱財旺：配偶不太照顧自己，也主怕配偶",
  正印: "配偶賢慧持家，會照顧自己",
  偏印: "配偶賢慧持家，會照顧自己",
  食神: "配偶愛惹是非、對自己嘮叨",
  傷官: "配偶愛惹是非、對自己嘮叨",
  比肩: "配偶剛毅，但會幫助自己的事業",
  劫財: "配偶剛毅，但會幫助自己的事業",
};

function marriageItems(ctx: BookTopicContext): BookItem[] {
  const { natal, gender } = ctx;
  const items: BookItem[] = [];
  const female = gender === "FEMALE";
  const day = stemOf(natal[2]);
  const me = STEM_ELEMENT[day];
  const palace = branchOf(natal[2]);
  const palaceGod = tenGodOf(day, mainStem(palace));
  const strong = strongOf(ctx);
  const all = spots(natal);
  const starFamily = female ? "官殺" : "財";
  const stars = all.filter((spot) => familyName(spot.god) === starFamily);
  const starElement = ELEMENTS.find((item) => familyOf(me, item) === starFamily) as Element;

  // 擇偶（妙算 p275–276）：日支與配偶星的喜忌。
  if (ctx.favorOf) {
    const palaceFavor = ctx.favorOf(mainElement(palace));
    const starFavor = ctx.favorOf(starElement);
    items.push({
      label: `夫妻宮${palace}${FAVOR_TEXT[palaceFavor]}、${female ? "夫" : "妻"}星${starElement}${FAVOR_TEXT[starFavor]}`,
      detail: `日支（夫妻宮）看配偶素質與相處品質，配偶星看感情與助力（妙算）；心訣第 45 訣：宮看「我對他的感覺」，星看「他對我的實質幫助」。${
        palaceFavor === "good" && starFavor === "good" ? "兩者都喜，書說最靈驗的良緣。" : palaceFavor === "bad" && starFavor === "bad" ? "兩者都忌，宜多溝通、晚婚。" : "一喜一忌，宮與星要分開看。"
      }`,
      tone: palaceFavor === "good" && starFavor === "good" ? "good" : palaceFavor === "bad" && starFavor === "bad" ? "bad" : "note",
      source: "妙算 p266、p275；心訣第 45 訣",
    });
  }

  // 日支坐什麼（妙算 p268–269，書寫男命，女命「論法同男」）。
  const table = strong ? MALE_STRONG_PALACE : MALE_WEAK_PALACE;
  if (table[palaceGod]) {
    items.push({
      label: `日支坐${palaceGod}（${strong ? "身強" : "身弱"}）`,
      detail: `${table[palaceGod]}。${female ? "書上這張表寫男命，女命論法相同，把「妻」換成「夫」看。" : ""}`,
      tone: "note",
      source: "妙算 p268–269",
    });
  }
  if (palaceGod === (female ? "正官" : "正財")) {
    items.push({ label: female ? "官填妻位" : "妻填妻位", detail: "日支坐正配偶星：子平正源說主配偶內助、賢良。", tone: "good", source: "子平正源 p149–155" });
  }

  // 配偶星透干又和日主五合（妙算 p276）。
  const partner = female ? "正官" : "正財";
  const combinedPartner = [1, 3, 0].find((index) => isStemCombine(day, stemOf(natal[index])) && tenGodOf(day, stemOf(natal[index])) === partner);
  if (combinedPartner !== undefined) {
    items.push({
      label: `${partner}合日主`,
      detail: `${PILLARS[combinedPartner]}干${stemOf(natal[combinedPartner])}是${partner}又和日干五合：書說配偶${female ? "溫柔體貼、能遷就自己" : "中規中矩、相敬如賓"}，是理想對象${combinedPartner === 0 ? "（但在年干，距離較遠）" : ""}。`,
      tone: "good",
      source: "妙算 p276",
    });
  }

  // 早婚晚婚、人緣（妙算 p279–282）。
  const branches = natal.map(branchOf);
  const hasGroup = [...TRIADS, ...MEETINGS].some((group) => group.split("").every((b) => branches.includes(b)));
  if (hasGroup) items.push({ label: "原局有三合或三會", detail: "書說容易和人結合，傾向早婚。", tone: "note", source: "妙算 p279" });
  const palaceCombiners = branches.filter((b, index) => index !== 2 && isSixCombine(b, palace));
  if (palaceCombiners.length >= 2) {
    items.push({ label: "日支被兩字合", detail: `日支${palace}被${palaceCombiners.join("、")}爭合（妒合）：書說感情複雜。`, tone: "bad", source: "妙算 p280" });
  }
  const palaceClashed = branches.some((b, index) => index !== 2 && isClash(b, palace));
  if (!female) {
    const c = counts(natal);
    const lateSigns = [
      c[starElement] >= 4 ? "財星過旺" : "",
      c[me] - 1 >= 3 ? "比劫重" : "",
      stars.length > 0 && stars.every((spot) => !spot.visible && STORAGE.includes(branchOf(natal[spot.pillar]))) ? "財星入墓" : "",
      palaceClashed ? "日支逢沖" : "",
    ].filter(Boolean);
    if (lateSigns.length) items.push({ label: "晚婚訊號", detail: `${lateSigns.join("、")}：書說男命容易晚婚（現代晚婚本就常見，書說命理推的婚期常比實際早）。`, tone: "note", source: "妙算 p272、p280" });
    // 財多身弱妻奪夫權、身強任財（子平正源妻妾表最後一列；心訣第 60 訣）。
    if (!strong && c[starElement] >= 3) items.push({ label: "財多身弱", detail: "子平正源：妻奪夫權（家裡配偶話事）；心訣：日主太弱財剋不住，主懼內，財旺之年要多經營。", tone: "note", source: "子平正源；心訣第 60 訣" });
    if (strong && c[starElement] <= 1) items.push({ label: "身太強財弱", detail: "心訣第 60 訣：日主太強、財星不足，歲運再不利財星時感情易起波折。", tone: "bad", source: "心訣第 60 訣" });
  } else {
    // 女命看夫君（妙算 p283–284）。
    const officers = stars.filter((spot) => spot.visible);
    const both = officers.some((spot) => spot.god === "正官") && officers.some((spot) => spot.god === "七殺");
    const hurt = all.filter((spot) => spot.visible && spot.god === "傷官").length;
    const wealth = all.filter((spot) => spot.visible && familyName(spot.god) === "財").length;
    if (both) items.push({ label: "官殺並見", detail: "書說異性緣佳、容易被感情迷惑；官殺混雜時家庭風波較多。", tone: "note", source: "妙算 p270" });
    if (ctx.favorOf?.(starElement) === "good" && wealth > 0) items.push({ label: "能得夫助", detail: "官殺是喜用、又有財生官。", tone: "good", source: "妙算 p283" });
    if (hurt >= 2 && wealth === 0) items.push({ label: "傷官重無財化", detail: "傷官剋官（夫星），中間沒有財把傷官轉成生官：書說不易得夫助。", tone: "bad", source: "妙算 p283" });
    if (both && hurt > 0 && palaceClashed) items.push({ label: "書列婚變條件", detail: "官殺混雜又傷官見官，日支又逢沖：書列為離婚條件之一，只當提醒。", tone: "bad", source: "妙算 p284" });
    const c = counts(natal);
    if (c[starElement] >= 4 || (!officers.length && stars.length > 0)) {
      items.push({ label: c[starElement] >= 4 ? "官殺過旺" : "官殺若有若無", detail: "書說夫緣較難美滿，最好晚婚。", tone: "note", source: "妙算 p270" });
    }
    // 食神暗合（林註 p286）。
    const food = STEMS[(STEMS.indexOf(day) + 4) % 10];
    const target = isYang(day) ? "正官" : "正印";
    const partnerStem = stems(natal).find((stem) => isStemCombine(food, stem) && tenGodOf(day, stem) === target);
    if (stems(natal).includes(food) && partnerStem) {
      items.push({ label: "食神暗合", detail: `食神${food}和${target}${partnerStem}相合：林註說是一種吸引異性的力量，喜忌決定是好緣還是困擾。`, tone: "note", source: "妙算 p286" });
    }
  }
  return items;
}

const stems = (natal: string[]) => natal.map(stemOf);

// ---------------------------------------------------------------------------
// 三、健康。

const ORGAN: Record<Element, string> = { 木: "肝膽、筋骨、四肢", 火: "心、血、眼、小腸", 土: "脾胃、皮肉", 金: "肺、大腸、氣管", 水: "腎、膀胱、耳" };
const DANGER: Record<Element, string> = { 木: "木器所傷", 火: "火災、燙傷、爆炸", 土: "土石、落石", 金: "刀械、鐵器", 水: "水邊活動" };
const FIGHT: Record<string, string> = {
  木土: "脾胃、腸炎、肝膽結石、偏頭痛、筋骨扭傷",
  火金: "肺、咽喉、皮膚、頭暈失眠、血氣失調",
  土水: "腎膀胱、健忘、動脈硬化、消化不良",
  金木: "筋骨關節、肝膽、眼目、四肢、哮喘",
  水火: "心血、中風、小腸、眼目、泌尿",
};

function healthItems(ctx: BookTopicContext): BookItem[] {
  const { natal } = ctx;
  const items: BookItem[] = [];
  const c = counts(natal);
  const day = stemOf(natal[2]);
  const me = STEM_ELEMENT[day];
  items.push({ label: `${day}${me}日主`, detail: `本身容易出問題的地方：${ORGAN[me]}（妙算 p288）。`, tone: "note", source: "妙算 p288" });
  ELEMENTS.forEach((element) => {
    if (c[element] >= 4) items.push({ label: `${element}太旺`, detail: `${element}有 ${c[element]} 份（月支算兩份）：實症，留意${ORGAN[element]}。`, tone: "bad", source: "妙算 p289" });
    else if (c[element] === 0) {
      items.push({
        label: hiddenHas(natal, element) ? `${element}只藏不現` : `${element}全缺`,
        detail: `虛症，留意${ORGAN[element]}。${hiddenHas(natal, element) ? "" : "妙算：五行和者一世無災。"}`,
        tone: hiddenHas(natal, element) ? "note" : "bad",
        source: "妙算 p289",
      });
    }
  });
  // 兩強相剋（心訣第 74 訣；妙算：兩強相剋又無通關易犯血光）。
  ELEMENTS.forEach((a) => {
    const b = CONTROLS[a];
    if (c[a] >= 3 && c[b] >= 3) {
      const key = [`${a}${b}`, `${b}${a}`].find((item) => FIGHT[item]) as string;
      const bridge = GENERATES[a];
      const bridged = c[bridge] > 0;
      items.push({
        label: `${a}${b}相戰`,
        detail: `${a}、${b}都旺而相剋：心訣第 74 訣容易的病是${FIGHT[key]}。${bridged ? `局中有${bridge}通關，較緩和。` : `局中沒有${bridge}通關，妙算說易犯血光，行運見${bridge}可解。`}`,
        tone: "bad",
        source: "心訣第 74 訣；妙算 p291",
      });
    }
  });
  // 意外災厄看忌什麼（妙算 p292）。
  if (ctx.favorOf) {
    const bad = ELEMENTS.filter((element) => ctx.favorOf!(element) === "bad");
    if (bad.length) items.push({ label: "生活提醒", detail: `忌${bad.join("、")}：書說要防${bad.map((element) => DANGER[element]).join("、")}。當生活提醒即可。`, tone: "note", source: "妙算 p292" });
  }
  // 精神健康（心訣第 75 訣）。
  const strong = strongOf(ctx);
  const output = ELEMENTS.find((item) => familyOf(me, item) === "食傷") as Element;
  const seal = ELEMENTS.find((item) => familyOf(me, item) === "印") as Element;
  if (strong && c[output] === 0) items.push({ label: "旺而無洩", detail: "日主旺又沒有食傷洩秀：心訣說痛苦無處宣洩，容易抑鬱；多找抒發管道。", tone: "note", source: "心訣第 75 訣" });
  if (c[output] >= 4 && !strong) items.push({ label: "食傷洩重", detail: "心訣：食傷洩得太重，神經系統較弱、易失眠。", tone: "note", source: "心訣第 75 訣" });
  if (!strong && c[seal] === 0) items.push({ label: "弱而缺印", detail: "心訣：日主弱又缺印，體質和意志較薄弱。", tone: "note", source: "心訣第 75 訣" });
  if (c[seal] >= 4) items.push({ label: "印多為病", detail: "心訣：印多容易精神恍惚、想太多。", tone: "note", source: "心訣第 75 訣" });
  if (c.土 >= 4 && c.木 === 0) items.push({ label: "土多無木", detail: "心訣：土多又沒有木疏導，不懂變通、易鑽牛角尖。", tone: "note", source: "心訣第 75 訣" });
  return items;
}

// ---------------------------------------------------------------------------
// 四、事業、田宅、官非。

const CAREER: Record<string, string> = {
  官殺: "有管理能力：政治、司法、公務；七殺偏軍警與競爭性行業",
  印: "文學、學術、教育、行政、宗教、慈善；偏印偏醫、科技、研究、五術，常身兼兩職",
  財: "金融、財政、營商；正財穩定開店，偏財貿易、運輸、觀光、推銷",
  食傷: "文學藝術；食神偏服務、幼教、演藝、餐飲，傷官偏口才（律師、教授）和技術",
  比劫: "自由業、服務業、合夥事業、貿易仲介，多白手起家",
};
const TRADE: Record<Element, string> = {
  木: "園藝、木器傢俱、文具出版、建材、農林",
  火: "電子電器、光學、能源、美容、餐飲、百貨",
  土: "房地產、營造、仲介、礦產、農牧",
  金: "金融保險、珠寶、機械五金、儀器",
  水: "運輸、航海、飲料冷凍、清潔、傳播、旅遊、水利",
};
const HORSE: Record<string, string> = { 申: "寅", 子: "寅", 辰: "寅", 寅: "申", 午: "申", 戌: "申", 巳: "亥", 酉: "亥", 丑: "亥", 亥: "巳", 卯: "巳", 未: "巳" };

function careerItems(ctx: BookTopicContext): BookItem[] {
  const { natal } = ctx;
  const items: BookItem[] = [];
  const day = stemOf(natal[2]);
  const me = STEM_ELEMENT[day];
  const strong = strongOf(ctx);
  (ctx.useElements ?? []).forEach((use) => {
    const family = familyOf(me, use);
    items.push({ label: `用神${use}（${family}）選行業`, detail: `${CAREER[family]}。`, tone: "good", source: "妙算 p293–296" });
  });
  // 透旺（妙算 p297–298）：日主以外最旺的天干五行；身強不取印比。
  const c = counts(natal);
  const candidates = ELEMENTS.filter((element) => stems(natal).some((stem, index) => index !== 2 && STEM_ELEMENT[stem] === element))
    .filter((element) => !strong || !["印", "比劫"].includes(familyOf(me, element)))
    .sort((a, b) => c[b] - c[a]);
  if (candidates.length) {
    const top = candidates[0];
    items.push({ label: `透旺：${top}`, detail: `日主以外最旺的透干五行是${top}（${familyOf(me, top)}${strong ? "，身強不取印比" : ""}），工作性質偏向：${TRADE[top]}。`, tone: "note", source: "妙算 p297–298" });
  }
  // 驛馬（妙算 p298–300）：以年支、日支查；在哪一柱就在那段年齡變動（每柱 16 年）。
  const branches = natal.map(branchOf);
  const horses = new Set([HORSE[branches[0]], HORSE[branches[2]]]);
  branches.forEach((branch, index) => {
    if (!horses.has(branch)) return;
    const ages = ["1–16", "17–32", "33–48", "49–64"][index];
    items.push({
      label: `驛馬在${PILLARS[index]}支${branch}`,
      detail: `一生閒不住、常出差外調；書說${ages} 歲前後因${["祖上父母", "父母兄姊", "自己或配偶", "自己或子女"][index]}而遷動。驛馬${ctx.favorOf ? FAVOR_TEXT[ctx.favorOf(mainElement(branch))] : ""}：逢沖時吉馬動而有成、凶馬雪上加霜；逢合被絆住。`,
      tone: "note",
      source: "妙算 p298–300",
    });
  });
  if (isClash(branches[0], branches[1])) items.push({ label: "年月支相沖", detail: "書說很早離開出生地。", tone: "note", source: "妙算 p300" });
  // 田宅（妙算 p301）：年干外觀、年支內部。
  if (ctx.favorOf) {
    const outside = ctx.favorOf(STEM_ELEMENT[stemOf(natal[0])]);
    const inside = ctx.favorOf(mainElement(branches[0]));
    items.push({
      label: "田宅",
      detail: `年干${FAVOR_TEXT[outside]}（外觀、環境）、年支${FAVOR_TEXT[inside]}（室內）。${outside === "good" && inside === "bad" ? "外牆不錯、室內凌亂。" : outside === "bad" && inside === "good" ? "外觀老舊、室內講究。" : ""}`,
      tone: outside === "good" && inside === "good" ? "good" : outside === "bad" && inside === "bad" ? "bad" : "note",
      source: "妙算 p301",
    });
  }
  // 官非（妙算 p302–303）。
  const all = spots(natal).filter((spot) => spot.visible);
  const hurt = all.filter((spot) => spot.god === "傷官");
  const officer = all.filter((spot) => spot.god === "正官");
  const near = hurt.some((h) => officer.some((o) => o.pillar === h.pillar || (o.part === h.part && Math.abs(o.pillar - h.pillar) === 1)));
  if (near) items.push({ label: "傷官見官", detail: "傷官和正官同柱或緊貼：書說「為禍百端」，作者認為官非這條最重要；文件、合約要謹慎。", tone: "bad", source: "妙算 p302" });
  const killers = c[ELEMENTS.find((item) => familyOf(me, item) === "官殺") as Element];
  if (!strong && killers >= 4) {
    const seal = c[ELEMENTS.find((item) => familyOf(me, item) === "印") as Element];
    items.push({ label: "身弱官殺旺", detail: `書說口舌是非多，走官殺運更明顯；${seal ? "有印轉化，較輕" : "沒有印轉化"}。`, tone: "bad", source: "妙算 p302" });
  }
  return items;
}

// ---------------------------------------------------------------------------
// 五、財富。

function wealthItems(ctx: BookTopicContext): BookItem[] {
  const { natal } = ctx;
  const items: BookItem[] = [];
  const day = stemOf(natal[2]);
  const me = STEM_ELEMENT[day];
  const strong = strongOf(ctx);
  const c = counts(natal);
  const wealth = ELEMENTS.find((item) => familyOf(me, item) === "財") as Element;
  const seal = ELEMENTS.find((item) => familyOf(me, item) === "印") as Element;
  const all = spots(natal);
  const wealthOut = all.some((spot) => spot.part === "干" && familyName(spot.god) === "財");
  const wealthAny = all.some((spot) => familyName(spot.god) === "財");
  const robOut = all.some((spot) => spot.part === "干" && familyName(spot.god) === "比劫");
  const sealOut = all.some((spot) => spot.part === "干" && familyName(spot.god) === "印");
  let base: BookItem;
  if (strong && c[wealth] >= 3) base = { label: "身強財旺", detail: "書說先天大富之命，歲運幫助則財源滾滾。", tone: "good", source: "妙算 p305" };
  else if (!strong && c[wealth] >= 3 && (c[seal] + c[me] - 1) >= 2) base = { label: "財多身弱有幫身", detail: "財多身弱但有印比幫身：書說也有發財之命。", tone: "good", source: "妙算 p305" };
  else if (!strong && c[wealth] >= 3) base = { label: "財多身弱", detail: "機會多但扛不起，宜合夥或靠印比運。", tone: "bad", source: "妙算 p305" };
  else base = { label: "財不算旺", detail: `財（${wealth}）${c[wealth]} 份，靠行運補。`, tone: "note", source: "妙算 p305" };
  items.push(base);
  if (wealthAny && !wealthOut && all.filter((spot) => familyName(spot.god) === "財").every((spot) => STORAGE.includes(branchOf(natal[spot.pillar])))) {
    items.push({ label: "財藏於庫", detail: "財星不透、只藏庫中：書說生性勤儉，不亂花錢。", tone: "good", source: "妙算 p305" });
  }
  if ((strong && robOut) || (!strong && wealthOut)) {
    items.push({ label: "能賺能花", detail: strong ? "身強比劫透干：朋友應酬多、出手大方，不易累積。" : "身弱財星透干：財擺在外面守不住。", tone: "bad", source: "妙算 p305" });
  } else if ((strong && wealthOut) || (!strong && sealOut)) {
    items.push({ label: "能賺而節儉", detail: strong ? "身強財星透干。" : "身弱印星透干。", tone: "good", source: "妙算 p305" });
  }
  items.push({
    label: "發財／破財的運",
    detail: strong ? "身強：食傷、財運發財（透干更好）；比劫、印運破財。" : "身弱：比劫、印運發財（合夥得財）；食傷、財、官殺運破財。",
    tone: "note",
    source: "妙算 p305–306",
  });
  // 財從哪裡來（妙算 p306–308）。
  if (ctx.favorOf) {
    const year = pillarFavor(ctx, 0);
    const month = pillarFavor(ctx, 1);
    if (year === "good") items.push({ label: "年柱喜用", detail: "書說有祖產可承接。", tone: "good", source: "妙算 p306" });
    if (year === "bad" && month === "bad") items.push({ label: "年月皆忌", detail: "書說多不得祖蔭，白手起家。", tone: "note", source: "妙算 p306" });
    const branches = natal.map(branchOf);
    const sameTriad = TRIADS.some((triad) => triad.includes(branches[0]) && triad.includes(branches[1]) && branches[0] !== branches[1]);
    if (year === "good" && (sameTriad || isSixCombine(branches[0], branches[1]))) items.push({ label: "宜在故鄉發展", detail: "年柱喜用、年月支又相合：書說喜歡出生地、最適合在故鄉發展。", tone: "good", source: "妙算 p307" });
    if (year === "bad" && isClash(branches[0], branches[1])) items.push({ label: "宜離鄉打拼", detail: "年柱為忌、年月支相沖：書說宜年少離鄉。", tone: "note", source: "妙算 p307" });
    const palaceHidden = HIDDEN[branches[2]];
    if (palaceHidden.includes(stemOf(natal[0])) && ctx.favorOf(STEM_ELEMENT[stemOf(natal[0])]) === "good") {
      items.push({ label: "日支透年干且喜", detail: "書說「嫁妝一牛車」，可能因結婚得到配偶家族的財。", tone: "good", source: "妙算 p307" });
    }
  }
  return items;
}

// ---------------------------------------------------------------------------
// 六、學業考運。

function studyItems(ctx: BookTopicContext): BookItem[] {
  const { natal } = ctx;
  const items: BookItem[] = [];
  const day = stemOf(natal[2]);
  const me = STEM_ELEMENT[day];
  const strong = strongOf(ctx);
  const c = counts(natal);
  const all = spots(natal);
  const seal = ELEMENTS.find((item) => familyOf(me, item) === "印") as Element;
  const output = ELEMENTS.find((item) => familyOf(me, item) === "食傷") as Element;
  const officer = ELEMENTS.find((item) => familyOf(me, item) === "官殺") as Element;
  const wealth = ELEMENTS.find((item) => familyOf(me, item) === "財") as Element;
  const sealOut = all.some((spot) => spot.part === "干" && familyName(spot.god) === "印");
  const sealAny = all.some((spot) => familyName(spot.god) === "印");
  items.push(
    sealOut
      ? { label: "印透干", detail: "印是文書知識的來源，透干的人手不釋卷、求知慾強。", tone: "good", source: "妙算 p309" }
      : sealAny
        ? { label: "印藏不透", detail: "有印但沒透出：喜歡學習，但不算愛讀書的類型。", tone: "note", source: "妙算 p309" }
        : { label: "命中無印", detail: "書說多不喜讀書、常識較少；行印運補上。", tone: "bad", source: "妙算 p309" },
  );
  if ((me === "金") && all.some((spot) => spot.part === "干" && STEM_ELEMENT[spot.stem] === "水")) {
    items.push({ label: "金水洩秀", detail: "庚辛日主見壬癸透：書云「金水洩秀，出身科甲」，聰明、學歷多高。", tone: "good", source: "妙算 p310" });
  } else if (c[output] >= 2 && strong) {
    items.push({ label: "食傷洩秀", detail: "身強有食傷：聰明、才華顯露。", tone: "good", source: "妙算 p310" });
  }
  const stemsList = stems(natal);
  const pairNear = (a: Element, b: Element) =>
    stemsList.some((s, i) => STEM_ELEMENT[s] === a && [i - 1, i + 1].some((j) => j >= 0 && j < 4 && STEM_ELEMENT[stemsList[j]] === b));
  if (pairNear(officer, seal)) items.push({ label: "官印相生", detail: "官殺和印相鄰透干：書說功課好、易得高學位（不見傷官剋官、財壞印時）。", tone: "good", source: "妙算 p310 細則 4" });
  const hurtOut = all.some((spot) => spot.part === "干" && spot.god === "傷官");
  if (hurtOut && sealOut) items.push({ label: "傷官佩印", detail: "書說天生聰穎，行運相助必獲高學歷（不見財壞印）。", tone: "good", source: "妙算 p311 細則 8" });
  if (strong && c[seal] >= 3 && c[output] <= 1) items.push({ label: "身強印重、食傷輕", detail: "書說才華難發揮，幼少年功課較吃力。", tone: "bad", source: "妙算 p311 細則 7" });
  // 喜用集中在哪兩柱（細則 9）。
  if (ctx.favorOf) {
    const goodCount = (indexes: number[]) => indexes.reduce((sum, index) => sum + [STEM_ELEMENT[stemOf(natal[index])], mainElement(branchOf(natal[index]))].filter((element) => ctx.favorOf!(element) === "good").length, 0);
    const early = goodCount([0, 1]);
    const late = goodCount([2, 3]);
    if (early !== late) {
      items.push({
        label: early > late ? "喜用集中年月" : "喜用集中日時",
        detail: early > late ? "書說少年就嶄露頭角。" : "書說中年後興起再讀書的念頭，多能如願。",
        tone: "note",
        source: "妙算 p311 細則 9",
      });
    }
    const studyLuck = (ctx.luck ?? []).slice(0, 2);
    if (studyLuck.length) {
      const marks = studyLuck.map((gz) => `${gz}（${FAVOR_TEXT[ctx.favorOf!(STEM_ELEMENT[stemOf(gz)])]}／${FAVOR_TEXT[ctx.favorOf!(mainElement(branchOf(gz)))]}）`);
      items.push({ label: "學歷看第一、二步大運", detail: `${marks.join("、")}。`, tone: "note", source: "妙算 p309" });
    }
  }
  // 考運：命局類型 → 流年宜走（妙算 p314）。
  const pairs: { when: boolean; label: string; go: string }[] = [
    { when: strong && c[seal] >= 3, label: "身印兩旺", go: `財星（${wealth}）年` },
    { when: strong && c[me] - 1 >= 3, label: "身比兩強", go: `官殺（${officer}）或食傷（${output}）年` },
    { when: strong && c[officer] >= 3, label: "身官殺兩強", go: `食傷（${output}）年（食神制殺、傷官駕官）` },
    { when: !strong && c[wealth] >= 3, label: "身衰財旺", go: `比劫（${me}）年` },
    { when: !strong && c[seal] >= 3, label: "身弱印旺", go: `官殺（${officer}）年` },
  ];
  const hit = pairs.find((pair) => pair.when);
  items.push({
    label: "考試年",
    detail: `${hit ? `命局屬「${hit.label}」，考試宜走${hit.go}；` : ""}流年最重要：日主和太歲宜相合，忌相沖；大運不好但流年好，考取機會仍大（流年重於大運）。`,
    tone: "note",
    source: "妙算 p312–314",
  });
  return items;
}

// ---------------------------------------------------------------------------
// 七、大運流年要訣：每一類十神的運會動到哪方面（妙算第十三篇；心訣第 57–60 訣）。

const DOMAIN: Record<string, { domain: string; good: string; bad: string }> = {
  官殺: { domain: "職位、名譽、主管、丈夫、子女（男）", good: "考試升遷、官司得直、女命好姻緣、男命得子", bad: "是非、丟職、衝動惹官司、意外、女命感情煩惱" },
  印: { domain: "貴人、長輩、母親、學習、宗教", good: "長輩貴人幫、學業職場順、名聲", bad: "長輩不幫、考試謀職波折、判斷錯誤損財、母親健康" },
  比劫: { domain: "兄弟朋友、合夥、父親與妻子（被比劫剋）", good: "朋友合夥相助、繼承家業、開創事業", bad: "被朋友連累、合夥失敗、和配偶衝突" },
  食傷: { domain: "創作、表演、子女（女）、部屬、女命丈夫", good: "作品得獎、企劃通過、才華發揮、女命懷孕", bad: "理想多實現少、不當求財惹官司、女命不利夫、男命子女" },
  財: { domain: "金錢、生意、財產、婚姻（男）、父親", good: "財運好、男命女人緣佳、未婚男結婚時機", bad: "破財、事業垮、桃花惹禍、父親健康" },
};
const XJ_MONEY: Record<string, string> = { 官殺: "耗財", 印: "投資耗財", 財: "錢財", 食傷: "財源", 比劫: "破財" };
const XJ_FAME: Record<string, string> = { 官殺: "壓力、頭銜", 印: "憂愁煩悶", 財: "錢財名譽", 食傷: "名譽", 比劫: "幫手" };
const XJ_CAREER: Record<string, string> = { 官殺: "官職", 印: "權力", 財: "官的根基", 食傷: "阻滯（意見不合）", 比劫: "阻滯（競爭）" };
const XJ_LOVE_F: Record<string, string> = { 官殺: "丈夫、情人", 印: "助力", 財: "官的根基", 食傷: "好勝、傷感情", 比劫: "是非波折" };
const XJ_LOVE_M: Record<string, string> = { 官殺: "困難", 印: "憂愁煩悶", 財: "妻子、情人", 食傷: "助力", 比劫: "是非波折" };

export function luckGodRows(ctx: BookTopicContext): LuckGodRow[] {
  const me = STEM_ELEMENT[stemOf(ctx.natal[2])];
  return ["官殺", "印", "比劫", "食傷", "財"].map((family) => {
    const element = ELEMENTS.find((item) => familyOf(me, item) === family) as Element;
    const favor = ctx.favorOf ? ctx.favorOf(element) : "neutral";
    const info = DOMAIN[family];
    return {
      family,
      element,
      favor,
      domain: info.domain,
      meaning: favor === "good" ? info.good : favor === "bad" ? info.bad : `喜：${info.good}；忌：${info.bad}`,
      money: XJ_MONEY[family],
      fame: XJ_FAME[family],
      career: XJ_CAREER[family],
      love: (ctx.gender === "FEMALE" ? XJ_LOVE_F : XJ_LOVE_M)[family],
    };
  });
}

function luckItems(): BookItem[] {
  return [
    { label: "看一年的步驟", detail: "原局喜忌 → 這個年齡是哪一柱（限運）→ 大運干支喜忌、蓋頭截腳（以地支為重）→ 流年喜忌（流年為君、比大運重）→ 刑沖合會 → 走到哪個十神，事情就發生在那方面。", tone: "note", source: "妙算第十三篇" },
    { label: "汽車比喻", detail: "命局是汽車、大運是路況、流年是天氣、流月是波浪；先把流年和大運合看，再加到原局。天干看合、地支看沖。", tone: "note", source: "心訣第 56 訣" },
    { label: "大運看支、歲君看干", detail: "原有原無、原輕原重：原局沒有根的東西，大運帶來也留不住。大運剋太歲是犯上（凶），太歲剋大運尚可；大運地支跨方位（轉角運）主環境大變。", tone: "note", source: "子平正源 p164–172" },
  ];
}

export function fourBookTopicSections(ctx: BookTopicContext): BookTopicSection[] {
  return [
    { key: "family", title: "四柱與六親", items: familyItems(ctx) },
    { key: "marriage", title: "姻緣婚嫁", items: marriageItems(ctx) },
    { key: "health", title: "健康", items: healthItems(ctx) },
    { key: "career", title: "事業・田宅・官非", items: careerItems(ctx) },
    { key: "wealth", title: "財富", items: wealthItems(ctx) },
    { key: "study", title: "學業考運", items: studyItems(ctx) },
    { key: "luck", title: "大運流年要訣", items: luckItems() },
  ];
}

// ---------------------------------------------------------------------------
// 運歲標籤（第二批）。

export interface TopicLuckOptions {
  natal: string[];
  gender: "MALE" | "FEMALE";
  favorOf?: (element: Element) => Favor;
  useElements?: Element[];
  /** 流年所在的大運，或大運的前一步大運。 */
  luck?: string;
  previousLuck?: string;
}

const DIRECTION: Record<string, string> = { 寅: "東", 卯: "東", 辰: "東", 巳: "南", 午: "南", 未: "南", 申: "西", 酉: "西", 戌: "西", 亥: "北", 子: "北", 丑: "北" };

export function fourBookTopicEvents(incoming: string, who: "運" | "歲", options: TopicLuckOptions): LuckEvent[] {
  const events: LuckEvent[] = [];
  const { natal } = options;
  if (natal.length < 4 || incoming.length < 2) return events;
  const stem = stemOf(incoming);
  const branch = branchOf(incoming);
  const day = stemOf(natal[2]);
  const palace = branchOf(natal[2]);

  // 轉角運（子平正源）：大運地支從一個方位跨到另一個方位。
  if (who === "運" && options.previousLuck) {
    const from = DIRECTION[branchOf(options.previousLuck)];
    const to = DIRECTION[branch];
    if (from && to && from !== to) {
      events.push({
        kind: "corner-luck",
        label: "轉角運",
        level: "info",
        detail: `大運地支由${from}方轉入${to}方（${branchOf(options.previousLuck)}→${branch}）：子平正源說人生環境有大變化，好壞看${to}方${options.favorOf ? `（${mainElement(branch)}，課堂${FAVOR_TEXT[options.favorOf(mainElement(branch))]}）` : ""}是不是用神。`,
      });
    }
  }

  // 運剋歲為犯上（子平正源 p171）。
  if (who === "歲" && options.luck) {
    const luckStem = stemOf(options.luck);
    if (CONTROLS[STEM_ELEMENT[luckStem]] === STEM_ELEMENT[stem]) {
      events.push({
        kind: "luck-offends-year",
        label: "運剋歲",
        level: "info",
        detail: `大運天干${luckStem}剋流年天干${stem}：《淵海子平》「運剋歲者凶」，子平正源稱「犯上」；太歲剋大運則尚可。`,
      });
    }
  }

  // 天地合日柱（心訣第 52 訣婚期）。
  if (isStemCombine(stem, day) && isSixCombine(branch, palace)) {
    events.push({
      kind: "marriage-heaven-earth-day",
      label: "天地合日柱",
      level: "info",
      topic: "marriage",
      detail: `${who === "運" ? "大運" : "流年"}${incoming}和日柱${natal[2]}天干五合、地支六合：心訣第 52 訣列為婚期良機（成不成在於人）。`,
    });
  }

  // 雙沖用神（妙算 p291）：大運、流年同時沖同一個用神地支。
  if (who === "歲" && options.luck && options.useElements?.length) {
    const luckBranch = branchOf(options.luck);
    natal.forEach((pillar, index) => {
      const target = branchOf(pillar);
      if (options.useElements!.includes(mainElement(target)) && isClash(branch, target) && isClash(luckBranch, target)) {
        events.push({
          kind: `double-clash-use-${index}`,
          label: "雙沖用神",
          level: "warn",
          topic: "health",
          detail: `大運${luckBranch}、流年${branch}同時沖${PILLARS[index]}支${target}（用神${mainElement(target)}）：妙算說必有大病或橫禍之虞，這段時間注意健康與交通安全。`,
        });
      }
    });
  }

  // 驛馬逢沖（妙算 p299）。
  const horses = new Set([HORSE[branchOf(natal[0])], HORSE[branchOf(natal[2])]]);
  natal.forEach((pillar, index) => {
    const target = branchOf(pillar);
    if (horses.has(target) && isClash(branch, target)) {
      const favor = options.favorOf ? options.favorOf(mainElement(target)) : "neutral";
      events.push({
        kind: `horse-clash-${index}`,
        label: "驛馬動",
        level: favor === "bad" ? "warn" : "info",
        detail: `${who === "運" ? "大運" : "流年"}${branch}沖${PILLARS[index]}支驛馬${target}：書說主遷動、出差、換環境；${favor === "good" ? "吉馬逢沖，動了反而好" : favor === "bad" ? "凶馬逢沖，雪上加霜" : "吉凶看驛馬喜忌"}（妙算 p299）。`,
      });
    }
  });
  return events;
}
