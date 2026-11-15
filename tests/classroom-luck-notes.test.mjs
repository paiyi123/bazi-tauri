import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

// 講義第 24 章第五節（2026-11-04）例一、例三、例八；analysis 是 Rust 課堂三步法的真實輸出。
const charts = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-1104-charts.json'), 'utf8'));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-classroom-luck-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { classroomLuckEvents } = await server.ssrLoadModule('/src/utils/classroomLuckNotes.ts');

const kinds = (events) => events.map((event) => event.kind);

test('例八 印格身旺：金運要帶土', () => {
  const { natal, analysis } = charts.example8;
  // 辛酉：純金，沒有土 → 金生水（印）
  assert.ok(kinds(classroomLuckEvents('辛酉', '運', { natal, analysis })).includes('classroom-officer-feeds-seal'));
  // 庚申：申藏戊土 → 金帶土
  assert.ok(kinds(classroomLuckEvents('庚申', '運', { natal, analysis })).includes('classroom-officer-with-wealth'));
  // 流年不套這條
  assert.deepEqual(kinds(classroomLuckEvents('辛酉', '歲', { natal, analysis })).filter((kind) => kind.startsWith('classroom-officer')), []);
});

test('例一 土厚埋金（母慈滅子）：木運印旺財折，金運比劫洩印', () => {
  const { natal, analysis } = charts.example1;
  assert.ok(analysis.strength.motherOverload);
  assert.ok(kinds(classroomLuckEvents('甲寅', '運', { natal, analysis })).includes('classroom-seal-breaks-wealth'));
  assert.ok(kinds(classroomLuckEvents('庚申', '運', { natal, analysis })).includes('classroom-peer-drains-seal'));
});

test('例三 中和：看運重點在沖', () => {
  const { natal, analysis } = charts.example3;
  assert.ok(analysis.strength.balanceHint);
  // 甲午運：午沖月支子
  const clash = classroomLuckEvents('甲午', '運', { natal, analysis });
  assert.ok(kinds(clash).includes('classroom-balance-clash'));
  assert.match(clash.find((event) => event.kind === 'classroom-balance-clash').detail, /月/);
  // 乙丑運：不沖
  assert.deepEqual(kinds(classroomLuckEvents('乙丑', '運', { natal, analysis })), []);
  // 流年和大運相沖（歲運逢沖）
  assert.ok(kinds(classroomLuckEvents('乙未', '歲', { natal, analysis, luck: '癸丑' })).includes('classroom-balance-clash'));
});
