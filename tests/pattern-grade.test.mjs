import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

// 蘇民峰《八字論命》第 12 章「格局高低」的命例（四柱照書，性別書上沒寫，一律當男命排大運）。
// fixture 由 App 的 Rust 核心排出（tools/bazi-report），只留總評用得到的欄位。
const cases = JSON.parse(readFileSync(resolve('tests/fixtures/pattern-grade-cases.json'), 'utf8'));
const byName = Object.fromEntries(cases.map((item) => [item.name, item.result]));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-pattern-grade-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { patternGrade } = await server.ssrLoadModule('/src/utils/patternGrade.ts');
const RANK = { 上: 4, 中上: 3, 中: 2, 中下: 1, 下: 0 };

test('總評：六個面向都有，每一條理由都有出處', () => {
  const grade = patternGrade(byName['春土坐亥']);
  assert.deepEqual(grade.dimensions.map((item) => item.key), ['formation', 'clarity', 'balance', 'useGod', 'flow', 'luck']);
  for (const dim of grade.dimensions) {
    assert.ok(dim.score <= 2 && dim.score >= -2, `${dim.title} 夾在 ±2`);
    for (const reason of dim.reasons) assert.ok(reason.source.length > 0, `${dim.title}：${reason.text} 沒有出處`);
  }
  assert.ok(grade.caveats[0].includes('不是命定'));
});

test('書上評為佳造的命，等級高於書上說濁、枯的命', () => {
  const good = ['春土坐亥', '陳濟棠'].map((name) => patternGrade(byName[name]));
  const poor = ['夏火變弱', '似清實枯'].map((name) => patternGrade(byName[name]));
  for (const item of good) assert.ok(RANK[item.level] >= RANK['中上'], `佳造得到 ${item.level}`);
  for (const item of poor) assert.ok(RANK[item.level] <= RANK['中下'], `濁、枯得到 ${item.level}`);
});

test('夏火變弱：用神、五行流通扣分', () => {
  const grade = patternGrade(byName['夏火變弱']);
  const dim = (key) => grade.dimensions.find((item) => item.key === key);
  assert.ok(dim('useGod').score < 0);
  assert.ok(dim('flow').score < 0);
});

test('課堂判「依據不足」時，用神與大運不計分，滿分跟著縮小', () => {
  const grade = patternGrade(byName['似清實枯']);
  const skipped = grade.dimensions.filter((item) => item.skipped).map((item) => item.key);
  assert.deepEqual(skipped, ['useGod', 'luck']);
  assert.equal(grade.max, 8);
});

test('沒有課堂三步分析時不給總評', () => {
  const result = { ...byName['陳濟棠'], quantModel: {} };
  assert.equal(patternGrade(result), null);
});
