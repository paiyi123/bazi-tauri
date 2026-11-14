import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

// 命例由 Rust 排盤核心實際算出（tools/bazi-report 的 core），只留課堂三步法的部分。
const charts = JSON.parse(readFileSync(resolve('tests/fixtures/strength-shift-charts.json'), 'utf8'));

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-strength-shift-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { strengthShift, cellStatesAt, bonusesAt, trendAt, helpsParty } = await server.ssrLoadModule('/src/utils/strengthShift.ts');

const effectsOf = (shift, title) => shift.steps.find((step) => step.title.includes(title))?.effects.map((effect) => `${effect.cell}${effect.dir}:${effect.text}`);

test('強弱變化圖：沒有課堂分析時不顯示', () => {
  assert.equal(strengthShift(null), null);
  assert.equal(strengthShift(undefined), null);
});

test('強弱變化圖：講義天干例三，丁壬合化木讓兩個天干換邊，子辰半合再加一', () => {
  const shift = strengthShift(charts.t3.analysis);
  assert.deepEqual([shift.before.same, shift.before.opp, shift.before.classification], [5, 3, '強候選']);
  assert.deepEqual([shift.after.same, shift.after.opp, shift.after.classification], [4, 5, '弱候選']);
  assert.equal(shift.changed, true);
  // 每一步之後的人數：原局 5:3 → 合化後 4:4 → 半合後 4:5，之後不變
  assert.deepEqual(shift.timeline.slice(0, 3).map((count) => `${count.same}:${count.opp}`), ['5:3', '4:4', '4:5']);
  assert.equal(shift.timeline[1].momentum, false, '4:4 平手而月令在異黨，不得勢');
  assert.equal(shift.timeline.at(-1).classification, '弱候選');

  const first = shift.steps[0];
  assert.equal(first.tag, '合而能化');
  assert.equal(first.counted, true);
  assert.deepEqual([first.deltaSame, first.deltaOpp], [-1, 1], '丁（印）換到異黨，壬本來就在異黨');
  assert.deepEqual(first.effects.map((effect) => `${effect.cell}${effect.text}`), ['年干化木', '月干化木']);

  // 第 1 步之後，年干丁變成木、歸異黨官殺；原局時還是同黨的印
  assert.equal(cellStatesAt(shift, 0).get('年干').party, 'same');
  const afterFirst = cellStatesAt(shift, 1).get('年干');
  assert.deepEqual([afterFirst.element, afterFirst.party, afterFirst.family, afterFirst.transformed], ['木', 'opp', '官殺', true]);

  const half = shift.steps[1];
  assert.equal(half.tag, '半合');
  assert.deepEqual(half.bonus, { party: 'opp', label: '水局' });
  assert.deepEqual(bonusesAt(shift, 1), []);
  assert.equal(bonusesAt(shift, 2).length, 1);
  assert.match(shift.caution, /翻轉/);
});

test('強弱變化圖：合化後的天干，同柱改用化神的五行來看', () => {
  const shift = strengthShift(charts.t3.analysis);
  // 丁亥原本是水剋火（截腳），丁化木之後是水生木
  assert.deepEqual(effectsOf(shift, '年柱丁亥'), ['年干up:坐下得生']);
  assert.match(shift.steps.find((step) => step.title.includes('年柱丁亥')).detail, /已合化為木/);
  // 沒有合化的柱照原樣：戊辰干支同類
  assert.deepEqual(effectsOf(shift, '日柱戊辰'), ['日干up:通根']);
});

test('強弱變化圖：講義地支例三，寅卯辰三會木讓平手的盤翻成弱', () => {
  const shift = strengthShift(charts.b3.analysis);
  assert.deepEqual([shift.before.same, shift.before.opp, shift.before.classification], [4, 4, '強候選']);
  assert.deepEqual([shift.after.same, shift.after.opp, shift.after.classification], [4, 5, '弱候選']);
  const meeting = shift.steps.find((step) => step.tag === '三會');
  assert.equal(meeting.counted, true);
  assert.deepEqual(meeting.effects.map((effect) => `${effect.cell}${effect.dir}:${effect.text}`), ['月支down:入木局受制', '日支up:成木局', '時支up:成木局']);
  // 步驟順序：天干 → 地支 → 同柱
  const groups = shift.steps.map((step) => step.group);
  assert.deepEqual([...new Set(groups)], ['stem', 'branch', 'pillar']);
});

test('強弱變化圖：沖、合來合去、六合只標方向，不改人數', () => {
  const shift = strengthShift(charts.x6.analysis);
  assert.deepEqual([shift.after.same, shift.after.opp], [shift.before.same, shift.before.opp]);
  assert.equal(shift.changed, false);
  assert.equal(shift.steps.some((step) => step.counted), false);
  assert.deepEqual(effectsOf(shift, '辛丙合而不化'), ['年干bind:被絆']);
  const coming = shift.steps.find((step) => step.tag === '合來');
  assert.equal(coming.effects[0].neutral, true, '合來只是貼向日主，不算增減');
  assert.deepEqual(effectsOf(shift, '申寅沖'), ['月支down:沖・兩傷', '時支down:沖・兩傷']);
  assert.deepEqual(effectsOf(shift, '亥寅合'), ['時支up:生合']);
  // 時支寅是日主的根，被沖：得地要打折
  assert.deepEqual(shift.lights.rootShaken, ['時支寅']);
  assert.ok(shift.summary.some((line) => line.includes('得地要打折')));
});

test('強弱變化圖：方向統計可以算到任一步，並分出對哪一邊有利', () => {
  const shift = strengthShift(charts.x6.analysis);
  assert.deepEqual(trendAt(shift, 0), { sameUp: 0, sameDown: 0, oppUp: 0, oppDown: 0, net: 0 });
  assert.deepEqual(trendAt(shift, shift.steps.length), shift.trend);
  // 異黨的字減力＝對日主這邊有利；同黨的字減力＝對異黨有利；合來不算
  assert.equal(helpsParty({ cell: '月支', dir: 'down', text: '沖敗' }, 'opp'), 'same');
  assert.equal(helpsParty({ cell: '日干', dir: 'down', text: '截腳' }, 'same'), 'opp');
  assert.equal(helpsParty({ cell: '時支', dir: 'up', text: '生合' }, 'same'), 'same');
  assert.equal(helpsParty({ cell: '時干', dir: 'bind', text: '合來', neutral: true }, 'opp'), null);
});

test('強弱變化圖：69 年女命，卯未半合只加一，刑在月令加強土，仍判旺', () => {
  const shift = strengthShift(charts.f69.analysis);
  assert.deepEqual([shift.before.same, shift.before.opp], [5, 3]);
  assert.deepEqual([shift.after.same, shift.after.opp, shift.after.classification], [5, 4, '旺候選']);
  assert.deepEqual(effectsOf(shift, '戌未相刑'), ['月支up:刑在月令・加強', '日支up:刑在月令・加強']);
  assert.deepEqual(effectsOf(shift, '戌卯合'), ['月支down:剋合（輕）']);
  // 和老師的判斷（身弱）不同：圖只照課堂規則數人頭，不會自動改成弱
  assert.equal(shift.changed, false);
});

test('強弱變化圖：刑落在日時而不在月令，標成鬆動', () => {
  const shift = strengthShift(charts.t3.analysis);
  assert.deepEqual(effectsOf(shift, '辰辰自刑'), ['日支down:刑在日時・鬆動', '時支down:刑在日時・鬆動']);
  assert.match(shift.steps.find((step) => step.title.includes('辰辰自刑')).detail, /老師 11-04/);
});

// ───────── 加上大運 ─────────
const luckSteps = (shift) => shift.steps.slice(shift.luck.firstStep);

test('強弱變化圖＋大運：不加大運時沒有大運資訊，加了之後原局的步驟與判定不變', () => {
  const plain = strengthShift(charts.t3.analysis);
  assert.equal(plain.luck, null);
  assert.equal(plain.cells.length, 8);
  const shift = strengthShift(charts.t3.analysis, { ganZhi: '戊午', label: '23–33 歲' });
  assert.equal(shift.luck.firstStep, plain.steps.length);
  assert.deepEqual(shift.steps.slice(0, plain.steps.length).map((step) => step.id), plain.steps.map((step) => step.id));
  assert.deepEqual(shift.after, plain.after, '原局的強弱不因大運改判');
  assert.equal(shift.cells.length, 10);
  assert.deepEqual(shift.cells.slice(8).map((cell) => `${cell.id}${cell.char}${cell.tenGod}${cell.party}`), ['運干戊比肩same', '運支午正印same']);
  // 大運的每一步：人數含大運，但判定仍是原局的
  const last = shift.timeline.at(-1);
  assert.deepEqual([last.same, last.opp, last.classification, last.withLuck], [6, 5, '弱候選', true]);
});

test('強弱變化圖＋大運：戊午運幫身弱的戊土，午沖月令子', () => {
  const shift = strengthShift(charts.t3.analysis, { ganZhi: '戊午' });
  const [enter, pillar, clash] = luckSteps(shift);
  assert.deepEqual([enter.tag, enter.counted, enter.deltaSame, enter.deltaOpp], ['大運', true, 2, 0]);
  assert.deepEqual(pillar.effects.map((effect) => `${effect.cell}${effect.dir}:${effect.text}`), ['運干up:坐下得生']);
  assert.deepEqual(clash.effects.map((effect) => `${effect.cell}${effect.dir}:${effect.text}`), ['月支down:沖提・月令動']);
  assert.ok(shift.luck.lean > 0);
  assert.match(shift.luck.advice, /原局偏弱.*順/);
  assert.ok(shift.summary.at(-1).includes('加上大運戊午'));
});

test('強弱變化圖＋大運：癸丑運填實亥子丑三會水，並合日主', () => {
  const shift = strengthShift(charts.t3.analysis, { ganZhi: '癸丑' });
  const steps = luckSteps(shift);
  const meeting = steps.find((step) => step.tag === '三會（填實）');
  assert.deepEqual([meeting.counted, meeting.deltaOpp, meeting.bonus.party, meeting.bonus.label], [true, 1, 'opp', '水局']);
  const day = steps.find((step) => step.tag === '合日主');
  assert.equal(day.effects[0].neutral, true, '日主不因大運而化');
  // 人數：大運癸（異黨）、丑（同黨）各一，三會水再給異黨加一 → 偏向異黨
  assert.ok(shift.luck.lean < 0);
  assert.match(shift.luck.advice, /原局偏弱.*逆/);
  assert.equal(shift.luck.bringsRoot, true);
});

test('強弱變化圖＋大運：命運相沖，原局無水的子運沖不動旺火', () => {
  const shift = strengthShift(charts.stir.analysis, { ganZhi: '壬子' });
  const clashes = luckSteps(shift).filter((step) => step.tag.startsWith('六沖'));
  assert.equal(clashes.length, 3);
  for (const step of clashes) {
    assert.equal(step.tag, '六沖（反激）');
    assert.deepEqual(step.effects.map((effect) => `${effect.dir}:${effect.text}`), ['up:激旺', 'down:沖不動']);
  }
  assert.deepEqual(shift.luck.rootShaken, [], '沖不動，根沒有被動到');
  assert.equal(shift.luck.advice, null, '旺極候選：扶抑方向不一定適用');
});

test('強弱變化圖＋大運：原局已有合的天干，大運再來合，先標待確認', () => {
  // x6 的時干壬已經和日主丁合來；丁卯運的丁再來合壬
  const shift = strengthShift(charts.x6.analysis, { ganZhi: '丁卯' });
  const steps = luckSteps(shift);
  const combine = steps.find((step) => step.id === 'luck-combine');
  assert.equal(combine.tag, '五合（待確認）');
  assert.equal(combine.counted, false);
  // 卯補上亥卯未：三合木局給同黨（印）加一
  const three = steps.find((step) => step.tag === '三合（填實）');
  assert.deepEqual([three.deltaSame, three.deltaOpp], [1, 0]);
});

test('強弱變化圖＋大運：大運天干五合，化不化看大運坐支', () => {
  // b3 的月干壬沒有別的合：丁卯運坐卯（木的旺地）→ 合化木；丁巳運不是木的旺地 → 合絆
  const transformed = strengthShift(charts.b3.analysis, { ganZhi: '丁卯' });
  const first = luckSteps(transformed).find((step) => step.id === 'luck-combine');
  assert.equal(first.tag, '合而能化');
  assert.deepEqual(first.effects.map((effect) => `${effect.cell}${effect.text}`), ['月干化木', '運干化木']);
  const bound = strengthShift(charts.b3.analysis, { ganZhi: '丁巳' });
  const second = luckSteps(bound).find((step) => step.id === 'luck-combine');
  assert.equal(second.tag, '合絆');
  assert.deepEqual(second.effects.map((effect) => `${effect.cell}${effect.dir}`), ['月干bind', '運干bind']);
});

test('強弱變化圖＋大運：四庫沖看土旺不旺；伏吟加重', () => {
  const storage = strengthShift(charts.b3.analysis, { ganZhi: '戊戌' });
  const clash = luckSteps(storage).find((step) => step.tag === '四庫沖');
  assert.deepEqual(clash.effects.map((effect) => `${effect.cell}${effect.text}`), ['運支愈沖愈旺', '月支愈沖愈旺']);
  const echo = strengthShift(charts.x6.analysis, { ganZhi: '庚寅' });
  const same = luckSteps(echo).find((step) => step.tag === '伏吟');
  assert.deepEqual(same.effects.map((effect) => `${effect.cell}${effect.dir}:${effect.text}`), ['時支up:同氣加重']);
});
