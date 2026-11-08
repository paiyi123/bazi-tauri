import type { LessonThreeStepAnalysis } from "../types/bazi";
import { lessonPreferences, type Favor } from "./lessonPreference";
import { specialPatternReport } from "./specialPatterns";

/**
 * 「用神喜忌判斷」卡：把課堂三步法的條件方案整理成一張看得懂的結論。
 *
 * 只整理、不改判：用神、喜神、忌神都來自課堂三步法的方案（src-tauri lesson_analysis/preferences.rs）
 * 與 lessonPreferences() 的合併結果。這裡另外做的事：
 * - 從各方案挑出最多方案共同的用神；方案彼此不一致時照實列出。
 * - 把「忌」細分成忌神（直接剋用神）與仇神（生忌神）；老師上課把兩者合稱為忌（2026-09-02）。
 * - 用神在原局的位置與力量（透干、通根、只藏支中、不見）。
 * - 相連檢查（2026-09-02「土要帶金」、10-21「金與水相連就沒關係」）：用神旁邊是誰在生它、剋它。
 * - 原局合沖碰到喜用神的項目（2026-10-14 四句總訣、09-30 沖去忌神為好），直接引用天干、地支卡的判斷。
 */

export type Element = "木" | "火" | "土" | "金" | "水";
export type GodRole = "use" | "favorable" | "unfavorable" | "enemy" | "idle" | "mixed";

export interface GodRow {
  role: GodRole;
  roleLabel: string;
  element: Element;
  family: string;
  reason: string;
}

export interface UsePosition {
  pillar: string;
  place: string;
  char: string;
  surface: boolean;
}

export interface NeighborNote {
  at: string;
  supporters: string[];
  attackers: string[];
}

export interface InteractionNote {
  verdict: string;
  label: string;
  reason: string;
}

export interface YongShenStep {
  label: string;
  text: string;
}

export interface YongShenGuide {
  status: "decided" | "conflict" | "pending" | "insufficient";
  headline: string;
  steps: YongShenStep[];
  gods: GodRow[];
  useElements: Element[];
  positions: UsePosition[];
  strengthVerdict: string;
  neighbors: NeighborNote[];
  interactions: InteractionNote[];
  specialWarnings: string[];
  teacherSummary: string;
}

const ELEMENTS: Element[] = ["木", "火", "土", "金", "水"];
const GENERATES: Record<Element, Element> = { 木: "火", 火: "土", 土: "金", 金: "水", 水: "木" };
const CONTROLS: Record<Element, Element> = { 木: "土", 土: "水", 水: "火", 火: "金", 金: "木" };
const WUXING_KEY: Record<Element, "wood" | "fire" | "earth" | "metal" | "water"> = {
  木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water",
};
const STEM_ELEMENT: Record<string, Element> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
const BRANCH_ELEMENT: Record<string, Element> = {
  子: "水", 丑: "土", 寅: "木", 卯: "木", 辰: "土", 巳: "火", 午: "火", 未: "土", 申: "金", 酉: "金", 戌: "土", 亥: "水",
};
const ROLE_LABELS: Record<GodRole, string> = {
  use: "用神",
  favorable: "喜神",
  unfavorable: "忌神",
  enemy: "仇神",
  idle: "閒神",
  mixed: "依方案而定",
};
const ROLE_ORDER: GodRole[] = ["use", "favorable", "mixed", "unfavorable", "enemy", "idle"];

const controllerOf = (element: Element) => ELEMENTS.find((item) => CONTROLS[item] === element)!;

export function familyOf(day: Element, element: Element): string {
  if (element === day) return "比劫";
  if (GENERATES[day] === element) return "食傷";
  if (CONTROLS[day] === element) return "財";
  if (CONTROLS[element] === day) return "官殺";
  return "印";
}

const charElement = (char: string): Element | undefined => STEM_ELEMENT[char] ?? BRANCH_ELEMENT[char];

function natalPillars(analysis: LessonThreeStepAnalysis): string[] {
  const order = ["年柱", "月柱", "日柱", "時柱"];
  return order.map((name) => analysis.facts.pillars.find((pillar) => pillar.pillar === name)?.ganZhi ?? "");
}

export function yongShenGuide(analysis?: LessonThreeStepAnalysis | null): YongShenGuide | null {
  if (!analysis) return null;
  const day = analysis.facts.dayMaster.element as Element;
  const plans = analysis.preferences.plans;
  const merged = lessonPreferences(analysis);
  const favorOf = (element: Element): Favor => merged?.elements[WUXING_KEY[element]].favor ?? "neutral";

  // 1. 用神：各方案的取用方向，取最多方案共同指向的五行。
  const useCount = new Map<Element, string[]>();
  plans.forEach((plan) =>
    plan.useDirections.forEach((direction) => {
      const element = direction.element as Element;
      useCount.set(element, [...(useCount.get(element) ?? []), plan.title]);
    }),
  );
  const top = Math.max(0, ...[...useCount.values()].map((titles) => titles.length));
  const useElements = ELEMENTS.filter((element) => (useCount.get(element)?.length ?? 0) === top && top > 0);
  const conditional = [...new Set(plans.flatMap((plan) => plan.conditionalDirections.map((d) => `${d.family}（${d.element}）`)))];

  const strength = analysis.strength;
  const pattern = analysis.pattern;
  const selected = pattern.ordinaryCandidates.find((item) => item.selection === "selected") ?? pattern.ordinaryCandidates[0];
  const steps: YongShenStep[] = [];
  steps.push({
    label: "定格",
    text: selected
      ? `月令${analysis.facts.monthCommand.branch}，取${selected.monthHiddenStem}（${selected.hiddenStemLayer}）為格：${selected.pattern}。${selected.transparentLocations.length ? `透干於${selected.transparentLocations.join("、")}。` : "未透干，取本氣。"}`
      : `普通格候選依據不足。${pattern.conclusion}`,
  });
  steps.push({
    label: "判強弱",
    text: `八個字分兩黨：同黨（印、比劫）${strength.samePartyCount}，異黨（食傷、財、官殺）${strength.opposingPartyCount}${
      strength.samePartyCount === strength.opposingPartyCount
        ? `，打平時看月令，月令在${strength.monthCommandSameParty ? "同黨" : "異黨"}（得令者為旺）`
        : ""
    }；地支根氣 ${strength.rootCount} 處。判為「${strength.classification}」。`,
  });
  const strongSide = ["強候選", "旺候選", "旺極候選"].includes(strength.classification);
  const weakSide = ["弱候選", "衰極候選"].includes(strength.classification);
  steps.push({
    label: "扶抑方向",
    text: strongSide
      ? "身強要「抑」：從食傷（洩）、財（耗）、官殺（剋）裡選；身強任財官，官藏就看財。"
      : weakSide
        ? "身弱要「扶」：從印（生）、比劫（助）裡選，看局中什麼太多就對付什麼。"
        : "強弱依據不足，先不定扶抑方向。",
  });

  let status: YongShenGuide["status"];
  if (!plans.length) {
    status = "insufficient";
    steps.push({ label: "取用神", text: analysis.preferences.noPlanReasons.join("；") || "條件不足，未產生方案。" });
  } else if (!useElements.length) {
    status = "pending";
    steps.push({ label: "取用神", text: `沒有方案直接指定用神，要再比較：${conditional.join("、")}的實際位置與力量。` });
  } else {
    const alternatives = [...useCount.entries()].filter(([element]) => !useElements.includes(element));
    status = useElements.length > 1 ? "conflict" : "decided";
    const lines = [...useCount.entries()].map(
      ([element, titles]) => `${familyOf(day, element)}（${element}）← ${titles.join("、")}`,
    );
    steps.push({
      label: "取用神",
      text: `${lines.join("；")}。${
        status === "conflict"
          ? "兩個方向一樣多，要看哪一個在原局比較有力、比較不受制。"
          : alternatives.length
            ? `多數方案指向${useElements[0]}，取${useElements[0]}為用神；其他方向列為次選。`
            : `取${useElements[0]}為用神。`
      }`,
    });
  }

  // 2. 五神表。
  const gods: GodRow[] = [];
  const assigned = new Set<Element>();
  const push = (role: GodRole, element: Element, reason: string) => {
    if (assigned.has(element)) return;
    assigned.add(element);
    gods.push({ role, roleLabel: ROLE_LABELS[role], element, family: familyOf(day, element), reason });
  };
  useElements.forEach((element) => push("use", element, "方案的取用方向"));
  ELEMENTS.filter((element) => favorOf(element) === "mixed").forEach((element) =>
    push("mixed", element, "有的方案列喜、有的列忌，看採用哪一個方案"),
  );
  if (useElements.length === 1) {
    const use = useElements[0];
    const attacker = controllerOf(use);
    if (favorOf(attacker) === "good") {
      // 別的方案以它為用或喜，但它剋這裡選的用神：兩個方案不能同時成立。
      push("mixed", attacker, `其他方案列為喜用，但${attacker}剋${use}，和取${use}為用的方案衝突`);
    }
    ELEMENTS.filter((element) => favorOf(element) === "good").forEach((element) =>
      push(
        "favorable",
        element,
        GENERATES[element] === use
          ? `${element}生${use}，幫用神`
          : GENERATES[use] === element
            ? `${use}生${element}，讓用神有去處（流通）`
            : CONTROLS[element] === controllerOf(use)
              ? `${element}制住剋用神的${controllerOf(use)}`
              : "方案列為喜",
      ),
    );
    if (favorOf(attacker) !== "good") push("unfavorable", attacker, `${attacker}剋${use}，直接破用神`);
    ELEMENTS.filter((element) => favorOf(element) === "bad").forEach((element) => {
      const enemyOf = gods.find((row) => row.role === "unfavorable" && GENERATES[element] === row.element);
      if (enemyOf) push("enemy", element, `${element}生${enemyOf.element}，幫忌神（老師併入「忌」）`);
      else push("unfavorable", element, "方案列為忌");
    });
  } else {
    ELEMENTS.filter((element) => favorOf(element) === "good").forEach((element) => push("favorable", element, "方案列為喜"));
    ELEMENTS.filter((element) => favorOf(element) === "bad").forEach((element) => push("unfavorable", element, "方案列為忌"));
  }
  ELEMENTS.forEach((element) => {
    if (assigned.has(element)) return;
    const use = useElements.length === 1 ? useElements[0] : undefined;
    const reason = !use
      ? "方案未列喜忌"
      : GENERATES[use] === element
        ? `${use}生${element}，會洩用神；方案未列喜忌，看相連`
        : CONTROLS[use] === element
          ? `被${use}所剋；方案未列喜忌，看相連`
          : "方案未列喜忌，看相連";
    push("idle", element, reason);
  });
  gods.sort((a, b) => ROLE_ORDER.indexOf(a.role) - ROLE_ORDER.indexOf(b.role));

  const pick = (roles: GodRole[]) => gods.filter((row) => roles.includes(row.role)).map((row) => row.element).join("");
  const teacherSummary = gods.length && plans.length
    ? `喜${pick(["use", "favorable"]) || "（未定）"}，忌${pick(["unfavorable", "enemy"]) || "（未定）"}${pick(["mixed"]) ? `；${pick(["mixed"])}依方案而定` : ""}`
    : "";
  if (useElements.length) {
    steps.push({ label: "定喜忌", text: `喜神生用神或讓用神流通，忌神剋用神，仇神生忌神。老師說法：${teacherSummary}。` });
  }

  // 3. 用神在原局的位置與力量。
  const positions: UsePosition[] = [];
  const useSet = new Set<string>(useElements);
  analysis.facts.pillars.forEach((pillar) => {
    if (pillar.pillar !== "日柱" && useSet.has(pillar.stemElement)) {
      positions.push({ pillar: pillar.pillar, place: "天干", char: pillar.stem, surface: true });
    }
    pillar.hiddenStems.forEach((hidden) => {
      if (useSet.has(hidden.element)) {
        positions.push({
          pillar: pillar.pillar,
          place: hidden.layer === "本氣" ? "地支本氣" : `${pillar.branch}藏${hidden.layer}`,
          char: hidden.layer === "本氣" ? pillar.branch : hidden.stem,
          surface: hidden.layer === "本氣",
        });
      }
    });
  });
  const stemShown = positions.some((item) => item.place === "天干");
  const rooted = positions.some((item) => item.place !== "天干" && !item.place.includes("餘氣"));
  const branchShown = positions.some((item) => item.place === "地支本氣");
  const strengthVerdict = !useElements.length
    ? "用神未定"
    : !positions.length
      ? (useSet.has(day) ? "除日主外原局不見比劫，要等大運補" : "原局不見用神，要等大運補上才會發（用神無力為下）")
      : stemShown && rooted
        ? "透干又通根，用神有力"
        : stemShown
          ? "透干但地支無根，虛浮，怕被剋"
          : branchShown
            ? "地支本氣可見（藏而顯），有一定力量"
            : "只藏在地支中氣、餘氣，力量弱，要等運引出";

  // 4. 相連：用神（天干或地支本氣）旁邊誰生它、誰剋它。
  const grid = analysis.facts.pillars.map((pillar) => ({
    pillar: pillar.pillar.replace("柱", ""),
    stem: pillar.stem,
    branch: pillar.branch,
  }));
  const describe = (char: string, label: string) => {
    const element = charElement(char)!;
    const favor = favorOf(element);
    const tag = favor === "good" ? "喜" : favor === "bad" ? "忌" : favor === "mixed" ? "依方案" : "閒";
    return `${label}${char}${element}（${tag}）`;
  };
  const neighbors: NeighborNote[] = [];
  grid.forEach((cell, index) => {
    const spots: Array<{ char: string; row: "stem" | "branch" }> = [
      { char: cell.stem, row: "stem" },
      { char: cell.branch, row: "branch" },
    ];
    spots.forEach(({ char, row }) => {
      const element = charElement(char)!;
      if (!useSet.has(element)) return;
      if (row === "stem" && cell.pillar === "日") return;
      const around: Array<{ char: string; label: string }> = [];
      if (row === "stem") around.push({ char: cell.branch, label: "下坐" });
      else around.push({ char: cell.stem, label: "上蓋" });
      [index - 1, index + 1].forEach((other) => {
        const next = grid[other];
        if (!next) return;
        around.push({ char: row === "stem" ? next.stem : next.branch, label: `鄰${next.pillar}${row === "stem" ? "干" : "支"}` });
      });
      const supporters = around.filter((item) => GENERATES[charElement(item.char)!] === element).map((item) => describe(item.char, item.label));
      const attackers = around.filter((item) => CONTROLS[charElement(item.char)!] === element).map((item) => describe(item.char, item.label));
      neighbors.push({ at: `${cell.pillar}${row === "stem" ? "干" : "支"}${char}`, supporters, attackers });
    });
  });

  // 5. 原局合沖碰到喜用神的項目（判斷沿用天干、地支卡）。
  const touches = (chars: string) =>
    [...chars].some((char) => {
      const element = charElement(char);
      return element ? favorOf(element) === "good" || useSet.has(element) : false;
    });
  const interactions: InteractionNote[] = [];
  analysis.stemInteractions?.combinations
    .filter((item) => touches(item.stems))
    .forEach((item) =>
      interactions.push({ verdict: item.verdict, label: `${item.stems}合：${item.kindLabel}${item.direction ? `・${item.direction}` : ""}`, reason: `${item.verdictLabel}。${item.reason}` }),
    );
  analysis.branchInteractions?.clashes
    .filter((item) => touches(item.branches))
    .forEach((item) =>
      interactions.push({ verdict: item.verdict, label: `${item.branches}沖：${item.outcomeLabel}`, reason: `${item.verdictLabel}。${item.reason}` }),
    );
  analysis.branchInteractions?.combinations
    .filter((item) => touches(item.branches))
    .forEach((item) =>
      interactions.push({ verdict: item.verdict, label: `${item.branches}六合：${item.kindLabel}`, reason: `${item.verdictLabel}。${item.reason}` }),
    );

  // 6. 特別格局：成立時喜忌要改成順勢，不能照扶抑。
  const specialWarnings: string[] = [];
  const report = specialPatternReport(natalPillars(analysis));
  if (report) {
    [...report.follow, report.prosperous, report.transform]
      .filter((item) => item && item.verdict !== "none")
      .forEach((item) =>
        specialWarnings.push(`${item!.name}（${item!.verdictLabel}）：若成立，喜忌改為順勢${item!.advice ? `——${item!.advice}` : ""}`),
      );
  }
  if (pattern.specialReview.required && !specialWarnings.length) {
    specialWarnings.push(`課堂特殊格審查：${pattern.specialReview.reasons.join("；")}`);
  }

  const headline =
    status === "decided"
      ? `用神${useElements[0]}（${familyOf(day, useElements[0])}）；${teacherSummary}`
      : status === "conflict"
        ? `用神候選${useElements.join("、")}並列，要比較原局力量`
        : status === "pending"
          ? `用神待比較：${conditional.join("、")}`
          : "條件不足，暫不定用神";

  return {
    status,
    headline,
    steps,
    gods,
    useElements,
    positions,
    strengthVerdict,
    neighbors,
    interactions,
    specialWarnings,
    teacherSummary,
  };
}
