/**
 * 特別格局檢查：依蘇民峰《玄學錦囊・八字論命》第十四章整理。
 *
 * 只列出「候選」與每一條件是否符合，不改五行力分數、不改課堂喜忌；
 * 書中同一八字也有前後判法不同的情形，所以這裡只當提醒。
 *
 * 計數方式（書沒有給分數，這是 App 的簡化）：
 * - 看日干以外的三個天干，加四個地支的本氣；月支算兩份（當令）。
 * - 地支三會或三合齊全時，三支都改算成局的五行（書：會方後「失去其原性」）。
 * - 「有根」只看地支本氣；藏干裡的比劫餘氣另外提醒（可能是假從）。
 */

import type { LuckEvent } from "./luckEvents";

type Element = "木" | "火" | "土" | "金" | "水";
type Family = "比劫" | "印" | "食傷" | "財" | "官殺";

export type CheckStatus = "pass" | "fail" | "partial";

export interface PatternCheck {
  label: string;
  status: CheckStatus;
  detail: string;
}

export interface PatternCandidate {
  key: string;
  name: string;
  /** formed＝條件都符合；partial＝大致符合但有瑕疵（書稱假從、假化）；none＝不成立。 */
  verdict: "formed" | "partial" | "none";
  verdictLabel: string;
  summary: string;
  checks: PatternCheck[];
  /** 書中的用神與喜忌提示。 */
  advice?: string;
  page: string;
}

export interface SpecialPatternReport {
  follow: PatternCandidate[];
  prosperous: PatternCandidate | null;
  transform: PatternCandidate | null;
  marks: PatternCandidate[];
  notes: string[];
}

const STEM_ELEMENT: Record<string, Element> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
/** 藏干依書第 23 頁，第一個是本氣。 */
const HIDDEN: Record<string, string[]> = {
  子: ["癸"], 丑: ["己", "癸", "辛"], 寅: ["甲", "丙", "戊"], 卯: ["乙"], 辰: ["戊", "乙", "癸"], 巳: ["丙", "戊", "庚"],
  午: ["丁", "己"], 未: ["己", "丁", "乙"], 申: ["庚", "壬", "戊"], 酉: ["辛"], 戌: ["戊", "辛", "丁"], 亥: ["壬", "甲"],
};
const GENERATES: Record<Element, Element> = { 木: "火", 火: "土", 土: "金", 金: "水", 水: "木" };
const CONTROLS: Record<Element, Element> = { 木: "土", 土: "水", 水: "火", 火: "金", 金: "木" };
const ELEMENTS: Element[] = ["木", "火", "土", "金", "水"];
const YANG = "甲丙戊庚壬";
const PILLARS = ["年", "月", "日", "時"];

/** 三會方先算，三合局次之。 */
const GATHERINGS: { branches: string; element: Element; name: string }[] = [
  { branches: "寅卯辰", element: "木", name: "寅卯辰三會木方" },
  { branches: "巳午未", element: "火", name: "巳午未三會火方" },
  { branches: "申酉戌", element: "金", name: "申酉戌三會金方" },
  { branches: "亥子丑", element: "水", name: "亥子丑三會水方" },
  { branches: "亥卯未", element: "木", name: "亥卯未三合木局" },
  { branches: "寅午戌", element: "火", name: "寅午戌三合火局" },
  { branches: "巳酉丑", element: "金", name: "巳酉丑三合金局" },
  { branches: "申子辰", element: "水", name: "申子辰三合水局" },
];

/** 從旺五格：日主五行、得時的月支、格名、書頁。 */
const PROSPEROUS: Record<Element, { months: string; name: string; page: string; virtue: string }> = {
  木: { months: "寅卯辰亥未", name: "曲直格", page: "p248–250", virtue: "木主仁，多慈悲" },
  火: { months: "巳午未寅戌", name: "炎上格", page: "p250–251", virtue: "火主禮，重禮貌" },
  土: { months: "辰戌丑未", name: "稼穡格", page: "p252–253", virtue: "土主信，信實無妄" },
  金: { months: "申酉戌巳丑", name: "從革格", page: "p253–254", virtue: "金主義，大義凜然" },
  水: { months: "亥子丑申辰", name: "潤下格", page: "p254–255", virtue: "水主智，足智多謀" },
};

/** 五合化氣：合化五行與化神當令的月支（p260–266）。 */
const TRANSFORMS: { pair: string; element: Element; months: string }[] = [
  { pair: "甲己", element: "土", months: "辰戌丑未" },
  { pair: "乙庚", element: "金", months: "巳酉丑申" },
  { pair: "丙辛", element: "水", months: "申子辰亥" },
  { pair: "丁壬", element: "木", months: "亥卯未寅" },
  { pair: "戊癸", element: "火", months: "寅午戌巳" },
];

// 五行的庫：水庫辰、火庫戌、木庫未、金庫丑。庫支不算剋自己的五行（p247 潤下格見辰、p263 清太宗得辰字）。
const TOMB: Record<Element, string> = { 水: "辰", 火: "戌", 木: "未", 金: "丑", 土: "" };

const KUI_GANG = ["戊戌", "壬辰", "庚戌", "庚辰"];
const JIN_SHEN = ["乙丑", "己巳", "癸酉"];
const CLASH_PAIRS = ["子午", "丑未", "寅申", "卯酉", "辰戌", "巳亥"];

const stemOf = (ganZhi: string) => ganZhi.charAt(0);
const branchOf = (ganZhi: string) => ganZhi.charAt(1);
const isClash = (a: string, b: string) => CLASH_PAIRS.some((pair) => pair === a + b || pair === b + a);
const mark = (ok: boolean): CheckStatus => (ok ? "pass" : "fail");

function familyOf(day: Element, other: Element): Family {
  if (day === other) return "比劫";
  if (GENERATES[other] === day) return "印";
  if (GENERATES[day] === other) return "食傷";
  if (CONTROLS[day] === other) return "財";
  return "官殺";
}

function elementOfFamily(day: Element, family: Family): Element {
  return ELEMENTS.find((element) => familyOf(day, element) === family) as Element;
}

interface Chart {
  natal: string[];
  stems: string[];
  branches: string[];
  dayStem: string;
  day: Element;
  /** 四個地支的本氣五行（已套用三會三合）。 */
  branchElements: Element[];
  /** 哪些地支被三會三合改變了五行。 */
  transformed: boolean[];
  gatherings: string[];
  /** 日干以外三干加四支、月支兩份的五類計數。 */
  weights: Record<Family, number>;
  /** 各類在天干（不含日干）出現的次數。 */
  stemCounts: Record<Family, number>;
}

function buildChart(natal: string[]): Chart {
  const stems = natal.map(stemOf);
  const branches = natal.map(branchOf);
  const dayStem = stems[2];
  const day = STEM_ELEMENT[dayStem];
  const branchElements = branches.map((branch) => STEM_ELEMENT[HIDDEN[branch][0]]);
  const transformed = branches.map(() => false);
  const gatherings: string[] = [];
  for (const gathering of GATHERINGS) {
    if (![...gathering.branches].every((branch) => branches.includes(branch))) continue;
    // 已被三會改過的地支不再重複算三合。
    const indexes = branches.map((branch, index) => (gathering.branches.includes(branch) ? index : -1)).filter((index) => index >= 0);
    if (indexes.some((index) => transformed[index])) continue;
    indexes.forEach((index) => {
      branchElements[index] = gathering.element;
      transformed[index] = true;
    });
    gatherings.push(gathering.name);
  }
  const empty = (): Record<Family, number> => ({ 比劫: 0, 印: 0, 食傷: 0, 財: 0, 官殺: 0 });
  const weights = empty();
  const stemCounts = empty();
  stems.forEach((stem, index) => {
    if (index === 2) return;
    const family = familyOf(day, STEM_ELEMENT[stem]);
    weights[family] += 1;
    stemCounts[family] += 1;
  });
  branchElements.forEach((element, index) => {
    weights[familyOf(day, element)] += index === 1 ? 2 : 1;
  });
  return { natal, stems, branches, dayStem, day, branchElements, transformed, gatherings, weights, stemCounts };
}

const familyElement = (chart: Chart, family: Family) => elementOfFamily(chart.day, family);
const monthFamily = (chart: Chart) => familyOf(chart.day, chart.branchElements[1]);

/** 地支本氣是比劫或印的位置，就是日主的根。 */
function strictRoots(chart: Chart): string[] {
  return chart.branchElements
    .map((element, index) => {
      const family = familyOf(chart.day, element);
      return family === "比劫" || family === "印" ? `${PILLARS[index]}支${chart.branches[index]}（${family}）` : "";
    })
    .filter(Boolean);
}

/** 藏干中氣、餘氣裡的比劫（寅申中的戊土不算），只提醒可能是假從。 */
function residualRoots(chart: Chart): string[] {
  const found: string[] = [];
  chart.branches.forEach((branch, index) => {
    if (chart.transformed[index]) return;
    HIDDEN[branch].slice(1).forEach((stem) => {
      if (stem === "戊" && (branch === "寅" || branch === "申")) return;
      if (STEM_ELEMENT[stem] === chart.day) found.push(`${PILLARS[index]}支${branch}藏${stem}`);
    });
  });
  return found;
}

/** 天干（不含日干）的比劫、印，在地支本氣有同五行就算有根。 */
function rootedHelpers(chart: Chart): { rooted: string[]; floating: string[] } {
  const rooted: string[] = [];
  const floating: string[] = [];
  chart.stems.forEach((stem, index) => {
    if (index === 2) return;
    const family = familyOf(chart.day, STEM_ELEMENT[stem]);
    if (family !== "比劫" && family !== "印") return;
    const label = `${PILLARS[index]}干${stem}（${family}）`;
    if (chart.branchElements.includes(STEM_ELEMENT[stem])) rooted.push(label);
    else floating.push(label);
  });
  return { rooted, floating };
}

/** 份量最多的一類；同分時比較透干數。 */
function strongestOf(chart: Chart, families: Family[]): Family {
  return families.reduce((best, family) => {
    const diff = chart.weights[family] - chart.weights[best];
    return diff > 0 || (diff === 0 && chart.stemCounts[family] > chart.stemCounts[best]) ? family : best;
  });
}

/** 份量嚴格最多（不與別類同分）才算「最旺的一行」。 */
function isStrictlyStrongest(chart: Chart, family: Family, among: Family[]): boolean {
  return among.every((other) => other === family || chart.weights[family] > chart.weights[other]);
}

function weightText(chart: Chart): string {
  return (["比劫", "印", "食傷", "財", "官殺"] as Family[])
    .map((family) => `${family}${familyElement(chart, family)} ${chart.weights[family]}`)
    .join("・");
}

/** 從格共同條件：失令、日主無根、天干印比無根（p239）。 */
function weakBaseChecks(chart: Chart): PatternCheck[] {
  const month = monthFamily(chart);
  const roots = strictRoots(chart);
  const helpers = rootedHelpers(chart);
  return [
    {
      label: "日主失令",
      status: mark(month !== "比劫" && month !== "印"),
      detail: `月支${chart.branches[1]}算${chart.branchElements[1]}，是日主的${month}。${month === "印" ? "月令是印，書說不能從（p187）。" : ""}`,
    },
    {
      label: "日主在地支無根",
      status: mark(roots.length === 0),
      detail: roots.length ? `有根：${roots.join("、")}。` : "四支本氣都不是比劫或印。",
    },
    {
      label: "天干印比無根（虛浮）",
      status: mark(helpers.rooted.length === 0),
      detail: helpers.rooted.length
        ? `${helpers.rooted.join("、")}在地支有根，能幫身。`
        : helpers.floating.length
          ? `${helpers.floating.join("、")}在地支沒有根，只是虛浮，不妨礙從。`
          : "天干沒有印比。",
    },
  ];
}

function verdictOf(checks: PatternCheck[], partialLabel = "大致符合"): Pick<PatternCandidate, "verdict" | "verdictLabel"> {
  if (checks.some((check) => check.status === "fail")) return { verdict: "none", verdictLabel: "不成立" };
  if (checks.some((check) => check.status === "partial")) return { verdict: "partial", verdictLabel: partialLabel };
  return { verdict: "formed", verdictLabel: "成立候選" };
}

function followCandidates(chart: Chart): PatternCandidate[] {
  const base = weakBaseChecks(chart);
  const residual = residualRoots(chart);
  const residualCheck: PatternCheck = {
    label: "藏干沒有比劫餘氣",
    status: residual.length ? "partial" : "pass",
    detail: residual.length ? `${residual.join("、")}：有一點餘氣，多半是假從（假從遇真運也能發，但運過就退）。` : "地支藏干也沒有比劫。",
  };
  const month = monthFamily(chart);
  const strongest = strongestOf(chart, ["食傷", "財", "官殺"]);
  const candidates: PatternCandidate[] = [];
  const w = chart.weights;
  const el = (family: Family) => familyElement(chart, family);

  const wealthChecks = [
    ...base,
    { label: "財星當令", status: mark(month === "財"), detail: `月令是${month}。` },
    { label: "財是最旺的一類", status: mark(isStrictlyStrongest(chart, "財", ["食傷", "財", "官殺"])), detail: weightText(chart) },
    residualCheck,
  ];
  candidates.push({
    key: "follow-wealth",
    name: "從財格",
    ...verdictOf(wealthChecks, "假從財"),
    summary: "日主失令無根，全局財多且旺，沒有印比；可見官或食傷，有官能貴。",
    checks: wealthChecks,
    advice: `用財（${el("財")}），喜食傷（${el("食傷")}）生財、官殺（${el("官殺")}）制比劫；忌比劫（${el("比劫")}）運大破財，印（${el("印")}）運也凶。`,
    page: "p239–242",
  });

  const killChecks = [
    ...base,
    { label: "官殺當令", status: mark(month === "官殺"), detail: `月令是${month}。` },
    { label: "官殺最旺且透干", status: mark(isStrictlyStrongest(chart, "官殺", ["食傷", "財", "官殺"]) && chart.stemCounts.官殺 > 0), detail: `${weightText(chart)}；官殺透干 ${chart.stemCounts.官殺} 個。` },
    { label: "沒有食傷制官殺", status: mark(w.食傷 === 0), detail: w.食傷 ? `食傷${el("食傷")}有 ${w.食傷} 份。` : "沒有食傷。" },
    residualCheck,
  ];
  candidates.push({
    key: "follow-killing",
    name: "從殺格",
    ...verdictOf(killChecks, "假從殺"),
    summary: "日主失令無根，官殺多又透干而旺，沒有食傷剋制（官多也作殺論）。",
    checks: killChecks,
    advice: `用官殺（${el("官殺")}），喜財（${el("財")}）生殺；最忌食傷（${el("食傷")}）運，次忌比劫、印運。`,
    page: "p242–244",
  });

  const childChecks = [
    ...base,
    { label: "生於食傷月令", status: mark(month === "食傷"), detail: `月令是${month}。` },
    { label: "食傷最旺且透干", status: mark(isStrictlyStrongest(chart, "食傷", ["比劫", "食傷", "財", "官殺"]) && chart.stemCounts.食傷 > 0), detail: `${weightText(chart)}；食傷透干 ${chart.stemCounts.食傷} 個。` },
    { label: "要見財星", status: mark(w.財 > 0), detail: w.財 ? "有財。" : "沒有財，書說不算從兒。" },
    { label: "不見印與官殺", status: mark(w.印 === 0 && w.官殺 === 0), detail: `印 ${w.印}、官殺 ${w.官殺}。` },
    residualCheck,
  ];
  candidates.push({
    key: "follow-child",
    name: "從兒格",
    ...verdictOf(childChecks, "假從兒"),
    summary: "生於食傷月，食傷最旺又透干，要見財，不見印和官殺。",
    checks: childChecks,
    advice: `用食傷（${el("食傷")}），喜財（${el("財")}），比劫也吉；最忌印（${el("印")}）運，官殺（${el("官殺")}）運多破敗。天干${chart.stemCounts.比劫 ? "透比肩，書說貴" : "不透比肩，書說只富不貴"}。`,
    page: "p244–245",
  });

  const anyStrict = candidates.some((candidate) => candidate.verdict !== "none");
  const groups = (["食傷", "財", "官殺"] as Family[]).filter((family) => w[family] > 0);
  const bridge = w.食傷 >= 2 && w.官殺 >= 2;
  const trendUse: Family = bridge ? "財" : strongest;
  const trendChecks = [
    ...base,
    {
      label: "食傷、財、官殺沒有一類能單獨成格",
      status: mark(!anyStrict && groups.length >= 2),
      detail: anyStrict ? "已有其他從格候選。" : `出現 ${groups.join("、") || "（無）"}；${weightText(chart)}。`,
    },
    residualCheck,
  ];
  candidates.push({
    key: "follow-trend",
    name: "從勢格",
    ...verdictOf(trendChecks, "假從勢"),
    summary: "日主無根，食傷、財、官殺競透，從較強之勢；兩邊相當時用財通關。",
    checks: trendChecks,
    advice: bridge
      ? `食傷與官殺都有力，取財（${el("財")}）通關為用；行財運最喜，官殺運次之，食傷又次之；印比劫祿刃運必凶。`
      : `從最強的${trendUse}（${el(trendUse)}）；印比劫祿刃運必凶。`,
    page: "p257–259",
  });

  const anyWealth = chart.branches.some((branch) => HIDDEN[branch].some((stem) => STEM_ELEMENT[stem] === el("財")))
    || chart.stems.some((stem, index) => index !== 2 && STEM_ELEMENT[stem] === el("財"));
  const strongChecks: PatternCheck[] = [
    { label: "印星當令", status: mark(month === "印"), detail: `月令是${month}。` },
    { label: "印最旺，印比合起來得地得勢", status: mark(strongestOf(chart, ["比劫", "印", "食傷", "財", "官殺"]) === "印" && w.印 + w.比劫 >= 5), detail: weightText(chart) },
    { label: "全局沒有一點財星（含藏干）", status: mark(!anyWealth), detail: anyWealth ? `有財（${el("財")}）。` : "天干地支藏干都沒有財。" },
  ];
  candidates.push({
    key: "follow-strong",
    name: "從強格",
    ...verdictOf(strongChecks),
    summary: "印星當令又得地得勢，母旺子衰，局中全無財星；可順不可逆。",
    checks: strongChecks,
    advice: `喜印（${el("印")}）比劫祿刃；官殺運有印化不忌；最忌財（${el("財")}）運剋旺神，食傷運有強印回剋也忌。`,
    page: "p246",
  });

  return candidates;
}

function prosperousCandidate(chart: Chart): PatternCandidate {
  const info = PROSPEROUS[chart.day];
  const w = chart.weights;
  const officerEl = familyElement(chart, "官殺");
  const sealEl0 = familyElement(chart, "印");
  // 官殺透干但與日干之間隔著印（官生印、印生身），書說引化不破格（p254 白虎格）。
  const shieldedStems: string[] = [];
  const officerStems = chart.stems
    .map((stem, index) => {
      if (index === 2 || STEM_ELEMENT[stem] !== officerEl) return "";
      const between = index < 2 ? chart.stems.slice(index + 1, 2) : chart.stems.slice(3, index);
      if (between.some((s) => STEM_ELEMENT[s] === sealEl0)) {
        shieldedStems.push(`${PILLARS[index]}干${stem}`);
        return "";
      }
      return `${PILLARS[index]}干${stem}`;
    })
    .filter(Boolean);
  // 日主五行之墓庫（水辰、火戌、木未、金丑）在從旺格中當成日主的庫，不算官殺。
  const officerSpots = [
    ...officerStems,
    ...chart.branchElements.map((element, index) =>
      element === officerEl && chart.branches[index] !== TOMB[chart.day] ? `${PILLARS[index]}支${chart.branches[index]}` : "",
    ),
  ].filter(Boolean);
  const own = chart.gatherings.filter((name) => name.includes(chart.day));
  const storage = [...chart.branches].filter((branch) => "辰戌丑未".includes(branch)).length;
  const hasFormation = chart.day === "土" ? storage >= 3 : own.length > 0;
  const checks: PatternCheck[] = [
    { label: "日主得時", status: mark(info.months.includes(chart.branches[1])), detail: `${info.name}要生在${[...info.months].join("、")}月；本命${chart.branches[1]}月。` },
    {
      label: "日主黨眾",
      status: w.比劫 >= 4 && w.比劫 + w.印 >= 6 ? "pass" : (w.比劫 >= 5 && own.length > 0) || (w.比劫 >= 3 && w.比劫 + w.印 >= 7) ? "partial" : "fail",
      detail: `${weightText(chart)}（比劫 4 份以上、印比合計 6 份以上；比劫 5 份且地支成本五行方局，或比劫 3 份但印比合計 7 份，算次一等）。`,
    },
    {
      label: "不見官殺",
      status: officerSpots.length ? "fail" : shieldedStems.length ? "partial" : "pass",
      detail: officerSpots.length
        ? `見官殺（${officerEl}）：${officerSpots.join("、")}，書說一見就破格。`
        : shieldedStems.length
          ? `${shieldedStems.join("、")}是官殺，但與日干之間隔著印，引化不破格。`
          : "天干和地支本氣都沒有官殺。",
    },
    {
      label: chart.day === "土" ? "四庫得三支以上" : "地支成方成局",
      status: hasFormation ? "pass" : "partial",
      detail: chart.day === "土" ? `辰戌丑未有 ${storage} 支。` : own.length ? own.join("、") : "沒有成方成局；書說只要黨多而強也可論，但較次。",
    },
  ];
  const outputEl = familyElement(chart, "食傷");
  const sealEl = familyElement(chart, "印");
  const useText = chart.stemCounts.食傷 > 0
    ? `透食傷（${outputEl}），以食傷吐秀為用`
    : chart.stemCounts.印 > 0
      ? `透印（${sealEl}），以印為用，官殺運有印化不忌`
      : "印、食傷都不透，日主一行孤獨無輔";
  return {
    key: "prosperous",
    name: `從旺・${info.name}`,
    ...verdictOf(checks, "次一等"),
    summary: `日主${chart.day}一行得氣（${info.virtue}）。成則從之，不成則照普通格局、建祿羊刃論。`,
    checks,
    advice: `${useText}。最忌旺神被沖、剋、合；原局沒有食傷時切忌財官殺運（官殺犯旺、財運群劫爭財）。`,
    page: info.page,
  };
}

/** 五合化氣格六條件（p260–266）。日干與月干或時干相合才檢查。 */
function transformCandidate(chart: Chart): PatternCandidate | null {
  const partners = [1, 3]
    .map((index) => ({ index, stem: chart.stems[index] }))
    .map(({ index, stem }) => ({ index, stem, rule: TRANSFORMS.find((t) => t.pair === chart.dayStem + stem || t.pair === stem + chart.dayStem) }))
    .filter((item) => item.rule);
  if (!partners.length) return null;
  const { index, stem, rule } = partners[0] as { index: number; stem: string; rule: (typeof TRANSFORMS)[number] };
  const transformEl = rule.element;
  const enemyEl = (Object.keys(CONTROLS) as Element[]).find((element) => CONTROLS[element] === transformEl) as Element;
  const motherEl = (Object.keys(GENERATES) as Element[]).find((element) => GENERATES[element] === transformEl) as Element;

  // 1. 日主無強根：本氣是比劫或印的地支。日干本身生化神時（如辛化水、壬化木），比劫根不算破（p270 第 6 種）。
  const dayFeedsTransform = GENERATES[chart.day] === transformEl;
  const roots = chart.branchElements
    .map((element, i) => {
      if (chart.branches[i] === TOMB[transformEl]) return "";
      const family = familyOf(chart.day, element);
      if (family === "印" || (family === "比劫" && !dayFeedsTransform)) return `${PILLARS[i]}支${chart.branches[i]}（${family}）`;
      return "";
    })
    .filter(Boolean);
  const transformBranches = chart.branchElements.filter((element) => element === transformEl).length;
  const enemies = chart.stems
    .map((s, i) => (i !== 2 && i !== index && STEM_ELEMENT[s] === enemyEl ? { label: `${PILLARS[i]}干${s}`, rooted: chart.branchElements.includes(enemyEl) } : null))
    .filter((item): item is { label: string; rooted: boolean } => item !== null);
  const enemyBranches = chart.branchElements.map((element, i) => (element === enemyEl && chart.branches[i] !== TOMB[transformEl] ? `${PILLARS[i]}支${chart.branches[i]}` : "")).filter(Boolean);
  const hourBranchEl = chart.branchElements[3];
  const sources = chart.stems.filter((s, i) => i !== 2 && i !== index && (STEM_ELEMENT[s] === transformEl)).length;
  const rivals = [0, 1, 3].filter((i) => i !== index && chart.stems[i] === stem && Math.abs(i - 2) <= 1);
  const rivalDay = chart.stems[0] === chart.dayStem && index === 1;

  const checks: PatternCheck[] = [
    {
      label: "日主無強根",
      status: roots.length ? "partial" : "pass",
      detail: roots.length ? `${roots.join("、")}：日主帶苗根，最多是假化，要等運去根。` : "地支本氣沒有日主的比劫或印。",
    },
    { label: "日干與月干或時干五合", status: "pass", detail: `日干${chart.dayStem}合${PILLARS[index]}干${stem}，化${transformEl}。` },
    {
      label: "月令是化神當令",
      status: mark(rule.months.includes(chart.branches[1]) || chart.branchElements[1] === transformEl),
      detail: `${rule.pair}化${transformEl}要生在${[...rule.months].join("、")}月，或月支會成${transformEl}局；本命${chart.branches[1]}月${chart.branchElements[1] === transformEl && !rule.months.includes(chart.branches[1]) ? "（已會成化神）" : ""}。`,
    },
    {
      label: "化神旺，不見剋化神",
      status: transformBranches < 2 || enemyBranches.length || enemies.some((e) => e.rooted) ? (transformBranches < 2 ? "fail" : "partial") : enemies.length ? "partial" : "pass",
      detail: `地支${transformEl} ${transformBranches} 支（要兩支以上）；剋化神的${enemyEl}：${[...enemies.map((e) => `${e.label}${e.rooted ? "有根" : "無根"}`), ...enemyBranches].join("、") || "沒有"}。`,
    },
    {
      label: "化神原神透出",
      status: sources > 0 ? "pass" : "partial",
      detail: sources ? `天干另有${transformEl}透出。` : `天干沒有另外透${transformEl}，力量較弱（地支化神旺也可）。`,
    },
    {
      label: "沒有爭合妒合",
      status: mark(rivals.length === 0 && !rivalDay),
      detail: rivals.length ? `另有${rivals.map((i) => `${PILLARS[i]}干${stem}`).join("、")}爭合，合不專，要當正格論（原局有剋合可解則轉敗為成）。` : "合得專一。",
    },
    {
      label: "時支不逆化神",
      status: hourBranchEl === enemyEl ? "partial" : "pass",
      detail: `時支${chart.branches[3]}算${hourBranchEl}${hourBranchEl === enemyEl ? "，剋化神" : hourBranchEl === transformEl || hourBranchEl === motherEl ? "，生助化神" : ""}。`,
    },
  ];
  return {
    key: "transform",
    name: `${rule.pair}化${transformEl}`,
    ...verdictOf(checks, "假化"),
    summary: "日干極弱、與緊貼的月干或時干五合，又逢化神當令，就順化神論。真化少、假化多；假化要等運補條件。",
    checks,
    advice: `用化神${transformEl}，喜生化神的${motherEl}；忌剋化神的${enemyEl}。歲運見剋合化神的天干叫「化神還原」，主破敗（大運流年格子會標出）。`,
    page: "p260–271",
  };
}

function kuiGangCandidate(chart: Chart, female: boolean): PatternCandidate | null {
  const dayPillar = chart.natal[2];
  if (!KUI_GANG.includes(dayPillar)) return null;
  const dayBranch = chart.branches[2];
  const clashed = [1, 3].filter((i) => isClash(chart.branches[i], dayBranch));
  const forbidden: Element = chart.dayStem === "戊" ? "水" : "火";
  const bad = chart.stems.map((s, i) => (i !== 2 && STEM_ELEMENT[s] === forbidden ? `${PILLARS[i]}干${s}` : "")).filter(Boolean);
  const checks: PatternCheck[] = [
    { label: "日柱是魁罡", status: "pass", detail: `${dayPillar}（魁罡只有戊戌、壬辰、庚戌、庚辰四日）。` },
    { label: "日支不被月支、時支沖", status: mark(clashed.length === 0), detail: clashed.length ? `${clashed.map((i) => `${PILLARS[i]}支${chart.branches[i]}`).join("、")}沖日支${dayBranch}，魁罡破格。` : "沒有沖。" },
    { label: `天干不透${forbidden}`, status: mark(bad.length === 0), detail: bad.length ? `${bad.join("、")}透出，破格。` : `${chart.dayStem}日魁罡忌天干見${forbidden}，本命沒有。` },
  ];
  const result = verdictOf(checks);
  return {
    key: "kui-gang",
    name: "魁罡",
    verdict: result.verdict,
    verdictLabel: result.verdict === "formed" ? "成格" : "破格",
    summary: "魁罡不破主富貴、有領導才能和決斷力，但性格剛強；犯破者學問難成。最忌運歲沖日支。",
    checks,
    advice: `宜身旺，逢強印比劫文章振發；行運照正格論。${female ? "女命日柱魁罡，書說心性過剛、多剋夫易離婚，只供參考。" : ""}`,
    page: "p275–278",
  };
}

function jinShenCandidate(chart: Chart): PatternCandidate | null {
  const dayHit = JIN_SHEN.includes(chart.natal[2]);
  const hourHit = JIN_SHEN.includes(chart.natal[3]) && (chart.dayStem === "甲" || chart.dayStem === "己");
  if (!dayHit && !hourHit) return null;
  const monthFire = HIDDEN[chart.branches[1]].some((stem) => STEM_ELEMENT[stem] === "火");
  const fireCount = chart.stems.filter((s) => STEM_ELEMENT[s] === "火").length + chart.branchElements.filter((e) => e === "火").length;
  const checks: PatternCheck[] = [
    { label: dayHit ? "日柱是金神" : "時柱是金神（甲、己日）", status: "pass", detail: `${dayHit ? `日柱${chart.natal[2]}` : `時柱${chart.natal[3]}`}；金神只有乙丑、己巳、癸酉。` },
    { label: "月支有火，或四柱火多", status: monthFire || fireCount >= 2 ? "pass" : "partial", detail: `月支${chart.branches[1]}${monthFire ? "藏火" : "不藏火"}；天干與地支本氣的火有 ${fireCount} 個。` },
  ];
  const result = verdictOf(checks, "假金神");
  return {
    key: "jin-shen",
    name: "金神格",
    verdict: result.verdict,
    verdictLabel: result.verdict === "formed" ? "真金神" : "假金神",
    summary: "金神要火制才顯貴，火愈多愈貴；性剛毅不屈，易遭人嫉。",
    checks,
    advice: `行運喜火忌水。${hourHit && !dayHit ? "時柱金神不一定用火，要看四柱配合。" : ""}假金神要等運配合才可能發達一時。`,
    page: "p279–282",
  };
}

function twoElementCandidate(chart: Chart): PatternCandidate | null {
  const counts = new Map<Element, { stems: number; branches: number }>();
  chart.stems.forEach((stem) => {
    const entry = counts.get(STEM_ELEMENT[stem]) ?? { stems: 0, branches: 0 };
    entry.stems += 1;
    counts.set(STEM_ELEMENT[stem], entry);
  });
  chart.branches.forEach((branch) => {
    const element = STEM_ELEMENT[HIDDEN[branch][0]];
    const entry = counts.get(element) ?? { stems: 0, branches: 0 };
    entry.branches += 1;
    counts.set(element, entry);
  });
  if (counts.size !== 2) return null;
  const [a, b] = [...counts.keys()];
  const even = [...counts.values()].every((entry) => entry.stems === 2 && entry.branches === 2);
  const generating = GENERATES[a] === b || GENERATES[b] === a;
  const other = a === chart.day ? b : a;
  const family = familyOf(chart.day, other);
  const kind = generating ? (family === "印" ? "生我之局" : "我生之局") : family === "財" ? "我剋之局" : "剋我之局";
  const advice: Record<string, string> = {
    生我之局: "類似從旺或母子同心：喜印比運，官殺有印轉化無大患，財星犯旺大忌。",
    我生之局: "不論日主強弱都以食傷為用，喜財流通，忌官殺犯旺、印破食傷。",
    我剋之局: "喜食傷通關，忌印制食傷、官殺剋身；財比日主旺不忌比劫。",
    剋我之局: "喜印洩官殺生身，比劫也可；忌財破印，食傷制官殺也凶。",
  };
  const checks: PatternCheck[] = [
    { label: "只有兩種五行", status: "pass", detail: `${a}、${b}（天干與地支本氣）。` },
    { label: "各佔兩干兩支", status: even ? "pass" : "fail", detail: [...counts.entries()].map(([el, c]) => `${el}：${c.stems}干${c.branches}支`).join("；") },
  ];
  return {
    key: "two-element",
    name: `兩神成象（${a}${b}${generating ? "相生" : "相剋"}・${kind}）`,
    ...verdictOf(checks),
    summary: "八字只有兩種五行、力量均等。入格者行運忌見混雜之物。",
    checks,
    advice: advice[kind],
    page: "p272–275",
  };
}

export function specialPatternReport(natal: string[], options: { female?: boolean } = {}): SpecialPatternReport | null {
  if (natal.length !== 4 || natal.some((pillar) => pillar.length < 2 || !STEM_ELEMENT[stemOf(pillar)] || !HIDDEN[branchOf(pillar)])) return null;
  const chart = buildChart(natal);
  const notes: string[] = [];
  if (chart.gatherings.length) notes.push(`地支${chart.gatherings.join("、")}，計數時三支都改算成局的五行。`);
  if (!YANG.includes(chart.dayStem)) notes.push("陰干比陽干容易從（p243：己土同樣的局，若是陽干戊就不從）。");
  const marks = [kuiGangCandidate(chart, Boolean(options.female)), jinShenCandidate(chart), twoElementCandidate(chart)].filter(
    (item): item is PatternCandidate => item !== null,
  );
  const follow = followCandidates(chart);
  const transform = transformCandidate(chart);
  if (transform && transform.verdict !== "none" && follow.some((item) => item.verdict !== "none")) {
    notes.push("化氣格和從格同時是候選時，書以化氣格論（日干已合化，不再從）；從格候選只供參考。");
  }
  return {
    follow,
    prosperous: prosperousCandidate(chart),
    transform,
    marks,
    notes,
  };
}

/**
 * 大運流年提示：化氣格的「化神還原」（p260–266）與魁罡被沖（p275）。
 * 只在原局化氣（真或假）或魁罡成格時出現。
 */
export function specialPatternLuckEvents(incoming: string, who: "運" | "歲", natal: string[]): LuckEvent[] {
  const report = specialPatternReport(natal);
  if (!report || incoming.length < 2) return [];
  const events: LuckEvent[] = [];
  const stem = stemOf(incoming);
  const branch = branchOf(incoming);
  const label = who === "運" ? `大運${incoming}` : `流年${incoming}`;
  const transform = report.transform;
  if (transform && transform.verdict !== "none") {
    const pair = transform.name.slice(0, 2);
    const yang = [...pair].find((s) => YANG.includes(s)) as string;
    const yin = [...pair].find((s) => !YANG.includes(s)) as string;
    const incomingEl = STEM_ELEMENT[stem];
    if (CONTROLS[incomingEl] === STEM_ELEMENT[yang]) {
      events.push({
        kind: "transform-revert",
        label: "化神還原",
        level: "warn",
        detail: `${label}天干${stem}剋${yang}，${pair}合化的${yang}被剋回原形，書說化格破、主破敗或災禍（p260–266）。原局是${transform.verdictLabel}。`,
      });
    } else if (CONTROLS[incomingEl] === STEM_ELEMENT[yin] && !YANG.includes(stem)) {
      events.push({
        kind: "transform-revert-light",
        label: "化神受剋",
        level: "info",
        detail: `${label}天干${stem}剋${yin}，書說也凶但較輕（p260–266）。`,
      });
    }
  }
  const kuiGang = report.marks.find((item) => item.key === "kui-gang");
  if (kuiGang && kuiGang.verdict === "formed" && isClash(branch, branchOf(natal[2]))) {
    events.push({
      kind: "kui-gang-clash",
      label: "沖魁罡",
      level: "warn",
      detail: `${label}地支${branch}沖日支${branchOf(natal[2])}。書說魁罡最忌沖破，日支逢運歲沖破必有大災（p275）。`,
    });
  }
  return events;
}
