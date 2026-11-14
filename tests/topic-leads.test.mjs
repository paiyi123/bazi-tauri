import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

// 2026-09-02 課堂命例：甲寅 / 癸酉 / 壬午 / 癸卯（年 / 月 / 日 / 時），課堂喜木火、忌金水。
const analysis = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-source.json'), 'utf8'));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-topic-leads-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { topicLeads } = await server.ssrLoadModule('/src/utils/topicLeads.ts');

const pillar = (ganZhi) => ({ ganZhi, stem: ganZhi[0], branch: ganZhi[1] });
const result = (gender) => ({
  dayMaster: '壬水',
  gender,
  yearPillar: pillar('甲寅'),
  monthPillar: pillar('癸酉'),
  dayPillar: pillar('壬午'),
  hourPillar: pillar('癸卯'),
  daYun: [],
  quantModel: { strengthLabel: '偏旺', luStrength: { verdict: '強', category: '強', roots: [{}] }, lessonContent: { threeStepAnalysis: analysis } },
});

test('六親：男命妻星是財（火），配偶宮午火，都是喜', () => {
  const [star, note] = topicLeads(result('MALE'), 'family');
  assert.match(star.text, /妻星（財）是火，為喜/);
  assert.match(star.text, /配偶宮（日支午，火）為喜/);
  assert.equal(note.tone, 'good');
});

test('六親：女命夫星是官殺（土），土在課堂喜忌未列', () => {
  const [star] = topicLeads(result('FEMALE'), 'family');
  assert.match(star.text, /夫星（官殺）是土/);
});

test('健康：數八個字的五行，最多木水、最少土', () => {
  const [line] = topicLeads(result('MALE'), 'health');
  assert.match(line.text, /最多的是木、水（3 個/);
  assert.match(line.text, /最少的是土（0 個/);
});

test('財富：財星火為喜，身強能任財', () => {
  const [line] = topicLeads(result('MALE'), 'career');
  assert.match(line.text, /財星是火，為喜/);
  assert.match(line.text, /身能任財/);
  assert.equal(line.tone, 'good');
});

test('強弱：主答案用課堂，各方法一致', () => {
  const [main, agree] = topicLeads(result('MALE'), 'strength');
  assert.match(main.text, /^強候選/);
  assert.equal(agree.text, '各方法一致');
});
