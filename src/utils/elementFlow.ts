import type { BaziResponse } from "../types/bazi";
import type { WuXing } from "../constants/wuxing";
import { STEM_WUXING } from "../constants/wuxing";
import { lessonPreferences, type Favor } from "./lessonPreference";
import { bonusesAt, cellStatesAt, strengthShift, type ShiftCell, type ShiftLuckInfo, type StrengthShift } from "./strengthShift";
import { BRANCH_HIDDEN_ORDERED, rootLevel } from "./stemRoots";

// 五行流通「重點圖」的資料：把八個字收成五行五個圈（以日主為起點，順著相生排），
// 再挑出這張盤最該先看的幾句話。只數字數、看方向，不打分數（老師：五行不能量化）。

const ORDER = ["木", "火", "土", "金", "水"];
const KEY: Record<string, WuXing> = { 木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water" };

export type FlowRole = "比劫" | "食傷" | "財" | "官殺" | "印";
const ROLES: FlowRole[] = ["比劫", "食傷", "財", "官殺", "印"];
const ROLE_TEXT: Record<FlowRole, string> = { 比劫: "同我", 食傷: "我生", 財: "我剋", 官殺: "剋我", 印: "生我" };

export interface FlowChar {
  id: string;
  char: string;
  row: "stem" | "branch";
  isDayMaster: boolean;
  isMonthCommand: boolean;
  /** 這個字合化了，五行已經換成現在這一行 */
  transformed: boolean;
  /** 不是原局的字：運＝大運、年＝流年 */
  extra?: "運" | "年";
}

export interface FlowElement {
  /** 0 比劫（日主這一行）起，順著相生：食傷、財、官殺、印 */
  index: number;
  key: WuXing;
  name: string;
  role: FlowRole;
  roleText: string;
  party: "same" | "opp";
  chars: FlowChar[];
  /** 成局多出來的合力（例：「水局」），算一個字 */
  bonus: string[];
  /** 字數＋合力，已經照合化、成局之後來算（和「同黨：異黨」同一套數法） */
  count: number;
  /** 只藏在地支中氣、餘氣裡的同五行（例：「巳中庚」）；本氣已經算在 chars 裡 */
  hidden: string[];
  favor: Favor;
  /** 課堂方案裡有把這一行列為用神 */
  isUseGod: boolean;
  /** 加了大運（流年）之後比原局多或少幾個（沒有加運時是 0） */
  delta: number;
}

export interface FlowEdge {
  from: number;
  to: number;
  kind: "generate" | "control";
  /** flow 兩邊都有字；broken 有一邊沒有字；clash 相剋而且兩邊都有兩個字以上 */
  state: "flow" | "broken" | "clash" | "plain";
  /** 這條線對日主有利還是不利：生喜、剋忌＝good；生忌、剋喜＝bad；被生／被剋的那一行沒有明確喜忌，或線是斷的＝none */
  effect: "good" | "bad" | "none";
}

export interface FlowPoint {
  key: string;
  tone: "good" | "bad" | "info";
  title: string;
  text: string;
  /** 點這一句時要亮起來的五行（index） */
  elements: number[];
}

export interface ElementFlow {
  dayStem: string;
  dayElement: string;
  /** 課堂三步法的判定（照原局人數） */
  classification: string;
  /** 合化、成局之後人數偏向另一邊時，寫那一邊的判定；沒有翻轉是 null */
  flipTo: string | null;
  /** 加進來的大運（流年），例如「大運庚午 29–39 歲」；只看原局時是 null */
  extraLabel: string | null;
  /** 合化、成局之後的人數（有加運時含運） */
  same: number;
  opp: number;
  elements: FlowElement[];
  generates: FlowEdge[];
  controls: FlowEdge[];
  points: FlowPoint[];
}

const list = (names: string[]) => names.join("、");

function chainsOf(elements: FlowElement[]): FlowElement[][] {
  // 順著相生把「有字」的五行串起來；遇到沒有字的那一行就斷開。
  const present = elements.map((item) => item.count > 0);
  if (present.every(Boolean)) return [elements];
  const firstGap = present.indexOf(false);
  const chains: FlowElement[][] = [];
  let current: FlowElement[] = [];
  for (let step = 1; step <= 5; step += 1) {
    const item = elements[(firstGap + step) % 5];
    if (item.count > 0) current.push(item);
    else if (current.length) {
      chains.push(current);
      current = [];
    }
  }
  if (current.length) chains.push(current);
  return chains;
}

function downMarks(shift: StrengthShift, cell: ShiftCell): string[] {
  return shift.steps
    .filter((step) => !step.inactive)
    .flatMap((step) => step.effects.filter((effect) => effect.cell === cell.id && (effect.dir === "down" || effect.dir === "bind") && !effect.light).map((effect) => effect.text));
}

/** 字是從哪裡來的：pillar 4 大運、5 流年（之後 strengthShift 加流年時沿用）。 */
const extraOf = (cell: ShiftCell): FlowChar["extra"] => (!cell.luck ? undefined : cell.pillar >= 5 ? "年" : "運");

/**
 * 五行流通重點圖。
 * shift 不給就只看原局；給了帶大運（流年）的 strengthShift（四柱圖上點了大運、流年），圈裡的字數、
 * 相生相剋、重點句都把運、年的字算進去，並多一句「這步大運」「這個流年」說明改變了什麼。
 */
export function elementFlow(result: BaziResponse, withLuck?: StrengthShift | null): ElementFlow | null {
  const analysis = result.quantModel?.lessonContent?.threeStepAnalysis;
  const base = buildFlow(analysis, strengthShift(analysis));
  if (!base || !withLuck || (!withLuck.luck && !withLuck.annual)) return base;
  const full = buildFlow(analysis, withLuck);
  if (!full) return base;
  for (const item of full.elements) item.delta = item.count - base.elements[item.index].count;

  const points: FlowPoint[] = [];
  if (withLuck.luck && withLuck.annual) {
    // 大運、流年都有：先比「原局→加大運」，再比「加大運→再加流年」
    const luckOnly = strengthShift(analysis, { ganZhi: withLuck.luck.ganZhi, label: withLuck.luck.label });
    const mid = buildFlow(analysis, luckOnly);
    if (luckOnly?.luck && mid) points.push(stagePoint(base, mid, luckOnly, luckOnly.luck, "大運"));
    if (mid) points.push(stagePoint(mid, full, withLuck, withLuck.annual, "流年"));
  } else if (withLuck.luck) points.push(stagePoint(base, full, withLuck, withLuck.luck, "大運"));
  else if (withLuck.annual) points.push(stagePoint(base, full, withLuck, withLuck.annual, "流年"));
  full.points.unshift(...points);

  // 強弱的判定仍是原局的（大運、流年不改判），沿用原局那一句
  const verdict = base.points.find((point) => point.key === "verdict");
  const at = full.points.findIndex((point) => point.key === "verdict");
  if (verdict && at >= 0) full.points[at] = verdict;
  full.classification = base.classification;
  full.flipTo = base.flipTo;
  full.extraLabel = [
    withLuck.luck ? `大運${withLuck.luck.ganZhi}${withLuck.luck.label ? ` ${withLuck.luck.label}` : ""}` : "",
    withLuck.annual ? `流年${withLuck.annual.ganZhi}${withLuck.annual.label ? ` ${withLuck.annual.label}` : ""}` : "",
  ]
    .filter(Boolean)
    .join("、");
  return full;
}

/** 「這步大運」「這個流年」：比前後兩張圖，說帶進什麼、補上或打斷了什麼、跟扶抑順不順。 */
function stagePoint(before: ElementFlow, after: ElementFlow, shift: StrengthShift, info: ShiftLuckInfo, kind: "大運" | "流年"): FlowPoint {
  const deltas = after.elements.map((item) => item.count - before.elements[item.index].count);
  const added = after.elements.filter((item) => deltas[item.index] > 0);
  const lost = after.elements.filter((item) => deltas[item.index] < 0);
  const isYear = kind === "流年";
  const chars = shift.cells.filter((cell) => cell.luck && (cell.pillar >= 5) === isYear);
  const states = cellStatesAt(shift, shift.steps.length);
  const charText = chars
    .map((cell) => {
      const now = states.get(cell.id)?.element ?? cell.element;
      const role = after.elements.find((item) => item.name === now);
      const fav = role?.favor === "good" ? "喜" : role?.favor === "bad" ? "忌" : "";
      return `${cell.char}${now}${role ? `（${role.role}${fav ? `，${fav}` : ""}）` : ""}`;
    })
    .join("、");
  const filled = before.elements.filter((item) => item.count === 0 && after.elements[item.index].count > 0);
  const nowMissing = after.elements.filter((item) => item.count === 0 && before.elements[item.index].count > 0);
  const joined = after.generates.filter((edge) => edge.state === "flow" && before.generates[edge.from].state === "broken");
  const newClash = after.controls.filter((edge) => edge.state === "clash" && before.controls[edge.from].state !== "clash");
  const group = isYear ? "annual" : "luck";
  const acts = shift.steps
    .slice(info.firstStep)
    .filter((step) => step.group === group && !step.inactive && !step.id.endsWith("-enter") && step.tag !== "同柱")
    .map((step) => step.title);
  const goodAdd = added.filter((item) => item.favor === "good");
  const badAdd = added.filter((item) => item.favor === "bad");
  const sentences = [
    `${kind}帶進 ${charText}。`,
    filled.length ? `補上${isYear ? "原局和大運" : "原局"}缺的${list(filled.map((item) => item.name))}${joined.length ? `，相生接通 ${joined.length} 處` : ""}。` : "",
    nowMissing.length ? `合化之後${list(nowMissing.map((item) => item.name))}沒有字了。` : "",
    newClash.length ? `多了 ${list(newClash.map((edge) => `${after.elements[edge.from].name}剋${after.elements[edge.to].name}`))} 兩邊都有力的相剋。` : "",
    acts.length ? `引動：${list(acts)}。` : "",
    info.rootShaken.length ? `日主的根 ${list(info.rootShaken)} 被沖動。` : info.bringsRoot ? `${kind}地支給日主帶來一個根。` : "",
    info.advice ?? "",
  ].filter(Boolean);
  return {
    key: isYear ? "annual" : "luck",
    tone: info.advice?.includes("是順的") || (goodAdd.length && !badAdd.length) ? "good" : info.advice?.includes("是逆的") || (badAdd.length && !goodAdd.length) ? "bad" : "info",
    title:
      `${isYear ? "這個流年" : "這步大運"}${info.ganZhi}：` +
      (added.length ? list(added.map((item) => `${item.name}＋${deltas[item.index]}`)) : "字數沒變") +
      (lost.length ? `、${list(lost.map((item) => `${item.name}${deltas[item.index]}`))}` : ""),
    text: sentences.join(""),
    elements: [...new Set([...added, ...lost, ...filled].map((item) => item.index))],
  };
}

function buildFlow(analysis: Parameters<typeof strengthShift>[0], shift: StrengthShift | null): ElementFlow | null {
  if (!shift) return null;
  const prefs = lessonPreferences(analysis);
  const states = cellStatesAt(shift, shift.steps.length);
  const bonuses = bonusesAt(shift, shift.steps.length);
  // 合化之後的樣子：五行、黨別都換成變化後的（圖上的字數才會和「同黨：異黨」對得起來）。
  // 有加大運時，大運的字也一起算（extra 標「運」）。
  const firstLuckStep = shift.luck?.firstStep ?? shift.annual?.firstStep ?? shift.steps.length;
  const natal = shift.cells
    .map((cell) => {
      const state = states.get(cell.id);
      return { ...cell, element: state?.element ?? cell.element, family: state?.family ?? cell.family, party: state?.party ?? cell.party, transformed: !!state?.transformed, extra: extraOf(cell) };
    });
  const dayCell = natal.find((cell) => cell.isDayMaster);
  if (!dayCell) return null;
  const start = ORDER.indexOf(shift.dayElement);
  if (start < 0) return null;

  const elements: FlowElement[] = ROLES.map((role, index) => {
    const name = ORDER[(start + index) % 5];
    const key = KEY[name];
    const cells = natal.filter((cell) => cell.element === name);
    const hidden = natal
      .filter((cell) => cell.row === "branch")
      .flatMap((cell) =>
        (BRANCH_HIDDEN_ORDERED[cell.char] ?? [])
          .slice(1)
          .filter((stem) => STEM_WUXING[stem] === key)
          .map((stem) => `${cell.char}中${stem}`),
      );
    const pref = prefs?.elements[key];
    return {
      index,
      key,
      name,
      role,
      roleText: ROLE_TEXT[role],
      party: role === "比劫" || role === "印" ? "same" : "opp",
      chars: cells.map((cell) => ({ id: cell.id, char: cell.char, row: cell.row, isDayMaster: cell.isDayMaster, isMonthCommand: cell.isMonthCommand, transformed: cell.transformed, extra: cell.extra })),
      bonus: bonuses.filter((item) => item.label.startsWith(name)).map((item) => item.label),
      count: cells.length + bonuses.filter((item) => item.label.startsWith(name)).length,
      hidden: [...new Set(hidden)],
      favor: pref?.favor ?? "neutral",
      isUseGod: !!pref?.notes.some((note) => note.includes("為用神")),
      delta: 0,
    };
  });

  const generates: FlowEdge[] = elements.map((item, index) => {
    const next = elements[(index + 1) % 5];
    const flowing = item.count > 0 && next.count > 0;
    const effect = !flowing ? "none" : next.favor === "good" ? "good" : next.favor === "bad" ? "bad" : "none";
    return { from: index, to: next.index, kind: "generate", state: flowing ? "flow" : "broken", effect };
  });
  const controls: FlowEdge[] = elements.map((item, index) => {
    const target = elements[(index + 2) % 5];
    const both = item.count > 0 && target.count > 0;
    const effect = !both ? "none" : target.favor === "bad" ? "good" : target.favor === "good" ? "bad" : "none";
    return { from: index, to: target.index, kind: "control", state: !both ? "broken" : item.count >= 2 && target.count >= 2 ? "clash" : "plain", effect };
  });

  const points: FlowPoint[] = [];
  const label = (item: FlowElement) => `${item.name}（${item.role}）`;

  // 0. 合化、成局改了五行（有才列）
  const changeSteps = shift.steps.slice(0, firstLuckStep).filter((step) => !step.inactive && (step.bonus || step.effects.some((effect) => effect.dir === "change")));
  if (changeSteps.length) {
    const touched = new Set<number>();
    const parts = changeSteps.map((step) => {
      const changed = step.effects.filter((effect) => effect.dir === "change" && effect.toElement);
      for (const effect of changed) {
        const index = elements.findIndex((item) => item.name === effect.toElement);
        if (index >= 0) touched.add(index);
      }
      if (step.bonus) {
        const index = elements.findIndex((item) => step.bonus!.label.startsWith(item.name));
        if (index >= 0) touched.add(index);
        return `${step.title}，${step.bonus.label}多算一份合力`;
      }
      const to = changed[0]?.toElement ?? "";
      const role = elements.find((item) => item.name === to)?.role;
      return `${step.title}，${list(changed.map((effect) => effect.cell))}改算${to}${role ? `（${role}）` : ""}`;
    });
    points.push({
      key: "change",
      tone: "info",
      title: "合化、成局改了五行",
      text: `${parts.join("；")}。圖上的字數已經照變化後來算${shift.changed ? `（同黨：異黨從 ${shift.before.same}：${shift.before.opp} 變成 ${shift.after.same}：${shift.after.opp}）` : ""}。`,
      elements: [...touched],
    });
  }

  // 1. 哪一行多、缺哪一行，氣流不流得過去
  const max = Math.max(...elements.map((item) => item.count));
  const most = elements.filter((item) => item.count === max);
  const missing = elements.filter((item) => item.count === 0);
  const single = elements.filter((item) => item.count === 1);
  const mostText = `${list(most.map(label))} 最多，${most.length > 1 ? "各 " : ""}${max} 個字`;
  if (!missing.length) {
    points.push({
      key: "spread",
      tone: "good",
      title: "五行俱全，相生不斷",
      text:
        `${mostText}${single.length ? `；${list(single.map(label))} 只有 1 個字` : ""}。` +
        `每一行都有字，${elements.map((item) => item.name).join(" → ")} → ${elements[0].name} 可以一路相生、循環不斷。`,
      elements: elements.map((item) => item.index),
    });
  } else {
    const missingText = missing
      .map((item) => `缺${label(item)}${item.hidden.length ? `（只藏在${list(item.hidden)}）` : "（連藏干都沒有）"}`)
      .join("、");
    const parts = chainsOf(elements).map((chain) => {
      const last = chain[chain.length - 1];
      const next = elements[(last.index + 1) % 5];
      const path = chain.map((item) => item.name).join(" → ");
      return chain.length > 1 ? `${path}，流到${last.name}就停（缺${next.name}接不下去）` : `${last.name}孤立，生不出去（缺${next.name}）`;
    });
    const seal = elements[4];
    const output = elements[1];
    const self = [seal.count === 0 ? `沒有${seal.name}（印）來生日主` : "", output.count === 0 ? `日主的氣沒有${output.name}（食傷）可以洩` : ""].filter(Boolean);
    points.push({
      key: "spread",
      tone: "bad",
      title: `缺${list(missing.map((item) => item.name))}，相生接不起來`,
      text: `${mostText}；${missingText}。相生：${parts.join("；")}。${self.length ? `${self.join("，")}。` : ""}`,
      elements: [...most, ...missing].map((item) => item.index),
    });
  }

  // 3. 月令
  const month = natal.find((cell) => cell.isMonthCommand);
  if (month) {
    const owner = elements.find((item) => item.name === month.element);
    const hits = downMarks(shift, month);
    // 月令參與的沖、刑、合（成立的才列）
    const monthActs = shift.steps
      .filter((step) => !step.inactive && (step.group === "branch" || step.group === "luck" || step.group === "annual") && step.cells.includes(month.id))
      .map((step) => step.title);
    points.push({
      key: "month",
      tone: shift.lights.season ? "good" : hits.length ? "bad" : "info",
      title: shift.lights.season ? "得令" : "不得令",
      text:
        `月令${month.char}是${month.element}${owner ? `（${owner.role}，${owner.roleText}）` : ""}，` +
        (shift.lights.season ? "和日主同一行，日主當令。" : owner?.party === "same" ? "不是日主同一行，但是生日主的印，月令站在日主這邊。" : "站在異黨那邊。") +
        (monthActs.length ? `月令有 ${list(monthActs)}${hits.length ? `，月令${list([...new Set(hits)])}，力量打折` : "，月令沒有受傷"}。` : ""),
      elements: owner ? [owner.index] : [],
    });
  }

  // 4. 相剋而且兩邊都有力：看有沒有通關（幾組都收在同一句）
  const clashes = controls.filter((item) => item.state === "clash");
  if (clashes.length) {
    const rows = clashes.map((edge) => ({ from: elements[edge.from], to: elements[edge.to], bridge: elements[(edge.from + 1) % 5] }));
    const lacking = rows.filter((row) => !row.bridge.count);
    points.push({
      key: "clash",
      tone: lacking.length ? "bad" : "info",
      title: lacking.length
        ? `${list(lacking.map((row) => `${row.from.name}剋${row.to.name}`))}，缺${list([...new Set(lacking.map((row) => row.bridge.name))])}通關`
        : rows.length > 2
          ? `${rows.length} 組有力的相剋，都有通關`
          : `${list(rows.map((row) => `${row.from.name}剋${row.to.name}`))}，有${list([...new Set(rows.map((row) => row.bridge.name))])}通關`,
      text:
        rows.length > 2
          ? // 組數多時只列一行一組，免得太長
            rows
              .map((row) => `${row.from.name}${row.from.count}剋${row.to.name}${row.to.count}，${row.bridge.count ? `有${row.bridge.name}通關` : `缺${row.bridge.name}通關`}`)
              .join("；") + `。（數字是字數；兩邊都有兩個字以上才算有力的相剋，中間那一行有字就能通關）`
          : rows
              .map(
                (row) =>
                  `${label(row.from)} ${row.from.count} 個字剋 ${label(row.to)} ${row.to.count} 個字，兩邊都有力；` +
                  (row.bridge.count
                    ? `中間有${label(row.bridge)}可以通關（${row.from.name}生${row.bridge.name}、${row.bridge.name}生${row.to.name}）。`
                    : `中間沒有${label(row.bridge)}通關，是直接相剋。`),
              )
              .join(""),
      elements: [...new Set(rows.flatMap((row) => [row.from.index, row.to.index, row.bridge.index]))],
    });
  }

  // 5. 天干虛實：無根、截腳的天干是虛的
  const branches = natal.filter((cell) => cell.row === "branch");
  const stems = natal.filter((cell) => cell.row === "stem");
  const who = (cell: Pick<ShiftCell, "id" | "char" | "family">) => `${cell.id}${cell.char}（${cell.family}）`;
  const rootless = stems.filter((stem) => !stem.transformed && !branches.some((branch) => rootLevel(stem.char, branch.char)));
  const cut = stems.filter((stem) =>
    shift.steps.some((step) => step.group === "pillar" && step.effects.some((effect) => effect.cell === stem.id && effect.text === "截腳")),
  );
  if (rootless.length || cut.length) {
    const weak = [...new Set([...rootless, ...cut])];
    points.push({
      key: "hollow",
      tone: weak.some((cell) => cell.isDayMaster || elements.find((item) => item.name === cell.element)?.favor === "good") ? "bad" : "info",
      title: "虛浮的天干",
      text:
        (rootless.length ? `${list(rootless.map(who))}在地支沒有根，浮在天干上。` : "") +
        (cut.length ? `${list(cut.map(who))}坐在剋它的地支上（截腳），有字但使不上力。` : "") +
        "其餘天干都有根。",
      elements: [...new Set(weak.map((cell) => elements.findIndex((item) => item.name === cell.element)))].filter((index) => index >= 0),
    });
  }

  // 6. 強弱與用神
  const after = shift.after;
  const good = elements.filter((item) => item.favor === "good");
  const bad = elements.filter((item) => item.favor === "bad");
  const useGods = elements.filter((item) => item.isUseGod);
  const useText = useGods
    .map((item) => {
      if (!item.count) return `用神${item.name}局中沒有字${item.hidden.length ? `（只藏在${list(item.hidden)}）` : ""}，要等大運流年補`;
      const hurt = natal
        .filter((cell) => cell.element === item.name && !cell.isDayMaster)
        .flatMap((cell) => [...new Set(downMarks(shift, cell))].map((mark) => `${cell.char}${mark}`));
      const own = natal.filter((cell) => cell.element === item.name && !cell.isDayMaster).map((cell) => `${cell.id}${cell.char}`);
      return `用神${item.name}${own.length ? `在${list(own)}` : "只有日主自己"}${hurt.length ? `，其中${list(hurt)}` : ""}`;
    })
    .join("；");
  // 課堂的判定和喜忌是照原局人數下的；合化、成局改了人數時，另外標出「可能翻轉」。
  const before = shift.before;
  const classification = before.classification || after.classification;
  const flipTo = after.classification && after.classification !== classification ? after.classification : null;
  if (classification) {
    points.push({
      key: "verdict",
      tone: flipTo ? "bad" : "info",
      title: classification + (good.length || bad.length ? `，喜${good.map((item) => item.name).join("") || "—"}、忌${bad.map((item) => item.name).join("") || "—"}` : "") + (flipTo ? "（可能翻轉）" : ""),
      text:
        (shift.changed
          ? `原局同黨（印、比劫）${before.same} 個字：異黨 ${before.opp} 個字，課堂判「${classification}」，喜忌照這個判定。合化、成局之後變成 ${after.same}：${after.opp}` +
            (flipTo ? `，偏向「${flipTo}」，判定可能翻轉，建議用走過的大運反驗。` : "，判定不變。")
          : `同黨（印、比劫）${after.same} 個字：異黨 ${after.opp} 個字，課堂判「${classification}」。`) +
        (useText ? `${useText}。` : "") +
        (!shift.changed && shift.caution ? "作用的方向和人數相反，是邊界盤，建議用走過的大運反驗。" : ""),
      elements: [...good, ...useGods].map((item) => item.index),
    });
  }

  return {
    dayStem: dayCell.char,
    dayElement: shift.dayElement,
    classification,
    flipTo,
    extraLabel: null,
    // 有加運時取時間軸最後一格（含運的人數）；判定仍是原局的
    same: shift.timeline[shift.timeline.length - 1]?.same ?? after.same,
    opp: shift.timeline[shift.timeline.length - 1]?.opp ?? after.opp,
    elements,
    generates,
    controls,
    points,
  };
}
