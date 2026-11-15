import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

// 2026-09-02 課堂命例：甲寅 / 癸酉 / 壬午 / 癸卯（年 / 月 / 日 / 時），課堂強候選，用火，喜木火、忌金水。
const analysis = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-source.json'), 'utf8'));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-learning-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const load = (path) => server.ssrLoadModule(path);
const { readingSteps, expectedMonthStem, expectedHourStem } = await load('/src/utils/readingSteps.ts');
const { checkNotes, chartKey } = await load('/src/utils/verifyNotes.ts');
const { similarCases } = await load('/src/utils/similarCases.ts');
const { splitGlossary, glossaryEntry } = await load('/src/constants/glossary.ts');
const { disagreementsOf } = await load('/src/utils/disagreements.ts');
const { isLearned } = await load('/src/composables/useLearning.ts');
const { judgeLuckRelation, lessonPreferences } = await load('/src/utils/lessonPreference.ts');
const { natalRelationVerdicts, findNatalVerdict, previewRelationVerdict, relationFamily } = await load('/src/utils/relationFavor.ts');
const prefs = lessonPreferences(analysis); // 喜木火、忌金水

const pillar = (ganZhi) => ({ ganZhi, stem: ganZhi[0], branch: ganZhi[1] });
const daYun = [
  { index: 1, ganZhi: '丙午', startYear: 2020, endYear: 2029, liuNian: [
    { year: 2024, ganZhi: '甲辰', age: 51 },
    { year: 2025, ganZhi: '乙巳', age: 52 },
    { year: 2026, ganZhi: '丙午', age: 53 },
    { year: 2028, ganZhi: '戊申', age: 55 },
    { year: 2029, ganZhi: '己酉', age: 56 },
  ] },
];
const makeResult = (over = {}) => ({
  baZi: '甲寅 癸酉 壬午 癸卯',
  dayMaster: '壬水',
  gender: 'MALE',
  solarDateTime: '1974-09-20 06:00:00',
  yearPillar: pillar('甲寅'),
  monthPillar: pillar('癸酉'),
  dayPillar: pillar('壬午'),
  hourPillar: pillar('癸卯'),
  daYun,
  quantModel: { strengthLabel: '偏旺', luStrength: { verdict: '強', category: '強', roots: [{}] }, lessonContent: { threeStepAnalysis: analysis } },
  ...over,
});

test('五虎遁、五鼠遁', () => {
  assert.equal(expectedMonthStem('甲', '寅'), '丙');
  assert.equal(expectedMonthStem('甲', '酉'), '癸');
  assert.equal(expectedMonthStem('癸', '丑'), '乙');
  assert.equal(expectedHourStem('壬', '卯'), '癸');
  assert.equal(expectedHourStem('甲', '子'), '甲');
  assert.equal(expectedHourStem('戊', '亥'), '癸');
});

test('解盤 10 步：課堂命例第 1 步核對通過，第 4、6 步有內容', () => {
  const steps = readingSteps(makeResult());
  assert.equal(steps.length, 10);
  assert.deepEqual(steps.map((s) => s.no), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
  assert.equal(steps[0].status, 'ok');
  assert.match(steps[3].summary, /強/);
  assert.match(steps[6].summary, /喜 .*木/);
  assert.notEqual(steps[5].status, 'none');
});

test('解盤第 1 步：月柱不合五虎遁、子時交界會警告', () => {
  const steps = readingSteps(makeResult({ monthPillar: pillar('甲酉'), solarDateTime: '1974-09-20 23:30:00' }));
  assert.equal(steps[0].status, 'warn');
  assert.match(steps[0].why.join('\n'), /應該是癸酉/);
  assert.match(steps[0].why.join('\n'), /子時/);
});

test('解盤第 1 步：後端的出生時間提醒（節氣、夏令時間）列進核對', () => {
  const warn = readingSteps(makeResult({ solarDateTime: '1974-07-06 08:30:00', birthWarnings: [{ kind: 'correction', level: 'warn', title: '時間校正改變了四柱', detail: '時柱丁巳→丙辰' }], timeCorrection: { lines: ['台灣夏令時間：鐘錶 09:30 → 標準時間 08:30。'] } }));
  assert.equal(warn[0].status, 'warn');
  assert.match(warn[0].summary, /時間校正改變了四柱/);
  assert.match(warn[0].why.join('\n'), /夏令時間/);
  const info = readingSteps(makeResult({ solarDateTime: '1974-03-06 23:30:00', birthWarnings: [{ kind: 'jie', level: 'info', title: '出生在驚蟄前 6 小時', detail: '月柱丙寅→丁卯' }] }));
  assert.equal(info[0].status, 'info');
  assert.doesNotMatch(info[0].why.join('\n'), /接近子時/);
});

test('反驗筆記：好事在喜年、壞事在忌年算符合', () => {
  const notes = [
    { id: '1', year: 2026, kind: 'good', text: '升職' }, // 丙午 火 喜
    { id: '2', year: 2029, kind: 'bad', text: '生病' }, // 己酉：土未定，酉金忌 → 看流年
    { id: '3', year: 2028, kind: 'good', text: '中獎' }, // 戊申：申金忌
    { id: '4', year: 2025, kind: 'marriage', text: '結婚' },
    { id: '5', year: 1990, kind: 'good', text: '不在表內' },
  ];
  const out = checkNotes(makeResult(), notes);
  const byId = Object.fromEntries(out.notes.map((n) => [n.id, n]));
  assert.equal(byId['1'].match, 'match');
  assert.equal(byId['4'].match, 'none');
  assert.equal(byId['5'].view, null);
  assert.equal(out.notes[0].id, '5'); // 依年份排
  assert.equal(out.hits + out.misses >= 1, true);
});

test('反驗筆記：三件以上、一半以上不符時提醒', () => {
  const notes = [2024, 2025, 2026].map((year, i) => ({ id: String(i), year, kind: 'bad', text: '壞事' }));
  const out = checkNotes(makeResult(), notes);
  assert.equal(out.misses, 3);
  assert.equal(out.warn, true);
  assert.equal(out.rate, 0);
});

test('命盤鍵依八字、性別、時間分開', () => {
  assert.notEqual(chartKey(makeResult()), chartKey(makeResult({ gender: 'FEMALE' })));
});

test('相似命例：四柱相同排第一，不相干的盤不列', () => {
  const cases = [
    { book: 'A', file: 'a.md', line: 1, heading: '例一', natal: ['乙卯', '丁亥', '庚辰', '丙子'] },
    { book: 'B', file: 'b.md', line: 2, heading: '例二', natal: ['丙子', '丁酉', '壬午', '甲辰'] },
    { book: 'C', file: 'c.md', line: 3, heading: '例三', natal: ['甲寅', '癸酉', '壬午', '癸卯'] },
  ];
  const out = similarCases(['甲寅', '癸酉', '壬午', '癸卯'], cases);
  assert.deepEqual(out.map((c) => c.book), ['C', 'B']);
  assert.match(out[0].why, /四柱相同/);
  assert.match(out[1].why, /日干壬/);
});

test('名詞表：同一詞只標第一次，長詞優先，別名可查', () => {
  const pieces = splitGlossary('歲運並臨時看用神，用神是火');
  const marked = pieces.filter((p) => p.entry).map((p) => p.text);
  assert.deepEqual(marked, ['歲運並臨', '用神']);
  assert.equal(pieces.map((p) => p.text).join(''), '歲運並臨時看用神，用神是火');
  assert.equal(glossaryEntry('日元')?.term, '日主');
});

test('各書分歧：課堂判強但量化判弱時列出「得勢怎麼數」', () => {
  const list = disagreementsOf(makeResult({ quantModel: { strengthLabel: '偏弱', luStrength: { verdict: '強', category: '強', roots: [{}] }, lessonContent: { threeStepAnalysis: analysis } } }));
  assert.ok(list.some((item) => item.no === 1));
  const same = disagreementsOf(makeResult());
  assert.ok(!same.some((item) => item.no === 1 || item.no === 2));
});

test('課程進度：沒設定時全部算學過，設到強弱時用神還沒學', () => {
  assert.equal(isLearned('all', 'yongshen'), true);
  assert.equal(isLearned('strength', 'pattern'), true);
  assert.equal(isLearned('strength', 'yongshen'), false);
  assert.equal(isLearned('pattern', null), true);
});

// ---- 2026-11-04：刑讀成生剋、命運相沖不一定大運贏、括號線箭頭 ----

test('11/04 大運與原局的刑讀成五行生剋：子運刑原局卯＝水生木，木是喜', () => {
  const out = judgeLuckRelation('子卯刑', 'branch', { char: '子', element: 'water' }, { char: '卯', element: 'wood' }, prefs);
  assert.equal(out.favor, 'good');
  assert.match(out.reason, /刑讀成五行生剋/);
});

test('11/04 命運相沖：沖月令而原局沒有這個五行，不判有利', () => {
  const luck = { char: '卯', element: 'wood' };
  const natal = { char: '酉', element: 'metal' }; // 月令酉金是忌
  assert.equal(judgeLuckRelation('卯酉沖', 'branch', luck, natal, prefs).favor, 'good');
  const guarded = judgeLuckRelation('卯酉沖', 'branch', luck, natal, prefs, { natalIsMonthBranch: true, natalHasLuckElement: false });
  assert.equal(guarded.favor, 'neutral');
  assert.match(guarded.reason, /不一定沖得贏/);
});

test('括號線箭頭：原局用課堂已判的結果', () => {
  const list = natalRelationVerdicts(analysis);
  assert.ok(list.length > 0);
  const clash = list.find((item) => item.family === 'clash');
  assert.ok(clash, '課堂命例有卯酉沖');
  const found = findNatalVerdict(list, 'branch', '沖', [...clash.pillars].map((char) => `${char}柱`));
  assert.equal(found.favor, clash.favor);
  assert.equal(findNatalVerdict(list, 'branch', '破', ['年柱', '月柱']), null);
  assert.equal(relationFamily('半合'), 'combine');
});

test('括號線箭頭：大運午沖原局子（忌）有利；三合成火局（喜）有利', () => {
  const natalElements = new Set(['wood', 'water', 'metal', 'fire']);
  const clash = previewRelationVerdict({ scope: 'branch', type: '沖', luck: { char: '子' }, natal: [{ char: '午', isMonth: false }], natalElements }, prefs);
  assert.equal(clash.favor, 'bad'); // 沖動喜用的午火
  const gather = previewRelationVerdict({ scope: 'branch', type: '三合', elementName: '火', luck: null, natal: [{ char: '寅', isMonth: false }, { char: '午', isMonth: false }], natalElements }, prefs);
  assert.equal(gather.favor, 'good');
  assert.equal(previewRelationVerdict({ scope: 'branch', type: '沖', luck: null, natal: [], natalElements }, prefs).favor, 'neutral');
});

test('11/04 解盤第 2 步：月令被沖時提醒取格要看本氣', () => {
  const steps = readingSteps(makeResult({ monthHiddenStems: ['辛'], monthBranchTenGods: ['正印'] }));
  const touched = (analysis.branchInteractions?.clashes ?? []).some((item) => item.positions.some((p) => p.startsWith('月支')));
  assert.equal(steps[1].status, touched ? 'warn' : 'ok');
  if (touched) assert.match(steps[1].why.join('\n'), /本氣是辛（正印）/);
});
