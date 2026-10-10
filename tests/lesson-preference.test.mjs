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

test('2026-09-02 chart: 金水喜用、木火土忌 across both plans', () => {
  const prefs = lessonPreferences(source);
  assert.deepEqual(
    Object.fromEntries(Object.values(prefs.elements).map((item) => [item.name, item.favor])),
    { 木: 'bad', 火: 'bad', 土: 'bad', 金: 'good', 水: 'good' },
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

test('沖、合、伏吟 follow the natal side and the combined element', () => {
  const prefs = lessonPreferences(source);
  assert.equal(judgeLuckRelation('子午相沖', '午', 'fire', prefs).favor, 'good');
  assert.equal(judgeLuckRelation('卯酉相沖', '酉', 'metal', prefs).favor, 'bad');
  // binds 喜用 癸水 and may turn into 忌 火: both unfavorable.
  assert.equal(judgeLuckRelation('戊癸合火', '癸', 'water', prefs).favor, 'bad');
  // removes 忌 午火 but may turn into 忌 火: depends on whether it really 化.
  const halfFire = judgeLuckRelation('寅午半合火', '午', 'fire', prefs);
  assert.equal(halfFire.favor, 'neutral');
  assert.match(halfFire.reason, /看是否成化/);
  assert.equal(judgeLuckRelation('伏吟', '癸', 'water', prefs).favor, 'good');
  assert.equal(judgeLuckRelation('卯午相破', '午', 'fire', prefs).favor, 'neutral');
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
