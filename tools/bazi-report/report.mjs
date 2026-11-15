#!/usr/bin/env node
// 命盤報告：一次跑完 App 的 Rust 排盤／量化／課堂三步法與前端 TS 規則，輸出精簡摘要。
// 用法見 README.md，例如：
//   node tools/bazi-report/report.mjs --date 1970-11-06 --time 05:00 --gender F --years 2024-2030
//   node tools/bazi-report/report.mjs --pillars "庚戌 丙戌 庚寅 己卯" --gender F --format json
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { REPO, runCore } from "./core.mjs";

const STEM_EL = { 甲: "木", 乙: "木", 丙: "火", 丁: "火", 戊: "土", 己: "土", 庚: "金", 辛: "金", 壬: "水", 癸: "水" };
const BRANCH_EL = { 子: "水", 丑: "土", 寅: "木", 卯: "木", 辰: "土", 巳: "火", 午: "火", 未: "土", 申: "金", 酉: "金", 戌: "土", 亥: "水" };
const EL_KEY = { 木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water" };
const TRIADS = [
  { kind: "三合", branches: "申子辰", element: "水" },
  { kind: "三合", branches: "亥卯未", element: "木" },
  { kind: "三合", branches: "寅午戌", element: "火" },
  { kind: "三合", branches: "巳酉丑", element: "金" },
  { kind: "三會", branches: "寅卯辰", element: "木" },
  { kind: "三會", branches: "巳午未", element: "火" },
  { kind: "三會", branches: "申酉戌", element: "金" },
  { kind: "三會", branches: "亥子丑", element: "水" },
];

// ---------- 輸入 ----------

export function buildRequest(input) {
  const gender = /^(f|female|女)$/i.test(String(input.gender ?? "")) ? "FEMALE" : /^(m|male|男)$/i.test(String(input.gender ?? "")) ? "MALE" : null;
  if (!gender) throw new Error("gender 必填：F/女 或 M/男");
  if (input.pillars) {
    const parts = String(input.pillars).trim().split(/[\s,，、]+/);
    if (parts.length !== 4) throw new Error("pillars 要四柱，依 年 月 日 時，例如「庚戌 丙戌 庚寅 己卯」");
    return {
      mode: "pillars",
      request: {
        yearPillar: parts[0], monthPillar: parts[1], dayPillar: parts[2], hourPillar: parts[3],
        gender, selectedGregorianYear: input.birthYear ? Number(input.birthYear) : null, inferGregorianYears: false,
      },
    };
  }
  const date = String(input.date ?? "").match(/^(\d{1,4})[-/.](\d{1,2})[-/.](\d{1,2})$/);
  if (!date) throw new Error("date 格式：YYYY-MM-DD（農曆加 --calendar lunar，民國年加 --era roc）");
  const time = String(input.time ?? "12:00").match(/^(\d{1,2}):(\d{2})(?::(\d{2}))?$/);
  if (!time) throw new Error("time 格式：HH:MM");
  return {
    mode: "calculate",
    request: {
      calendarType: /^lunar|農/i.test(String(input.calendar ?? "")) ? "LUNAR" : "SOLAR",
      gender,
      year: Number(date[1]),
      yearEra: /^roc|民/i.test(String(input.era ?? "")) ? "ROC" : "AD",
      month: Number(date[2]),
      day: Number(date[3]),
      hour: Number(time[1]),
      minute: Number(time[2]),
      second: Number(time[3] ?? 0),
      baziSect: Number(input.baziSect ?? 2),
      yunSect: Number(input.yunSect ?? 1), // App 輸入表單預設 1
      leapMonth: Boolean(input.leap),
    },
  };
}

function parseYears(spec, fallback) {
  if (!spec) return fallback;
  const m = String(spec).match(/^(\d{4})(?:\s*[-~到]\s*(\d{4}))?$/);
  if (!m) throw new Error("years 格式：2026 或 2024-2030");
  const from = Number(m[1]);
  const to = Number(m[2] ?? m[1]);
  return Array.from({ length: Math.max(0, to - from) + 1 }, (_, i) => from + i);
}

// ---------- 載入前端 TS 規則 ----------

let tsModules;
async function loadTs() {
  if (tsModules) return tsModules;
  const { createServer } = await import("vite");
  const cacheDir = mkdtempSync(join(tmpdir(), "bazi-report-"));
  const server = await createServer({
    root: REPO, cacheDir, logLevel: "error", configFile: false,
    server: { middlewareMode: true, hmr: false, watch: null }, appType: "custom",
    optimizeDeps: { noDiscovery: true, entries: [] },
  });
  const load = (path) => server.ssrLoadModule(path);
  tsModules = {
    life: await load("/src/utils/lifeTopics.ts"),
    events: await load("/src/utils/luckEvents.ts"),
    basic: await load("/src/utils/basicStructure.ts"),
    special: await load("/src/utils/specialPatterns.ts"),
    tenGod: await load("/src/utils/tenGodCombos.ts"),
    pref: await load("/src/utils/lessonPreference.ts"),
    close: async () => {
      await server.close();
      rmSync(cacheDir, { recursive: true, force: true });
    },
  };
  return tsModules;
}

export async function closeTs() {
  if (tsModules) await tsModules.close();
  tsModules = undefined;
}

// ---------- 小工具 ----------

/** 藏干照 App 排法：本氣置中（三個：中 本 餘；兩個：中 本）。 */
function hiddenLayout(stems, gods) {
  const order = stems.length === 3 ? [1, 0, 2] : stems.length === 2 ? [1, 0] : [0];
  return order.map((i) => ({ stem: stems[i], tenGod: gods[i], main: i === 0 }));
}

function chartMarkdown(pillars) {
  const cols = [...pillars].reverse(); // 時 日 月 年
  const hidden = (p, key) => p.hidden.map((h) => (h.main ? `**${h[key]}**` : h[key])).join(" ");
  return [
    "| | 時 | 日 | 月 | 年 |",
    "|---|:-:|:-:|:-:|:-:|",
    `| 十神 | ${cols.map((p) => p.stemTenGod).join(" | ")} |`,
    `| 天干 | ${cols.map((p) => `**${p.stem}**<sub>${p.stemElement}</sub>`).join(" | ")} |`,
    `| 地支 | ${cols.map((p) => `**${p.branch}**<sub>${p.branchElement}</sub>`).join(" | ")} |`,
    `| 藏干 | ${cols.map((p) => hidden(p, "stem")).join(" | ")} |`,
    `| 藏干十神 | ${cols.map((p) => hidden(p, "tenGod")).join(" | ")} |`,
  ].join("\n");
}

/** 依原局強弱解讀量化分數：作用後命局離 0 越近越中和（有利），越遠越偏（不利）。從格要反過來看。 */
function quantReading(natal, effective, cong) {
  if (effective == null) return "";
  const diff = Math.abs(effective) - Math.abs(natal);
  const flip = Math.sign(effective) !== Math.sign(natal) && Math.abs(effective) > 20;
  let text = diff < -8 ? "趨中和（有利）" : diff > 8 ? "更偏（不利）" : "變化不大";
  if (flip) text += "，強弱翻轉";
  if (cong && cong !== "不從") text += "（從格：此讀法要反過來）";
  return text;
}

function favorOf(prefs, char) {
  const element = STEM_EL[char] ?? BRANCH_EL[char];
  const entry = prefs?.elements?.[EL_KEY[element]];
  return entry ? `${element}${entry.label}` : element;
}

/** 運歲湊成原局沒有的三合／三會（填實）。 */
function triadFills(natalBranches, incoming) {
  const out = [];
  for (const triad of TRIADS) {
    const chars = [...triad.branches];
    const natalHas = chars.filter((c) => natalBranches.includes(c));
    if (natalHas.length === 3) continue;
    const pool = [...natalBranches, ...incoming.map((x) => x.branch)];
    if (!chars.every((c) => pool.includes(c))) continue;
    const added = incoming.filter((x) => chars.includes(x.branch) && !natalBranches.includes(x.branch));
    if (!added.length) continue;
    out.push(
      `${added.map((x) => `${x.who}${x.branch}`).join("、")}與原局${natalHas.join("") || "（無）"}湊成${triad.branches}${triad.kind}${triad.element}局`,
    );
  }
  return out;
}

const eventText = (events) => events.map((e) => ({ label: e.label, tone: e.tone ?? e.topic ?? "", detail: e.detail ?? e.tooltip ?? "" }));

// ---------- 組報告 ----------

export async function buildReport(input = {}) {
  let { mode, request } = buildRequest(input);
  let pillarNote = "";
  if (mode === "pillars") {
    // 直輸四柱沒有公曆年份，大運流年排不出來：先回推出生時間，再照一般排盤跑。
    const hint = runCore("pillars", { ...request, inferGregorianYears: true }).directPillarYearHint;
    const candidates = hint?.candidates ?? [];
    const wanted = input.birthYear ? candidates.find((c) => c.year === Number(input.birthYear)) : undefined;
    const now = new Date().getFullYear();
    const chosen = wanted ?? [...candidates].filter((c) => c.year <= now).sort((a, b) => b.year - a.year)[0];
    if (!chosen) throw new Error(`找不到符合 ${input.pillars} 的公曆年份（${hint?.note ?? ""}）`);
    pillarNote = `四柱回推公曆 ${chosen.label}（候選：${candidates.map((c) => c.label).join("、")}；可用 --birthYear 指定）`;
    request = {
      calendarType: "SOLAR", gender: request.gender, year: chosen.year, yearEra: "AD", month: chosen.month, day: chosen.day,
      hour: chosen.hour, minute: chosen.minute, second: chosen.second, baziSect: 2, yunSect: Number(input.yunSect ?? 1), leapMonth: false,
    };
    mode = "calculate";
  }
  const r = runCore(mode, request);
  const ts = await loadTs();
  const q = r.quantModel ?? {};
  const t = q.lessonContent?.threeStepAnalysis;
  const lu = q.luStrength;
  const posNames = ["年", "月", "日", "時"];
  const keys = ["year", "month", "day", "hour"];
  const pillars = keys.map((k, i) => {
    const p = r[`${k}Pillar`];
    return {
      position: posNames[i],
      ganZhi: p.ganZhi,
      stem: p.stem,
      stemElement: STEM_EL[p.stem],
      stemTenGod: r[`${k}StemTenGod`] === "日元" ? "日主" : r[`${k}StemTenGod`],
      branch: p.branch,
      branchElement: BRANCH_EL[p.branch],
      hidden: hiddenLayout(r[`${k}HiddenStems`], r[`${k}BranchTenGods`]),
      naYin: p.naYin,
      diShi: p.diShi,
    };
  });
  const natal = pillars.map((p) => p.ganZhi);
  const natalBranches = pillars.map((p) => p.branch);
  const count = { 木: 0, 火: 0, 土: 0, 金: 0, 水: 0 };
  const hiddenCount = { 木: 0, 火: 0, 土: 0, 金: 0, 水: 0 };
  pillars.forEach((p) => {
    count[p.stemElement] += 1;
    count[p.branchElement] += 1;
    p.hidden.forEach((h) => (hiddenCount[STEM_EL[h.stem]] += 1));
  });

  const prefs = t ? ts.pref.lessonPreferences(t) : null;
  const strength = lu?.category;
  const pattern = ts.basic.basicStructure(natal, lu?.verdict);
  const luckCtx = { natal, strength, rootless: lu ? lu.roots.length === 0 : undefined, verdict: lu?.verdict, pattern: pattern.pattern.name };
  const topicCtx = { natal, gender: r.gender, strength };
  const congPattern = q.congPattern?.pattern;
  const natalScore = q.totalScore;

  const luckScores = new Map((q.luckScores ?? []).map((l) => [l.index, l]));
  const allAnnual = (r.daYun ?? []).flatMap((y) => (y.liuNian ?? []).map((n) => ({ ...n, luck: y })));
  const spouse = ts.life.spouseStorageYears(topicCtx, allAnnual.map((n) => ({ year: n.year, ganZhi: n.ganZhi, age: n.age })));

  const daYun = (r.daYun ?? []).map((y) => {
    const ls = luckScores.get(y.index);
    const events = [
      ...ts.events.majorLuckEvents(y.ganZhi, luckCtx),
      ...ts.life.topicEvents(y.ganZhi, "運", topicCtx),
      ...ts.tenGod.movingGodEvents(y.ganZhi, "運", natal),
      ...ts.basic.patternLuckEvents(y.ganZhi, "運", natal, luckCtx.pattern),
      ...ts.special.specialPatternLuckEvents(y.ganZhi, "運", natal),
    ];
    return {
      ganZhi: y.ganZhi,
      age: `${y.startAge}–${y.endAge}`,
      years: `${y.startYear}–${y.endYear}`,
      tenGods: `${y.stemTenGod}／${y.branchTenGods[0] ?? ""}`,
      classroom: prefs ? `${favorOf(prefs, y.ganZhi[0])}、${favorOf(prefs, y.ganZhi[1])}` : "",
      quant: ls
        ? {
            stemHalf: ls.firstHalfScore,
            branchHalf: ls.secondHalfScore,
            effective: ls.effectiveNatalScore,
            reading: quantReading(natalScore, ls.effectiveNatalScore, congPattern),
            notes: ls.overviewLines.filter((x) => !/^(前|後)五年看|整柱參考|R值：|作用後命局：|運(干|支)入局令/.test(x)),
            interactions: ls.interactionLines,
          }
        : null,
      fills: triadFills(natalBranches, [{ who: "運", branch: y.ganZhi[1] }]),
      events: eventText(events),
    };
  });

  const thisYear = new Date().getFullYear();
  const years = parseYears(input.years, [thisYear]);
  const annual = years.map((year) => {
    // 交運年同一年會出現在兩步運，取後一步（交運在年初節氣後不久）。
    const hits = allAnnual.filter((n) => n.year === year);
    const n = hits[hits.length - 1];
    if (!n) return { year, missing: true };
    const ls = luckScores.get(n.luck.index);
    const a = ls?.annualScores?.find((x) => x.year === year);
    const events = [
      ...ts.events.annualEvents(n.ganZhi, n.luck.ganZhi, luckCtx),
      ...ts.life.topicEvents(n.ganZhi, "歲", topicCtx, n.luck.ganZhi),
      ...ts.tenGod.movingGodEvents(n.ganZhi, "歲", natal, n.luck.ganZhi),
      ...ts.basic.patternLuckEvents(n.ganZhi, "歲", natal, luckCtx.pattern),
      ...ts.special.specialPatternLuckEvents(n.ganZhi, "歲", natal),
    ];
    const storage = spouse.get(year);
    if (storage) events.push({ label: storage === "first" ? "夫妻星庫首次沖開（婚期候選）" : "夫妻星庫再沖（婚變留意）", tone: "marriage" });
    const half = a?.activeLuckLabel ?? "";
    const activeLuckChar = /運干/.test(half) ? n.luck.ganZhi[0] : n.luck.ganZhi[1];
    return {
      year,
      ganZhi: n.ganZhi,
      age: n.age,
      luck: n.luck.ganZhi,
      transitionYear: hits.length > 1,
      tenGods: `${n.stemTenGod}／${n.branchTenGods[0] ?? ""}`,
      classroom: prefs ? `${favorOf(prefs, n.ganZhi[0])}、${favorOf(prefs, n.ganZhi[1])}` : "",
      quant: a
        ? {
            activeLuck: `${half}（${activeLuckChar}）`,
            stem: a.annualStemScore,
            branch: a.annualBranchScore,
            combined: a.combinedScore,
            effective: a.effectiveNatalScore,
            r: a.impactRatio,
            reading: quantReading(natalScore, a.effectiveNatalScore, congPattern),
            interactions: a.interactionLines,
          }
        : null,
      fills: triadFills(natalBranches, [{ who: "運", branch: n.luck.ganZhi[1] }, { who: "歲", branch: n.ganZhi[1] }]),
      events: eventText(events),
    };
  });

  const special = ts.special.specialPatternReport(natal, { female: r.gender === "FEMALE" });
  const specialHits = special
    ? [...(special.follow ?? []), special.prosperous, special.transform].filter((x) => x && x.verdict && x.verdict !== "none").map((x) => `${x.name}：${x.verdictLabel}`)
    : [];
  const tenGod = ts.tenGod.tenGodOverview(natal, prefs);

  return {
    input: { ...request, mode },
    pillarNote,
    solar: r.solarDateTime,
    lunar: r.lunarDateTime,
    bazi: r.baZi,
    gender: r.gender === "FEMALE" ? "女" : "男",
    chart: { pillars, markdown: chartMarkdown(pillars) },
    elementCount: { surface: count, hidden: hiddenCount },
    luckStart: r.luckStart ? `${r.luckStart.startSummary}；${r.luckStart.transitionSummary}（${r.luckStart.forward ? "順" : "逆"}排）` : "",
    classroom: t
      ? {
          pattern: t.pattern?.conclusion,
          strength: t.strength?.classification,
          strengthReason: t.strength?.verdictReason,
          party: `同黨 ${t.strength?.samePartyCount}：異黨 ${t.strength?.opposingPartyCount}`,
          checks: (t.strength?.checks ?? []).map((c) => `${c.item}${c.hit ? "✓" : "✗"}：${c.finding}`),
          plans: (t.preferences?.plans ?? []).map((p) => {
            const fmt = (xs) => xs.map((d) => `${d.family}(${d.element})`).join("、") || "—";
            return `${p.title}｜用 ${fmt(p.useDirections)}｜喜 ${fmt(p.favorableDirections)}｜忌 ${fmt(p.unfavorableDirections)}｜待比較 ${fmt(p.conditionalDirections)}`;
          }),
          merged: prefs ? Object.values(prefs.elements).map((e) => `${e.name}${e.label}`).join("、") : "",
          stemCard: t.stemInteractions?.conclusion,
          branchCard: t.branchInteractions?.conclusion,
          pillarCard: t.pillarRelations?.conclusion,
          recheck: t.interactionStrength?.conclusion,
          comparison: t.comparison,
        }
      : null,
    quant: {
      total: q.totalScore,
      label: q.strengthLabel,
      yongShen: q.yongShen?.conclusion,
      yongShenProcess: q.yongShen?.process,
      cong: q.congPattern?.conclusion,
      special: q.specialPattern?.conclusion,
      interactions: (q.interactions ?? []).map((x) => `${x.scope}${x.target}${x.type}（${x.pillars}）${x.detail}`),
      readingRule: "量化分數正＝幫日主、負＝剋洩耗；App「傾向」欄已依身強身弱判讀（身強喜負分、身弱喜正分，從格反過來）。這份報告另看作用後命局離 0 遠近：靠近 0 為趨中和（有利）。",
    },
    lu: lu ? { verdict: lu.verdict, score: lu.score, threshold: lu.threshold, conclusion: lu.conclusion, flags: lu.flags.map((f) => `${f.title}：${f.detail}`) } : null,
    basic: { pattern: pattern.pattern, useGod: pattern.useGod, checks: pattern.checks, climate: `${pattern.climate.total}（${pattern.climate.level}）` },
    lifeTopics: ts.life.lifeTopicSections(topicCtx).map((s) => ({ title: s.title, summary: s.summary, items: s.items.map((i) => ({ label: i.label, tone: i.tone, detail: i.detail })) })),
    tenGod: tenGod ? { leader: `${tenGod.leader}（${tenGod.leaderSource}）`, items: tenGod.items.map((i) => `${i.label}：${i.detail}${i.favorNote ? `〔${i.favorNote}〕` : ""}`) } : null,
    specialPatterns: specialHits.length ? specialHits : ["蘇民峰特別格局皆不成立"],
    shenSha: r.shenSha,
    daYun,
    annual,
  };
}

// ---------- Markdown ----------

export function toMarkdown(rep, { detail = false } = {}) {
  const L = [];
  const ev = (events) => events.map((e) => (detail ? `${e.label}（${e.detail}）` : e.label)).join("、") || "—";
  L.push(`# ${rep.bazi}（${rep.gender}）`, "", `國曆 ${rep.solar}｜${rep.lunar}`, "");
  if (rep.pillarNote) L.push(`> ${rep.pillarNote}`, "");
  L.push(rep.chart.markdown, "");
  const c = rep.elementCount;
  L.push(`五行（干＋支本氣）：${Object.entries(c.surface).map(([k, v]) => `${k}${v}`).join(" ")}；藏干：${Object.entries(c.hidden).map(([k, v]) => `${k}${v}`).join(" ")}`);
  L.push(`起運：${rep.luckStart}`, "");
  if (rep.classroom) {
    const cl = rep.classroom;
    L.push("## 課堂三步法", `- 取格：${cl.pattern}`, `- 強弱：${cl.strength}（${cl.party}）— ${cl.strengthReason}`);
    cl.checks.forEach((x) => L.push(`  - ${x}`));
    L.push("- 喜忌方案：");
    cl.plans.forEach((x) => L.push(`  - ${x}`));
    L.push(`- 合併喜忌：${cl.merged}`, `- 天干：${cl.stemCard}`, `- 地支：${cl.branchCard}`, `- 同柱：${cl.pillarCard}`, `- 複核：${cl.recheck}`, "");
  }
  const q = rep.quant;
  L.push("## 其他方法", `- 量化五行力：${q.total}（${q.label}）；${q.yongShen}；${q.cong}`, `  - 用神推導：${q.yongShenProcess}`);
  if (q.interactions.length) L.push(`  - 原局沖合：${q.interactions.join("；")}`);
  if (rep.lu) L.push(`- 陸致極：${rep.lu.conclusion.replace(/。$/, "")}${rep.lu.flags.length ? `；${rep.lu.flags.join("；")}` : ""}`);
  L.push(`- 基礎教程：${rep.basic.pattern.name}；${rep.basic.useGod.note}；寒暖燥濕 ${rep.basic.climate}`);
  if (rep.tenGod) L.push(`- 十神洩天機：領袖 ${rep.tenGod.leader}；${rep.tenGod.items.join("；") || "無組合"}`);
  L.push(`- 蘇民峰特別格局：${rep.specialPatterns.join("；")}`, "");
  L.push("## 婚姻・財運・健康（App 面板）");
  rep.lifeTopics.forEach((s) => {
    L.push(`- **${s.title}**：${s.summary}`);
    s.items.forEach((i) => L.push(`  - ${i.label}${detail ? `：${i.detail}` : ""}`));
  });
  L.push("", "## 大運", `> ${q.readingRule}`, "", "| 年齡 | 年份 | 大運 | 十神 | 課堂喜忌 | 量化 干/支 → 作用後命局 | 填實 | 提示 |", "|---|---|---|---|---|---|---|---|");
  rep.daYun.forEach((y) =>
    L.push(`| ${y.age} | ${y.years} | ${y.ganZhi} | ${y.tenGods} | ${y.classroom} | ${y.quant ? `${y.quant.stemHalf}/${y.quant.branchHalf} → ${y.quant.effective}，${y.quant.reading}` : ""} | ${y.fills.join("；") || "—"} | ${ev(y.events)} |`),
  );
  L.push("", "## 流年", "| 年 | 干支 | 歲 | 大運 | 十神 | 課堂喜忌 | 量化（當值運）合參 → 作用後命局 | 填實 | 提示 |", "|---|---|---|---|---|---|---|---|---|");
  rep.annual.forEach((a) => {
    if (a.missing) return L.push(`| ${a.year} | 不在大運範圍 |||||||| `);
    L.push(
      `| ${a.year} | ${a.ganZhi} | ${a.age} | ${a.luck}${a.transitionYear ? "（交運年）" : ""} | ${a.tenGods} | ${a.classroom} | ${a.quant ? `${a.quant.activeLuck} ${a.quant.combined} → ${a.quant.effective}，${a.quant.reading}` : ""} | ${a.fills.join("；") || "—"} | ${ev(a.events)} |`,
    );
  });
  if (detail) {
    L.push("", "## 量化計分明細（流年）");
    rep.annual.forEach((a) => a.quant && L.push(`- ${a.year} ${a.ganZhi}：${a.quant.interactions.join("；")}`));
  }
  return L.join("\n");
}

// ---------- CLI ----------

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (!a.startsWith("--")) continue;
    const key = a.slice(2);
    const next = argv[i + 1];
    if (next === undefined || next.startsWith("--")) out[key] = true;
    else {
      out[key] = next;
      i++;
    }
  }
  return out;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help || (!args.date && !args.pillars)) {
    console.log(`用法：
  node tools/bazi-report/report.mjs --date 1970-11-06 --time 05:00 --gender F [--years 2024-2030]
  node tools/bazi-report/report.mjs --date 59-10-08 --era roc --calendar lunar --time 05:00 --gender 女
  node tools/bazi-report/report.mjs --pillars "庚戌 丙戌 庚寅 己卯" --gender F
選項：--format md|json（預設 md）、--detail（列出提示說明與計分明細）、--yunSect 1|2（預設 1，同 App）、--leap（農曆閏月）`);
    return;
  }
  try {
    const rep = await buildReport(args);
    console.log(args.format === "json" ? JSON.stringify(rep, null, 1) : toMarkdown(rep, { detail: Boolean(args.detail) }));
  } finally {
    await closeTs();
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  main().catch((error) => {
    console.error(error.message ?? error);
    process.exit(1);
  });
}
