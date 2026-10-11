import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-ten-god-combos-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { tenGodOverview, monthLeader, movingGodEvents } = await server.ssrLoadModule('/src/utils/tenGodCombos.ts');
const labels = (natal) => tenGodOverview(natal).items.map((item) => item.label);

// 命例皆取自潘樂德《十神洩天機》，四柱依年、月、日、時。

test('薛相公（p67）申月取本氣：正官為領袖，官印相生、財官並舉', () => {
  const natal = ['甲申', '壬申', '乙巳', '戊寅'];
  assert.equal(monthLeader(natal).god, '正官');
  const result = labels(natal);
  assert.ok(result.includes('官印相生'));
  assert.ok(result.includes('財官並舉'));
});

test('偏印格逢財（p68）：酉月偏印為領袖，見財反辱為榮', () => {
  const natal = ['丙辰', '丁酉', '癸未', '丙辰'];
  assert.equal(monthLeader(natal).god, '偏印');
  assert.ok(labels(natal).includes('偏印逢財'));
});

test('食神格（p68–69）：丑月取透干的癸，食神為領袖', () => {
  assert.equal(monthLeader(['丙辰', '辛丑', '辛巳', '癸巳']).god, '食神');
});

test('顧鼎臣（p78）：卯月正印為領袖', () => {
  assert.equal(monthLeader(['癸巳', '乙卯', '丙戌', '乙未']).god, '正印');
});

test('閻錫山（p136）：七殺有印化又有食神制', () => {
  const result = labels(['癸未', '辛酉', '乙酉', '丁亥']);
  assert.ok(result.includes('殺印相生'));
  assert.ok(result.includes('食神制殺'));
});

test('伍廷芳（p134）：時干七殺得月干偏印化', () => {
  assert.ok(labels(['壬寅', '丁未', '己卯', '乙亥']).includes('殺印相生'));
});

test('馮玉祥（p30）：強傷官有印制', () => {
  assert.ok(labels(['壬午', '庚戌', '己酉', '庚午']).includes('傷官配印'));
});

test('p221 女命：傷官遇劫財', () => {
  assert.ok(labels(['壬戌', '庚戌', '辛未', '己亥']).includes('傷官劫財'));
});

test('女醫科博士（p277）：七個字中六個偏神', () => {
  const overview = tenGodOverview(['丙寅', '己亥', '癸酉', '甲寅']);
  assert.equal(overview.irregular, 6);
  assert.equal(overview.regular, 1);
});

test('p60 乾造：乙巳運偏財在庚寅、辛卯年得根，主為錢變動', () => {
  const natal = ['丙辰', '辛丑', '辛巳', '癸巳'];
  for (const annual of ['庚寅', '辛卯']) {
    const events = movingGodEvents(annual, '歲', natal, '乙巳');
    assert.ok(events.some((event) => event.kind === 'moving-偏財'), annual);
  }
});

test('動神無根時不提示', () => {
  // 辛日見乙為偏財，乙在酉年無根。
  assert.deepEqual(movingGodEvents('乙酉', '運', ['丙辰', '辛丑', '辛巳', '癸巳']), []);
});
