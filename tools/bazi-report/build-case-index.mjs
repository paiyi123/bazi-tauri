#!/usr/bin/env node
// 把 docs/books 筆記裡用 App 排法畫的命盤做成索引，給 App 的「書上相似命例」用。
//   node tools/bazi-report/build-case-index.mjs
// 輸出 src/data/bookCases.json。新增書本筆記後重跑一次即可。
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { REPO } from "./core.mjs";
import { noteCharts } from "./notes.mjs";

const seen = new Set();
const cases = noteCharts()
  .filter((chart) => chart.file.startsWith("docs/books/"))
  .filter((chart) => {
    const key = `${chart.natal.join(" ")}|${chart.file}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  })
  .map((chart) => ({
    book: chart.file.split("/")[2]?.replace(/\.md$/, "") ?? "",
    file: chart.file.replace(/^docs\/books\//, ""),
    line: chart.line,
    heading: chart.heading.replace(/[*`]/g, "").slice(0, 80),
    natal: chart.natal,
  }));

writeFileSync(join(REPO, "src", "data", "bookCases.json"), `${JSON.stringify(cases)}\n`);
console.log(`寫入 ${cases.length} 個命盤到 src/data/bookCases.json`);
