import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { createSSRApp, h } from 'vue';
import { renderToString } from 'vue/server-renderer';
import ElementPlus from 'element-plus';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';

const fixture = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-general.json'), 'utf8'));
const sourceFixture = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-source.json'), 'utf8'));
const lesson0902 = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-0902-content.json'), 'utf8'));
const singleFixture = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-single.json'), 'utf8'));
const insufficientFixture = JSON.parse(readFileSync(resolve('tests/fixtures/lesson-insufficient.json'), 'utf8'));

const cacheParent = resolve(tmpdir());
const cachePrefix = 'bazi-lesson-ui-';
const cacheDir = resolve(mkdtempSync(join(cacheParent, cachePrefix)));
if (dirname(cacheDir) !== cacheParent || !basename(cacheDir).startsWith(cachePrefix)) {
  throw new Error(`Refusing to use unowned Vite cache directory: ${cacheDir}`);
}

const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  if (dirname(cacheDir) !== cacheParent || !basename(cacheDir).startsWith(cachePrefix)) {
    throw new Error(`Refusing to remove unowned Vite cache directory: ${cacheDir}`);
  }
  rmSync(cacheDir, { recursive: true, force: true });
});
async function render(component, props = {}) {
  const mod = await server.ssrLoadModule(component);
  const app = createSSRApp({ render: () => h(mod.default, props) });
  app.use(ElementPlus);
  return await renderToString(app);
}

function planCard(html, title) {
  const cards = html.match(/<article class="plan-card"[\s\S]*?<\/article>/g) || [];
  const card = cards.find(item => item.includes(title));
  assert.ok(card, `missing plan card: ${title}`);
  return card;
}

function directionGroup(card, label) {
  const groups = card.match(/<div class="direction-group"[\s\S]*?<\/div>/g) || [];
  const group = groups.find(item => item.includes(label));
  assert.ok(group, `missing direction group: ${label}`);
  return group;
}

test('renders graceful absent-analysis state', async () => {
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: null });
  assert.match(html, /此結果尚無三步分析/);
});

test('renders the four 0902 source cards with evidence layers beside the engineering result', async () => {
  const html = await render('/src/components/CourseContentPanel.vue', { result: { quantModel: { lessonContent: lesson0902 } } });
  for (const text of ['本命：正印格與取用對照', '戊寅：喜木為何未直接判好', '己卯：沖忌印的課堂解釋', '庚辰：保留結論與證據缺口', '事實：', '本課派說法：', '補充推論：', 'D:/Data/20260902.txt', '字元', '作用鏈未交代', '強候選', '身強', '用神候選：火']) {
    assert.ok(html.includes(text), `missing 0902 content: ${text}`);
  }
  assert.equal((html.match(/2026\/09\/02 男命原例｜/g) || []).length, 4);
});

test('hides all 0902 source cards for a nonmatching chart', async () => {
  // Synthetic display-contract variant; backend rejection of each pillar is tested in Rust.
  const nonmatch = structuredClone(lesson0902);
  for (const item of nonmatch.inferences) {
    if (item.ruleId.startsWith('20260902-male-')) item.matchStatus = 'not_matched';
  }
  const html = await render('/src/components/CourseContentPanel.vue', { result: { quantModel: { lessonContent: nonmatch } } });
  assert.ok(!html.includes('2026/09/02 男命原例｜'));
  assert.ok(html.includes('第一步：格局判定'));
});

test('renders three steps, evidence, separate direction roles and source details', async () => {
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: fixture });
  for (const text of ['第一步：格局判定', '第二步：日主強弱', '第三步：用神與喜忌', '偏印格', '強候選', '互動後有效力量未定', '財', '食傷', '比劫', '官殺', '2026-09-02', 'strong-hidden-wealth-visible-kill']) assert.match(html, new RegExp(text));
  assert.match(html, /老師的精確來源命例結論不同/);
  assert.match(html, /選定候選|候選/);
});

test('renders multiple plans and escapes untrusted lesson text', async () => {
  const second = structuredClone(fixture);
  second.facts.conclusion = '<script>alert("x")</script>';
  second.preferences.plans = [...second.preferences.plans, { ...second.preferences.plans[0], ruleId: 'plan-2', title: '另一方案', condition: '依據不足，待核對' }];
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: second });
  assert.match(html, /另一方案/);
  assert.match(html, /&lt;script&gt;alert/);
  assert.doesNotMatch(html, /<script>alert/);
});

test('renders undefined and insufficient analysis without inventing plans', async () => {
  const absent = await render('/src/components/LessonThreeStepPanel.vue', {});
  assert.match(absent, /此結果尚無三步分析/);
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: insufficientFixture });
  assert.match(html, /依據不足/);
  assert.doesNotMatch(html, /未知狀態（依據不足）/);
  assert.match(html, /目前沒有足夠依據形成方案/);
  assert.equal((html.match(/class="plan-card/g) || []).length, 0);
});

test('renders one-plan fallback separately from four-plan general analysis', async () => {
  const general = await render('/src/components/LessonThreeStepPanel.vue', { analysis: fixture });
  const single = await render('/src/components/LessonThreeStepPanel.vue', { analysis: singleFixture });
  assert.equal((general.match(/class="plan-card/g) || []).length, 4);
  assert.equal((single.match(/class="plan-card/g) || []).length, 1);
  assert.match(single, /有條件/);

  const officerPlan = planCard(general, '財藏殺顯：官殺候選');
  assert.match(directionGroup(officerPlan, '忌神方向'), /印（土）/);
  assert.match(directionGroup(officerPlan, '有條件方向'), /食傷（水）/);
  assert.doesNotMatch(directionGroup(officerPlan, '取用方向'), /食傷（水）/);

  const outputPlan = planCard(general, '強旺比劫：食傷洩比');
  assert.match(directionGroup(outputPlan, '取用方向'), /食傷（水）/);
  assert.equal((outputPlan.match(/有條件方向/g) || []).length, 0);
});

test('preserves selected candidate, both source versions, and null seasonal ownership', async () => {
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: sourceFixture });
  assert.match(html, /選定候選/);
  assert.match(html, /2026-07-08/);
  assert.match(html, /2026-09-02/);
  assert.match(html, /方案 未指定/);
});

test('renders known production codes as exact Chinese labels without leaking raw codes', async () => {
  const rendered = await Promise.all([
    render('/src/components/LessonThreeStepPanel.vue', { analysis: fixture }),
    render('/src/components/LessonThreeStepPanel.vue', { analysis: sourceFixture }),
    render('/src/components/LessonThreeStepPanel.vue', { analysis: singleFixture }),
    render('/src/components/LessonThreeStepPanel.vue', { analysis: insufficientFixture }),
  ]);

  assert.match(rendered[0], /強弱：偏強／強候選（一致）/);
  assert.match(rendered[0], /喜忌：木、水／木、水、火（部分一致）/);
  assert.match(rendered[0], /調候本版範圍外/);
  assert.match(rendered[3], /調候冬季/);
  for (const html of rendered) {
    assert.doesNotMatch(html, /outside_bounded_scope|>winter（|未知狀態（依據不足）/);
  }
});

test('labels supported bridge and summer codes', async () => {
  // Synthetic display-contract variants; original Rust fixture remains unchanged.
  const analysis = structuredClone(fixture);
  analysis.pattern.ordinaryCandidates[0].selection = 'fallback_candidate';
  analysis.preferences.bridgePaths[0].status = 'missing_intermediary';
  analysis.preferences.seasonalCompatibility[0].season = 'summer';
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis });

  assert.match(html, /缺少通關中介/);
  assert.match(html, /調候夏季/);
  assert.match(html, /備選候選/);
  assert.doesNotMatch(html, /fallback_candidate|missing_intermediary|>summer（/);
});

test('keeps unsupported status and season codes explicit', async () => {
  const analysis = structuredClone(fixture);
  analysis.preferences.bridgePaths[0].status = 'future_backend_code';
  analysis.preferences.seasonalCompatibility[0].season = 'future_season_code';
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis });

  assert.match(html, /未知狀態（future_backend_code）/);
  assert.match(html, /未知季節（future_season_code）/);
});

test('keeps legacy course annotations reachable beside absent new analysis', async () => {
  const legacy = { title: '上課內容', status: 'legacy', analysisOrder: [], futureRuleNotice: '舊資料', inferences: [{ ruleId: 'legacy-1', title: '舊註解', status: '候選', sourceVerification: 'legacy', algorithmStatus: 'candidate', matchStatus: 'matched', outputMode: 'annotation_only', fixtureScope: 'none', agreementStatus: 'not_comparable', fallbackPolicy: 'preserve_normal_outputs', appReference: { geJu: '舊格', strengthLabel: '偏強', specialPattern: '無' }, sensitiveClaimsPolicy: 'excluded', conclusion: '保留', evidence: [], rejectedAlternatives: [] }] };
  const html = await render('/src/components/CourseContentPanel.vue', { result: { quantModel: { lessonContent: legacy } } });
  assert.match(html, /此結果尚無三步分析/);
  assert.match(html, /課堂命例補充/);
  assert.match(html, /舊註解/);
});

test('strength comparison card spells out 得令、得勢、得地 for the classroom method', async () => {
  const html = await render('/src/components/StrengthComparisonPanel.vue', {
    result: { quantModel: { strengthLabel: '偏強', totalScore: 1, lessonContent: { threeStepAnalysis: sourceFixture } } },
  });
  for (const text of ['課堂三步法強弱分析', '得令✗ 得勢✓ 得地✗', '月支酉（正印）', '年干甲（食神）', '同黨（印、比劫）4 個', '得令者為旺', '強候選']) {
    assert.ok(html.includes(text), `missing ${text}`);
  }
});

test('2026-10-07／10-14：天干生剋合化一節列出五合變化與生剋', async () => {
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: fixture });
  assert.match(html, /天干生剋合化/);
  assert.match(html, /戊癸合火：合而不合/);
  assert.match(html, /中間隔著月干丙、日干庚/);
  assert.match(html, /天干生剋（遠近、通關、先受剋）/);
  // Older payloads without the section still render.
  const { stemInteractions, ...legacy } = fixture;
  assert.doesNotMatch(await render('/src/components/LessonThreeStepPanel.vue', { analysis: legacy }), /天干生剋合化/);
});

test('2026-10-21：地支沖合一節列出相沖勝負與六合生剋', async () => {
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: sourceFixture });
  assert.match(html, /地支沖合/);
  assert.match(html, /酉卯沖：勝負分明/);
  assert.match(html, /去喜存忌，凶/);
  const { branchInteractions, ...legacy } = sourceFixture;
  assert.doesNotMatch(await render('/src/components/LessonThreeStepPanel.vue', { analysis: legacy }), /地支沖合/);
});

test('講義第24–26章：三合三會、刑害、同柱干支與作用變化後強弱複核', async () => {
  const html = await render('/src/components/LessonThreeStepPanel.vue', { analysis: insufficientFixture });
  assert.match(html, /地支沖合會刑害/);
  assert.match(html, /丑戌相刑（丑戌未三刑缺未）：同屬土，加強土力/);
  assert.match(html, /丑午相害：火生土/);
  assert.match(html, /同柱干支/);
  assert.match(html, /日柱甲午：天干生地支(<!--\[-->)?・暗合/);
  assert.doesNotMatch(html, /地支沒有沖、合、會、刑、害/);
  assert.match(html, /作用變化後強弱複核/);
  const general = await render('/src/components/LessonThreeStepPanel.vue', { analysis: fixture });
  assert.match(general, /申辰缺中神/);
  // A synthetic reversal (cloned data, not a Rust output) is highlighted.
  const changed = structuredClone(fixture);
  changed.interactionStrength.changed = true;
  changed.interactionStrength.conclusion = '測試：可能由強轉弱';
  assert.match(await render('/src/components/LessonThreeStepPanel.vue', { analysis: changed }), /review-changed/);
  // Older payloads without the new sections still render.
  const { pillarRelations, interactionStrength, ...legacy } = fixture;
  const legacyHtml = await render('/src/components/LessonThreeStepPanel.vue', { analysis: { ...legacy, branchInteractions: { ...legacy.branchInteractions, gatherings: undefined, punishments: undefined } } });
  assert.doesNotMatch(legacyHtml, /lesson-step pillar-relations|lesson-step interaction-strength|combo-list gathering-list/);
});
