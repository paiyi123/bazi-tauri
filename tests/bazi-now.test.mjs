import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-now-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { liChunTime, baziYearAt, currentLuckIndex, luckStartTimes } = await server.ssrLoadModule('/src/utils/baziNow.ts');
const { luckRows } = await server.ssrLoadModule('/src/utils/luckPreview.ts');
const { conclusionOf } = await server.ssrLoadModule('/src/utils/conclusion.ts');

// 台灣時間（UTC+8）
const tw = (y, mo, d, h = 12, mi = 0) => new Date(Date.UTC(y, mo - 1, d, h - 8, mi));

test('立春時刻：和 tyme4rs 相差不到 10 分鐘', () => {
  // tyme4rs：2026 立春 2026-02-04 04:02:08、2025 立春 2025-02-03 22:10:28（北京時間）
  for (const [year, expected] of [[2026, tw(2026, 2, 4, 4, 2)], [2025, tw(2025, 2, 3, 22, 10)], [1985, tw(1985, 2, 4, 5, 12)]]) {
    const diff = Math.abs(liChunTime(year).getTime() - expected.getTime()) / 60000;
    assert.ok(diff < 10, `${year} 差 ${diff.toFixed(1)} 分`);
  }
});

test('今年以立春分年：1/1 到立春前還是上一個干支年', () => {
  assert.equal(baziYearAt(tw(2026, 1, 15)), 2025);
  assert.equal(baziYearAt(tw(2026, 2, 3, 23)), 2025);
  assert.equal(baziYearAt(tw(2026, 2, 4, 12)), 2026);
  assert.equal(baziYearAt(tw(2026, 12, 31)), 2026);
});

// 起運 2019-06-10 → 大運 1：2019–2028、大運 2：2029–2038（每步列 11 個流年，交運年 2029 兩步都有）
const years = (start) => Array.from({ length: 11 }, (_, i) => ({ index: i, year: start + i, ganZhi: '甲子' }));
const result = {
  luckStart: { startSolar: '2019-06-10 08:00:00' },
  daYun: [
    { index: 1, ganZhi: '丙午', startYear: 2019, endYear: 2028, startAge: 3, endAge: 12, liuNian: years(2019) },
    { index: 2, ganZhi: '丁未', startYear: 2029, endYear: 2038, startAge: 13, endAge: 22, liuNian: years(2029) },
    { index: 3, ganZhi: '戊申', startYear: 2039, endYear: 2048, startAge: 23, endAge: 32, liuNian: years(2039) },
  ],
};

test('目前大運看交運時刻：交運那年交運前仍是舊大運', () => {
  assert.equal(luckStartTimes(result).get(2).toISOString(), tw(2029, 6, 10, 8).toISOString());
  assert.equal(currentLuckIndex(result, tw(2029, 3, 1)), 1);
  assert.equal(currentLuckIndex(result, tw(2029, 6, 10, 9)), 2);
  assert.equal(currentLuckIndex(result, tw(2019, 1, 1)), null, '還沒起運');
  assert.deepEqual(luckRows(result, tw(2029, 8, 1)).map((row) => row.isCurrent), [false, true, false]);
  assert.deepEqual(luckRows(result, tw(2029, 3, 1)).map((row) => row.isCurrent), [true, false, false]);
});

test('結論卡：交運前看舊大運、立春前看上一年', () => {
  const analysis = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-source.json'), 'utf8'));
  const pillar = (ganZhi) => ({ ganZhi, stem: ganZhi[0], branch: ganZhi[1] });
  const full = { ...result, dayMaster: '壬水', yearPillar: pillar('甲寅'), monthPillar: pillar('癸酉'), dayPillar: pillar('壬午'), hourPillar: pillar('癸卯'), quantModel: { strengthLabel: '偏旺', luStrength: { verdict: '強', category: '強', roots: [{}] }, lessonContent: { threeStepAnalysis: analysis } } };
  const before = conclusionOf(full, tw(2029, 1, 20));
  assert.equal(before?.luck?.ganZhi, '丙午');
  assert.equal(before?.annual?.range, '2028');
  const after = conclusionOf(full, tw(2029, 7, 1));
  assert.equal(after?.luck?.ganZhi, '丁未');
  assert.equal(after?.annual?.range, '2029');
});
