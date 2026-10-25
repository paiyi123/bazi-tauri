import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const load = (name) => JSON.parse(readFileSync(resolve('tests/fixtures', name), 'utf8'));
const source = load('lesson-source.json');
const general = load('lesson-general.json');
const insufficient = load('lesson-insufficient.json');

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-lesson-preference-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { lessonPreferences, judgeLuckRelation, combineJudgements } = await server.ssrLoadModule('/src/utils/lessonPreference.ts');

const side = (char, element) => ({ char, element });

test('2026-09-02 chart: 身強，用火財、喜木，忌金水', () => {
  const prefs = lessonPreferences(source);
  assert.deepEqual(
    Object.fromEntries(Object.values(prefs.elements).map((item) => [item.name, item.favor])),
    { 木: 'good', 火: 'good', 土: 'neutral', 金: 'bad', 水: 'bad' },
  );
});

test('conflicting plans are not merged into one verdict', () => {
  const prefs = lessonPreferences(general);
  // 印土 is 忌 in every plan; 比劫金 is 忌 in two plans and unmentioned elsewhere.
  assert.equal(prefs.elements.earth.favor, 'bad');
  assert.equal(prefs.elements.metal.favor, 'bad');
  // 食傷水 is 用 in one plan, 喜 in another and 待比較 in two: no plan says 忌.
  assert.equal(prefs.elements.water.favor, 'good');
  assert.ok(prefs.elements.water.notes.some((note) => note.includes('待比較')));
});

test('no plans means no 喜忌 judgement', () => {
  assert.equal(lessonPreferences(insufficient), null);
});

test('沖 follows the natal side: 沖到喜神不可，沖到忌神可以', () => {
  const prefs = lessonPreferences(source);
  // 2026-09-02 己卯運：卯沖酉，忌神（印）受傷 → 有利。
  assert.equal(judgeLuckRelation('卯酉相沖', 'branch', side('卯', 'wood'), side('酉', 'metal'), prefs).favor, 'good');
  // 2026-09-02 丙子運：子午沖入夫妻宮喜神財星 → 不利。
  assert.equal(judgeLuckRelation('子午相沖', 'branch', side('子', 'water'), side('午', 'fire'), prefs).favor, 'bad');
});

test('地支有合無化：branch combinations are judged as plain 生剋', () => {
  const prefs = lessonPreferences(source);
  // 寅亥合看作水生木：亥水（忌）生原局寅木（喜）→ 有利。
  const yinHai = judgeLuckRelation('寅亥合木', 'branch', side('亥', 'water'), side('寅', 'wood'), prefs);
  assert.equal(yinHai.favor, 'good');
  assert.match(yinHai.reason, /地支有合無化/);
  // 辰酉合看作土生金：辰土生原局酉金（忌）→ 不利。
  assert.equal(judgeLuckRelation('辰酉合金', 'branch', side('辰', 'earth'), side('酉', 'metal'), prefs).favor, 'bad');
});

test('天干五合 weighs 合絆 against the possible 合化', () => {
  const prefs = lessonPreferences(source);
  // 戊癸合火：合去原局癸水（忌），若化火也是喜用 → 有利。
  assert.equal(judgeLuckRelation('戊癸合火', 'stem', side('戊', 'earth'), side('癸', 'water'), prefs).favor, 'good');
  // 甲己合土：合絆原局甲木（喜），化土未列喜忌 → 不利。
  assert.equal(judgeLuckRelation('甲己合土', 'stem', side('己', 'earth'), side('甲', 'wood'), prefs).favor, 'bad');
});

test('伏吟 follows the repeated element; 害 is not used', () => {
  const prefs = lessonPreferences(source);
  assert.equal(judgeLuckRelation('伏吟', 'stem', side('癸', 'water'), side('癸', 'water'), prefs).favor, 'bad');
  const harm = judgeLuckRelation('卯辰相害', 'branch', side('辰', 'earth'), side('卯', 'wood'), prefs);
  assert.equal(harm.favor, 'neutral');
  assert.match(harm.reason, /神煞/);
});

test('opposite verdicts on one tag cancel out', () => {
  assert.equal(
    combineJudgements([
      { favor: 'good', reason: 'a' },
      { favor: 'bad', reason: 'b' },
    ]).favor,
    'neutral',
  );
});

test('2026-10-07：大運天干五合，化不化看大運坐支', async () => {
  const { seatTransforms } = await server.ssrLoadModule('/src/utils/lessonPreference.ts');
  // 68 年次男命（己未 壬申 甲戌 己巳）喜水木、忌火土金。
  const elements = Object.fromEntries(
    [['wood', '木', 'good'], ['water', '水', 'good'], ['fire', '火', 'bad'], ['earth', '土', 'bad'], ['metal', '金', 'bad']].map(
      ([element, name, favor]) => [element, { element, name, favor, label: favor === 'good' ? '喜用' : '忌', notes: [] }],
    ),
  );
  const prefs = { elements, planTitles: ['身弱財強：比劫分財'] };
  // 丁卯運：丁壬合，大運坐卯 → 化木，喜 → 有利（老師：財運走上坡）。
  const dingMao = judgeLuckRelation('丁壬合木', 'stem', { char: '丁', element: 'fire', seat: '卯' }, side('壬', 'water'), prefs);
  assert.equal(dingMao.favor, 'good');
  assert.match(dingMao.reason, /坐卯.*化為木/);
  // 同樣丁壬合，大運坐未（不是木的旺地）→ 合而不化，合絆原局壬水（喜）→ 減吉。
  const dingWei = judgeLuckRelation('丁壬合木', 'stem', { char: '丁', element: 'fire', seat: '未' }, side('壬', 'water'), prefs);
  assert.equal(dingWei.favor, 'bad');
  assert.match(dingWei.reason, /合而不化.*減吉/);
  // 甲運坐丑：甲己合化土（10/07 例）；本盤土為忌 → 不利。
  assert.equal(judgeLuckRelation('甲己合土', 'stem', { char: '甲', element: 'wood', seat: '丑' }, side('己', 'earth'), prefs).favor, 'bad');

  assert.equal(seatTransforms('卯', 'wood'), true);
  assert.equal(seatTransforms('丑', 'earth'), true);
  assert.equal(seatTransforms('申', 'water'), true); // 申金生水且藏壬
  assert.equal(seatTransforms('酉', 'water'), false); // 酉不藏水
  assert.equal(seatTransforms('亥', 'fire'), false);
});
