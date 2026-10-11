/**
 * 十神組合與四大動神：依潘樂德《十神洩天機》（增訂第二版）整理。
 *
 * 書的前提是喜忌已知，所以這裡只標出書中講的十神組合與變動時機，
 * 再附上課堂三步法的喜忌給你自己判斷；不改分數，也不另算強弱。
 * 官殺混雜、傷官見官、比劫爭財已在「婚姻・財運・健康」面板，這裡不重複。
 */

import type { LessonPreferences } from "./lessonPreference";
import { elementFromName } from "./lessonPreference";
import type { LuckEvent } from "./luckEvents";
import { tenGod } from "./lifeTopics";

export interface ComboItem {
  key: string;
  label: string;
  detail: string;
  /** 書中的看法：good＝成格或化凶，bad＝破格或助凶，note＝中性提醒。 */
  tone: "good" | "bad" | "note";
  /** 課堂三步法對相關五行的喜忌，例如「七殺(金)忌、正印(水)喜用」。 */
  favorNote?: string;
}

export interface TenGodOverview {
  /** 月令取出的十神領袖。 */
  leader: string;
  leaderSource: string;
  regular: number;
  irregular: number;
  items: ComboItem[];
}

const STEM_ELEMENT: Record<string, string> = {
  甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水",
};
const HIDDEN: Record<string, string[]> = {
  子: ["癸"], 丑: ["己", "癸", "辛"], 寅: ["甲", "丙", "戊"], 卯: ["乙"], 辰: ["戊", "乙", "癸"], 巳: ["丙", "戊", "庚"],
  午: ["丁", "己"], 未: ["己", "丁", "乙"], 申: ["庚", "壬", "戊"], 酉: ["辛"], 戌: ["戊", "辛", "丁"], 亥: ["壬", "甲"],
};
const REN: Record<string, string> = { 甲: "卯", 丙: "午", 戊: "午", 庚: "酉", 壬: "子" };
const STORAGE = "辰戌丑未";
const PILLARS = ["年", "月", "日", "時"];
/** 四正神順用，四偏神逆用（p36–44）。 */
const REGULAR = ["正官", "正印", "正財", "偏財", "食神"];
const MOVING: Record<string, { label: string; detail: string }> = {
  七殺: { label: "七殺", detail: "七殺主被迫變動：被裁、人事鬥爭、壓力太大而離開" },
  傷官: { label: "傷官", detail: "傷官主主動求變：嫌舊環境沉悶、想接受新挑戰，書稱「傷官的驛馬」" },
  偏財: { label: "偏財", detail: "偏財主為錢而動：被待遇吸引轉職、兼職副業、轉行" },
  比肩: { label: "比劫", detail: "比劫主一時衝動：一時不快就突然辭職，沒有特別理由" },
  劫財: { label: "比劫", detail: "比劫主一時衝動：一時不快就突然辭職，沒有特別理由" },
};

const stemOf = (ganZhi: string) => ganZhi.charAt(0);
const branchOf = (ganZhi: string) => ganZhi.charAt(1);

interface Spot {
  god: string;
  stem: string;
  pillar: number;
  visible: boolean;
}

/** 七個字：三個透出的天干（不含日干）加四個地支本氣。 */
function spots(natal: string[]): Spot[] {
  const dayStem = stemOf(natal[2]);
  const result: Spot[] = [];
  natal.forEach((pillar, index) => {
    if (index !== 2) result.push({ god: tenGod(dayStem, stemOf(pillar)), stem: stemOf(pillar), pillar: index, visible: true });
    const main = HIDDEN[branchOf(pillar)]?.[0];
    if (main) result.push({ god: tenGod(dayStem, main), stem: main, pillar: index, visible: false });
  });
  return result;
}

/** 月令取十神領袖：子午卯酉寅申巳亥取本氣，辰戌丑未取透干的藏干，都不透則取本氣（p65–69）。 */
export function monthLeader(natal: string[]): { god: string; source: string } {
  const dayStem = stemOf(natal[2]);
  const month = branchOf(natal[1]);
  const hidden = HIDDEN[month] ?? [];
  const shown = natal.filter((_, index) => index !== 2).map(stemOf);
  if (STORAGE.includes(month)) {
    const out = hidden.find((stem) => shown.includes(stem));
    if (out) return { god: tenGod(dayStem, out), source: `月令${month}中${out}透干` };
  }
  return { god: tenGod(dayStem, hidden[0]), source: `月令${month}本氣${hidden[0]}` };
}

function favorText(prefs: LessonPreferences | null | undefined, natal: string[], gods: string[]): string | undefined {
  if (!prefs) return undefined;
  const dayStem = stemOf(natal[2]);
  const parts = gods.map((god) => {
    const stem = "甲乙丙丁戊己庚辛壬癸".split("").find((item) => tenGod(dayStem, item) === god);
    const name = stem ? STEM_ELEMENT[stem] : "";
    const element = elementFromName(name);
    const label = element ? prefs.elements[element].label : "未列喜忌";
    return `${god}(${name})${label}`;
  });
  return `課堂喜忌：${parts.join("、")}`;
}

function where(list: Spot[], god: string): string {
  return list
    .filter((spot) => spot.god === god)
    .map((spot) => `${PILLARS[spot.pillar]}${spot.visible ? "干" : "支"}${spot.stem}`)
    .join("、");
}

export function tenGodOverview(natal: string[], prefs?: LessonPreferences | null): TenGodOverview | null {
  if (natal.length !== 4 || natal.some((pillar) => pillar.length < 2)) return null;
  const list = spots(natal);
  const has = (...gods: string[]) => list.some((spot) => gods.includes(spot.god));
  const shown = (...gods: string[]) => list.some((spot) => spot.visible && gods.includes(spot.god));
  const leader = monthLeader(natal);
  const items: ComboItem[] = [];
  const push = (key: string, label: string, detail: string, tone: ComboItem["tone"], gods: string[]) =>
    items.push({ key, label, detail, tone, favorNote: favorText(prefs, natal, gods) });

  const seal = has("正印") ? "正印" : "偏印";
  if (has("七殺") && has("正印", "偏印")) {
    const broken = shown("正財", "偏財");
    push(
      "kill-seal",
      "殺印相生",
      `七殺（${where(list, "七殺")}）得${seal}（${where(list, seal)}）化。書中列為七殺最佳益友，「制殺不如化殺高」，主文武雙全（p146）。${
        broken ? "但天干又見財星：殺配印再遇財，財壞印而生殺，書中說轉為不好（p49–51）。" : ""
      }`,
      broken ? "note" : "good",
      ["七殺", seal],
    );
  }
  if (has("七殺") && has("食神", "傷官")) {
    const output = has("食神") ? "食神" : "傷官";
    push(
      "food-control-kill",
      output === "食神" ? "食神制殺" : "傷官駕殺",
      `七殺（${where(list, "七殺")}）有${output}（${where(list, output)}）制。主技藝安身、剛柔並重；女命七殺遇食神，婚姻由差轉好（p146、p196）。`,
      "good",
      ["七殺", output],
    );
  }
  if (has("七殺") && !has("正印", "偏印", "食神", "傷官")) {
    const fed = shown("正財", "偏財");
    push(
      "kill-unchecked",
      fed ? "財生七殺無制" : "七殺無制",
      `七殺（${where(list, "七殺")}）沒有印化、也沒有食傷制。「有制伏為偏官，無制伏為七殺」，主缺乏安全感、猜疑、小人多${
        fed ? "；又有財星生殺，書中說最忌" : ""
      }。歲運見印或食神是轉機（p130–151）。`,
      "bad",
      fed ? ["七殺", "偏財"] : ["七殺"],
    );
  }
  if (has("傷官") && has("正印", "偏印")) {
    push(
      "hurt-seal",
      "傷官配印",
      `傷官（${where(list, "傷官")}）有${seal}（${where(list, seal)}）制。書中說是大才子、可掌權，善於管理，宜文藝教育路線（p210、p218）。`,
      "good",
      ["傷官", seal],
    );
  }
  if (has("傷官") && has("正財", "偏財")) {
    push(
      "hurt-wealth",
      "傷官生財",
      `傷官（${where(list, "傷官")}）生財。「傷官原是產業神」，傷官加偏財可成暴發格；缺點是說話浮誇、男命桃花多（p218）。`,
      "good",
      ["傷官", has("偏財") ? "偏財" : "正財"],
    );
  } else if (shown("傷官") && !has("正財", "偏財")) {
    push("hurt-no-wealth", "傷官無財", "傷官透出而原局無財。書引「傷官無財可恃，雖巧必貧」（p218）。", "note", ["傷官"]);
  }
  if (has("傷官") && has("劫財")) {
    push(
      "hurt-rob",
      "傷官劫財",
      `傷官（${where(list, "傷官")}）遇劫財（${where(list, "劫財")}）。書中說「傷官劫財，拉幫結派」，易結交損友，${
        has("正印", "偏印") ? "有印可緩和" : "又無印制"
      }（p25、p220–221）。`,
      has("正印", "偏印") ? "note" : "bad",
      ["傷官", "劫財"],
    );
  }
  if (has("偏印") && has("食神")) {
    const checked = has("正財", "偏財");
    push(
      "owl-food",
      "梟神奪食",
      `偏印（${where(list, "偏印")}）剋食神（${where(list, "食神")}），又叫「倒食」，不利健康與晚輩${
        checked ? "；原局有財可制偏印，力量減輕" : ""
      }（p93–96、p273–276 黃霑例）。`,
      checked ? "note" : "bad",
      ["偏印", "食神"],
    );
  }

  // 主副配對：以月令領袖為主（p49–54、p67–68）。
  if (leader.god === "正印" && has("正財", "偏財")) {
    push(
      "seal-then-wealth",
      "先印後財",
      `月令領袖是正印，又見財星。「先印後財，反成其辱」：財遏正印之善，書中說易見利忘義、失貴人、文書受損（p48–51）。`,
      "bad",
      ["正印", has("偏財") ? "偏財" : "正財"],
    );
  }
  if (leader.god === "偏印" && has("正財", "偏財")) {
    push(
      "owl-meets-wealth",
      "偏印逢財",
      "月令領袖是偏印，又見財星。偏神宜制，偏印格逢財「反辱為榮」，書中例為富貴之命（p68）。",
      "good",
      ["偏印", has("偏財") ? "偏財" : "正財"],
    );
  }
  if ((leader.god === "正財" || leader.god === "偏財") && has("正印", "偏印")) {
    push(
      "wealth-then-seal",
      "先財後印",
      `月令領袖是${leader.god}，又見印星。「先財後印，反成其福」：印揚財之善，書中稱儒商（p50、p181）。`,
      "good",
      [leader.god, seal],
    );
  }
  if (leader.god === "正官" && has("正印", "偏印")) {
    push("officer-seal", "官印相生", "月令領袖是正官，又有印星。官主職位、印主實權，「有官有印，無破作廊廟之材」（p124）。", "good", ["正官", seal]);
  }
  if (leader.god === "正官" && has("正財", "偏財")) {
    push("wealth-officer", "財官並舉", "月令領袖是正官，又有財星生官。官弱時財星比印更好，主外圓內方（p124–125）。", "good", ["正官", has("偏財") ? "偏財" : "正財"]);
  }

  const dayStem = stemOf(natal[2]);
  const ren = REN[dayStem];
  if (ren && has("七殺") && natal.some((pillar) => branchOf(pillar) === ren)) {
    push("kill-blade", "殺刃", `七殺與陽刃${ren}並見。「殺無刃不顯，刃無殺不威」，主武職或需膽識的行業（p251–254）。`, "good", ["七殺", "劫財"]);
  }
  natal.forEach((pillar, index) => {
    const stemGod = index === 2 ? "" : tenGod(dayStem, stemOf(pillar));
    const branchGod = tenGod(dayStem, HIDDEN[branchOf(pillar)]?.[0] ?? dayStem);
    if ((stemGod === "正財" || stemGod === "偏財") && branchGod === "劫財") {
      push(
        `blade-head-wealth-${index}`,
        "刃頭財",
        `${PILLARS[index]}柱${pillar}：財星坐劫財。書中說是高風險、不道德之財，發後易敗或因財惹禍（p246–248）。`,
        "bad",
        [stemGod, "劫財"],
      );
    }
  });

  const regular = list.filter((spot) => REGULAR.includes(spot.god)).length;
  return {
    leader: leader.god,
    leaderSource: leader.source,
    regular,
    irregular: list.length - regular,
    items,
  };
}

/**
 * 四大動神（p58–61）：運歲中七殺、傷官、偏財、比劫透干又有根，主人事變動。
 * 大運看本身干支；流年看流年天干與大運天干，是否在流年地支得根（p60 命例：乙巳運遇寅、卯年）。
 */
export function movingGodEvents(incoming: string, who: "運" | "歲", natal: string[], luck = ""): LuckEvent[] {
  if (natal.length !== 4 || incoming.length < 2) return [];
  const dayStem = stemOf(natal[2]);
  const branch = branchOf(incoming);
  const rootElements = (HIDDEN[branch] ?? []).map((stem) => STEM_ELEMENT[stem]);
  const candidates = who === "運" ? [stemOf(incoming)] : [stemOf(incoming), luck ? stemOf(luck) : ""].filter(Boolean);
  const events: LuckEvent[] = [];
  const seen = new Set<string>();
  candidates.forEach((stem, index) => {
    const god = tenGod(dayStem, stem);
    const moving = MOVING[god];
    if (!moving || seen.has(moving.label) || !rootElements.includes(STEM_ELEMENT[stem])) return;
    seen.add(moving.label);
    const from = who === "運" ? `大運${incoming}` : index === 0 ? `流年天干${stem}` : `大運天干${stem}`;
    events.push({
      kind: `moving-${god}`,
      label: moving.label,
      level: "info",
      topic: "move",
      detail: `${from}是${god}，在${who === "運" ? "本身地支" : `流年地支${branch}`}得根。${moving.detail}（十神洩天機 p58–61）。是否有利要看${god}是喜是忌。`,
    });
  });
  return events;
}
