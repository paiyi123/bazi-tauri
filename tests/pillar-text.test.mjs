import test, { after } from "node:test";
import assert from "node:assert/strict";
import { createServer } from "vite";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const cacheDir = mkdtempSync(join(tmpdir(), "bazi-pillar-text-"));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: "custom", optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { parsePillarText, expectedMonthStem, expectedHourStem } = await server.ssrLoadModule("/src/utils/pillarText.ts");

test("五虎遁、五鼠遁", () => {
  assert.equal(expectedMonthStem("壬", "酉"), "己");
  assert.equal(expectedMonthStem("甲", "寅"), "丙");
  assert.equal(expectedHourStem("甲", "子"), "甲");
  assert.equal(expectedHourStem("乙", "巳"), "辛");
});

test("年月日時的書例自動判斷", () => {
  const r = parsePillarText("壬午 己酉 甲申 甲子");
  assert.ok(r.ok);
  assert.equal(r.order, "ymdh");
  assert.deepEqual(r.pillars, { year: "壬午", month: "己酉", day: "甲申", hour: "甲子" });
});

test("日時月年的寫法自動判斷", () => {
  // 同一張盤寫成 日 時 月 年
  const r = parsePillarText("甲申、甲子、己酉、壬午");
  assert.ok(r.ok);
  assert.equal(r.order, "dhmy");
  assert.deepEqual(r.pillars, { year: "壬午", month: "己酉", day: "甲申", hour: "甲子" });
});

test("指定順序、錯誤輸入", () => {
  const r = parsePillarText("年 己巳 月 乙亥 日 乙未 時 辛巳", "ymdh");
  assert.ok(r.ok);
  assert.equal(r.pillars.day, "乙未");
  assert.equal(r.note, "");
  assert.equal(parsePillarText("壬午 己酉 甲申").ok, false);
  assert.equal(parsePillarText("甲丑 己酉 甲申 甲子").ok, false);
});
