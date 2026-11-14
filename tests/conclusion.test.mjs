import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

// 2026-09-02 課堂命例：甲寅 / 癸酉 / 壬午 / 癸卯（年 / 月 / 日 / 時）。
const analysis = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-source.json'), 'utf8'));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-conclusion-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { conclusionOf, findCurrentLuck } = await server.ssrLoadModule('/src/utils/conclusion.ts');

const pillar = (ganZhi) => ({ ganZhi, stem: ganZhi[0], branch: ganZhi[1] });
function makeResult({ quantLabel = '偏旺', luVerdict = '強', daYun = [] } = {}) {
  return {
    dayMaster: '壬水',
    yearPillar: pillar('甲寅'),
    monthPillar: pillar('癸酉'),
    dayPillar: pillar('壬午'),
    hourPillar: pillar('癸卯'),
    daYun,
    quantModel: {
      strengthLabel: quantLabel,
      luStrength: { verdict: luVerdict, category: luVerdict, roots: [{}] },
      lessonContent: { threeStepAnalysis: analysis },
    },
  };
}

test('結論卡：強弱、用神、喜忌都來自課堂三步法', () => {
  const data = conclusionOf(makeResult());
  assert.equal(data.strength.label, '強候選');
  assert.equal(data.strength.checks, '得令✗ 得勢✓ 得地✗');
  assert.equal(data.strength.party, '同黨 4：異黨 4');
  assert.deepEqual(data.useElements.map((item) => item.name), ['火']);
  assert.deepEqual(data.favorable.map((item) => item.name), ['木', '火']);
  assert.deepEqual(data.unfavorable.map((item) => item.name), ['金', '水']);
});

test('結論卡：三方法同一邊為一致，有方法判另一邊為邊界盤', () => {
  assert.equal(conclusionOf(makeResult()).agreement.status, 'agree');
  const split = conclusionOf(makeResult({ quantLabel: '偏弱' })).agreement;
  assert.equal(split.status, 'split');
  assert.match(split.text, /算分法判為身弱/);
});

test('結論卡：目前大運與今年依喜忌標 ▲▼', () => {
  const daYun = [
    { index: 1, ganZhi: '丙午', startYear: 2020, endYear: 2029, liuNian: [{ year: 2026, ganZhi: '丙午' }, { year: 2027, ganZhi: '丁未' }] },
    { index: 2, ganZhi: '丁未', startYear: 2030, endYear: 2039, liuNian: [{ year: 2030, ganZhi: '庚戌' }] },
  ];
  const { luck, annual } = findCurrentLuck(daYun, 2026);
  assert.equal(luck.ganZhi, '丙午');
  assert.equal(annual.ganZhi, '丙午');
  const data = conclusionOf(makeResult({ daYun }), 2026);
  assert.equal(data.luck.overall, 'good');
  assert.equal(data.annual.overall, 'good');
  // 丙午與日柱壬午：流年同大運 → 歲運並臨（陸致極較嚴重的提示）
  assert.ok(data.highlights.some((item) => item.label.includes('歲運並臨')));
  assert.ok(data.highlights.length <= 3);
});

test('結論卡：沒有課堂分析時回傳 null', () => {
  const result = makeResult();
  result.quantModel.lessonContent = undefined;
  assert.equal(conclusionOf(result), null);
});
