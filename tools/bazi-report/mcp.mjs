#!/usr/bin/env node
// bazi MCP server（stdio）。不用額外套件：直接實作 MCP 的 JSON-RPC（一行一則訊息）。
// 工具：bazi_report（命盤報告）、bazi_search_notes（查筆記）、bazi_similar_cases（書中相似命例）。
// 註冊方式見 README.md（專案根目錄 .mcp.json 已設定）。
import { createInterface } from "node:readline";
import { buildReport, closeTs, toMarkdown } from "./report.mjs";
import { searchMarkdown, searchNotes, similarCases, similarMarkdown } from "./notes.mjs";

const birthProps = {
  date: { type: "string", description: "出生日期 YYYY-MM-DD（國曆；農曆加 calendar=lunar；民國年加 era=roc）" },
  time: { type: "string", description: "出生時間 HH:MM，預設 12:00" },
  gender: { type: "string", description: "F/女 或 M/男" },
  calendar: { type: "string", enum: ["solar", "lunar"], description: "預設 solar" },
  era: { type: "string", enum: ["ad", "roc"], description: "預設 ad（西元）" },
  leap: { type: "boolean", description: "農曆閏月" },
  pillars: { type: "string", description: "直接給四柱（年 月 日 時），例如「庚戌 丙戌 庚寅 己卯」；會回推公曆出生時間" },
  birthYear: { type: "number", description: "給四柱時指定出生的公曆年份（有兩個候選時用）" },
  yunSect: { type: "number", description: "起運流派，預設 1（同 App）" },
};

const TOOLS = [
  {
    name: "bazi_report",
    description:
      "用 bazi App 的程式排盤並分析：App 排法命盤表、課堂三步法（取格→強弱→喜忌方案與合併喜忌、天干地支同柱卡片）、量化五行力、陸致極強弱、基礎教程格局、十神洩天機組合、蘇民峰特別格局、婚姻財運健康面板，以及大運與指定流年的課堂喜忌、量化分數（已依強弱解讀）、三合三會填實與 App 提示標籤。",
    inputSchema: {
      type: "object",
      properties: {
        ...birthProps,
        years: { type: "string", description: "流年範圍，例如 2026 或 2024-2030；預設今年" },
        format: { type: "string", enum: ["md", "json"], description: "md（預設，精簡）或 json（完整欄位）" },
        detail: { type: "boolean", description: "md 時列出每個提示的說明與量化計分明細" },
      },
      required: ["gender"],
    },
  },
  {
    name: "bazi_search_notes",
    description: "在專案的書本筆記（docs/books/）與上課筆記裡用關鍵字搜尋（空白分隔，全部都要出現在同一段），回傳檔案、行號、標題與摘錄。",
    inputSchema: {
      type: "object",
      properties: { query: { type: "string" }, limit: { type: "number", description: "預設 12" } },
      required: ["query"],
    },
  },
  {
    name: "bazi_similar_cases",
    description: "從筆記裡約 1,900 張 App 排法命盤中，找日干、月令、日柱相同或同字多的命例，回傳四柱、相似處與出處行號。",
    inputSchema: {
      type: "object",
      properties: { pillars: { type: "string", description: "年 月 日 時，例如「庚戌 丙戌 庚寅 己卯」" }, limit: { type: "number", description: "預設 15" } },
      required: ["pillars"],
    },
  },
];

async function callTool(name, args = {}) {
  if (name === "bazi_report") {
    const report = await buildReport(args);
    return args.format === "json" ? JSON.stringify(report, null, 1) : toMarkdown(report, { detail: Boolean(args.detail) });
  }
  if (name === "bazi_search_notes") return searchMarkdown(searchNotes(args.query, { limit: args.limit ?? 12 }));
  if (name === "bazi_similar_cases") return similarMarkdown(similarCases(args.pillars, { limit: args.limit ?? 15 }));
  throw new Error(`unknown tool: ${name}`);
}

const send = (message) => process.stdout.write(`${JSON.stringify(message)}\n`);

async function handle(message) {
  const { id, method, params } = message;
  if (id === undefined) return; // notification
  try {
    if (method === "initialize") {
      return send({
        jsonrpc: "2.0",
        id,
        result: {
          protocolVersion: params?.protocolVersion ?? "2025-06-18",
          capabilities: { tools: {} },
          serverInfo: { name: "bazi", version: "0.1.0" },
        },
      });
    }
    if (method === "ping") return send({ jsonrpc: "2.0", id, result: {} });
    if (method === "tools/list") return send({ jsonrpc: "2.0", id, result: { tools: TOOLS } });
    if (method === "tools/call") {
      try {
        const text = await callTool(params?.name, params?.arguments);
        return send({ jsonrpc: "2.0", id, result: { content: [{ type: "text", text }] } });
      } catch (error) {
        return send({ jsonrpc: "2.0", id, result: { isError: true, content: [{ type: "text", text: String(error?.message ?? error) }] } });
      }
    }
    return send({ jsonrpc: "2.0", id, error: { code: -32601, message: `Method not found: ${method}` } });
  } catch (error) {
    return send({ jsonrpc: "2.0", id, error: { code: -32603, message: String(error?.message ?? error) } });
  }
}

// Requests are handled one at a time so a slow first build doesn't interleave output.
let queue = Promise.resolve();
const rl = createInterface({ input: process.stdin });
rl.on("line", (line) => {
  if (!line.trim()) return;
  let message;
  try {
    message = JSON.parse(line);
  } catch {
    return send({ jsonrpc: "2.0", id: null, error: { code: -32700, message: "Parse error" } });
  }
  queue = queue.then(() => handle(message));
});
rl.on("close", () => queue.then(() => closeTs()).finally(() => process.exit(0)));
