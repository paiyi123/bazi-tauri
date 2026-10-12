/**
 * 陸致極《八字命理學基礎教程》附錄「八字結構分析圖解流程」（p295–302）。
 *
 * 照書上的入門手算法：五行字數不加權（天干＋地支本氣，地支附屬之氣另列括號），
 * 比較日主同方與異方；十神計分取天干與全部藏干；以主導勢力定格局，再查表 7.1 取用神，
 * 並用表 7.2、7.3 與第七章的成敗條件檢查格局。只作參考，不改變量化五行力分數。
 */

import type { LuckEvent } from "./luckEvents";
import { climateScore, tenGod, type ClimateScore, type Element } from "./lifeTopics";

export type Season = "春" | "夏" | "秋" | "冬" | "季";
export type WangState = "旺" | "相" | "休" | "囚" | "死";
export type Relation = "日主" | "食傷" | "財" | "官殺" | "印";

export interface BasicElementRow {
  element: Element;
  relation: Relation;
  party: "同方" | "異方";
  state: WangState;
  /** 天干＋地支本氣的字數。 */
  main: number;
  /** 地支中氣、餘氣的字數（書上寫在括號內）。 */
  attached: number;
  tenGods: { name: string; count: number }[];
}

export interface BasicPattern {
  name: string;
  tenGod: string;
  reason: string;
}

export interface BasicUseGod {
  strength: "強" | "弱";
  most: string;
  useGod: string | null;
  note: string;
}

export interface BasicCheck {
  label: string;
  tone: "good" | "bad" | "note";
  detail: string;
}

export interface BasicStructure {
  season: Season;
  elements: BasicElementRow[];
  same: { main: number; attached: number };
  opposite: { main: number; attached: number };
  tenGodCounts: Record<string, number>;
  pattern: BasicPattern;
  useGod: BasicUseGod;
  checks: BasicCheck[];
  climate: ClimateScore;
}

const ELEMENTS: Element[] = ["木", "火", "土", "金", "水"];
const STEM_ELEMENT: Record<string, Element> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
/** 表 3.4 地支藏遁表（第一位為本氣；亥只藏壬、甲）。 */
const HIDDEN: Record<string, string[]> = {
  子: ["癸"], 丑: ["己", "癸", "辛"], 寅: ["甲", "丙", "戊"], 卯: ["乙"], 辰: ["戊", "乙", "癸"], 巳: ["丙", "庚", "戊"],
  午: ["丁", "己"], 未: ["己", "丁", "乙"], 申: ["庚", "壬", "戊"], 酉: ["辛"], 戌: ["戊", "辛", "丁"], 亥: ["壬", "甲"],
};
const SEASON: Record<string, Season> = {
  寅: "春", 卯: "春", 巳: "夏", 午: "夏", 申: "秋", 酉: "秋", 亥: "冬", 子: "冬", 辰: "季", 未: "季", 戌: "季", 丑: "季",
};
const SEASON_LEAD: Record<Season, Element> = { 春: "木", 夏: "火", 秋: "金", 冬: "水", 季: "土" };
/** 表 3.1 規律：當令者旺、我生者相、生我者休、剋我者囚、我剋者死。 */
const STATE_BY_OFFSET: WangState[] = ["旺", "相", "死", "囚", "休"];
const STATE_RANK: Record<WangState, number> = { 旺: 5, 相: 4, 休: 3, 囚: 2, 死: 1 };
const RELATIONS: Relation[] = ["日主", "食傷", "財", "官殺", "印"];
const TEN_GOD_PAIRS: [string, string][] = [
  ["比肩", "劫財"],
  ["食神", "傷官"],
  ["偏財", "正財"],
  ["七殺", "正官"],
  ["偏印", "正印"],
];
/** 表 7.4：祿與陽刃（陰干無刃；戊午有爭議，不列入劫刃格）。 */
const LU: Record<string, string> = { 甲: "寅", 乙: "卯", 丙: "巳", 丁: "午", 戊: "巳", 己: "午", 庚: "申", 辛: "酉", 壬: "亥", 癸: "子" };
const YANG_REN: Record<string, string> = { 甲: "卯", 丙: "午", 庚: "酉", 壬: "子" };
const CATEGORY: Record<string, string> = {
  比肩: "比劫", 劫財: "比劫", 食神: "食傷", 傷官: "食傷", 偏財: "財星", 正財: "財星", 七殺: "官殺", 正官: "官殺", 偏印: "印星", 正印: "印星",
};

/** 表 7.1 用神選取表（p137，參考陳柏諭《四柱八字闡微與實務》）。 */
const USE_GOD_TABLE: { group: string; strength: "強" | "弱"; most: string[]; useGod: string }[] = [
  { group: "正官", strength: "弱", most: ["財星"], useGod: "比劫、印星" },
  { group: "正官", strength: "弱", most: ["食傷"], useGod: "印星" },
  { group: "正官", strength: "弱", most: ["官殺"], useGod: "印星" },
  { group: "正官", strength: "強", most: ["比劫", "印星"], useGod: "官星" },
  { group: "正官", strength: "強", most: ["印星"], useGod: "財星" },
  { group: "正官", strength: "強", most: ["食傷"], useGod: "財星" },
  { group: "七殺", strength: "弱", most: ["財星"], useGod: "比劫" },
  { group: "七殺", strength: "弱", most: ["食傷"], useGod: "印星" },
  { group: "七殺", strength: "弱", most: ["官殺"], useGod: "印星" },
  { group: "七殺", strength: "強", most: ["比劫"], useGod: "七殺" },
  { group: "七殺", strength: "強", most: ["印星"], useGod: "財星" },
  { group: "七殺", strength: "強", most: ["官殺"], useGod: "食傷" },
  { group: "財星", strength: "弱", most: ["食傷"], useGod: "印星" },
  { group: "財星", strength: "弱", most: ["財星"], useGod: "比劫" },
  { group: "財星", strength: "弱", most: ["官殺"], useGod: "印星" },
  { group: "財星", strength: "強", most: ["比劫"], useGod: "食傷、官殺" },
  { group: "財星", strength: "強", most: ["印星"], useGod: "財星" },
  { group: "印星", strength: "弱", most: ["官殺"], useGod: "印星" },
  { group: "印星", strength: "弱", most: ["食傷"], useGod: "印星" },
  { group: "印星", strength: "弱", most: ["財星"], useGod: "比劫" },
  { group: "印星", strength: "強", most: ["比劫"], useGod: "官殺、食傷" },
  { group: "印星", strength: "強", most: ["印星"], useGod: "財星" },
  { group: "印星", strength: "強", most: ["財星"], useGod: "食傷、財星" },
  { group: "食神", strength: "弱", most: ["官殺"], useGod: "印星" },
  { group: "食神", strength: "弱", most: ["財星"], useGod: "比劫" },
  { group: "食神", strength: "弱", most: ["食傷"], useGod: "印星" },
  { group: "食神", strength: "強", most: ["印星"], useGod: "財星" },
  { group: "食神", strength: "強", most: ["比劫"], useGod: "食神" },
  { group: "食神", strength: "強", most: ["財星"], useGod: "官殺" },
  { group: "傷官", strength: "弱", most: ["財星"], useGod: "比劫" },
  { group: "傷官", strength: "弱", most: ["官殺"], useGod: "印星" },
  { group: "傷官", strength: "弱", most: ["食傷"], useGod: "印星" },
  { group: "傷官", strength: "強", most: ["比劫"], useGod: "官殺" },
  { group: "傷官", strength: "強", most: ["印星"], useGod: "財星" },
];

/** 表 7.2「善」格所忌、表 7.3「不善」格所喜（p141）。 */
const GOOD_PATTERN_AVOID: Record<string, { gods: string[]; text: string }> = {
  正官格: { gods: ["傷官"], text: "傷官見官，用之為官不可傷" },
  正財格: { gods: ["比肩", "劫財"], text: "比劫奪財，用之為財不可劫" },
  偏財格: { gods: ["比肩", "劫財"], text: "比劫奪財，用之為財不可劫" },
  正印格: { gods: ["正財", "偏財"], text: "財星壞印，用之為印不可破" },
  食神格: { gods: ["偏印"], text: "梟神奪食，用為食神不可梟" },
};
const BAD_PATTERN_WANT: Record<string, { gods: string[]; text: string }> = {
  七殺格: { gods: ["食神"], text: "食神制殺" },
  傷官格: { gods: ["正印", "偏印"], text: "傷官佩印" },
  偏印格: { gods: ["正財", "偏財"], text: "以財制梟" },
  劫刃格: { gods: ["七殺"], text: "七殺制刃" },
};

const stemOf = (ganZhi: string) => ganZhi.charAt(0);
const branchOf = (ganZhi: string) => ganZhi.charAt(1);

export function wangState(element: Element, monthBranch: string): WangState {
  const lead = SEASON_LEAD[SEASON[monthBranch] ?? "季"];
  return STATE_BY_OFFSET[(ELEMENTS.indexOf(element) - ELEMENTS.indexOf(lead) + 5) % 5];
}

/** 十神計分：天干（不含日干）＋全部藏干，各算 1（附錄步驟 16）。 */
export function tenGodCounts(natal: string[]): Record<string, number> {
  const dayStem = stemOf(natal[2]);
  const counts: Record<string, number> = {};
  const add = (stem: string) => {
    const god = tenGod(dayStem, stem);
    counts[god] = (counts[god] ?? 0) + 1;
  };
  natal.forEach((pillar, index) => {
    if (index !== 2) add(stemOf(pillar));
    (HIDDEN[branchOf(pillar)] ?? []).forEach(add);
  });
  return counts;
}

/** 依主導勢力定格局（附錄步驟 17；第六章 p117–122、第七章 p143–144）。 */
export function dominantPattern(natal: string[]): BasicPattern {
  const dayStem = stemOf(natal[2]);
  const month = branchOf(natal[1]);
  if (LU[dayStem] === month) {
    return { name: "建祿格", tenGod: "比肩", reason: `月令${month}是日干${dayStem}的祿（表 7.4）。` };
  }
  if (YANG_REN[dayStem] === month) {
    return { name: "劫刃格", tenGod: "劫財", reason: `陽干${dayStem}生於陽刃月${month}（表 7.4）。` };
  }
  const counts = tenGodCounts(natal);
  const dayElement = STEM_ELEMENT[dayStem];
  const visible = new Set(natal.filter((_, index) => index !== 2).map((pillar) => tenGod(dayStem, stemOf(pillar))));
  const candidates = Object.entries(counts).filter(([god]) => CATEGORY[god] !== "比劫");
  if (!candidates.length) {
    return { name: "（無）", tenGod: "", reason: "除比劫外沒有其他十神。" };
  }
  const max = Math.max(...candidates.map(([, count]) => count));
  const monthGod = tenGod(dayStem, HIDDEN[month][0]);
  if (CATEGORY[monthGod] !== "比劫") {
    // 月令本氣與同五行的另一個十神合看（正偏同類），計分接近最多者時以月令為格，取其中計分較高者。
    const partner = TEN_GOD_PAIRS.find((pair) => pair.includes(monthGod))!.find((god) => god !== monthGod)!;
    const own = counts[monthGod] ?? 0;
    const other = counts[partner] ?? 0;
    if (own >= max - 1 || own + other >= max) {
      const chosen = other > own ? partner : monthGod;
      return {
        name: `${chosen}格`,
        tenGod: chosen,
        reason: `月令${month}本氣${HIDDEN[month][0]}為${monthGod}（${own}）${other ? `，同類${partner}（${other}）` : ""}，${own + other >= max && Math.max(own, other) >= max ? "計分最多" : `與最多者（${max}）相當`}，以月令${chosen === monthGod ? "" : "同類中較多的" + chosen}為格。`,
      };
    }
  }
  const elementOfGod = (god: string) => {
    const offset = TEN_GOD_PAIRS.findIndex((pair) => pair.includes(god));
    return ELEMENTS[(ELEMENTS.indexOf(dayElement) + offset) % 5];
  };
  const best = candidates
    .filter(([, count]) => count === max)
    .sort(
      ([a], [b]) =>
        STATE_RANK[wangState(elementOfGod(b), month)] - STATE_RANK[wangState(elementOfGod(a), month)] ||
        Number(visible.has(b)) - Number(visible.has(a)),
    )[0][0];
  const tied = candidates.filter(([, count]) => count === max).length > 1;
  return {
    name: `${best}格`,
    tenGod: best,
    reason: `${best}計分 ${max} 最多${tied ? `（同分時取${wangState(elementOfGod(best), month)}令者）` : ""}，月令本氣${monthGod}只有 ${counts[monthGod] ?? 0}，以主導勢力為格。`,
  };
}

function patternGroup(name: string): string {
  if (name === "正財格" || name === "偏財格") return "財星";
  if (name === "正印格" || name === "偏印格") return "印星";
  return name.replace("格", "");
}

function lookupUseGod(pattern: BasicPattern, strength: "強" | "弱", counts: Record<string, number>): BasicUseGod {
  const categoryCounts: Record<string, number> = { 比劫: 0, 印星: 0, 食傷: 0, 財星: 0, 官殺: 0 };
  Object.entries(counts).forEach(([god, count]) => {
    categoryCounts[CATEGORY[god]] += count;
  });
  const ranked = Object.entries(categoryCounts).sort((a, b) => b[1] - a[1]);
  const most = ranked[0][0];
  if (pattern.name === "建祿格" || pattern.name === "劫刃格") {
    return {
      strength,
      most,
      useGod: null,
      note: "建祿、劫刃格一般身強，用神從異方找，優先「食傷生財」或「財滋官殺」；劫刃格不喜單獨用財，可用七殺制刃（p143–148）。",
    };
  }
  const group = patternGroup(pattern.name);
  const tops = ranked.filter(([, count]) => count === ranked[0][1]).map(([category]) => category);
  for (const category of tops) {
    const row = USE_GOD_TABLE.find((item) => item.group === group && item.strength === strength && item.most.includes(category));
    if (row) {
      return {
        strength,
        most: category,
        useGod: row.useGod,
        note: `表 7.1：${group}格、日主${strength}、命局${category}多 → 取${row.useGod}。`,
      };
    }
  }
  return {
    strength,
    most,
    useGod: null,
    note: `表 7.1 沒有「${group}格、日主${strength}、命局${tops.join("／")}多」這一列，請依扶抑另取。`,
  };
}

function patternChecks(natal: string[], pattern: BasicPattern): BasicCheck[] {
  const dayStem = stemOf(natal[2]);
  const visibleGods = natal.filter((_, index) => index !== 2).map((pillar) => tenGod(dayStem, stemOf(pillar)));
  const hiddenGods = natal.flatMap((pillar) => (HIDDEN[branchOf(pillar)] ?? []).map((stem) => tenGod(dayStem, stem)));
  const checks: BasicCheck[] = [];
  const avoid = GOOD_PATTERN_AVOID[pattern.name];
  if (avoid) {
    const seen = avoid.gods.filter((god) => visibleGods.includes(god));
    const hidden = avoid.gods.filter((god) => !seen.includes(god) && hiddenGods.includes(god));
    checks.push(
      seen.length
        ? { label: `破格：見${seen.join("、")}`, tone: "bad", detail: `「善」格宜順用，${pattern.name}忌${avoid.gods.join("、")}（${avoid.text}，表 7.2）。天干透出${seen.join("、")}。` }
        : {
            label: hidden.length ? `${avoid.gods.join("、")}藏而未透` : `未見${avoid.gods.join("、")}`,
            tone: hidden.length ? "note" : "good",
            detail: `${pattern.name}忌${avoid.gods.join("、")}（表 7.2）。${hidden.length ? "只藏在地支，運歲引出時要留意。" : "原局沒有，行運再見才要留意。"}`,
          },
    );
  }
  const want = BAD_PATTERN_WANT[pattern.name];
  if (want) {
    const seen = want.gods.filter((god) => visibleGods.includes(god));
    checks.push(
      seen.length
        ? { label: `成格：${want.text}`, tone: "good", detail: `「不善」格宜逆用，${pattern.name}喜${want.gods.join("、")}（表 7.3）。天干透出${seen.join("、")}。` }
        : { label: `未見${want.gods.join("、")}`, tone: "note", detail: `${pattern.name}喜${want.gods.join("、")}來${want.text}（表 7.3）；原局天干沒有，可看運歲是否補上。` },
    );
  }
  if ((pattern.name === "正官格" || pattern.name === "七殺格") && visibleGods.includes("正官") && visibleGods.includes("七殺")) {
    checks.push({ label: "官殺混雜", tone: "bad", detail: "正官、七殺同透天干，格局不清（p148–151）；有合殺或合官者可解。" });
  }
  const single = { 正官格: "正官", 七殺格: "七殺", 食神格: "食神" }[pattern.name as "正官格" | "七殺格" | "食神格"];
  if (single && visibleGods.filter((god) => god === single).length >= 2) {
    checks.push({ label: `${single}重見`, tone: "bad", detail: `${single}只宜一位，重疊多見反而不美（p148–157）。` });
  }
  if (pattern.name === "七殺格" && visibleGods.includes("食神") && visibleGods.includes("偏印")) {
    checks.push({ label: "食神制殺逢梟", tone: "bad", detail: "食神制殺而又透偏印奪食，制殺失效，書云「不貧則夭」（p150–151）。" });
  }
  if (pattern.name === "傷官格" && visibleGods.includes("正官")) {
    const metalWater = "庚辛".includes(dayStem) && "亥子丑".includes(branchOf(natal[1]));
    checks.push(
      metalWater
        ? { label: "金水傷官見官", tone: "good", detail: "冬月金水傷官，調候為急，喜見官（火），不作破格論（p155–156）。" }
        : { label: "傷官見官", tone: "bad", detail: "傷官格忌見正官，除非傷盡或金水傷官（p155–156）。" },
    );
  }
  return checks;
}

export function basicStructure(natal: string[], verdict?: string): BasicStructure {
  const dayStem = stemOf(natal[2]);
  const dayElement = STEM_ELEMENT[dayStem];
  const month = branchOf(natal[1]);
  const counts = tenGodCounts(natal);
  const elements: BasicElementRow[] = RELATIONS.map((relation, offset) => {
    const element = ELEMENTS[(ELEMENTS.indexOf(dayElement) + offset) % 5];
    let main = 0;
    let attached = 0;
    natal.forEach((pillar) => {
      if (STEM_ELEMENT[stemOf(pillar)] === element) main += 1;
      (HIDDEN[branchOf(pillar)] ?? []).forEach((stem, layer) => {
        if (STEM_ELEMENT[stem] !== element) return;
        if (layer === 0) main += 1;
        else attached += 1;
      });
    });
    return {
      element,
      relation,
      party: relation === "日主" || relation === "印" ? "同方" : "異方",
      state: wangState(element, month),
      main,
      attached,
      tenGods: TEN_GOD_PAIRS[offset].map((name) => ({ name, count: counts[name] ?? 0 })),
    };
  });
  const sum = (party: "同方" | "異方") =>
    elements.filter((row) => row.party === party).reduce((acc, row) => ({ main: acc.main + row.main, attached: acc.attached + row.attached }), { main: 0, attached: 0 });
  const same = sum("同方");
  const opposite = sum("異方");
  const strength: "強" | "弱" = verdict ? (verdict.includes("強") ? "強" : "弱") : same.main >= opposite.main ? "強" : "弱";
  const pattern = dominantPattern(natal);
  return {
    season: SEASON[month] ?? "季",
    elements,
    same,
    opposite,
    tenGodCounts: counts,
    pattern,
    useGod: lookupUseGod(pattern, strength, counts),
    checks: patternChecks(natal, pattern),
    climate: climateScore(natal),
  };
}

/** 運歲天干對格局的成敗（表 7.2、7.3；杜月笙晚年傷官運破官格，p140）。 */
export function patternLuckEvents(incoming: string, who: "運" | "歲", natal: string[], pattern: string): LuckEvent[] {
  if (natal.length !== 4 || incoming.length < 2 || !pattern) return [];
  const god = tenGod(stemOf(natal[2]), stemOf(incoming));
  const whoName = who === "運" ? "大運" : "流年";
  const events: LuckEvent[] = [];
  const avoid = GOOD_PATTERN_AVOID[pattern];
  if (avoid?.gods.includes(god)) {
    events.push({
      kind: "pattern-break",
      label: "破格",
      level: "warn",
      detail: `${whoName}天干${stemOf(incoming)}是${god}，${pattern}忌之（${avoid.text}，基礎 表 7.2）。杜月笙正官格晚年走傷官運即破格（p140）。`,
    });
  }
  const want = BAD_PATTERN_WANT[pattern];
  if (want?.gods.includes(god)) {
    events.push({
      kind: "pattern-made",
      label: "成格",
      level: "info",
      detail: `${whoName}天干${stemOf(incoming)}是${god}，${pattern}喜之（${want.text}，基礎 表 7.3）。`,
    });
  }
  if ((pattern === "正官格" && god === "七殺") || (pattern === "七殺格" && god === "正官")) {
    events.push({
      kind: "pattern-mixed",
      label: "官殺混雜",
      level: "warn",
      detail: `${whoName}天干${stemOf(incoming)}帶來${god}，與格局的${pattern === "正官格" ? "正官" : "七殺"}混雜；杜月笙辛酉運七殺混官（基礎 p205–206）。`,
    });
  }
  return events;
}
