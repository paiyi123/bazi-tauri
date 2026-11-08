/**
 * 四本書的可計算規則（2026-10-30 整理進 App）：
 * - 鄭偉雄《子平正源——轉盤論法》：轉盤（以某位六親那個字為新日主重排十神）、合而解沖、原局沒有的五行在大運出現。
 * - 覺慧居士《八字心訣》：借盤優先位置、旺衰三選二、流通訣（同柱干支、鄰干、鄰支才相生）、
 *   清濁三法、寒暖例外（暖無氣反以無暖為美）、五合三種化法與暗五合、四正四庫齊全。
 * - 林信銘《妙算人生八字》：用神四品質（有力、有情、真假、虛實）、戀愛方式（日支藏干透在哪一柱）、
 *   限運、蓋頭截腳、大運×流年喜忌組合。
 * - 錢思吾《輕鬆學算命》：不同日干先看哪一種取用法、丑辰濕土吸亢陽、未戌燥土暖寒濕。
 *
 * 全部只是「提示」：不改五行力分數、不改課堂三步法的強弱與喜忌。
 * 喜忌一律用課堂三步法的結果（lessonPreference.ts 合併後的喜／忌），用神用 yongShen.ts 的用神五行。
 * 書沒有給分數的地方，App 的簡化都寫在各函式的註解裡。
 */

import type { LuckEvent } from "./luckEvents";

export type Element = "木" | "火" | "土" | "金" | "水";
export type Favor = "good" | "bad" | "mixed" | "neutral";
export type CheckStatus = "pass" | "fail" | "partial";

export interface BookCheck {
  label: string;
  status: CheckStatus;
  detail: string;
}

const STEMS = "甲乙丙丁戊己庚辛壬癸";
const STEM_ELEMENT: Record<string, Element> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
/** 藏干，第一個是本氣。 */
const HIDDEN: Record<string, string[]> = {
  子: ["癸"], 丑: ["己", "癸", "辛"], 寅: ["甲", "丙", "戊"], 卯: ["乙"], 辰: ["戊", "乙", "癸"], 巳: ["丙", "戊", "庚"],
  午: ["丁", "己"], 未: ["己", "丁", "乙"], 申: ["庚", "壬", "戊"], 酉: ["辛"], 戌: ["戊", "辛", "丁"], 亥: ["壬", "甲"],
};
const GENERATES: Record<Element, Element> = { 木: "火", 火: "土", 土: "金", 金: "水", 水: "木" };
const CONTROLS: Record<Element, Element> = { 木: "土", 土: "水", 水: "火", 火: "金", 金: "木" };
const ELEMENTS: Element[] = ["木", "火", "土", "金", "水"];
const PILLARS = ["年", "月", "日", "時"];
const CLASH_PAIRS = ["子午", "丑未", "寅申", "卯酉", "辰戌", "巳亥"];
const SIX_COMBINE = ["子丑", "寅亥", "卯戌", "辰酉", "巳申", "午未"];
const STEM_COMBINE = ["甲己", "乙庚", "丙辛", "丁壬", "戊癸"];

const stemOf = (ganZhi: string) => ganZhi.charAt(0);
const branchOf = (ganZhi: string) => ganZhi.charAt(1);
const isYang = (stem: string) => STEMS.indexOf(stem) % 2 === 0;
const pairIn = (list: string[], a: string, b: string) => list.some((pair) => pair === a + b || pair === b + a);
const isClash = (a: string, b: string) => pairIn(CLASH_PAIRS, a, b);
const isSixCombine = (a: string, b: string) => pairIn(SIX_COMBINE, a, b);
const isStemCombine = (a: string, b: string) => pairIn(STEM_COMBINE, a, b);
const mainStem = (branch: string) => HIDDEN[branch]?.[0] ?? "";
const mainElement = (branch: string): Element | undefined => STEM_ELEMENT[mainStem(branch)];
const controllerOf = (element: Element) => ELEMENTS.find((item) => CONTROLS[item] === element) as Element;

/** 十神名稱（以 master 為日主）。 */
export function tenGodOf(master: string, stem: string): string {
  const me = STEM_ELEMENT[master];
  const other = STEM_ELEMENT[stem];
  if (!me || !other) return "";
  const same = isYang(master) === isYang(stem);
  if (me === other) return same ? "比肩" : "劫財";
  if (GENERATES[other] === me) return same ? "偏印" : "正印";
  if (GENERATES[me] === other) return same ? "食神" : "傷官";
  if (CONTROLS[me] === other) return same ? "偏財" : "正財";
  return same ? "七殺" : "正官";
}

export function familyOf(me: Element, other: Element): "比劫" | "印" | "食傷" | "財" | "官殺" {
  if (me === other) return "比劫";
  if (GENERATES[other] === me) return "印";
  if (GENERATES[me] === other) return "食傷";
  if (CONTROLS[me] === other) return "財";
  return "官殺";
}

/** 旺相休囚死：月令本氣五行為旺，旺所生為相（辰戌丑未月以土為旺）。 */
function seasonState(element: Element, monthBranch: string): "旺" | "相" | "休" | "囚" | "死" {
  const lead = mainElement(monthBranch) as Element;
  const order: Element[] = ["木", "火", "土", "金", "水"];
  return (["旺", "相", "死", "囚", "休"] as const)[(order.indexOf(element) - order.indexOf(lead) + 5) % 5];
}

// ---------------------------------------------------------------------------
// 八個字的位置（天干 0–3、地支 4–7，依年月日時）。

export interface ChartNode {
  index: number;
  pillar: number;
  part: "干" | "支";
  char: string;
  /** 地支用本氣的天干。 */
  stem: string;
  element: Element;
  name: string;
}

function nodesOf(natal: string[]): ChartNode[] {
  const nodes: ChartNode[] = [];
  natal.forEach((pillar, index) => {
    const stem = stemOf(pillar);
    nodes.push({ index, pillar: index, part: "干", char: stem, stem, element: STEM_ELEMENT[stem], name: `${PILLARS[index]}干${stem}` });
  });
  natal.forEach((pillar, index) => {
    const branch = branchOf(pillar);
    nodes.push({
      index: index + 4,
      pillar: index,
      part: "支",
      char: branch,
      stem: mainStem(branch),
      element: mainElement(branch) as Element,
      name: `${PILLARS[index]}支${branch}`,
    });
  });
  return nodes;
}

/** 《八字心訣》第 14、80 訣：同柱干支、相鄰天干、相鄰地支才相通（異柱干支不相生）。 */
function adjacent(a: ChartNode, b: ChartNode): boolean {
  if (a.index === b.index) return false;
  if (a.pillar === b.pillar) return true;
  return a.part === b.part && Math.abs(a.pillar - b.pillar) === 1;
}

const label = (node: ChartNode) => `${node.name}${node.element}`;

// ---------------------------------------------------------------------------
// 《八字心訣》第 15 訣：得令、得地、得勢三選二。

export interface ThreeOfTwo {
  master: string;
  element: Element;
  deLing: boolean;
  deDi: boolean;
  deShi: boolean;
  sameParty: number;
  strong: boolean;
  detail: string;
}

/**
 * 以某個天干為日主做三選二。
 * App 的簡化（書沒有給計數方法）：
 * - 得令：月支本氣是它的印或比劫。
 * - 得地：坐支藏有同五行，或任一地支本氣和它同五行（祿、刃這類強根）。
 * - 得勢：除了它自己，其他七個字（天干＋地支本氣）裡印比過半（至少 4 個）。
 *   第 99 訣「一清到底」丙火（印比 3 個）書說偏弱；尚書命 5 個、寒雖甚而暖無氣的庚金 4 個書說身旺。
 */
export function threeOfTwo(natal: string[], master: string, at: { pillar: number; part: "干" | "支" } = { pillar: 2, part: "干" }): ThreeOfTwo {
  const element = STEM_ELEMENT[master];
  const nodes = nodesOf(natal);
  const monthElement = mainElement(branchOf(natal[1])) as Element;
  const deLing = monthElement === element || GENERATES[monthElement] === element;
  const seat = branchOf(natal[at.pillar]);
  const seatRoot = (HIDDEN[seat] ?? []).some((stem) => STEM_ELEMENT[stem] === element);
  const strongRoot = nodes.some((node) => node.part === "支" && node.element === element && !(at.part === "支" && node.pillar === at.pillar));
  const deDi = seatRoot || strongRoot;
  const self = at.part === "干" ? at.pillar : at.pillar + 4;
  const others = nodes.filter((node) => node.index !== self);
  const sameParty = others.filter((node) => node.element === element || GENERATES[node.element] === element).length;
  const deShi = sameParty >= 4;
  const count = [deLing, deDi, deShi].filter(Boolean).length;
  const strong = count >= 2;
  const parts = [
    `得令${deLing ? "✓" : "✗"}（月令${branchOf(natal[1])}本氣${monthElement}）`,
    `得地${deDi ? "✓" : "✗"}${seatRoot ? `（坐${seat}有根）` : strongRoot ? "（他支本氣同類）" : ""}`,
    `得勢${deShi ? "✓" : "✗"}（其他七字印比 ${sameParty} 個）`,
  ];
  return { master, element, deLing, deDi, deShi, sameParty, strong, detail: `${parts.join("、")} → 得 ${count} 項，${strong ? "身旺" : "身弱"}` };
}

// ---------------------------------------------------------------------------
// 《八字心訣》第 14、80、81 訣：流通路線。

export interface FlowReport {
  /** 依心訣規則能相生的箭頭。 */
  edges: { from: string; to: string }[];
  /** 最長的幾條相生鏈。 */
  chains: string[];
  /** 其他書會算、心訣不算的相生（異柱干支）。 */
  blocked: string[];
  inflow: string | null;
  outflow: string | null;
  strength: ThreeOfTwo;
  check: BookCheck;
}

function longestPaths(nodes: ChartNode[], edges: Map<number, number[]>): number[][] {
  const paths: number[][] = [];
  const walk = (path: number[]) => {
    const next = (edges.get(path[path.length - 1]) ?? []).filter((item) => !path.includes(item));
    if (!next.length) {
      paths.push(path);
      return;
    }
    next.forEach((item) => walk([...path, item]));
  };
  nodes.forEach((node) => walk([node.index]));
  const contains = (big: number[], small: number[]) =>
    big.length > small.length && big.join(",").includes(small.join(","));
  return paths
    .filter((path) => path.length >= 2)
    .filter((path) => !paths.some((other) => contains(other, path)))
    .sort((a, b) => b.length - a.length);
}

export function flowReport(natal: string[]): FlowReport {
  const nodes = nodesOf(natal);
  const edges = new Map<number, number[]>();
  const edgeList: { from: string; to: string }[] = [];
  const blocked: string[] = [];
  nodes.forEach((from) =>
    nodes.forEach((to) => {
      if (from.index === to.index || GENERATES[from.element] !== to.element) return;
      if (adjacent(from, to)) {
        edges.set(from.index, [...(edges.get(from.index) ?? []), to.index]);
        edgeList.push({ from: label(from), to: label(to) });
      } else if (from.part !== to.part && Math.abs(from.pillar - to.pillar) === 1) {
        blocked.push(`${label(from)}→${label(to)}`);
      }
    }),
  );
  const paths = longestPaths(nodes, edges);
  const chains = paths.slice(0, 4).map((path) => path.map((index) => label(nodes[index])).join(" → "));

  const day = 2;
  // 流進日主的最長路線、從日主流出去的最長路線。
  const intoDay = paths
    .map((path) => path.slice(0, path.indexOf(day) + 1))
    .filter((path) => path.length >= 2 && path[path.length - 1] === day)
    .sort((a, b) => b.length - a.length)[0];
  const outOfDay = paths
    .map((path) => path.slice(path.indexOf(day)))
    .filter((path) => path.length >= 2 && path[0] === day)
    .sort((a, b) => b.length - a.length)[0];
  const text = (path?: number[]) => (path ? path.map((index) => label(nodes[index])).join(" → ") : null);
  const strength = threeOfTwo(natal, stemOf(natal[2]));
  const me = strength.element;
  const familyText = (path: number[]) => path.map((index) => familyOf(me, nodes[index].element)).join("→");

  let check: BookCheck;
  if (strength.strong) {
    if (outOfDay) {
      const reach = familyText(outOfDay);
      const stop = reach.includes("官殺") ? "流到官殺而停，主富貴" : reach.includes("財") ? "流到財，主富" : "流到食傷";
      check = { label: "順逆", status: "pass", detail: `身旺喜順流：${text(outOfDay)}（${reach}），${stop}。` };
    } else if (intoDay) {
      check = { label: "順逆", status: "fail", detail: `身旺卻只有逆流進日主（${text(intoDay)}），沒有出口：流錯方向。` };
    } else {
      check = { label: "順逆", status: "fail", detail: "日干和鄰干、坐支都不相生：流不通。" };
    }
  } else if (intoDay) {
    const chain = familyText(intoDay);
    const noble = chain.includes("官殺→印") ? "官生印、印生身，主貴" : chain.includes("財→官殺") ? "財生官、官生印，主富貴" : "印生身";
    check = { label: "順逆", status: "pass", detail: `身弱喜逆流：${text(intoDay)}（${chain}），${noble}。` };
  } else if (outOfDay) {
    check = { label: "順逆", status: "fail", detail: `身弱卻只有順流洩出（${text(outOfDay)}），沒有印比流進日主：流錯方向。` };
  } else {
    check = { label: "順逆", status: "fail", detail: "日干和鄰干、坐支都不相生：流不通。" };
  }
  return { edges: edgeList, chains, blocked, inflow: text(intoDay), outflow: text(outOfDay), strength, check };
}

// ---------------------------------------------------------------------------
// 寒暖燥濕：《八字心訣》第 92 訣＋《輕鬆學算命》濕土燥土。

const STEM_CLIMATE: Record<string, number> = { 甲: 3, 乙: 1, 丙: 6, 丁: 4, 戊: 5, 己: -4, 庚: -1, 辛: -3, 壬: -5, 癸: -6 };
const BRANCH_CLIMATE: Record<string, number> = {
  寅: 3, 卯: 1, 辰: -4, 巳: 5, 午: 6, 未: 3, 申: -2, 酉: -3, 戌: 4, 亥: -5, 子: -6, 丑: -4,
};
const MONTH_CLIMATE: Record<string, number> = {
  寅: 0, 卯: 1, 辰: 2, 巳: 3, 午: 4, 未: 3, 申: 0, 酉: -1, 戌: -2, 亥: -3, 子: -4, 丑: -4,
};

export interface ClimateReport {
  /** 和「婚姻・財運・健康」卡片同一套寒暖燥濕分數（陸致極）。 */
  total: number;
  side: "寒" | "暖" | "中和";
  check: BookCheck;
  /** 濕土、燥土提示（輕鬆學算命）。 */
  soilNotes: string[];
}

/**
 * 寒暖判斷：分數 < -6，或冬月（亥子丑）且分數 < 0 → 寒；分數 > 6，或夏月（巳午未）且分數 > 0 → 暖。
 * 「暖有氣」：有火的天干在地支藏干有根（寅巳午未戌），或地支本氣有火；「寒有根」同理看水（亥子申辰丑）。
 */
export function climateReport(natal: string[]): ClimateReport {
  let total = 0;
  natal.forEach((pillar) => {
    total += (STEM_CLIMATE[stemOf(pillar)] ?? 0) + (BRANCH_CLIMATE[branchOf(pillar)] ?? 0);
  });
  const month = branchOf(natal[1]);
  total += MONTH_CLIMATE[month] ?? 0;
  const cold = total < -6 || ("亥子丑".includes(month) && total < 0);
  const hot = total > 6 || ("巳午未".includes(month) && total > 0);
  const side = cold ? "寒" : hot ? "暖" : "中和";
  const stems = natal.map(stemOf);
  const branches = natal.map(branchOf);
  const hiddenHas = (element: Element) => branches.filter((branch) => HIDDEN[branch].some((stem) => STEM_ELEMENT[stem] === element));

  const relief = (element: Element) => {
    const stemHits = stems.filter((stem) => STEM_ELEMENT[stem] === element);
    const mainHits = branches.filter((branch) => mainElement(branch) === element);
    const roots = hiddenHas(element);
    return { present: stemHits.length + roots.length > 0, rooted: mainHits.length > 0 || (stemHits.length > 0 && roots.length > 0), stemHits, roots };
  };

  const soilNotes: string[] = [];
  let check: BookCheck;
  if (side === "寒") {
    const fire = relief("火");
    if (fire.rooted) {
      check = { label: "寒暖", status: "pass", detail: `寒（${total}），但火有根（${[...fire.stemHits, ...fire.roots].join("")}）：寒雖甚而暖有氣，能調候（心訣第 92 訣）。` };
    } else if (fire.present) {
      check = {
        label: "寒暖",
        status: "partial",
        detail: `寒（${total}），火（${[...fire.stemHits, ...fire.roots].join("")}）沒有根：寒雖甚而暖無氣，書說「反以無暖為美」，順著寒勢、不必硬補火；行運火得根反而不利（心訣第 92 訣）。`,
      };
    } else {
      check = { label: "寒暖", status: "fail", detail: `寒（${total}），局中完全沒有火：過於濕寒（心訣第 99 訣第一法）。` };
    }
    const dry = branches.filter((branch) => "未戌".includes(branch));
    soilNotes.push(
      dry.length
        ? `局中有燥土${dry.join("")}，能溫暖寒濕（輕鬆學算命第一章）。`
        : "局中沒有未、戌燥土；行運見未、戌可暖局（輕鬆學算命第一章）。",
    );
  } else if (side === "暖") {
    const water = relief("水");
    if (water.rooted) {
      check = { label: "寒暖", status: "pass", detail: `暖（${total}），但水有根（${[...water.stemHits, ...water.roots].join("")}）：暖雖至而寒有根，能調候（心訣第 92 訣）。` };
    } else if (water.present) {
      check = {
        label: "寒暖",
        status: "partial",
        detail: `暖（${total}），水（${[...water.stemHits, ...water.roots].join("")}）沒有根：暖雖至而寒無根，書說「反以無寒為美」，順著火勢、不必硬補水；行運水得根反而不利（心訣第 92 訣）。`,
      };
    } else {
      check = { label: "寒暖", status: "fail", detail: `暖（${total}），局中完全沒有水：過於燥熱（心訣第 99 訣第一法）。` };
    }
    const wet = branches.filter((branch) => "丑辰".includes(branch));
    soilNotes.push(
      wet.length
        ? `局中有濕土${wet.join("")}，能吸收亢陽、洩火生金（輕鬆學算命第一章「丑辰吸亢陽」）。`
        : "局中沒有丑、辰濕土；行運見丑、辰可吸收燥熱（輕鬆學算命第一章）。",
    );
  } else {
    check = { label: "寒暖", status: "pass", detail: `寒暖適中（${total}）。` };
  }
  return { total, side, check, soilNotes };
}

// ---------------------------------------------------------------------------
// 用神：被剋、被合（心訣「神枯」）與妙算四品質。

interface UseNode {
  node: ChartNode;
  attackers: ChartNode[];
  helpers: ChartNode[];
  combined: ChartNode[];
  rooted: boolean;
  /** 天干坐支藏有同類（通根）。 */
  seatRooted: boolean;
}

function useNodes(natal: string[], use: Element): UseNode[] {
  const nodes = nodesOf(natal);
  const branches = natal.map(branchOf);
  return nodes
    .filter((node) => node.element === use && node.index !== 2)
    .map((node) => {
      const neighbors = nodes.filter((other) => adjacent(node, other));
      const seat = branches[node.pillar];
      // 天干坐下的地支藏有同類（通根，例如壬坐辰、癸坐丑）也算有援。
      const seatRooted = node.part === "干" && HIDDEN[seat].some((stem) => STEM_ELEMENT[stem] === use);
      return {
        node,
        attackers: neighbors.filter((other) => CONTROLS[other.element] === use),
        helpers: neighbors.filter((other) => GENERATES[other.element] === use || (other.element === use && other.index !== 2)),
        seatRooted,
        combined: neighbors.filter(
          (other) =>
            other.part === node.part &&
            (node.part === "干" ? isStemCombine(node.char, other.char) : isSixCombine(node.char, other.char)),
        ),
        rooted: node.part === "支" || branches.some((branch) => HIDDEN[branch].some((stem) => STEM_ELEMENT[stem] === use)),
      };
    });
}

function hiddenOnly(natal: string[], use: Element): string[] {
  return natal
    .map(branchOf)
    .filter((branch) => mainElement(branch) !== use && HIDDEN[branch].slice(1).some((stem) => STEM_ELEMENT[stem] === use));
}

const nodeText = (items: ChartNode[]) => items.map((item) => `${item.name}`).join("、");

/** 心訣第 99 訣第三法「神枯」：用神被剋無援、被合走，或根本不現。 */
export function useWitherCheck(natal: string[], useElements: Element[]): BookCheck {
  if (!useElements.length) {
    return { label: "神枯", status: "partial", detail: "課堂三步法還沒有定出用神，無法判斷。" };
  }
  const lines: string[] = [];
  let healthy = false;
  useElements.forEach((use) => {
    const items = useNodes(natal, use);
    if (!items.length) {
      const hidden = hiddenOnly(natal, use);
      lines.push(hidden.length ? `用神${use}只藏在${hidden.join("")}的餘氣，虛而無力` : `原局不見用神${use}`);
      return;
    }
    items.forEach((item) => {
      const problems: string[] = [];
      if (item.combined.length) problems.push(`被${nodeText(item.combined)}合住`);
      const supported = item.helpers.length > 0 || item.seatRooted;
      if (item.attackers.length && !supported) problems.push(`被${nodeText(item.attackers)}剋而無援`);
      if (problems.length) {
        lines.push(`${item.node.name}（${use}）${problems.join("、")}`);
      } else {
        healthy = true;
        lines.push(
          `${item.node.name}（${use}）${
            item.attackers.length
              ? `雖受${nodeText(item.attackers)}剋，${item.helpers.length ? `有${nodeText(item.helpers)}生扶` : `坐${branchOf(natal[item.node.pillar])}通根`}`
              : "不受剋合"
          }`,
        );
      }
    });
  });
  return {
    label: "神枯",
    status: healthy ? "pass" : "fail",
    detail: `${lines.join("；")}。${healthy ? "用神不枯。" : "用神枯：例如身弱用印而財星壞印、身旺用食傷而印制食傷（心訣第 99 訣第三法）。"}`,
  };
}

/** 《妙算人生八字》第 245 頁：各日干最「有情」的用神（書上沒有列丙、丁）。 */
const AFFECTION: Record<string, { elements: Element[]; reason: string }> = {
  甲: { elements: ["金"], reason: "甲為棟樑之材，要金來修剪" },
  乙: { elements: ["火"], reason: "乙為花木，要陽光（火）才開" },
  戊: { elements: ["水"], reason: "戊為高山，最喜水潤" },
  己: { elements: ["木"], reason: "己為田園，尤喜甲木疏土、忌水混濁" },
  庚: { elements: ["火"], reason: "庚為刀劍，要火煉" },
  辛: { elements: ["水"], reason: "辛為寶石，要水洗" },
  壬: { elements: ["土", "火"], reason: "壬為河海，喜燥土作堤、丙火映照" },
  癸: { elements: ["木"], reason: "癸為雨露，最喜木來順洩" },
};

export interface UseQuality {
  use: Element;
  checks: BookCheck[];
  summary: string;
}

/** 《妙算人生八字》第六篇第五節：用神有力／有情／真假／虛實。 */
export function useQuality(natal: string[], use: Element): UseQuality {
  const dayStem = stemOf(natal[2]);
  const month = branchOf(natal[1]);
  const items = useNodes(natal, use);
  const stems = items.filter((item) => item.node.part === "干");
  const free = items.filter((item) => !item.combined.length && !(item.attackers.length && !item.helpers.length && !item.seatRooted));
  const state = seasonState(use, month);
  const inMonth = mainElement(month) === use;
  const count = items.length;

  // 有力（p244–245）：在月支或得令；透干或得地得勢；不被剋合。
  // 得地得勢：地支本氣同類、坐支通根，或鄰位有兩個以上生扶（例二十四丁火靠甲、卯）。
  const rootedOrBacked = items.some(
    (item) => item.node.part === "支" || item.seatRooted || item.helpers.length >= 2,
  );
  const seasonal = inMonth || ["旺", "相"].includes(state);
  const exposed = stems.length > 0;
  const freeOk = free.length > 0;
  const force: BookCheck = {
    label: "有力",
    status:
      freeOk && seasonal && (exposed || rootedOrBacked)
        ? "pass"
        : !freeOk || (!seasonal && !rootedOrBacked)
          ? "fail"
          : "partial",
    detail: [
      `在月支或得令（${state}）${seasonal ? "✓" : "✗"}`,
      `透干${exposed ? "✓" : "✗"}`,
      `得地得勢${rootedOrBacked ? "✓" : "✗"}（局中 ${count} 個）`,
      `不被剋合${freeOk ? "✓" : "✗"}`,
    ].join("、"),
  };

  // 有情（p245–246）：扶身的用神（印、比劫）要有一條相生鏈流進日主；
  // 剋洩的用神（食傷、財、官殺）要緊貼日主、不受剋合，而且不能被鄰字洩去轉生它要制的字（例二十六）。
  const me = STEM_ELEMENT[dayStem];
  const family = familyOf(me, use);
  // 扶身鏈：鄰位之間相生，或同類相助（例二十五 庚→壬→癸：金生水、壬水幫癸水）。
  const nodes = nodesOf(natal);
  const reachesDay = items.filter((item) => {
    const seen = new Set([item.node.index]);
    const queue = [item.node];
    while (queue.length) {
      const current = queue.shift() as ChartNode;
      if (current.index === 2) return true;
      nodes
        .filter((next) => !seen.has(next.index) && adjacent(current, next))
        .filter((next) => GENERATES[current.element] === next.element || (current.element === next.element && next.element === me))
        .forEach((next) => {
          seen.add(next.index);
          queue.push(next);
        });
    }
    return false;
  });
  const close = items.filter((item) => [1, 3, 6].includes(item.node.index) && !item.attackers.length && !item.combined.length);
  const target = CONTROLS[use];
  const leaks = items.flatMap((item) =>
    nodes
      .filter(
        (other) =>
          other.index !== 2 &&
          adjacent(item.node, other) &&
          GENERATES[use] === other.element &&
          GENERATES[other.element] === target &&
          nodes.some((node) => node.element === target && node.index !== 2),
      )
      .map((other) => `${item.node.name}被${other.name}洩去、轉生${target}`),
  );
  const nature = AFFECTION[dayStem];
  const natureFit = nature?.elements.includes(use) ?? false;
  const natureText = nature
    ? natureFit
      ? `合日干本性（${nature.reason}）`
      : `日干本性另喜${nature.elements.join("、")}（${nature.reason}）`
    : "書上沒有列丙、丁的本性用神";
  let affection: BookCheck;
  if (family === "印" || family === "比劫") {
    affection = reachesDay.length
      ? { label: "有情", status: "pass", detail: `${nodeText(reachesDay.map((item) => item.node))}順生流進日主；${natureText}` }
      : { label: "有情", status: natureFit ? "partial" : "fail", detail: `用神${use}沒有一條相生鏈流到日主；${natureText}` };
  } else if (leaks.length) {
    affection = { label: "有情", status: "fail", detail: `${leaks.join("；")}：用神反助忌神，無情（例二十六）；${natureText}` };
  } else if (close.length) {
    affection = { label: "有情", status: "pass", detail: `${nodeText(close.map((item) => item.node))}緊貼日主、不受剋合；${natureText}` };
  } else {
    affection = { label: "有情", status: natureFit ? "partial" : "fail", detail: `沒有緊貼日主又不受剋的用神字；${natureText}` };
  }

  const truth: BookCheck = inMonth && stems.length
    ? { label: "真假", status: "pass", detail: `用神${use}秉令在提綱（月支${month}）又透干：真神，一生較貴。` }
    : ["休", "囚", "死"].includes(state)
      ? { label: "真假", status: "fail", detail: `用神${use}在${month}月${state}，退氣失時：假神，書說「用假終為勞碌人」。` }
      : { label: "真假", status: "partial", detail: `用神${use}在${month}月${state}，但${inMonth ? "沒有透干" : "不在月令"}：不算最真。` };

  const hidden = hiddenOnly(natal, use);
  const places = [...items.map((item) => item.node.char), ...hidden];
  const storageOnly = !stems.length && places.length > 0 && places.every((branch) => "辰戌丑未".includes(branch));
  const solid: BookCheck = stems.length
    ? { label: "虛實", status: "pass", detail: `用神透在${nodeText(stems.map((item) => item.node))}：實。` }
    : items.length && !storageOnly
      ? { label: "虛實", status: "partial", detail: `用神只在地支${nodeText(items.map((item) => item.node))}，沒有透干：半虛半實。` }
      : { label: "虛實", status: "fail", detail: hidden.length || items.length ? `用神只藏在${[...hidden, ...items.map((item) => item.node.char)].join("")}${storageOnly ? "（庫中）" : ""}：虛。` : `原局不見用神${use}：虛。` };

  const checks = [force, affection, truth, solid];
  const passes = checks.filter((check) => check.status === "pass").length;
  const fails = checks.filter((check) => check.status === "fail").length;
  const summary = passes >= 3 ? "用神品質好" : fails >= 3 ? "用神品質差（書稱偏枯、常人）" : "用神品質中等";
  return { use, checks, summary };
}

// ---------------------------------------------------------------------------
// 清濁三法（心訣第 99 訣）。

export interface ClarityReport {
  checks: BookCheck[];
  grade: string;
  detail: string;
}

export function clarityReport(natal: string[], useElements: Element[]): ClarityReport {
  const climate = climateReport(natal);
  const flow = flowReport(natal);
  const wither = useWitherCheck(natal, useElements);
  const checks = [climate.check, flow.check, wither];
  const clear = checks.filter((check) => check.status === "pass").length;
  const half = checks.filter((check) => check.status === "partial").length;
  const score = clear + half * 0.5;
  const grade = score >= 3 ? "一清到底" : score >= 2 ? "澄濁求清" : score >= 1 ? "半清半濁" : "滿盤濁氣";
  const detail = {
    一清到底: "三清齊得，書說富貴必真。",
    澄濁求清: "三得其二，也算佳造；行運能補上濁的那一項更好。",
    半清半濁: "三清只得其一，書說多成多敗；看行運能不能掃除濁氣。",
    滿盤濁氣: "三項都濁，書列為劣等；但「時來寒谷也生春」，行運送來能澄濁的字仍可翻身。",
  }[grade] as string;
  return { checks, grade, detail };
}

// ---------------------------------------------------------------------------
// 《輕鬆學算命》第 240 頁：不同日干先看哪一種取用法。

const FIRST_METHOD: Record<Element, string> = {
  木: "調候",
  火: "扶抑",
  土: "通關、病藥",
  金: "通關、調候",
  水: "調候、通關",
};

export function firstMethodHint(dayStem: string): string {
  const element = STEM_ELEMENT[dayStem];
  return element ? `${dayStem}${element}日主先看${FIRST_METHOD[element]}（輕鬆學算命 p240 作者經驗表；課堂仍以扶抑為本）。` : "";
}

// ---------------------------------------------------------------------------
// 《妙算人生八字》第 273–275 頁：戀愛方式（日支藏干透在哪一柱）。

const LOVE_WAY: Record<number, { attract: string; via: string }> = {
  0: { attract: "多參加公益、廟會、演講、郊遊、旅行等團體活動", via: "長輩、長官、父執輩介紹" },
  1: { attract: "言行舉止表現胸懷磊落、寬宏大量", via: "兄弟姊妹、朋友、同事" },
  3: { attract: "在事業、工作、技藝上展現能力", via: "子女、姪輩、下屬、員工等晚輩" },
};

export interface LoveStyle {
  dayBranch: string;
  hits: { pillar: string; stem: string; attract: string; via: string }[];
  selfExposed: boolean;
  summary: string;
}

export function loveStyle(natal: string[]): LoveStyle {
  const dayBranch = branchOf(natal[2]);
  const hidden = HIDDEN[dayBranch] ?? [];
  const hits: LoveStyle["hits"] = [];
  [1, 3, 0].forEach((index) => {
    const stem = stemOf(natal[index]);
    if (hidden.includes(stem)) hits.push({ pillar: `${PILLARS[index]}干`, stem, ...LOVE_WAY[index] });
  });
  const selfExposed = hidden.includes(stemOf(natal[2]));
  const summary = hits.length
    ? `日支${dayBranch}藏${hidden.join("")}，透在${hits.map((hit) => `${hit.pillar}${hit.stem}`).join("、")}${hits.length >= 2 ? "：透得多，戀愛機會多" : ""}。`
    : selfExposed
      ? `日支${dayBranch}的藏干只透在日干本身（日支自透）。`
      : `日支${dayBranch}藏${hidden.join("")}，都沒有透出：書說戀愛機率較低，請人介紹（含相親）較易成功。`;
  return { dayBranch, hits, selfExposed, summary };
}

// ---------------------------------------------------------------------------
// 限運（妙算第 319–320 頁每柱 17–18 年；心訣第 42 訣每柱 16 年）。

export interface LimitRow {
  pillar: string;
  ganZhi: string;
  miaoSuan: string;
  xinJue: string;
  stemFavor: Favor;
  branchFavor: Favor;
  note: string;
}

export function limitRows(natal: string[], favorOf?: (element: Element) => Favor): LimitRow[] {
  const miao = ["1–18 歲", "19–36 歲", "37–55 歲", "56 歲以後"];
  const xin = ["1–16 歲", "17–32 歲", "33–48 歲", "49–64 歲"];
  const notes = ["根：發源地，變化不明顯", "苗：命局關鍵，這段的事特別應驗", "花：一生富貴貧困從此分", "果：老來吉凶的期末考"];
  return natal.map((pillar, index) => ({
    pillar: `${PILLARS[index]}柱`,
    ganZhi: pillar,
    miaoSuan: miao[index],
    xinJue: xin[index],
    stemFavor: favorOf ? favorOf(STEM_ELEMENT[stemOf(pillar)]) : "neutral",
    branchFavor: favorOf ? favorOf(mainElement(branchOf(pillar)) as Element) : "neutral",
    note: notes[index],
  }));
}

// ---------------------------------------------------------------------------
// 五合三種化法與暗五合（心訣第 96 訣）。

const COMBINE_RESULT: Record<string, { husband: string; wife: string; proper: Element }> = {
  甲己: { husband: "甲", wife: "己", proper: "土" },
  乙庚: { husband: "庚", wife: "乙", proper: "金" },
  丙辛: { husband: "丙", wife: "辛", proper: "水" },
  丁壬: { husband: "壬", wife: "丁", proper: "木" },
  戊癸: { husband: "戊", wife: "癸", proper: "火" },
};
const DARK_COMBINE: Record<string, string> = {
  甲午: "甲合午中己",
  丁亥: "丁合亥中壬",
  戊子: "戊合子中癸",
  辛巳: "辛合巳中丙",
  壬午: "壬合午中丁",
};

export function combineNotes(natal: string[]): string[] {
  const notes: string[] = [];
  const stems = natal.map(stemOf);
  for (let index = 0; index < 3; index += 1) {
    const key = STEM_COMBINE.find((pair) => pair === stems[index] + stems[index + 1] || pair === stems[index + 1] + stems[index]);
    if (!key) continue;
    const info = COMBINE_RESULT[key];
    notes.push(
      `${PILLARS[index]}干${stems[index]}、${PILLARS[index + 1]}干${stems[index + 1]}相合：正化${info.proper}；夫從妻化${STEM_ELEMENT[info.wife]}（化成陰干${info.wife}的五行）；妻從夫化${STEM_ELEMENT[info.husband]}（化成陽干${info.husband}的五行）。化不化仍照「天干生剋合化」卡片的條件。`,
    );
  }
  natal.forEach((pillar, index) => {
    if (DARK_COMBINE[pillar]) notes.push(`${PILLARS[index]}柱${pillar}：暗五合（${DARK_COMBINE[pillar]}）。`);
  });
  return notes;
}

// ---------------------------------------------------------------------------
// 轉盤／借盤（子平正源第二章、心訣第 46 訣）。

export interface RelativeOption {
  key: string;
  label: string;
  gods: string[];
  /** 心訣：天干優先，父母取年干、夫妻取月干、子女取時干。 */
  order: number[];
  note: string;
}

export function relativeOptions(gender: "MALE" | "FEMALE"): RelativeOption[] {
  const female = gender === "FEMALE";
  return [
    { key: "spouse", label: female ? "丈夫" : "妻子", gods: female ? ["正官", "七殺"] : ["正財", "偏財"], order: [1, 3, 0], note: "夫妻取月干，再時干、年干（心訣 p115）" },
    { key: "father", label: "父親", gods: ["偏財", "正財"], order: [0, 1, 3], note: "父為偏財（母親的丈夫）；父母取年干（心訣）" },
    { key: "mother", label: "母親", gods: ["正印", "偏印"], order: [0, 1, 3], note: "母為正印，偏印＝繼母、乳母；父母取年干（心訣）" },
    {
      key: "child",
      label: "子女",
      gods: female ? ["傷官", "食神"] : ["七殺", "正官", "食神", "傷官"],
      order: [3, 1, 0],
      note: female ? "女命食傷為子女；子女取時干（心訣）" : "男命官殺為子女，不見官殺借食傷（子平正源）；子女取時干",
    },
    { key: "sibling", label: "兄弟姊妹", gods: ["比肩", "劫財"], order: [1, 0, 3], note: "比劫，也代表朋友、合伙人、情敵" },
    { key: "boss", label: "上司、公司", gods: ["正官", "七殺"], order: [1, 0, 3], note: "官殺：管我者（子平正源）" },
  ];
}

export interface RotationTarget {
  pillar: number;
  part: "干" | "支";
}

/** 依優先位置找代表這位親人的字：先天干，再地支本氣（日支最後）。找不到就不能借盤。 */
export function findRelative(natal: string[], option: RelativeOption): RotationTarget | null {
  const dayStem = stemOf(natal[2]);
  // 心訣：天干優於地支。先在天干依優先位置找（正星先於偏星），找不到才看地支本氣。
  // 男命子女：官殺全盤都找不到，才借食傷（子平正源）。
  const groups = option.key === "child" && option.gods.length > 2 ? [option.gods.slice(0, 2), option.gods.slice(2)] : [option.gods];
  for (const gods of groups) {
    for (const god of gods) {
      const stemHit = option.order.find((index) => tenGodOf(dayStem, stemOf(natal[index])) === god);
      if (stemHit !== undefined) return { pillar: stemHit, part: "干" };
    }
    for (const god of gods) {
      const branchHit = [...option.order, 2].find((index) => tenGodOf(dayStem, mainStem(branchOf(natal[index]))) === god);
      if (branchHit !== undefined) return { pillar: branchHit, part: "支" };
    }
  }
  return null;
}

export interface RotatedCell {
  pillar: string;
  stem: string;
  stemGod: string;
  branch: string;
  branchGod: string;
  hidden: { stem: string; god: string }[];
  isTarget: "干" | "支" | null;
  isSelf: boolean;
}

export interface RotationLuck {
  ganZhi: string;
  years: string;
  forSelf: string;
  forThem: string;
  effect: "good" | "bad" | "neutral";
  effectText: string;
}

export interface RotationReport {
  master: string;
  masterName: string;
  selfRole: string;
  cells: RotatedCell[];
  strength: ThreeOfTwo;
  threats: string[];
  luck: RotationLuck[];
}

function familyName(god: string): string {
  if (god === "比肩" || god === "劫財") return "比劫";
  if (god.endsWith("印")) return "印";
  if (god === "食神" || god === "傷官") return "食傷";
  if (god.endsWith("財")) return "財";
  return "官殺";
}

export function rotateChart(
  natal: string[],
  target: RotationTarget,
  luckRows: { ganZhi: string; startYear?: number | null; endYear?: number | null }[] = [],
): RotationReport {
  const dayStem = stemOf(natal[2]);
  const char = target.part === "干" ? stemOf(natal[target.pillar]) : branchOf(natal[target.pillar]);
  const master = target.part === "干" ? char : mainStem(char);
  const cells: RotatedCell[] = natal.map((pillar, index) => {
    const branch = branchOf(pillar);
    return {
      pillar: PILLARS[index],
      stem: stemOf(pillar),
      stemGod: index === target.pillar && target.part === "干" ? "新日主" : tenGodOf(master, stemOf(pillar)),
      branch,
      branchGod: tenGodOf(master, mainStem(branch)),
      hidden: (HIDDEN[branch] ?? []).map((stem) => ({ stem, god: tenGodOf(master, stem) })),
      isTarget: index === target.pillar ? target.part : null,
      isSelf: index === 2,
    };
  });
  const strength = threeOfTwo(natal, master, target);
  const element = STEM_ELEMENT[master];

  // 這個字在原局受到的威脅：鄰近的官殺（受剋）、五合／六合（被合）、沖。
  const nodes = nodesOf(natal);
  const self = nodes[target.part === "干" ? target.pillar : target.pillar + 4];
  const threats: string[] = [];
  nodes
    .filter((node) => adjacent(self, node))
    .forEach((node) => {
      if (CONTROLS[node.element] === element) threats.push(`${node.name}剋它（它的官殺）`);
      if (node.part === self.part && (self.part === "干" ? isStemCombine(self.char, node.char) : isSixCombine(self.char, node.char))) {
        threats.push(`和${node.name}相合（被合絆）`);
      }
      if (self.part === "支" && node.part === "支" && isClash(self.char, node.char)) threats.push(`被${node.name}沖`);
    });

  const luck: RotationLuck[] = luckRows.map((row) => {
    const stemGod = tenGodOf(master, stemOf(row.ganZhi));
    const branchGod = tenGodOf(master, mainStem(branchOf(row.ganZhi)));
    const families = [familyName(stemGod), familyName(branchGod)];
    const helps = families.filter((item) => item === "印" || item === "比劫").length;
    const hurts = families.filter((item) => item === "官殺").length;
    // 弱的字喜印比、怕官殺；旺的字喜洩耗剋，怕再生扶過頭（書：親人那個字要強才任得起財官）。
    let effect: RotationLuck["effect"] = "neutral";
    if (strength.strong) {
      effect = helps === 2 ? "bad" : helps === 0 ? "good" : "neutral";
    } else {
      effect = hurts > 0 && helps === 0 ? "bad" : helps > 0 && hurts === 0 ? "good" : "neutral";
    }
    const years = row.startYear ? `${row.startYear}–${row.endYear ?? ""}` : "";
    return {
      ganZhi: row.ganZhi,
      years,
      forSelf: `${tenGodOf(dayStem, stemOf(row.ganZhi))}／${tenGodOf(dayStem, mainStem(branchOf(row.ganZhi)))}`,
      forThem: `${stemGod}／${branchGod}`,
      effect,
      effectText:
        effect === "good"
          ? strength.strong
            ? "它旺，行洩耗剋之運較順"
            : "它弱，得印比生扶"
          : effect === "bad"
            ? strength.strong
              ? "它已旺，再逢印比過頭"
              : "它弱，又逢官殺剋"
            : "平",
    };
  });

  return {
    master,
    masterName: `${PILLARS[target.pillar]}${target.part}${char}${target.part === "支" ? `（本氣${master}）` : ""}`,
    selfRole: tenGodOf(master, dayStem),
    cells,
    strength,
    threats,
    luck,
  };
}

// ---------------------------------------------------------------------------
// 大運、流年標籤。

export interface FourBookLuckOptions {
  natal: string[];
  /** 課堂三步法合併後的喜忌；沒有時不出喜忌類標籤。 */
  favorOf?: (element: Element) => Favor;
  /** 流年所在的大運干支。 */
  luck?: string;
}

type PillarMood = "全喜" | "全忌" | "一喜一忌" | "平" | "小吉" | "小凶";

function moodOf(ganZhi: string, favorOf: (element: Element) => Favor): PillarMood | null {
  const stem = favorOf(STEM_ELEMENT[stemOf(ganZhi)]);
  const branch = favorOf(mainElement(branchOf(ganZhi)) as Element);
  if (stem === "mixed" || branch === "mixed") return null;
  const good = [stem, branch].filter((item) => item === "good").length;
  const bad = [stem, branch].filter((item) => item === "bad").length;
  if (good === 2) return "全喜";
  if (bad === 2) return "全忌";
  if (good && bad) return "一喜一忌";
  if (good) return "小吉";
  if (bad) return "小凶";
  return "平";
}

/** 妙算第 321–322 頁大運×流年組合（只標書上點名的幾種）。 */
const COMBO_TABLE: { luck: PillarMood; annual: PillarMood; label: string; level: "warn" | "info"; detail: string }[] = [
  { luck: "全喜", annual: "全喜", label: "鴻運當頭", level: "info", detail: "大運、流年干支全喜" },
  { luck: "全忌", annual: "全忌", label: "禍不單行", level: "warn", detail: "大運、流年干支全忌" },
  { luck: "一喜一忌", annual: "全忌", label: "凶禍立至", level: "warn", detail: "大運一喜一忌、流年全忌；流年為君，比大運全忌更危險" },
  { luck: "全忌", annual: "一喜一忌", label: "凶中有吉", level: "info", detail: "大運全忌、流年一喜一忌，十年辛苦中這一年有轉機" },
  { luck: "全喜", annual: "一喜一忌", label: "大有可為", level: "info", detail: "大運全喜、流年一喜一忌" },
  { luck: "一喜一忌", annual: "全喜", label: "大有可為", level: "info", detail: "流年全喜、大運一喜一忌" },
];

export function fourBookLuckEvents(incoming: string, who: "運" | "歲", options: FourBookLuckOptions): LuckEvent[] {
  const events: LuckEvent[] = [];
  const { natal, favorOf } = options;
  if (natal.length < 4 || incoming.length < 2) return events;
  const stem = stemOf(incoming);
  const branch = branchOf(incoming);
  const whoName = who === "運" ? "大運" : "流年";
  const natalBranches = natal.map(branchOf);
  const stemElement = STEM_ELEMENT[stem];
  const branchElement = mainElement(branch) as Element;

  // 蓋頭、截腳（妙算 p315–316）：只看大運，喜用的一方被另一方剋。
  if (who === "運" && favorOf) {
    if (CONTROLS[stemElement] === branchElement && favorOf(branchElement) === "good" && favorOf(stemElement) !== "good") {
      const relief = natal.some((pillar) => [STEM_ELEMENT[stemOf(pillar)], mainElement(branchOf(pillar))].some((element) => element === GENERATES[stemElement] || element === controllerOf(stemElement)));
      events.push({
        kind: "gai-tou",
        label: "蓋頭",
        level: "info",
        detail: `大運地支${branch}${branchElement}是喜用，頭上蓋著${stem}${stemElement}剋它：以地支為主，吉減半。${relief ? `原局有${GENERATES[stemElement]}或${controllerOf(stemElement)}可通關或制${stemElement}，妨礙較小。` : ""}（妙算 p315）`,
      });
    }
    if (CONTROLS[branchElement] === stemElement && favorOf(stemElement) === "good" && favorOf(branchElement) !== "good") {
      events.push({
        kind: "jie-jiao",
        label: "截腳",
        level: "info",
        detail: `大運天干${stem}${stemElement}是喜用，卻坐在剋它的${branch}${branchElement}上（支不載干）：「吉而無吉，謂之虛也」；原局或流年再有${controllerOf(stemElement)}剋${stemElement}反而變凶（妙算 p316）。`,
      });
    }
  }

  // 大運×流年喜忌組合（妙算 p321–322）。
  if (who === "歲" && favorOf && options.luck) {
    const luckMood = moodOf(options.luck, favorOf);
    const annualMood = moodOf(incoming, favorOf);
    const hit = COMBO_TABLE.find((row) => row.luck === luckMood && row.annual === annualMood);
    if (hit) {
      events.push({
        kind: `combo-${hit.label}`,
        label: hit.label,
        level: hit.level,
        detail: `大運${options.luck}（${luckMood}）× 流年${incoming}（${annualMood}）：${hit.detail}（妙算 p321–322，喜忌用課堂三步法）。`,
      });
    }
  }

  // 原局沒有的五行在大運出現（子平正源）。
  if (who === "運") {
    const present = new Set<Element>();
    natal.forEach((pillar) => {
      present.add(STEM_ELEMENT[stemOf(pillar)]);
      HIDDEN[branchOf(pillar)].forEach((item) => present.add(STEM_ELEMENT[item]));
    });
    const fresh = [...new Set([stemElement, ...HIDDEN[branch].map((item) => STEM_ELEMENT[item])])].filter((element) => !present.has(element));
    if (fresh.length) {
      events.push({
        kind: "new-element",
        label: `新見${fresh.join("")}`,
        level: "info",
        detail: `原局（含藏干）沒有${fresh.join("、")}，這步大運帶進來；子平正源作者觀察常關乎人生大事，吉凶看它對日主是喜是忌${favorOf ? `（課堂喜忌：${fresh.map((element) => `${element}${{ good: "喜", bad: "忌", mixed: "依方案", neutral: "未列" }[favorOf(element)]}`).join("、")}）` : ""}。`,
      });
    }
  }

  // 合而解沖（子平正源、妙算 p322）：運歲地支六合原局一組沖的其中一支。
  const clashes: [number, number][] = [];
  for (let a = 0; a < 4; a += 1) {
    for (let b = a + 1; b < 4; b += 1) {
      if (isClash(natalBranches[a], natalBranches[b])) clashes.push([a, b]);
    }
  }
  clashes.forEach(([a, b]) => {
    const bound = [a, b].find((index) => isSixCombine(branch, natalBranches[index]));
    if (bound === undefined) return;
    const other = bound === a ? b : a;
    events.push({
      kind: `combine-release-${a}${b}`,
      label: "合解沖",
      level: "info",
      detail: `原局${PILLARS[a]}支${natalBranches[a]}沖${PILLARS[b]}支${natalBranches[b]}，${whoName}${branch}合住${PILLARS[bound]}支${natalBranches[bound]}，${natalBranches[other]}就沖不動：命局本有沖、歲運來合可以化解（妙算 p322）；子平正源多例對應結婚、轉運。`,
    });
  });

  // 四正、四庫齊全（心訣第 64 訣）。寅申巳亥齊全已由「四沖」標籤提示。
  const all = [...natalBranches, branch, ...(who === "歲" && options.luck ? [branchOf(options.luck)] : [])];
  [
    { members: "子午卯酉", name: "四桃花（子午卯酉）齊全", warn: "防色情、身體劫難（三毒之「癡」）" },
    { members: "辰戌丑未", name: "四墓庫（辰戌丑未）齊全", warn: "防家宅、錢財、凶災（三毒之「貪」）" },
  ].forEach((group) => {
    const complete = group.members.split("").every((member) => all.includes(member));
    const natalComplete = group.members.split("").every((member) => natalBranches.includes(member));
    if (complete && !natalComplete && group.members.includes(branch)) {
      events.push({
        kind: `four-${group.members}`,
        label: group.members === "子午卯酉" ? "四正全" : "四庫全",
        level: "warn",
        detail: `原局加${who === "歲" ? "大運、流年" : "大運"}湊齊${group.name}：${group.warn}（心訣第 64 訣，作者自己的經驗法則）。`,
      });
    }
  });

  return events;
}
