import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-basic-structure-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { basicStructure, dominantPattern, patternLuckEvents } = await server.ssrLoadModule('/src/utils/basicStructure.ts');
const { annualEvents, majorLuckEvents } = await server.ssrLoadModule('/src/utils/luckEvents.ts');
const { organStates } = await server.ssrLoadModule('/src/utils/lifeTopics.ts');
const labels = (events) => events.map((event) => event.label);

test('附錄範例杜月笙：五行字數、旺相休囚死、調候 −6、正官格、身弱取印比', () => {
  const result = basicStructure(['戊子', '庚申', '乙丑', '壬午'], '弱');
  const cells = Object.fromEntries(result.elements.map((row) => [row.element, `${row.main}(${row.attached})${row.state}`]));
  assert.deepEqual(cells, { 木: '1(0)死', 火: '1(0)囚', 土: '2(2)休', 金: '2(1)旺', 水: '2(2)相' });
  assert.deepEqual(result.same, { main: 3, attached: 2 });
  assert.deepEqual(result.opposite, { main: 5, attached: 3 });
  assert.equal(result.climate.total, -6);
  assert.equal(result.climate.level, '中和');
  assert.equal(result.pattern.name, '正官格');
  assert.equal(result.useGod.useGod, '比劫、印星');
});

test('主導勢力取格：書中 15 個命例至少 13 個與作者相同', () => {
  const cases = [
    ['杜月笙', ['戊子', '庚申', '乙丑', '壬午'], '正官格'],
    ['證嚴法師', ['丁丑', '甲辰', '辛卯', '己丑'], '偏印格'],
    ['張敬堯', ['辛巳', '丁酉', '乙卯', '己卯'], '七殺格'],
    ['李師科', ['丁卯', '壬寅', '戊戌', '庚申'], '七殺格'],
    ['毛澤東', ['癸巳', '甲子', '丁酉', '甲辰'], '七殺格'],
    ['徐樂吾', ['丙戌', '壬辰', '丙申', '丙申'], '七殺格'],
    ['戴笠', ['丁酉', '乙巳', '丙戌', '丁酉'], '建祿格'],
    ['南懷瑾', ['戊午', '乙卯', '甲子', '乙亥'], '劫刃格'],
    ['唐子培', ['丁丑', '己酉', '丙午', '己丑'], '正財格'],
    ['葉澄衷', ['庚子', '癸未', '戊寅', '壬子'], '正財格'],
    ['趙孟頫', ['甲寅', '甲戌', '己酉', '己巳'], '正官格'],
    ['張大千', ['己亥', '己巳', '戊寅', '辛酉'], '建祿格'],
    ['曾國藩', ['辛未', '己亥', '丙辰', '己亥'], '七殺格'],
    ['教授', ['己丑', '壬申', '壬辰', '辛亥'], '正印格'],
    ['少將', ['癸巳', '庚申', '乙巳', '己卯'], '正官格'],
  ];
  const misses = cases.filter(([, natal, expected]) => dominantPattern(natal).name !== expected).map(([name]) => name);
  assert.ok(misses.length <= 2, `不符：${misses.join('、')}`);
});

test('證嚴法師：偏印格、身強、印多 → 表 7.1 取財星（書取甲木正財）', () => {
  const result = basicStructure(['丁丑', '甲辰', '辛卯', '己丑'], '強');
  assert.equal(result.useGod.most, '印星');
  assert.equal(result.useGod.useGod, '財星');
});

test('張敬堯：七殺格透丁火食神 → 成格（食神制殺）', () => {
  const result = basicStructure(['辛巳', '丁酉', '乙卯', '己卯'], '稍強');
  assert.ok(result.checks.some((check) => check.label === '成格：食神制殺'));
});

test('杜月笙運程：丙寅傷官運破官格，辛酉運官殺混雜', () => {
  const natal = ['戊子', '庚申', '乙丑', '壬午'];
  assert.ok(labels(patternLuckEvents('丙寅', '運', natal, '正官格')).includes('破格'));
  assert.ok(labels(patternLuckEvents('辛酉', '運', natal, '正官格')).includes('官殺混雜'));
  assert.deepEqual(patternLuckEvents('壬戌', '運', natal, '正官格'), []);
});

test('毛澤東七殺格：丙辰年辰為水墓 → 旺殺投墓', () => {
  const ctx = { natal: ['癸巳', '甲子', '丁酉', '甲辰'], pattern: '七殺格' };
  assert.ok(labels(annualEvents('丙辰', '壬戌', ctx)).includes('旺殺投墓'));
  assert.ok(!labels(annualEvents('丙辰', '壬戌', { ...ctx, pattern: '正官格' })).includes('旺殺投墓'));
});

test('張作霖：甲戌運沖日柱庚辰，戊辰年又沖大運 → 三連環沖', () => {
  const ctx = { natal: ['乙亥', '己卯', '庚辰', '丁丑'] };
  assert.ok(labels(annualEvents('戊辰', '甲戌', ctx)).includes('三連環沖'));
});

test('瞿鴻禨：丁未年與時柱丁丑天比地沖', () => {
  const ctx = { natal: ['庚戌', '癸未', '乙亥', '丁丑'] };
  assert.ok(labels(annualEvents('丁未', '己卯', ctx)).includes('天比地沖(時)'));
});

test('張大千：原局寅巳亥，運歲補上申 → 四沖', () => {
  const ctx = { natal: ['己亥', '己巳', '戊寅', '辛酉'] };
  assert.ok(labels(majorLuckEvents('庚申', ctx)).includes('四沖'));
  assert.ok(!labels(majorLuckEvents('庚午', ctx)).includes('四沖'));
});

test('晦氣流年與敗亡地', () => {
  assert.ok(labels(annualEvents('己丑', '丙寅', { natal: ['庚午', '戊寅', '甲子', '丙寅'] })).includes('晦氣(日)'));
  const weak = { natal: ['庚申', '戊子', '甲申', '庚午'], verdict: '很弱' };
  assert.ok(labels(majorLuckEvents('壬申', weak)).includes('敗亡地'));
  assert.ok(!labels(majorLuckEvents('壬申', { ...weak, verdict: '弱' })).includes('敗亡地'));
});

test('健康：命例 35 火最弱（亥月死位）；命例 36 水最弱、己土太過戊土不及', () => {
  const heart = organStates(['己丑', '乙亥', '辛酉', '壬辰']);
  assert.ok(heart.some((state) => state.kind === 'weak' && state.element === '火'));
  const kidney = organStates(['甲午', '辛未', '壬午', '己酉']);
  assert.ok(kidney.some((state) => state.kind === 'weak' && state.element === '水'));
  assert.ok(kidney.some((state) => state.kind === 'imbalance' && state.element === '土'));
});
