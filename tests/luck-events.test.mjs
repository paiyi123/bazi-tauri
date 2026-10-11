import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-luck-events-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { majorLuckEvents, annualEvents } = await server.ssrLoadModule('/src/utils/luckEvents.ts');
const labels = (events) => events.map((event) => event.label);

// 三毛（進階 命例 40）：癸未 乙卯 癸未 甲寅。
const sanmao = { natal: ['癸未', '乙卯', '癸未', '甲寅'], strength: '弱', rootless: true };

test('三毛 1972 壬子年在戊午運：歲運交戰', () => {
  assert.ok(labels(annualEvents('壬子', '戊午', sanmao)).includes('歲運交戰'));
});

test('三毛 1979 己未年在己未運：歲運並臨', () => {
  assert.ok(labels(annualEvents('己未', '己未', sanmao)).includes('歲運並臨'));
});

test('三毛庚申運直沖甲寅時柱：反吟(時)', () => {
  assert.ok(labels(majorLuckEvents('庚申', sanmao)).includes('反吟(時)'));
});

// 朱熹（進階 p30）：庚戌 丙戌 甲寅 庚午；1200 庚申年在癸巳運。
const zhuxi = { natal: ['庚戌', '丙戌', '甲寅', '庚午'], strength: '強' };

test('朱熹 1200 庚申年：天剋地沖日柱，寅巳申連環三刑', () => {
  const result = labels(annualEvents('庚申', '癸巳', zhuxi));
  assert.ok(result.includes('反吟(日)'));
  assert.ok(result.includes('三刑'));
});

test('朱熹 1196 丙辰年：辰沖戌庫', () => {
  const result = labels(annualEvents('丙辰', '癸巳', zhuxi));
  assert.ok(result.includes('沖提'));
  assert.ok(result.some((label) => label.startsWith('沖庫')));
});

test('丁榮錦第六步戊午運沖月令子：沖提', () => {
  assert.ok(labels(majorLuckEvents('戊午', { natal: ['癸卯', '甲子', '丙寅', '丁酉'] })).includes('沖提'));
});

test('日主剋流年天干：日犯歲君', () => {
  assert.ok(labels(annualEvents('戊辰', '乙卯', { natal: ['甲子', '丙寅', '甲午', '丁卯'] })).includes('日犯歲君'));
});

test('原局有日祿而運歲沖之：沖祿', () => {
  // 丙日祿在巳，時支巳被乙亥年沖。
  assert.ok(labels(annualEvents('乙亥', '庚子', { natal: ['辛未', '己亥', '丙午', '癸巳'] })).includes('沖祿'));
});

test('沒有特殊組合時不產生提示', () => {
  assert.deepEqual(annualEvents('乙丑', '丁卯', { natal: ['甲子', '丙寅', '戊辰', '庚申'] }), []);
});
