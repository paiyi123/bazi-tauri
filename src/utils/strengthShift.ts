import type { LessonThreeStepAnalysis } from "../types/bazi";
import { elementFromName, seatTransforms } from "./lessonPreference";

// 「強弱變化圖」：把課堂三步法已經判好的合、沖、刑、會，整理成「哪個字被加強、被削弱、換邊」，
// 讓畫面可以一步一步演給初學者看。這裡不做新的判斷：
// - 人數（同黨：異黨）的增減，和 Rust 的「作用變化後強弱複核」完全相同（合化改黨、成局該黨加一）。
// - 其他作用（沖、合絆、六合、刑、同柱、天干相剋）只標方向（▲增力、▼減力），不給分數；
//   方向都取自課堂卡片已有的結論（10-07、10-14、10-21、10-28、11-04 與講義第 23–26 章）。

export type ShiftParty = "same" | "opp";
/** up 增力、down 減力、change 換五行（合化）、bind 被合絆 */
export type ShiftDir = "up" | "down" | "change" | "bind";
export type ShiftGroup = "stem" | "branch" | "pillar" | "luck" | "annual";

export interface ShiftCell {
  /** 例：「年干」「月支」 */
  id: string;
  /** 0 年、1 月、2 日、3 時、4 大運 */
  pillar: number;
  /** 這個字是後來加上的大運，不是原局 */
  luck?: boolean;
  row: "stem" | "branch";
  char: string;
  /** 五行中文名（地支取本氣） */
  element: string;
  family: string;
  tenGod: string;
  party: ShiftParty;
  isDayMaster: boolean;
  isMonthCommand: boolean;
  /** 藏干裡有日主同五行（得地的根） */
  isRoot: boolean;
}

export interface ShiftEffect {
  cell: string;
  dir: ShiftDir;
  /** 貼在字上的短標籤，例如「化木」「沖敗」「截腳」 */
  text: string;
  /** 力量較輕（隔位）：畫面用淡色，不列入方向統計 */
  light?: boolean;
  /** 合化後的五行、黨別、十神類別 */
  toElement?: string;
  toParty?: ShiftParty;
  toFamily?: string;
  /** 不列入方向統計（例如合來：只是貼向日主，不算增減） */
  neutral?: boolean;
}

export interface ShiftStep {
  id: string;
  group: ShiftGroup;
  /** 作用種類，例如「合而能化」「三會」「六沖」 */
  tag: string;
  title: string;
  cells: string[];
  effects: ShiftEffect[];
  /** 會改變同黨、異黨人數 */
  counted: boolean;
  deltaSame: number;
  deltaOpp: number;
  /** 成局給哪一黨加一（畫在拔河條上的「合力」） */
  bonus?: { party: ShiftParty; label: string };
  /** 不成立的作用（合而不合、不成局）：列出來讓人知道為什麼不算 */
  inactive: boolean;
  /** 課堂卡片對這張盤的說明 */
  detail: string;
  /** 這一類作用的通則（一句話） */
  rule: string;
}

export interface ShiftCount {
  same: number;
  opp: number;
  momentum: boolean;
  classification: string;
  /** 人數已經把大運的字算進去（判定仍是原局的，大運不改判） */
  withLuck?: boolean;
}

export interface ShiftLuckInput {
  /** 大運干支，例如「戊辰」 */
  ganZhi: string;
  /** 顯示用的說明，例如「23–33 歲」 */
  label?: string;
}

export interface ShiftLuckInfo {
  ganZhi: string;
  label: string;
  /** 大運的第一步在 steps 裡的位置（之前都是原局的作用） */
  firstStep: number;
  /** 正數：這步運把力量推向日主這邊；負數：推向異黨 */
  lean: number;
  leanText: string;
  /** 照扶抑大方向的一句話；特別格局候選時為 null */
  advice: string | null;
  /** 被大運沖到的日主的根 */
  rootShaken: string[];
  /** 大運地支藏有日主同五行（給日主帶根） */
  bringsRoot: boolean;
  trend: ShiftTrend;
}

export interface ShiftTrend {
  sameUp: number;
  sameDown: number;
  oppUp: number;
  oppDown: number;
  /** 正數偏向日主這邊，負數偏向異黨 */
  net: number;
}

export interface StrengthShift {
  dayElement: string;
  cells: ShiftCell[];
  steps: ShiftStep[];
  before: ShiftCount;
  after: ShiftCount;
  changed: boolean;
  monthCommandSameParty: boolean;
  lights: { season: boolean; rooted: boolean; rootShaken: string[] };
  /** 每一步之後的人數與判定（index 0 是原局，index k 是做完第 k 步） */
  timeline: ShiftCount[];
  trend: ShiftTrend;
  summary: string[];
  /** 人數判定和方向相反：容易有爭議的盤 */
  caution: string | null;
  /** 有加大運時的資訊 */
  luck: ShiftLuckInfo | null;
  /** 有加流年時的資訊（流年接在大運後面） */
  annual: ShiftLuckInfo | null;
}

const GENERATES: Record<string, string> = { 木: "火", 火: "土", 土: "金", 金: "水", 水: "木" };
const CONTROLS: Record<string, string> = { 木: "土", 土: "水", 水: "火", 火: "金", 金: "木" };
const PILLARS = "年月日時";
const PRESSURE = ["食傷", "財", "官殺"];

export function familyOf(dayElement: string, element: string): string {
  if (element === dayElement) return "比劫";
  if (GENERATES[element] === dayElement) return "印";
  if (GENERATES[dayElement] === element) return "食傷";
  if (CONTROLS[dayElement] === element) return "財";
  return "官殺";
}

const partyOf = (family: string): ShiftParty => (family === "印" || family === "比劫" ? "same" : "opp");
const partyName = (party: ShiftParty) => (party === "same" ? "同黨" : "異黨");

/** 「年干丁（正印）」「月支戌（土，比劫）」「時支寅」→「年干」「月支」「時支」 */
function cellIdOf(position: string): string | null {
  const match = position.match(/^([年月日時])([干支])/);
  return match ? `${match[1]}${match[2]}` : null;
}

function classify(analysis: LessonThreeStepAnalysis, same: number, opp: number): ShiftCount {
  const strength = analysis.strength;
  const tie = same === opp;
  const momentum = same > opp || (tie && strength.monthCommandSameParty);
  const observations = Number(strength.sameSeasonElement) + Number(momentum) + Number(strength.rootCount > 0);
  const surface = strength.pressureFamiliesSurface ?? [];
  let classification: string;
  if (observations === 3) classification = surface.length ? "旺候選" : "旺極候選";
  else if (observations === 2) classification = "強候選";
  else if (observations === 1) classification = tie && strength.monthCommandSameParty ? "強候選" : "弱候選";
  else classification = PRESSURE.every((family) => surface.includes(family)) ? "衰極候選" : "依據不足";
  return { same, opp, momentum, classification };
}

function buildCells(analysis: LessonThreeStepAnalysis): ShiftCell[] {
  const dayElement = analysis.facts.dayMaster.element;
  const rootIds = new Set(
    (analysis.facts.roots ?? []).map((root) => cellIdOf(root.pillar)).filter((id): id is string => !!id),
  );
  const cells: ShiftCell[] = [];
  analysis.facts.pillars.forEach((pillar, index) => {
    const name = pillar.pillar.charAt(0);
    const pillarIndex = PILLARS.indexOf(name) >= 0 ? PILLARS.indexOf(name) : index;
    const isDay = pillarIndex === 2;
    const stemFamily = familyOf(dayElement, pillar.stemElement);
    cells.push({
      id: `${name}干`,
      pillar: pillarIndex,
      row: "stem",
      char: pillar.stem,
      element: pillar.stemElement,
      family: isDay ? "日主" : stemFamily,
      tenGod: pillar.stemTenGod,
      party: partyOf(stemFamily),
      isDayMaster: isDay,
      isMonthCommand: false,
      isRoot: false,
    });
    const branchFamily = familyOf(dayElement, pillar.branchMainElement);
    cells.push({
      id: `${name}支`,
      pillar: pillarIndex,
      row: "branch",
      char: pillar.branch,
      element: pillar.branchMainElement,
      family: branchFamily,
      tenGod: pillar.hiddenStems?.[0]?.tenGod ?? "",
      party: partyOf(branchFamily),
      isDayMaster: false,
      isMonthCommand: pillarIndex === 1,
      isRoot: rootIds.has(`${name}支`),
    });
  });
  return cells;
}

const RULES = {
  transformed: "合而能化：兩個天干都改用化神的五行來看，所以可能換邊（年月之合要時支見辰、坐支是化神的根，10-21）。",
  dayTransformed: "日干自己參與的合如果能化，是化氣格，要改看特別格局，不在這裡數人頭（10-14）。",
  bound: "合而不化＝互相牽絆。年月之合叫「合去」，被絆的字不為日主所用；日干之合叫「合來」，那個字心向日主（10-14）。",
  blocked: "合而不合：中間隔了一個字，或其中一干被旁邊剋住，就不論合，照一般生剋看（10-14）。",
  contested: "爭合、妒合：兩個字搶合同一個字，合力分散，只論牽絆（10-14）。",
  stemControl: "天干生剋看遠近：相鄰力大、隔一位力小、遙隔更輕；被剋的字作用減弱（10-07）。",
  stemGenerate: "天干相鄰相生，被生的字得力；去生別人的字會「貪生忘剋」（10-07）。",
  gathering: "三合、三會不論化，只看合力成不成氣。成局後那個五行力量大增（三會＞三合＞半合），成員原本的五行還在；局神剋到的成員會受制（10-28、講義第 26 章）。",
  unformed: "不成局就照一般生剋看：局神不在月令又被隔開、或缺中神的半合（要歲運填實）都不算（10-28）。",
  clash: "沖看貼近、得令、黨多定勝負：敗的一方被削弱；弱的去沖旺的，反而把旺的激得更旺。沖到月令的不易倒，沖在日時根容易鬆動（10-21）。",
  storage: "辰戌丑未互沖：土得令或黨多時愈沖愈旺；土不旺時根基反而動搖，不可套「墓庫逢沖則發」（10-21）。",
  sixCombine: "地支六合不化：生合（寅亥、辰酉、午未）加強被生的一方，剋合（子丑、卯戌、巳申）削減被剋的一方，被合的字都還在（10-21）。",
  punish: "刑只讀成本氣的生剋，不另論吉凶；刑卡到月令那個五行更旺，刑在日時反而鬆動（11-04）。",
  pillar: "同一柱：天干坐在生它、同它的地支上有力（通根）；坐在剋它的地支上是截腳、虛浮無力（講義第 25 章）。",
};

const PILLAR_LABELS: Record<string, string> = {
  same: "干支同類",
  branch_generates_stem: "地支生天干",
  branch_controls_stem: "地支剋天干",
  stem_controls_branch: "天干剋地支",
  stem_generates_branch: "天干生地支",
};
const PILLAR_NOTES: Record<string, string> = {
  same: "干支同類，通根有力。",
  branch_generates_stem: "地支生天干，坐下得生。",
  branch_controls_stem: "地支剋天干，截腳無力。",
  stem_controls_branch: "天干剋地支，蓋頭。",
  stem_generates_branch: "天干生地支，坐下洩氣。",
};

/** 把課堂卡片的結果整理成步驟。順序照講義「各個擊破」：天干 → 地支 → 同柱。 */
export function strengthShift(analysis?: LessonThreeStepAnalysis | null, luck?: ShiftLuckInput | null, annual?: ShiftLuckInput | null): StrengthShift | null {
  if (!analysis?.facts?.pillars?.length || !analysis.strength) return null;
  const dayElement = analysis.facts.dayMaster.element;
  const cells = buildCells(analysis);
  const cellMap = new Map(cells.map((cell) => [cell.id, cell]));
  const idsOf = (positions: string[]) =>
    [...new Set(positions.map(cellIdOf).filter((id): id is string => !!id && cellMap.has(id)))];
  const steps: ShiftStep[] = [];
  const push = (step: Omit<ShiftStep, "counted" | "deltaSame" | "deltaOpp" | "inactive"> & Partial<ShiftStep>) =>
    steps.push({ counted: false, deltaSame: 0, deltaOpp: 0, inactive: false, ...step });

  // ── 天干五合 ──
  for (const combo of analysis.stemInteractions?.combinations ?? []) {
    const ids = idsOf(combo.positions);
    const title = `${combo.stems}${combo.kindLabel}${combo.kind === "transformed" ? combo.element : ""}`;
    if (combo.kind === "transformed" && combo.scope === "日干以外之合") {
      const family = familyOf(dayElement, combo.element);
      const toParty = partyOf(family);
      const effects: ShiftEffect[] = [];
      let deltaSame = 0;
      let deltaOpp = 0;
      for (const id of ids) {
        const cell = cellMap.get(id)!;
        if (cell.isDayMaster || cell.element === combo.element) continue;
        effects.push({ cell: id, dir: "change", text: `化${combo.element}`, toElement: combo.element, toParty, toFamily: family });
        if (cell.party !== toParty) {
          deltaSame += toParty === "same" ? 1 : -1;
          deltaOpp += toParty === "opp" ? 1 : -1;
        }
      }
      push({ id: combo.id, group: "stem", tag: "合而能化", title, cells: ids, effects, counted: true, deltaSame, deltaOpp, detail: combo.reason, rule: RULES.transformed });
    } else if (combo.kind === "transformed") {
      push({ id: combo.id, group: "stem", tag: "化氣", title, cells: ids, effects: [], detail: combo.reason, rule: RULES.dayTransformed });
    } else if (combo.kind === "bound" || combo.kind === "contested") {
      const coming = combo.direction === "合來";
      const bound = idsOf(combo.boundStems?.length ? combo.boundStems : combo.positions).filter((id) => !cellMap.get(id)!.isDayMaster);
      const effects: ShiftEffect[] = bound.map((id) => ({
        cell: id,
        dir: "bind",
        text: combo.kind === "contested" ? "爭合" : coming ? "合來" : "被絆",
        neutral: coming || combo.kind === "contested",
      }));
      push({
        id: combo.id,
        group: "stem",
        tag: combo.kind === "contested" ? "爭合妒合" : coming ? "合來" : "合去",
        title,
        cells: ids,
        effects,
        detail: combo.reason,
        rule: combo.kind === "contested" ? RULES.contested : RULES.bound,
      });
    } else {
      push({ id: combo.id, group: "stem", tag: "合而不合", title, cells: ids, effects: [], inactive: true, detail: combo.reason, rule: RULES.blocked });
    }
  }

  // ── 天干生剋：只列日主以外、看得出強弱方向的（相鄰的生、相鄰或隔一位的剋） ──
  for (const relation of analysis.stemInteractions?.relations ?? []) {
    if (relation.status === "ignored") continue;
    const from = cellIdOf(relation.from);
    const to = cellIdOf(relation.to);
    if (!from || !to || !cellMap.has(from) || !cellMap.has(to)) continue;
    if (cellMap.get(from)!.isDayMaster || cellMap.get(to)!.isDayMaster) continue;
    const adjacent = relation.distance === "鄰近";
    const near = adjacent || relation.distance === "隔一位";
    const light = !adjacent || relation.status === "weak";
    if (relation.relation === "剋" && near) {
      push({
        id: relation.id,
        group: "stem",
        tag: "天干相剋",
        title: `${from}${cellMap.get(from)!.char}剋${to}${cellMap.get(to)!.char}`,
        cells: [from, to],
        effects: [{ cell: to, dir: "down", text: light ? "受剋（輕）" : "受剋", light }],
        detail: relation.reason,
        rule: RULES.stemControl,
      });
    } else if (relation.relation === "生" && adjacent && relation.status === "active") {
      push({
        id: relation.id,
        group: "stem",
        tag: "天干相生",
        title: `${from}${cellMap.get(from)!.char}生${to}${cellMap.get(to)!.char}`,
        cells: [from, to],
        effects: [{ cell: to, dir: "up", text: "受生" }],
        detail: relation.reason,
        rule: RULES.stemGenerate,
      });
    }
  }

  // ── 地支：三會、三合、半合 ──
  const branches = analysis.branchInteractions;
  for (const item of branches?.gatherings ?? []) {
    const ids = idsOf(item.positions);
    const counts = item.formed && ["three_combination", "three_meeting", "half_combination", "earth_full"].includes(item.kind);
    if (!counts) {
      push({ id: item.id, group: "branch", tag: "不成局", title: item.kindLabel, cells: ids, effects: [], inactive: true, detail: item.reason, rule: RULES.unformed });
      continue;
    }
    const party = partyOf(familyOf(dayElement, item.element));
    const effects: ShiftEffect[] = [];
    for (const id of ids) {
      const cell = cellMap.get(id)!;
      if (cell.element === item.element) effects.push({ cell: id, dir: "up", text: `成${item.element}局` });
      else if (CONTROLS[item.element] === cell.element) effects.push({ cell: id, dir: "down", text: `入${item.element}局受制` });
    }
    push({
      id: item.id,
      group: "branch",
      tag: item.kind === "three_meeting" ? "三會" : item.kind === "three_combination" ? "三合" : item.kind === "earth_full" ? "四庫全" : "半合",
      title: item.kindLabel,
      cells: ids,
      effects,
      counted: true,
      deltaSame: party === "same" ? 1 : 0,
      deltaOpp: party === "opp" ? 1 : 0,
      bonus: { party, label: `${item.element}局` },
      detail: `${item.reason}（${item.forceLabel}）`,
      rule: RULES.gathering,
    });
  }

  // ── 地支：六沖 ──
  for (const item of branches?.clashes ?? []) {
    const ids = idsOf(item.positions);
    const effects: ShiftEffect[] = [];
    const light = item.distance === "遙隔";
    if (item.outcome === "storage") {
      const stronger = `${item.verdictLabel}${item.reason}`.includes("愈旺");
      for (const id of ids) effects.push({ cell: id, dir: stronger ? "up" : "down", text: stronger ? "愈沖愈旺" : "根基動搖", light });
    } else if (item.outcome === "both_damaged") {
      for (const id of ids) effects.push({ cell: id, dir: "down", text: "沖・兩傷", light });
    } else {
      const loser = item.loser ? cellIdOf(item.loser) : null;
      const winner = item.winner ? cellIdOf(item.winner) : null;
      if (loser && cellMap.has(loser)) effects.push({ cell: loser, dir: "down", text: "沖敗", light });
      if (winner && cellMap.has(winner) && (item.notes ?? []).some((note) => note.includes("衰神沖旺"))) {
        effects.push({ cell: winner, dir: "up", text: "激旺", light });
      }
    }
    push({
      id: item.id,
      group: "branch",
      tag: item.outcome === "storage" ? "四庫沖" : "六沖",
      title: `${item.branches}沖（${item.outcomeLabel}）`,
      cells: ids,
      effects,
      detail: item.reason,
      rule: item.outcome === "storage" ? RULES.storage : RULES.clash,
    });
  }

  // ── 地支：六合 ──
  for (const item of branches?.combinations ?? []) {
    const ids = idsOf(item.positions);
    const [first, second] = ids.map((id) => cellMap.get(id)!);
    const effects: ShiftEffect[] = [];
    if (first && second) {
      const light = !item.adjacent;
      const suffix = light ? "（輕）" : "";
      if (item.kind === "generating") {
        const receiver = GENERATES[first.element] === second.element ? second : GENERATES[second.element] === first.element ? first : null;
        if (receiver) effects.push({ cell: receiver.id, dir: "up", text: `生合${suffix}`, light });
      } else {
        const victim = CONTROLS[first.element] === second.element ? second : CONTROLS[second.element] === first.element ? first : null;
        if (victim) effects.push({ cell: victim.id, dir: "down", text: `剋合${suffix}`, light });
      }
    }
    push({ id: item.id, group: "branch", tag: item.kindLabel, title: `${item.branches}合（${item.kindLabel}）`, cells: ids, effects, detail: item.reason, rule: RULES.sixCombine });
  }

  // ── 地支：刑（害屬神煞，不列） ──
  for (const item of branches?.punishments ?? []) {
    if (item.kind === "harm") continue;
    const ids = idsOf(item.positions);
    const members = ids.map((id) => cellMap.get(id)!);
    const pillars = [...new Set(members.map((cell) => PILLARS[cell.pillar]))].join("");
    const effects: ShiftEffect[] = [];
    const same = item.relation.match(/同屬(.)/);
    if (same) {
      const atMonth = pillars.includes("月");
      const loose = !atMonth && members.every((cell) => cell.pillar >= 2);
      for (const cell of members) {
        effects.push({ cell: cell.id, dir: loose ? "down" : "up", text: loose ? "刑在日時・鬆動" : atMonth ? "刑在月令・加強" : "同氣加強" });
      }
    } else {
      for (const match of item.relation.matchAll(/(.)(生|剋)(.)/g)) {
        for (const cell of members.filter((member) => member.element === match[3])) {
          if (effects.some((effect) => effect.cell === cell.id)) continue;
          effects.push({ cell: cell.id, dir: match[2] === "生" ? "up" : "down", text: match[2] === "生" ? "刑・受生" : "刑・受剋" });
        }
      }
    }
    const loosened = effects.some((effect) => effect.text.includes("鬆動"));
    push({
      id: item.id,
      group: "branch",
      tag: item.kind === "self_punishment" ? "自刑" : "三刑",
      title: item.kindLabel.replace(/（兼六害）/, ""),
      cells: ids,
      effects,
      detail: `${item.relation}。${item.reason}${loosened ? "（老師 11-04 補充：這個刑落在日時，沒有卡到月令，這個五行反而鬆動、減弱，所以圖上標成減力。）" : ""}`,
      rule: RULES.punish,
    });
  }

  // ── 同柱干支 ──
  const transformedTo = new Map<string, string>();
  for (const step of steps) for (const effect of step.effects) if (effect.dir === "change" && effect.toElement) transformedTo.set(effect.cell, effect.toElement);
  for (const item of analysis.pillarRelations?.relations ?? []) {
    const name = item.pillar.charAt(0);
    const stem = cellMap.get(`${name}干`);
    const branch = cellMap.get(`${name}支`);
    if (!stem || !branch) continue;
    const effects: ShiftEffect[] = [];
    // 天干已經合化的，改用化神的五行來看它坐的地支
    const changedTo = transformedTo.get(stem.id);
    const stemElement = changedTo ?? stem.element;
    const kind = !changedTo
      ? item.kind
      : stemElement === branch.element
        ? "same"
        : GENERATES[branch.element] === stemElement
          ? "branch_generates_stem"
          : CONTROLS[branch.element] === stemElement
            ? "branch_controls_stem"
            : CONTROLS[stemElement] === branch.element
              ? "stem_controls_branch"
              : "stem_generates_branch";
    if (kind === "same") effects.push({ cell: stem.id, dir: "up", text: "通根" });
    else if (kind === "branch_generates_stem") effects.push({ cell: stem.id, dir: "up", text: "坐下得生" });
    else if (kind === "branch_controls_stem") effects.push({ cell: stem.id, dir: "down", text: "截腳" });
    else if (kind === "stem_controls_branch") effects.push({ cell: branch.id, dir: "down", text: "蓋頭", light: true });
    else if (kind === "stem_generates_branch") effects.push({ cell: stem.id, dir: "down", text: "坐下洩氣", light: true });
    const changedNote = changedTo
      ? `（${stem.id}${stem.char}已合化為${changedTo}，圖上改用${changedTo}來看：${PILLAR_NOTES[kind] ?? ""}）`
      : "";
    push({
      id: item.id,
      group: "pillar",
      tag: "同柱",
      title: `${item.pillar}${item.ganZhi}：${changedTo ? PILLAR_LABELS[kind] ?? item.kindLabel : item.kindLabel}`,
      cells: [stem.id, branch.id],
      effects,
      detail: `${item.reason}${changedNote}`,
      rule: RULES.pillar,
    });
  }

  // ── 人數與判定 ──
  const strength = analysis.strength;
  const review = analysis.interactionStrength;
  const before = classify(analysis, strength.samePartyCount, strength.opposingPartyCount);
  before.classification = strength.classification;
  before.momentum = strength.momentumProxy ?? before.momentum;
  const timeline: ShiftCount[] = [before];
  let same = strength.samePartyCount;
  let opp = strength.opposingPartyCount;
  for (const step of steps) {
    same += step.deltaSame;
    opp += step.deltaOpp;
    timeline.push(classify(analysis, Math.max(same, 0), Math.max(opp, 0)));
  }
  const computed = timeline[timeline.length - 1];
  const after: ShiftCount = review
    ? { same: review.samePartyCount, opp: review.opposingPartyCount, momentum: review.momentum, classification: review.classification }
    : computed;
  // 步驟加總若和 Rust 的複核不同，以 Rust 為準：中間各步只顯示人數，不顯示判定。
  const consistent = computed.same === after.same && computed.opp === after.opp;
  if (!consistent || computed.classification !== after.classification) {
    for (let index = 1; index < timeline.length; index += 1) timeline[index].classification = "";
  }
  timeline[timeline.length - 1] = after;

  // ── 方向統計（不含已經算進人數的合化、成局；淡色的輕微作用不列） ──
  const trend = tallyTrend(cells, steps, steps.length);

  const rootShaken = cells
    .filter((cell) => cell.isRoot)
    .filter((cell) => steps.some((step) => step.tag.includes("沖") && step.effects.some((effect) => effect.cell === cell.id && effect.dir === "down" && !effect.light)))
    .map((cell) => `${cell.id}${cell.char}`);

  const changed = review?.changed ?? after.classification !== before.classification;
  const active = steps.filter((step) => !step.inactive);
  const counted = active.filter((step) => step.counted);
  const summary: string[] = [];
  if (!active.length) {
    summary.push("原局沒有會改變強弱的合、沖、刑、會，強弱照得令、得勢、得地三項看就可以。");
  } else if (!counted.length) {
    summary.push(`同黨 ${before.same}：異黨 ${before.opp}，沒有合化或成局，人數不變，判定仍是「${before.classification}」。`);
  } else {
    summary.push(
      `作用前同黨 ${before.same}：異黨 ${before.opp}（${before.classification}）→ 作用後 ${after.same}：${after.opp}（${after.classification}）。改變人數的是：${counted.map((step) => step.title).join("、")}。`,
    );
  }
  const directional = trend.sameUp + trend.sameDown + trend.oppUp + trend.oppDown;
  if (directional) {
    summary.push(
      `只看方向的作用：對日主這邊有利 ${trend.sameUp + trend.oppDown} 項（同黨增力 ${trend.sameUp}、異黨減力 ${trend.oppDown}），對異黨那邊有利 ${trend.sameDown + trend.oppUp} 項（同黨減力 ${trend.sameDown}、異黨增力 ${trend.oppUp}），${
        trend.net > 0 ? "整體偏向日主這一邊" : trend.net < 0 ? "整體偏向異黨那一邊" : "兩邊大致相抵"
      }。`,
    );
  }
  if (rootShaken.length) summary.push(`日主的根 ${rootShaken.join("、")} 被沖，得地要打折。`);

  const strongSide = /強|旺/.test(after.classification);
  const weakSide = /弱|衰/.test(after.classification);
  let caution: string | null = null;
  if (changed) caution = "作用變化後判定可能翻轉，請人工複核（課堂判定仍以三步法為主）。";
  else if (strongSide && trend.net <= -2) caution = "人數判在身強這邊，但多項作用在削弱同黨或加強異黨，屬於容易有爭議的盤，建議用過去的大運反驗，或在課堂上問老師。";
  else if (weakSide && trend.net >= 2) caution = "人數判在身弱這邊，但多項作用在加強同黨或削弱異黨，屬於容易有爭議的盤，建議用過去的大運反驗，或在課堂上問老師。";

  const shift: StrengthShift = {
    dayElement,
    cells,
    steps,
    before,
    after,
    changed,
    monthCommandSameParty: strength.monthCommandSameParty,
    lights: { season: strength.sameSeasonElement, rooted: strength.rootCount > 0, rootShaken },
    timeline,
    trend,
    summary,
    caution,
    luck: null,
    annual: null,
  };
  if (luck?.ganZhi) appendLuck(shift, luck);
  if (annual?.ganZhi) appendLuck(shift, annual, "流年");
  return shift;
}

// ───────────────────────── 大運 ─────────────────────────
// 原局的強弱是「命」，不因大運改判。加上大運只回答一個問題：這十年把天平往哪一邊推。
// 規則都是課堂已講過的：大運天干五合化不化看坐支（10-07）、合而不化為合絆（10-14）、
// 沖與六合（10-21）、三合三會填實（10-28）、刑讀成本氣生剋與命運相沖（11-04）。

const STEMS = "甲乙丙丁戊己庚辛壬癸";
const STEM_ELEMENT: Record<string, string> = { 甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水" };
const BRANCH_MAIN_STEM: Record<string, string> = { 子: "癸", 丑: "己", 寅: "甲", 卯: "乙", 辰: "戊", 巳: "丙", 午: "丁", 未: "己", 申: "庚", 酉: "辛", 戌: "戊", 亥: "壬" };
const BRANCH_HIDDEN_ELEMENTS: Record<string, string> = { 子: "水", 丑: "土水金", 寅: "木火土", 卯: "木", 辰: "土木水", 巳: "火金土", 午: "火土", 未: "土火木", 申: "金水土", 酉: "金", 戌: "土金火", 亥: "水木" };
const STEM_COMBINE: Record<string, [string, string]> = { 甲: ["己", "土"], 己: ["甲", "土"], 乙: ["庚", "金"], 庚: ["乙", "金"], 丙: ["辛", "水"], 辛: ["丙", "水"], 丁: ["壬", "木"], 壬: ["丁", "木"], 戊: ["癸", "火"], 癸: ["戊", "火"] };
const BRANCH_CLASH: Record<string, string> = { 子: "午", 午: "子", 丑: "未", 未: "丑", 寅: "申", 申: "寅", 卯: "酉", 酉: "卯", 辰: "戌", 戌: "辰", 巳: "亥", 亥: "巳" };
const BRANCH_SIX: Record<string, string> = { 子: "丑", 丑: "子", 寅: "亥", 亥: "寅", 卯: "戌", 戌: "卯", 辰: "酉", 酉: "辰", 巳: "申", 申: "巳", 午: "未", 未: "午" };
const THREE_SETS: Array<{ kind: "三會" | "三合"; members: string; element: string }> = [
  { kind: "三會", members: "寅卯辰", element: "木" },
  { kind: "三會", members: "巳午未", element: "火" },
  { kind: "三會", members: "申酉戌", element: "金" },
  { kind: "三會", members: "亥子丑", element: "水" },
  { kind: "三合", members: "申子辰", element: "水" },
  { kind: "三合", members: "亥卯未", element: "木" },
  { kind: "三合", members: "寅午戌", element: "火" },
  { kind: "三合", members: "巳酉丑", element: "金" },
];
const PUNISH_PAIRS = ["寅巳", "巳申", "丑戌", "戌未", "子卯"];
const SELF_PUNISH = "辰午酉亥";

function tenGodOf(dayStem: string, stem: string): string {
  const day = STEMS.indexOf(dayStem);
  const other = STEMS.indexOf(stem);
  if (day < 0 || other < 0) return "";
  const samePolarity = day % 2 === other % 2;
  const names: Record<string, [string, string]> = {
    比劫: ["比肩", "劫財"],
    食傷: ["食神", "傷官"],
    財: ["偏財", "正財"],
    官殺: ["七殺", "正官"],
    印: ["偏印", "正印"],
  };
  return names[familyOf(STEM_ELEMENT[dayStem], STEM_ELEMENT[stem])]?.[samePolarity ? 0 : 1] ?? "";
}

const LUCK_RULES = {
  enter: "大運是十年的環境。原局的強弱（命）不因大運改判，大運只看它的天干、地支各幫哪一邊，把天平往哪邊推。",
  pillar: RULES.pillar,
  combine: "大運天干和原局天干五合，化不化看大運自己坐的地支（10-07）：坐支是化神的旺地就化，兩個字都當化神看；不化就是互相牽絆，被絆的字暫時不好用（10-14）。",
  contested: RULES.contested,
  dayCombine: "大運天干合日主：日主不因大運而化，這個字貼向日主。課堂講的「合來」是原局的規則，用在大運上屬於推論。",
  clash: "大運地支沖原局地支：被沖的字動搖、減力；被沖的若是日主的根，得地要打折（10-21）。",
  stir: "命運相沖不一定是大運贏：大運沖月令，而原局沒有大運那個五行時，沖不動得令的月令，反而把它激得更旺（11-04）。",
  storage: RULES.storage,
  six: RULES.sixCombine,
  gathering: "大運的地支補上原局缺的那個字（填實），三合或三會成局，那個五行在這十年力量大增；課堂只論合力、不論化（10-28）。",
  half: "大運和原局一個地支半合（要有子午卯酉當中神），只算那個五行略增，不算成局（10-28）。",
  punish: RULES.punish,
  echo: "大運地支和原局地支相同（伏吟）：那個五行再多一份力；辰午酉亥自己見自己也叫自刑，同樣讀成同氣加強（11-04）。",
};

function appendLuck(shift: StrengthShift, luck: ShiftLuckInput, which: "大運" | "流年" = "大運"): void {
  // 大運、流年用同一套規則：流年再和原局＋大運比（流年的字標「歲干／歲支」）。
  const K = which;
  const SPAN = which === "大運" ? "這十年" : "這一年";
  const STEP = which === "大運" ? "這步運" : "這個流年";
  const IDP = which === "大運" ? "luck" : "annual";
  const PFX = which === "大運" ? "運" : "歲";
  const [stemChar, branchChar] = [...luck.ganZhi];
  const stemElement = STEM_ELEMENT[stemChar];
  const branchMain = BRANCH_MAIN_STEM[branchChar];
  const dayCell = shift.cells.find((cell) => cell.isDayMaster);
  if (!stemElement || !branchMain || !dayCell) return;
  const branchElement = STEM_ELEMENT[branchMain];
  const day = shift.dayElement;
  const firstStep = shift.steps.length;
  const natal = cellStatesAt(shift, firstStep);
  // 流年進來時，大運的字也算在「已經在的字」裡
  const natalCells = [...shift.cells];
  const natalStems = natalCells.filter((cell) => cell.row === "stem");
  const natalBranches = natalCells.filter((cell) => cell.row === "branch");
  const elementNow = (cell: ShiftCell) => natal.get(cell.id)?.element ?? cell.element;
  const partyNow = (cell: ShiftCell) => natal.get(cell.id)?.party ?? cell.party;
  const where = (cell: ShiftCell) => `${cell.id}${cell.char}`;

  const stemFamily = familyOf(day, stemElement);
  const branchFamily = familyOf(day, branchElement);
  const bringsRoot = (BRANCH_HIDDEN_ELEMENTS[branchChar] ?? "").includes(day);
  const stemCell: ShiftCell = { id: `${PFX}干`, pillar: which === "大運" ? 4 : 5, luck: true, row: "stem", char: stemChar, element: stemElement, family: stemFamily, tenGod: tenGodOf(dayCell.char, stemChar), party: partyOf(stemFamily), isDayMaster: false, isMonthCommand: false, isRoot: false };
  const branchCell: ShiftCell = { id: `${PFX}支`, pillar: which === "大運" ? 4 : 5, luck: true, row: "branch", char: branchChar, element: branchElement, family: branchFamily, tenGod: tenGodOf(dayCell.char, branchMain), party: partyOf(branchFamily), isDayMaster: false, isMonthCommand: false, isRoot: bringsRoot };
  shift.cells.push(stemCell, branchCell);

  const steps: ShiftStep[] = [];
  const push = (step: Omit<ShiftStep, "counted" | "deltaSame" | "deltaOpp" | "inactive" | "group"> & Partial<ShiftStep>) =>
    steps.push({ counted: false, deltaSame: 0, deltaOpp: 0, inactive: false, group: which === "大運" ? "luck" : "annual", ...step });
  const rootShaken: string[] = [];

  // 1. 大運兩個字進來
  const count = (party: ShiftParty) => Number(stemCell.party === party) + Number(branchCell.party === party);
  push({
    id: `${IDP}-enter`,
    tag: K,
    title: `${K}${luck.ganZhi}進來`,
    cells: [stemCell.id, branchCell.id],
    effects: [],
    counted: true,
    deltaSame: count("same"),
    deltaOpp: count("opp"),
    detail: `天干${stemChar}屬${stemElement}，是${stemCell.tenGod}，站在${partyName(stemCell.party)}；地支${branchChar}本氣屬${branchElement}，是${branchCell.tenGod}，站在${partyName(branchCell.party)}。${
      bringsRoot ? `${branchChar}裡藏有${day}，日主在這十年多一個根${shift.lights.rooted ? "" : "（原局無根，這是「無根見根」）"}。` : ""
    }`,
    rule: LUCK_RULES.enter,
  });

  // 2. 大運這一柱自己：天干坐在什麼地支上
  {
    const kind =
      stemElement === branchElement
        ? "same"
        : GENERATES[branchElement] === stemElement
          ? "branch_generates_stem"
          : CONTROLS[branchElement] === stemElement
            ? "branch_controls_stem"
            : CONTROLS[stemElement] === branchElement
              ? "stem_controls_branch"
              : "stem_generates_branch";
    const effects: ShiftEffect[] = [];
    if (kind === "same") effects.push({ cell: stemCell.id, dir: "up", text: "通根" });
    else if (kind === "branch_generates_stem") effects.push({ cell: stemCell.id, dir: "up", text: "坐下得生" });
    else if (kind === "branch_controls_stem") effects.push({ cell: stemCell.id, dir: "down", text: "截腳" });
    else if (kind === "stem_controls_branch") effects.push({ cell: branchCell.id, dir: "down", text: "蓋頭", light: true });
    else effects.push({ cell: stemCell.id, dir: "down", text: "坐下洩氣", light: true });
    push({
      id: `${IDP}-pillar`,
      tag: "同柱",
      title: `${K}${luck.ganZhi}：${PILLAR_LABELS[kind]}`,
      cells: [stemCell.id, branchCell.id],
      effects,
      detail: `${stemChar}${stemElement}坐${branchChar}${branchElement}：${PILLAR_NOTES[kind]}`,
      rule: LUCK_RULES.pillar,
    });
  }

  // 3. 大運天干五合原局天干（已經在原局合化的天干不再算）
  const combine = STEM_COMBINE[stemChar];
  if (combine) {
    const [partner, combined] = combine;
    const matches = natalStems.filter((cell) => cell.char === partner && !natal.get(cell.id)?.transformed);
    const name = `${stemChar}${partner}`;
    if (matches.length > 1) {
      push({
        id: `${IDP}-combine`,
        tag: "爭合妒合",
        title: `${K}${stemChar}與${matches.map(where).join("、")}爭合`,
        cells: [stemCell.id, ...matches.map((cell) => cell.id)],
        effects: matches.filter((cell) => !cell.isDayMaster).map((cell) => ({ cell: cell.id, dir: "bind" as const, text: "爭合", neutral: true })),
        detail: `原局有 ${matches.length} 個${partner}，一個${stemChar}合不了兩邊，合力分散，只論牽絆。`,
        rule: LUCK_RULES.contested,
      });
    } else if (matches.length === 1) {
      const target = matches[0];
      const combinedKey = elementFromName(combined);
      const engaged = shift.steps.find((step) => step.group === "stem" && ["合來", "合去", "爭合妒合"].includes(step.tag) && step.cells.includes(target.id));
      if (engaged && !target.isDayMaster) {
        push({
          id: `${IDP}-combine`,
          tag: "五合（待確認）",
          title: `${K}${stemChar}也來合${where(target)}`,
          cells: [stemCell.id, target.id],
          effects: [{ cell: target.id, dir: "bind", text: "又被運合", neutral: true }],
          detail: `${where(target)}在原局已經有合（${engaged.title}），${K}再來一個${stemChar}。這算爭合，還是照樣看${K}坐支論化，課堂還沒講到，圖上先不算，請自己判斷或問老師。`,
          rule: LUCK_RULES.contested,
        });
      } else if (target.isDayMaster) {
        push({
          id: `${IDP}-combine`,
          tag: "合日主",
          title: `${K}${stemChar}合日主${partner}`,
          cells: [stemCell.id, target.id],
          effects: [{ cell: stemCell.id, dir: "bind", text: "合日主", neutral: true }],
          detail: `${K}${stemChar}（${stemCell.tenGod}）與日主${partner}相合，這個字在${SPAN}貼向日主。日主本身不因${K}而化。`,
          rule: LUCK_RULES.dayCombine,
        });
      } else if (combinedKey && seatTransforms(branchChar, combinedKey)) {
        const family = familyOf(day, combined);
        const toParty = partyOf(family);
        const effects: ShiftEffect[] = [];
        let deltaSame = 0;
        let deltaOpp = 0;
        for (const [cell, party, element] of [
          [target, partyNow(target), elementNow(target)],
          [stemCell, stemCell.party, stemCell.element],
        ] as Array<[ShiftCell, ShiftParty, string]>) {
          if (element === combined) continue;
          effects.push({ cell: cell.id, dir: "change", text: `化${combined}`, toElement: combined, toParty, toFamily: family });
          if (party !== toParty) {
            deltaSame += toParty === "same" ? 1 : -1;
            deltaOpp += toParty === "opp" ? 1 : -1;
          }
        }
        push({
          id: `${IDP}-combine`,
          tag: "合而能化",
          title: `${K}${name}合化${combined}`,
          cells: [stemCell.id, target.id],
          effects,
          counted: true,
          deltaSame,
          deltaOpp,
          detail: `${K}${stemChar}與${where(target)}相合，${K}坐${branchChar}是${combined}的旺地，所以合而能化：${SPAN}兩個字都當${combined}（${family}，${partyName(toParty)}）看。`,
          rule: LUCK_RULES.combine,
        });
      } else {
        push({
          id: `${IDP}-combine`,
          tag: "合絆",
          title: `${K}${name}合而不化`,
          cells: [stemCell.id, target.id],
          effects: [
            { cell: target.id, dir: "bind", text: "被運合絆" },
            { cell: stemCell.id, dir: "bind", text: "合絆" },
          ],
          detail: `${K}${stemChar}與${where(target)}相合，但${K}坐${branchChar}不是${combined}的旺地，合而不化：兩個字互相牽絆，${SPAN}都不太好用。`,
          rule: LUCK_RULES.combine,
        });
      }
    }
  }

  // 4. 大運地支填實三合、三會；半合
  const natalChars = natalBranches.map((cell) => cell.char);
  const formedElements = new Set<string>();
  for (const set of THREE_SETS) {
    if (!set.members.includes(branchChar)) continue;
    const others = [...set.members].filter((char) => char !== branchChar);
    if (natalChars.includes(branchChar) && others.every((char) => natalChars.includes(char))) continue; // 原局已經三字全
    const present = others.filter((char) => natalChars.includes(char));
    const members = natalBranches.filter((cell) => present.includes(cell.char));
    if (present.length === 2) {
      const party = partyOf(familyOf(day, set.element));
      const effects: ShiftEffect[] = [];
      for (const cell of [...members, branchCell]) {
        if (cell.element === set.element) effects.push({ cell: cell.id, dir: "up", text: `成${set.element}局` });
        else if (CONTROLS[set.element] === cell.element) effects.push({ cell: cell.id, dir: "down", text: `入${set.element}局受制` });
      }
      formedElements.add(set.element);
      push({
        id: `${IDP}-${set.members}`,
        tag: `${set.kind}（填實）`,
        title: `${K}${branchChar}填實${set.members}${set.kind}${set.element}局`,
        cells: [branchCell.id, ...members.map((cell) => cell.id)],
        effects,
        counted: true,
        deltaSame: party === "same" ? 1 : 0,
        deltaOpp: party === "opp" ? 1 : 0,
        bonus: { party, label: `${set.element}局` },
        detail: `原局有${members.map(where).join("、")}，${K}補上${branchChar}，${set.members}${set.kind}成${set.element}局；${set.element}是${familyOf(day, set.element)}，給${partyName(party)}加一。`,
        rule: LUCK_RULES.gathering,
      });
    } else if (set.kind === "三合" && present.length === 1 && !formedElements.has(set.element)) {
      const pair = branchChar + present[0];
      if (![..."子午卯酉"].some((middle) => pair.includes(middle))) continue;
      const effects: ShiftEffect[] = [...members, branchCell]
        .filter((cell) => cell.element === set.element)
        .map((cell) => ({ cell: cell.id, dir: "up" as const, text: `半合${set.element}`, light: true }));
      push({
        id: `${IDP}-half-${set.members}`,
        tag: "半合",
        title: `${K}${branchChar}與${members.map(where).join("、")}半合${set.element}`,
        cells: [branchCell.id, ...members.map((cell) => cell.id)],
        effects,
        detail: `${pair}半合${set.element}，缺第三個字，不算成局，只算${set.element}略增。`,
        rule: LUCK_RULES.half,
      });
    }
  }

  // 5. 大運地支對原局每一個地支：沖、六合、刑、伏吟
  const natalHasLuckElement = natalCells.some((cell) => elementNow(cell) === branchElement);
  const earthCount = natalCells.filter((cell) => elementNow(cell) === "土").length;
  const monthBranch = natalBranches.find((cell) => cell.isMonthCommand);
  for (const cell of natalBranches) {
    const pairCells = [branchCell.id, cell.id];
    if (BRANCH_CLASH[branchChar] === cell.char) {
      if (branchElement === "土" && cell.element === "土") {
        const stronger = monthBranch?.element === "土" || earthCount >= 3;
        push({
          id: `${IDP}-clash-${cell.id}`,
          tag: "四庫沖",
          title: `${K}${branchChar}沖${where(cell)}（四庫相沖）`,
          cells: pairCells,
          effects: pairCells.map((id) => ({ cell: id, dir: stronger ? ("up" as const) : ("down" as const), text: stronger ? "愈沖愈旺" : "根基動搖" })),
          detail: stronger ? `原局土${monthBranch?.element === "土" ? "得令" : `有 ${earthCount} 個`}，庫沖之後土反而更旺。` : "原局土不旺，庫沖之後根基動搖。",
          rule: LUCK_RULES.storage,
        });
      } else if (monthBranch && cell.element === monthBranch.element && !natalHasLuckElement && !formedElements.has(branchElement)) {
        push({
          id: `${IDP}-clash-${cell.id}`,
          tag: "六沖（反激）",
          title: `${K}${branchChar}沖${where(cell)}（沖不動${cell.isMonthCommand ? "月令" : "旺神"}）`,
          cells: pairCells,
          effects: [
            { cell: cell.id, dir: "up", text: "激旺" },
            { cell: branchCell.id, dir: "down", text: "沖不動" },
          ],
          detail: `原局沒有${branchElement}，${K}${branchChar}孤身來沖得令的${cell.element}（${where(cell)}），沖不贏，反而把${cell.element}激得更旺。`,
          rule: LUCK_RULES.stir,
        });
      } else {
        if (cell.isRoot) rootShaken.push(where(cell));
        push({
          id: `${IDP}-clash-${cell.id}`,
          tag: "六沖",
          title: `${K}${branchChar}沖${where(cell)}`,
          cells: pairCells,
          effects: [{ cell: cell.id, dir: "down", text: cell.isMonthCommand ? "沖提・月令動" : "被運沖" }],
          detail: `${branchChar}${cell.char}相沖，原局${where(cell)}（${cell.element}，${partyName(partyNow(cell))}）在${SPAN}被沖動、減力。${cell.isRoot ? "它是日主的根，得地要打折。" : ""}${
            cell.isMonthCommand && !natalHasLuckElement ? `原局雖然沒有${branchElement}，但這步運同時填實成${branchElement}局，不算孤身來沖。` : ""
          }`,
          rule: LUCK_RULES.clash,
        });
      }
      continue;
    }
    if (BRANCH_SIX[branchChar] === cell.char) {
      const effects: ShiftEffect[] = [];
      let kind = "合";
      let note = "";
      if (GENERATES[branchElement] === cell.element || GENERATES[cell.element] === branchElement) {
        const receiver = GENERATES[branchElement] === cell.element ? cell : branchCell;
        effects.push({ cell: receiver.id, dir: "up", text: "生合" });
        kind = "生合";
        note = `加強${receiver.element}`;
      } else if (CONTROLS[branchElement] === cell.element || CONTROLS[cell.element] === branchElement) {
        const victim = CONTROLS[branchElement] === cell.element ? cell : branchCell;
        effects.push({ cell: victim.id, dir: "down", text: "剋合" });
        kind = "剋合";
        note = `削減${victim.element}`;
      }
      push({
        id: `${IDP}-six-${cell.id}`,
        tag: kind,
        title: `${K}${branchChar}合${where(cell)}（${kind}）`,
        cells: pairCells,
        effects,
        detail: `${branchChar}${cell.char}六合不化，讀成${branchElement}與${cell.element}的生剋：${note}，兩個字都還在。`,
        rule: LUCK_RULES.six,
      });
      continue;
    }
    if (branchChar === cell.char) {
      push({
        id: `${IDP}-echo-${cell.id}`,
        tag: SELF_PUNISH.includes(branchChar) ? "伏吟・自刑" : "伏吟",
        title: `${K}${branchChar}與${where(cell)}相同`,
        cells: pairCells,
        effects: [{ cell: cell.id, dir: "up", text: "同氣加重" }],
        detail: `${K}地支和${where(cell)}同一個字，${cell.element}（${partyName(partyNow(cell))}）再多一份力。`,
        rule: LUCK_RULES.echo,
      });
      continue;
    }
    if (PUNISH_PAIRS.some((pair) => pair.includes(branchChar) && pair.includes(cell.char))) {
      const effects: ShiftEffect[] = [];
      let note: string;
      if (branchElement === cell.element) {
        effects.push({ cell: cell.id, dir: "up", text: "同氣加強" });
        note = `同屬${cell.element}，加強${cell.element}`;
      } else if (GENERATES[branchElement] === cell.element || GENERATES[cell.element] === branchElement) {
        const receiver = GENERATES[branchElement] === cell.element ? cell : branchCell;
        effects.push({ cell: receiver.id, dir: "up", text: "刑・受生" });
        note = `${receiver === cell ? branchElement : cell.element}生${receiver.element}`;
      } else {
        const victim = CONTROLS[branchElement] === cell.element ? cell : branchCell;
        effects.push({ cell: victim.id, dir: "down", text: "刑・受剋" });
        note = `${victim === cell ? branchElement : cell.element}剋${victim.element}`;
      }
      push({
        id: `${IDP}-punish-${cell.id}`,
        tag: "刑",
        title: `${K}${branchChar}刑${where(cell)}`,
        cells: pairCells,
        effects,
        detail: `${branchChar}${cell.char}相刑，只讀成本氣生剋：${note}。`,
        rule: LUCK_RULES.punish,
      });
    }
  }

  // ── 接到步驟與時間軸後面 ──
  shift.steps.push(...steps);
  const base = shift.timeline[shift.timeline.length - 1] ?? shift.after;
  let { same, opp } = base;
  for (const step of steps) {
    same += step.deltaSame;
    opp += step.deltaOpp;
    shift.timeline.push({ same: Math.max(same, 0), opp: Math.max(opp, 0), momentum: shift.after.momentum, classification: shift.after.classification, withLuck: true });
  }
  const trend = tallyTrend(shift.cells, shift.steps, shift.steps.length, firstStep);
  const headcount = same - base.same - (opp - base.opp);
  // 先看人數（大運兩個字、合化換邊、填實成局），人數相抵時才看只標方向的作用。
  const lean = headcount !== 0 ? headcount : trend.net;
  const leanText = lean > 0 ? "把力量推向日主這一邊（同黨得助）" : lean < 0 ? "把力量推向異黨那一邊（剋、洩、耗增加）" : "兩邊大致相抵";
  const label = shift.after.classification;
  const special = /旺極|衰極|依據不足/.test(label);
  const strong = !special && /強|旺/.test(label);
  const weak = !special && /弱/.test(label);
  let advice: string | null = null;
  if (lean === 0) advice = `${STEP}對強弱的影響不大，重點看它引動了哪幾個字。`;
  else if (strong) advice = lean < 0 ? `原局偏強，${STEP}增加剋、洩、耗，照扶抑的大方向是順的。` : `原局偏強，${STEP}又幫同黨，強者更強，照扶抑的大方向是逆的。`;
  else if (weak) advice = lean > 0 ? `原局偏弱，${STEP}來生扶，照扶抑的大方向是順的。` : `原局偏弱，${STEP}又增加剋、洩、耗，照扶抑的大方向是逆的。`;
  const info: ShiftLuckInfo = { ganZhi: luck.ganZhi, label: luck.label ?? "", firstStep, lean, leanText, advice, rootShaken, bringsRoot, trend };
  if (which === "大運") shift.luck = info;
  else shift.annual = info;
  shift.summary.push(
    `加上${K}${luck.ganZhi}${luck.label ? `（${luck.label}）` : ""}：${leanText}。${advice ?? "原局是特別格局候選或依據不足，扶抑的方向不一定適用，要先確認格局。"}細的喜忌仍以用神卡和${K}面板的 ▲▼ 為準。`,
  );
  if (rootShaken.length) shift.summary.push(`這步${K}沖到日主的根 ${rootShaken.join("、")}，${SPAN}得地要打折。`);
}

function tallyTrend(cells: ShiftCell[], steps: ShiftStep[], count: number, from = 0): ShiftTrend {
  const applied = steps.slice(0, Math.max(0, count));
  const party = new Map(cells.map((cell) => [cell.id, cell.party]));
  for (const step of applied) for (const effect of step.effects) if (effect.dir === "change" && effect.toParty) party.set(effect.cell, effect.toParty);
  const trend: ShiftTrend = { sameUp: 0, sameDown: 0, oppUp: 0, oppDown: 0, net: 0 };
  for (const step of applied.slice(from)) {
    if (step.inactive) continue;
    for (const effect of step.effects) {
      if (effect.light || effect.neutral || effect.dir === "change") continue;
      if (step.counted && effect.dir === "up") continue; // 成局的「合力」已經算成人數
      const up = effect.dir === "up";
      if (party.get(effect.cell) === "same") up ? (trend.sameUp += 1) : (trend.sameDown += 1);
      else up ? (trend.oppUp += 1) : (trend.oppDown += 1);
    }
  }
  trend.net = trend.sameUp - trend.sameDown - (trend.oppUp - trend.oppDown);
  return trend;
}

/** 做完前 count 步之後，「只看方向」的作用統計。 */
export function trendAt(shift: StrengthShift, count: number): ShiftTrend {
  return tallyTrend(shift.cells, shift.steps, count);
}

/** 這個作用對哪一邊有利：字增力就是幫它那一黨，字減力就是幫對面。 */
export function helpsParty(effect: ShiftEffect, party: ShiftParty): ShiftParty | null {
  if (effect.neutral) return null;
  if (effect.dir === "up") return party;
  if (effect.dir === "down" || effect.dir === "bind") return party === "same" ? "opp" : "same";
  return effect.toParty ?? null;
}

export interface ShiftCellState {
  element: string;
  party: ShiftParty;
  family: string;
  marks: ShiftEffect[];
  transformed: boolean;
}

/** 做完前 count 步之後，每個字的樣子（count = 0 是原局）。 */
export function cellStatesAt(shift: StrengthShift, count: number): Map<string, ShiftCellState> {
  const states = new Map<string, ShiftCellState>(
    shift.cells.map((cell) => [cell.id, { element: cell.element, party: cell.party, family: cell.family, marks: [], transformed: false }]),
  );
  for (const step of shift.steps.slice(0, Math.max(0, count))) {
    for (const effect of step.effects) {
      const state = states.get(effect.cell);
      if (!state) continue;
      state.marks.push(effect);
      if (effect.dir === "change" && effect.toElement && effect.toParty) {
        state.element = effect.toElement;
        state.party = effect.toParty;
        state.family = effect.toFamily ?? state.family;
        state.transformed = true;
      }
    }
  }
  return states;
}

/** 做完前 count 步之後，成局加給兩黨的「合力」。 */
export function bonusesAt(shift: StrengthShift, count: number): Array<{ id: string; party: ShiftParty; label: string }> {
  return shift.steps
    .slice(0, Math.max(0, count))
    .filter((step) => step.bonus)
    .map((step) => ({ id: step.id, party: step.bonus!.party, label: step.bonus!.label }));
}

export { partyName };
