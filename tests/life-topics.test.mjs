import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-life-topics-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { natalMarriage, natalWealth, natalHealth, climateScore, organStates, topicEvents, spouseStorageYears } =
  await server.ssrLoadModule('/src/utils/lifeTopics.ts');
const labels = (items) => items.map((item) => item.label);
const weakOrgans = (natal) => organStates(natal).filter((state) => state.kind === 'weak').map((state) => state.element);

test('寒暖燥濕：左宗棠 −4（中和），章太炎 −14（寒濕）', () => {
  assert.equal(climateScore(['壬申', '辛亥', '丙午', '庚寅']).total, -4);
  const zhang = climateScore(['戊辰', '乙丑', '癸卯', '庚申']);
  assert.equal(zhang.total, -14);
  assert.equal(zhang.level, '寒濕');
});

test('命例 98：唯一七殺藏未庫，30 歲乙丑沖開結婚、42 歲丁丑再沖離婚', () => {
  const ctx = { natal: ['乙未', '甲申', '辛亥', '戊子'], gender: 'FEMALE' };
  assert.ok(labels(natalMarriage(ctx).items).includes('夫星入庫'));
  const years = Array.from({ length: 60 }, (_, i) => {
    const year = 1955 + i;
    const stems = '甲乙丙丁戊己庚辛壬癸';
    const branches = '子丑寅卯辰巳午未申酉戌亥';
    return { year, age: i, ganZhi: stems[(year - 4) % 10] + branches[(year - 4) % 12] };
  });
  const result = spouseStorageYears(ctx, years);
  assert.equal(result.get(1985), 'first');
  assert.equal(result.get(1997), 'again');
  assert.equal(result.get(1973), undefined);
});

test('命例 85：唯一偏財藏丑庫，妻星入庫', () => {
  assert.ok(labels(natalMarriage({ natal: ['壬子', '甲辰', '丁丑', '丁未'], gender: 'MALE' }).items).includes('妻星入庫'));
});

test('馬雲丑運：財庫合入日支', () => {
  const ctx = { natal: ['甲辰', '甲戌', '丁酉', '戊申'], gender: 'MALE', strength: '強' };
  assert.ok(labels(topicEvents('丁丑', '運', ctx)).includes('財庫合入日支'));
});

test('命例 148：財露天干，戊運比劫奪財', () => {
  const ctx = { natal: ['辛丑', '癸巳', '戊辰', '戊午'], gender: 'MALE', strength: '強' };
  assert.ok(labels(natalWealth(ctx).items).includes('財露逢比劫'));
  assert.ok(labels(topicEvents('戊子', '運', ctx)).includes('比劫奪財'));
});

test('命例 150：丁火自坐丑財庫，原局刑沖已開庫', () => {
  const item = natalWealth({ natal: ['乙未', '己卯', '丁丑', '庚戌'], gender: 'MALE' }).items.find((entry) => entry.label.includes('財庫'));
  assert.ok(item);
  assert.equal(item.tone, 'good');
});

test('弱臟：命例 172 心（火）、174 肺（金）、176 肝（木）、179 腎（水）', () => {
  assert.ok(weakOrgans(['癸卯', '丁巳', '己酉', '癸酉']).includes('火'));
  assert.ok(weakOrgans(['癸卯', '戊午', '庚寅', '己卯']).includes('金'));
  assert.ok(weakOrgans(['庚子', '乙酉', '丙午', '庚寅']).includes('木'));
  assert.ok(weakOrgans(['壬戌', '丙午', '乙卯', '丙戌']).includes('水'));
});

test('強臟：命例 180 土旺、182 寅午戌火局', () => {
  assert.ok(organStates(['己丑', '己巳', '己亥', '戊辰']).some((s) => s.kind === 'strong' && s.element === '土'));
  assert.ok(organStates(['丙戌', '丙申', '甲寅', '庚午']).some((s) => s.kind === 'strong' && s.element === '火'));
});

test('命例 176 己丑運土生金剋木：運歲剋弱臟', () => {
  const ctx = { natal: ['庚子', '乙酉', '丙午', '庚寅'], gender: 'MALE' };
  assert.ok(labels(topicEvents('辛酉', '歲', ctx)).some((label) => label.startsWith('剋弱臟')));
});

test('曾國藩寒濕土虛：皮膚病提示', () => {
  assert.ok(labels(natalHealth({ natal: ['辛未', '己亥', '丙辰', '己亥'], gender: 'MALE' }).items).includes('水多土虛'));
});
