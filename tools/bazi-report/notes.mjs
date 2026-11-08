#!/usr/bin/env node
// 查書本／上課筆記：關鍵字搜尋，以及找和某張盤相似的書中命例。
//   node tools/bazi-report/notes.mjs search 土多金埋 庚
//   node tools/bazi-report/notes.mjs similar "庚戌 丙戌 庚寅 己卯"
// 預設搜尋 docs/books/、上課筆記整理.md、bazi-skills/；可用環境變數 BAZI_NOTES_DIRS（Windows 用 ; 分隔，Linux/macOS 用 :）加資料夾。
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { delimiter, join, relative } from "node:path";
import { pathToFileURL } from "node:url";
import { REPO } from "./core.mjs";

function roots() {
  const base = [join(REPO, "docs", "books"), join(REPO, "上課筆記整理.md"), join(REPO, "bazi-skills")];
  const extra = (process.env.BAZI_NOTES_DIRS ?? "").split(delimiter).map((s) => s.trim()).filter(Boolean);
  return [...base, ...extra].filter((p) => existsSync(p));
}

function listMarkdown(path, out = []) {
  const st = statSync(path);
  if (st.isFile()) {
    if (path.endsWith(".md")) out.push(path);
    return out;
  }
  for (const name of readdirSync(path)) {
    if (name.startsWith(".") || name === "_raw" || name === "node_modules") continue;
    listMarkdown(join(path, name), out);
  }
  return out;
}

let cache;
function documents() {
  if (cache) return cache;
  cache = roots().flatMap((root) => listMarkdown(root)).map((file) => ({ file, lines: readFileSync(file, "utf8").replace(/\r\n/g, "\n").split("\n") }));
  return cache;
}

const label = (file) => {
  const rel = relative(REPO, file);
  return (rel.startsWith("..") ? file : rel).replaceAll("\\", "/");
};

/** 把檔案切成段落（以標題為界），回傳全部關鍵字都出現的段落。 */
export function searchNotes(query, { limit = 12, context = 600 } = {}) {
  const terms = String(query).split(/\s+/).filter(Boolean);
  if (!terms.length) return [];
  const hits = [];
  for (const doc of documents()) {
    let heading = "";
    let start = 0;
    const flush = (end) => {
      const text = doc.lines.slice(start, end).join("\n");
      if (terms.every((t) => text.includes(t))) {
        const score = terms.reduce((n, t) => n + text.split(t).length - 1, 0);
        const first = Math.max(0, text.indexOf(terms[0]) - 120);
        hits.push({ file: label(doc.file), line: start + 1, heading, score, snippet: text.slice(first, first + context).trim() });
      }
    };
    doc.lines.forEach((line, i) => {
      if (/^#{1,4}\s/.test(line)) {
        flush(i);
        heading = line.replace(/^#+\s*/, "");
        start = i;
      }
    });
    flush(doc.lines.length);
  }
  return hits.sort((a, b) => b.score - a.score).slice(0, limit);
}

const clean = (cell) => cell.replace(/<sub>.*?<\/sub>/g, "").replace(/[*\s]/g, "");

/** 找出筆記裡用 App 排法畫的命盤（| 天干 | 時 | 日 | 月 | 年 |）。 */
export function noteCharts() {
  const charts = [];
  for (const doc of documents()) {
    let heading = "";
    doc.lines.forEach((line, i) => {
      if (/^#{1,5}\s/.test(line)) heading = line.replace(/^#+\s*/, "");
      if (!line.startsWith("| 天干 |")) return;
      const next = doc.lines[i + 1] ?? "";
      if (!next.startsWith("| 地支 |")) return;
      const stems = line.split("|").slice(2, 6).map(clean);
      const branches = next.split("|").slice(2, 6).map(clean);
      if (stems.length !== 4 || branches.length !== 4 || stems.some((s) => s.length !== 1) || branches.some((b) => b.length !== 1)) return;
      // 表格是 時 日 月 年，轉成 年 月 日 時
      const natal = [3, 2, 1, 0].map((k) => stems[k] + branches[k]);
      charts.push({ file: label(doc.file), line: i + 1, heading, natal });
    });
  }
  return charts;
}

/** 依日干、月令、日柱、相同字數排序，找相似命例。 */
export function similarCases(pillars, { limit = 15 } = {}) {
  const natal = String(pillars).trim().split(/[\s,，、]+/);
  if (natal.length !== 4) throw new Error("pillars 要四柱，依 年 月 日 時");
  const chars = (n) => n.flatMap((p) => [...p]);
  const seen = new Set();
  return noteCharts()
    .map((c) => {
      let score = 0;
      const why = [];
      if (c.natal.join(" ") === natal.join(" ")) (score += 20), why.push("四柱相同");
      if (c.natal[2][0] === natal[2][0]) (score += 4), why.push(`日干${natal[2][0]}`);
      if (c.natal[1][1] === natal[1][1]) (score += 4), why.push(`月令${natal[1][1]}`);
      if (c.natal[2] === natal[2]) (score += 2), why.push(`日柱${natal[2]}`);
      if (c.natal[1] === natal[1]) (score += 1), why.push(`月柱${natal[1]}`);
      const pool = chars(natal);
      const shared = chars(c.natal).filter((ch) => {
        const i = pool.indexOf(ch);
        if (i < 0) return false;
        pool.splice(i, 1);
        return true;
      }).length;
      score += shared * 0.5;
      return { ...c, natal: c.natal.join(" "), score, why: `${why.join("、")}${why.length ? "、" : ""}同字 ${shared}/8` };
    })
    .filter((c) => c.score >= 8)
    .sort((a, b) => b.score - a.score)
    .filter((c) => {
      const key = `${c.natal}|${c.file}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    })
    .slice(0, limit);
}

export function searchMarkdown(hits) {
  if (!hits.length) return "沒有找到。";
  return hits.map((h) => `### ${h.file}:${h.line}　${h.heading}\n${h.snippet}`).join("\n\n");
}

export function similarMarkdown(cases) {
  if (!cases.length) return "沒有找到相似命例。";
  return ["| 四柱（年月日時） | 相似處 | 出處 |", "|---|---|---|", ...cases.map((c) => `| ${c.natal} | ${c.why} | ${c.file}:${c.line}（${c.heading}） |`)].join("\n");
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const [cmd, ...rest] = process.argv.slice(2);
  if (cmd === "search") console.log(searchMarkdown(searchNotes(rest.join(" "))));
  else if (cmd === "similar") console.log(similarMarkdown(similarCases(rest.join(" "))));
  else console.log('用法：notes.mjs search 關鍵字…｜notes.mjs similar "年 月 日 時"');
}
