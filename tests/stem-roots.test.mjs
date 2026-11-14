import test, { after } from "node:test";
import assert from "node:assert/strict";
import { createServer } from "vite";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const cacheDir = mkdtempSync(join(tmpdir(), "bazi-stem-roots-"));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: "custom", optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { rootLevel, stemRoots, mainQiMatches, BRANCH_HIDDEN_ORDERED } = await server.ssrLoadModule("/src/utils/stemRoots.ts");

test("每個地支的本氣都和地支五行相同", () => {
  for (const branch of Object.keys(BRANCH_HIDDEN_ORDERED)) assert.ok(mainQiMatches(branch), branch);
});

test("通根分本氣、中氣、餘氣，陰陽同五行都算", () => {
  assert.equal(rootLevel("戊", "辰"), "本氣");
  assert.equal(rootLevel("甲", "亥"), "中氣");
  assert.equal(rootLevel("乙", "辰"), "中氣");
  assert.equal(rootLevel("丙", "戌"), "餘氣");
  assert.equal(rootLevel("甲", "卯"), "本氣");
  assert.equal(rootLevel("壬", "午"), null);
});

test("講義例：戊日 丙辰 戊辰 壬子 丁亥 — 日主根在兩個辰，坐下優先", () => {
  const branches = [{ col: 0, branch: "辰" }, { col: 1, branch: "辰" }, { col: 2, branch: "子" }, { col: 3, branch: "亥" }];
  const roots = stemRoots("戊", 1, branches);
  assert.deepEqual(roots.map((r) => [r.col, r.level, r.seated]), [[1, "本氣", true], [0, "本氣", false]]);
  assert.deepEqual(stemRoots("丁", 3, branches), []);
});
