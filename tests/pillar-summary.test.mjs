import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const source = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-source.json'), 'utf8'));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-pillar-summary-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { pillarSummary } = await server.ssrLoadModule('/src/utils/pillarSummary.ts');

test('四柱摘要：強弱、用神、喜忌沿用課堂三步法', () => {
  const summary = pillarSummary(source);
  assert.equal(summary.strength.label, '強候選');
  assert.equal(summary.strength.detail, '得令✗ 得勢✓ 得地✗；同黨 4：異黨 4');
  assert.equal(summary.strength.review, null);
  assert.deepEqual(summary.useGods, ['財（火）', '食傷（木）']);
  assert.equal(summary.planCount, 3);
  assert.deepEqual(summary.favorable.map((item) => item.name), ['木', '火']);
  assert.deepEqual(summary.unfavorable.map((item) => item.name), ['金', '水']);
});

test('四柱摘要：天干生剋與地支沖合只列結果', () => {
  const summary = pillarSummary(source);
  assert.deepEqual(summary.stemLines, []);
  assert.deepEqual(summary.stemRelations.map((item) => item.text), ['月癸生年甲', '日壬生年甲', '時癸生年甲']);
  assert.deepEqual(summary.branchLines.map((item) => [item.label, item.where, item.verdict]), [['酉卯沖・勝負分明', '月時', 'bad']]);
});

test('四柱摘要：作用變化改變強弱時提示複核', () => {
  const changed = structuredClone(source);
  changed.interactionStrength = { ...changed.interactionStrength, changed: true, classification: '弱候選', samePartyCount: 3, opposingPartyCount: 5 };
  assert.match(pillarSummary(changed).strength.review, /可能變為「弱候選」/);
  assert.equal(pillarSummary(null), null);
});
