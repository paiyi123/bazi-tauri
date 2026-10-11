/**
 * 大運、流年的「事件提示」，依陸致極《八字命理學進階教程》《動態分析教程》整理。
 *
 * 這些提示只標出書中認為「容易有事」的干支組合（沖提、歲運並臨、天剋地沖……），
 * 不改變五行力分數，也不判斷吉凶好壞；是否應驗要配合喜忌與原局再看。
 */

export type LuckEventLevel = "warn" | "info";

export interface LuckEvent {
  kind: string;
  label: string;
  level: LuckEventLevel;
  detail: string;
  /** 婚姻、財運、健康類與十神動神的提示另外上色。 */
  topic?: "marriage" | "wealth" | "health" | "move";
}

export interface LuckEventContext {
  /** 原局四柱干支，依年、月、日、時。 */
  natal: string[];
  /** 陸致極強弱檢核的結論（強／弱），用於五指傘凶年等需要強弱的規則。 */
  strength?: string;
  /** 日主在原局是否無根。 */
  rootless?: boolean;
}

const PILLAR_NAMES = ["年", "月", "日", "時"];
const STEM_ELEMENT: Record<string, string> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
const HIDDEN: Record<string, string[]> = {
  子: ["癸"], 丑: ["己", "癸", "辛"], 寅: ["甲", "丙", "戊"], 卯: ["乙"], 辰: ["戊", "乙", "癸"], 巳: ["丙", "戊", "庚"],
  午: ["丁", "己"], 未: ["己", "丁", "乙"], 申: ["庚", "壬", "戊"], 酉: ["辛"], 戌: ["戊", "辛", "丁"], 亥: ["壬", "甲"],
};
const CONTROLS: Record<string, string> = { 木: "土", 土: "水", 水: "火", 火: "金", 金: "木" };
const GENERATED_BY: Record<string, string> = { 木: "水", 火: "木", 土: "火", 金: "土", 水: "金" };
const CLASH_PAIRS = ["子午", "丑未", "寅申", "卯酉", "辰戌", "巳亥"];
const THREE_PUNISHMENTS = ["寅巳申", "丑戌未"];
const LU: Record<string, string> = { 甲: "寅", 乙: "卯", 丙: "巳", 丁: "午", 戊: "巳", 己: "午", 庚: "申", 辛: "酉", 壬: "亥", 癸: "子" };
const REN: Record<string, string> = { 甲: "卯", 丙: "午", 戊: "午", 庚: "酉", 壬: "子" };
const STORAGE = "辰戌丑未";

const stemOf = (ganZhi: string) => ganZhi.charAt(0);
const branchOf = (ganZhi: string) => ganZhi.charAt(1);

export function isClash(a: string, b: string): boolean {
  return CLASH_PAIRS.some((pair) => pair === a + b || pair === b + a);
}

function stemControls(a: string, b: string): boolean {
  const left = STEM_ELEMENT[a];
  const right = STEM_ELEMENT[b];
  return Boolean(left && right && CONTROLS[left] === right);
}

/** 天剋地沖：天干相剋（任一方向）且地支相沖。 */
export function isHeavenEarthClash(a: string, b: string): boolean {
  const stemFight = stemControls(stemOf(a), stemOf(b)) || stemControls(stemOf(b), stemOf(a));
  return stemFight && isClash(branchOf(a), branchOf(b));
}

function mainElement(branch: string): string | undefined {
  const first = HIDDEN[branch]?.[0];
  return first ? STEM_ELEMENT[first] : undefined;
}

/** 運歲帶進來的干支（大運、流年）共用的規則。 */
function commonEvents(incoming: string, who: "運" | "歲", ctx: LuckEventContext): LuckEvent[] {
  const events: LuckEvent[] = [];
  const natal = ctx.natal;
  if (natal.length !== 4) return events;
  const branch = branchOf(incoming);
  const dayStem = stemOf(natal[2]);
  const whoName = who === "運" ? "大運" : "流年";

  if (isClash(branch, branchOf(natal[1]))) {
    events.push({
      kind: "clash-month",
      label: "沖提",
      level: "warn",
      detail: `${whoName}${branch}沖月令${branchOf(natal[1])}。月令提綱如堤壩，被沖垮則變動大；作者指出第六步大運必是沖提，喜用在月柱者要特別留意（進階 p113、p276）。`,
    });
  }

  natal.forEach((pillar, index) => {
    if (isHeavenEarthClash(incoming, pillar)) {
      events.push({
        kind: `fan-yin-${index}`,
        label: `反吟(${PILLAR_NAMES[index]})`,
        level: index === 2 ? "warn" : "info",
        detail: `${whoName}${incoming}與${PILLAR_NAMES[index]}柱${pillar}天剋地沖。${
          index === 2 ? "沖到日柱（自身與配偶宮），作者視為較重的反吟（動態 p335–336）。" : "反吟主該柱所代表的人事變動（動態 p335–336）。"
        }`,
      });
    } else if (incoming === pillar && index === 2) {
      events.push({
        kind: "fu-yin-day",
        label: "伏吟(日)",
        level: "info",
        detail: `${whoName}${incoming}與日柱相同（伏吟），日主與配偶宮被重複引動（動態 第十章）。`,
      });
    }
  });

  const lu = LU[dayStem];
  const luIndex = natal.findIndex((pillar) => branchOf(pillar) === lu);
  if (lu && luIndex >= 0 && isClash(branch, lu)) {
    events.push({
      kind: "clash-lu",
      label: "沖祿",
      level: "warn",
      detail: `${whoName}${branch}沖${PILLAR_NAMES[luIndex]}支的日祿${lu}。祿為日主之身，日主根弱時祿被沖最危險；即使是用神運，祿被沖仍可能為凶（進階 p113、命例 168）。`,
    });
  }

  natal.forEach((pillar, index) => {
    const natalBranch = branchOf(pillar);
    if (STORAGE.includes(natalBranch) && isClash(branch, natalBranch) && !(lu && natalBranch === lu)) {
      events.push({
        kind: `open-storage-${index}`,
        label: `沖庫(${PILLAR_NAMES[index]})`,
        level: "info",
        detail: `${whoName}${branch}沖開${PILLAR_NAMES[index]}支${natalBranch}庫。墓需沖才開；土金坐墓喜沖，木火水坐墓忌沖，沖太過則破庫（進階 p101–103）。`,
      });
    }
  });

  const ren = REN[dayStem];
  const renIndex = natal.findIndex((pillar) => branchOf(pillar) === ren);
  if (ren && renIndex >= 0 && isClash(branch, ren)) {
    events.push({
      kind: "clash-ren",
      label: "沖刃",
      level: "warn",
      detail: `${whoName}${branch}沖${PILLAR_NAMES[renIndex]}支的陽刃${ren}。《繼善篇》「陽刃沖合歲君，悖然禍至」，身旺而刃為忌者尤其要留意（進階 第十二章）。`,
    });
  }

  if (ctx.rootless && mainElement(branch) === STEM_ELEMENT[dayStem]) {
    events.push({
      kind: "rootless-meets-root",
      label: "無根見根",
      level: "info",
      detail: `原局日主無根，${whoName}${branch}帶來強根（本氣同類）。作者提醒無根者運歲見根，反而可能招剋（動態 p187，命例 54）。`,
    });
  }
  return events;
}

/** 多個地支湊成三沖一、三刑時，需由運歲帶進至少一字。 */
function groupEvents(natalBranches: string[], incoming: string[], whoName: string): LuckEvent[] {
  const events: LuckEvent[] = [];
  const all = [...natalBranches, ...incoming];
  const count = (branch: string) => all.filter((item) => item === branch).length;
  for (const pair of CLASH_PAIRS) {
    const [a, b] = [pair.charAt(0), pair.charAt(1)];
    for (const [many, one] of [[a, b], [b, a]]) {
      if (count(many) >= 3 && count(one) >= 1 && incoming.some((item) => item === many || item === one)) {
        events.push({
          kind: `three-clash-${many}${one}`,
          label: "三沖一",
          level: "warn",
          detail: `${whoName}使地支出現三個${many}沖一個${one}。身弱而根藏於被沖之支時，三沖一可以拔根（進階 p289）。`,
        });
      }
    }
  }
  for (const group of THREE_PUNISHMENTS) {
    const members = group.split("");
    const complete = members.every((member) => all.includes(member));
    const natalComplete = members.every((member) => natalBranches.includes(member));
    if (complete && !natalComplete && incoming.some((item) => members.includes(item))) {
      events.push({
        kind: `three-punish-${group}`,
        label: "三刑",
        level: "warn",
        detail: `${whoName}湊成${group}三刑。朱熹庚申年即命、運、歲構成寅巳申連環三刑；作者也認為對六親星與宮位，刑比沖更凶（進階 第一、五章）。`,
      });
    }
  }
  return events;
}

/** 大運本身的提示。 */
export function majorLuckEvents(luck: string, ctx: LuckEventContext): LuckEvent[] {
  if (ctx.natal.length !== 4 || luck.length < 2) return [];
  return [
    ...commonEvents(luck, "運", ctx),
    ...groupEvents(ctx.natal.map(branchOf), [branchOf(luck)], "大運"),
  ];
}

/** 流年在所屬大運之下的提示。 */
export function annualEvents(annual: string, luck: string, ctx: LuckEventContext): LuckEvent[] {
  if (ctx.natal.length !== 4 || annual.length < 2) return [];
  const events: LuckEvent[] = [];
  const dayStem = stemOf(ctx.natal[2]);

  if (luck && annual === luck) {
    events.push({
      kind: "sui-yun-bing-lin",
      label: "歲運並臨",
      level: "warn",
      detail: `流年${annual}與大運相同。歲運並臨（伏吟），該干支的力量加倍；書中三毛在己未運己未年喪夫，也有多個命例在此時出事（進階 第四章、p276）。`,
    });
  } else if (luck && isHeavenEarthClash(annual, luck)) {
    events.push({
      kind: "sui-yun-jiao-zhan",
      label: "歲運交戰",
      level: "warn",
      detail: `流年${annual}與大運${luck}天剋地沖。歲運交戰（反吟），流年先和大運作用，君臣相沖，書中多例的事故時機（進階 第四章、p276）。`,
    });
  }

  if (stemControls(dayStem, stemOf(annual))) {
    events.push({
      kind: "day-offends-year",
      label: "日犯歲君",
      level: "info",
      detail: `日干${dayStem}剋流年天干${stemOf(annual)}。太歲剋日主即使有災也不是大害；日主剋太歲可能引起大災，叫「日犯歲君」（進階 第四章）。`,
    });
  }

  events.push(...commonEvents(annual, "歲", ctx));
  const incoming = [branchOf(annual)];
  const natalAndLuck = luck ? [...ctx.natal.map(branchOf), branchOf(luck)] : ctx.natal.map(branchOf);
  events.push(...groupEvents(natalAndLuck, incoming, "流年"));

  // 表 6.1：陽干日主祿、刃、印散見於柱、運、歲，身強者為凶年（動態 p180–181）。
  const ren = REN[dayStem];
  if (ctx.strength === "強" && ren) {
    const dayElement = STEM_ELEMENT[dayStem];
    const sealElement = GENERATED_BY[dayElement];
    const pillars = luck ? [...ctx.natal, luck, annual] : [...ctx.natal, annual];
    const branches = pillars.map(branchOf);
    const hasSeal = pillars.some(
      (pillar) => STEM_ELEMENT[stemOf(pillar)] === sealElement || mainElement(branchOf(pillar)) === sealElement,
    );
    const annualAdds =
      branchOf(annual) === LU[dayStem] ||
      branchOf(annual) === ren ||
      STEM_ELEMENT[stemOf(annual)] === sealElement ||
      mainElement(branchOf(annual)) === sealElement;
    if (branches.includes(LU[dayStem]) && branches.includes(ren) && hasSeal && annualAdds) {
      events.push({
        kind: "lu-ren-seal",
        label: "祿刃印全",
        level: "warn",
        detail: `祿${LU[dayStem]}、刃${ren}、印在原局、大運、流年中俱全。身強者逢之為「五指傘」凶年（動態 p179–181，表 6.1）。`,
      });
    }
  }
  return dedupe(events);
}

function dedupe(events: LuckEvent[]): LuckEvent[] {
  const seen = new Set<string>();
  return events.filter((event) => {
    const key = `${event.kind}-${event.label}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
