import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const load = (name) => JSON.parse(readFileSync(resolve('tests/fixtures', name), 'utf8'));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-yong-shen-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { yongShenGuide, familyOf } = await server.ssrLoadModule('/src/utils/yongShen.ts');

const roles = (guide) => Object.fromEntries(guide.gods.map((row) => [row.element, row.roleLabel]));

test('2026-09-02 課堂命例（甲寅 癸酉 壬午 癸卯）：用火、喜木、忌水、仇金、閒土', () => {
  const guide = yongShenGuide(load('lesson-source.json'));
  assert.equal(guide.status, 'decided');
  assert.deepEqual(guide.useElements, ['火']);
  assert.deepEqual(roles(guide), { 火: '用神', 木: '喜神', 水: '忌神', 金: '仇神', 土: '閒神' });
  assert.equal(guide.teacherSummary, '喜火木，忌水金');
  assert.match(guide.headline, /用神火（財）/);
});

test('用神位置：午本氣可見、寅中藏丙，未透干', () => {
  const guide = yongShenGuide(load('lesson-source.json'));
  assert.deepEqual(
    guide.positions.map((item) => `${item.pillar}${item.place}${item.char}`),
    ['年柱寅藏中氣丙', '日柱地支本氣午'],
  );
  assert.match(guide.strengthVerdict, /藏而顯/);
});

test('相連：日支午火上有壬水蓋頭（忌）、鄰時支卯木生火（喜）', () => {
  const guide = yongShenGuide(load('lesson-source.json'));
  const noon = guide.neighbors.find((item) => item.at === '日支午');
  assert.deepEqual(noon.supporters, ['鄰時支卯木（喜）']);
  assert.deepEqual(noon.attackers, ['上蓋壬水（忌）']);
});

test('強弱步驟寫出同黨異黨與得令者為旺', () => {
  const guide = yongShenGuide(load('lesson-source.json'));
  const step = guide.steps.find((item) => item.label === '判強弱');
  assert.match(step.text, /同黨（印、比劫）4，異黨（食傷、財、官殺）4/);
  assert.match(step.text, /得令者為旺/);
});

test('條件不足的盤不硬定用神', () => {
  const guide = yongShenGuide(load('lesson-insufficient.json'));
  assert.equal(guide.status, 'insufficient');
  assert.deepEqual(guide.useElements, []);
  assert.match(guide.headline, /暫不定用神/);
});

test('多個方案時照實列出（lesson-general）', () => {
  const guide = yongShenGuide(load('lesson-general.json'));
  assert.ok(['decided', 'conflict', 'pending'].includes(guide.status));
  assert.equal(guide.gods.length, 5);
});

test('十神家族換算', () => {
  assert.equal(familyOf('水', '火'), '財');
  assert.equal(familyOf('水', '木'), '食傷');
  assert.equal(familyOf('水', '土'), '官殺');
  assert.equal(familyOf('水', '金'), '印');
  assert.equal(familyOf('水', '水'), '比劫');
});

test('沒有分析時回傳 null', () => {
  assert.equal(yongShenGuide(null), null);
});

test('剋用神的五行即使別的方案列喜，也不算喜神（lesson-general：用火，水改列依方案而定）', () => {
  const guide = yongShenGuide(load('lesson-general.json'));
  assert.deepEqual(guide.useElements, ['火']);
  const water = guide.gods.find((row) => row.element === '水');
  assert.equal(water.role, 'mixed');
  assert.match(water.reason, /水剋火/);
});
