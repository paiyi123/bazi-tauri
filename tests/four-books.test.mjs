import test from "node:test";
import assert from "node:assert/strict";
import { createServer } from "vite";

const server = await createServer({ server: { middlewareMode: true }, appType: "custom", logLevel: "silent" });
const m = await server.ssrLoadModule("/src/utils/fourBooks.ts");
test.after(() => server.close());

const c = (chart) => chart.split(" ");
const status = (checks, label) => checks.find((check) => check.label === label).status;

// 命例出自 docs/books/ 各書 命例.tsv（依年 月 日 時）。
test("心訣第 15 訣：得令、得地、得勢三選二", () => {
  const strong = (chart) => m.threeOfTwo(c(chart), chart.split(" ")[2][0]).strong;
  assert.equal(strong("戊辰 戊午 甲申 癸酉"), false, "李嘉誠：甲木衰弱");
  assert.equal(strong("己酉 癸酉 庚子 丙子"), true, "孔子：庚金身旺");
  assert.equal(strong("癸卯 壬戌 丙戌 丁酉"), false, "梅艷芳：丙火身弱");
  assert.equal(strong("癸丑 丁巳 丁巳 丙午"), true, "尚書命：丁火強旺");
  assert.equal(strong("己酉 丙子 庚辰 甲申"), true, "第 92 訣：庚金身旺");
  assert.equal(strong("癸亥 甲子 丙寅 乙未"), false, "第 99 訣：三冬丙火失時而弱");
});

test("心訣第 14 訣：流通只走同柱、鄰干、鄰支", () => {
  const sun = m.flowReport(c("丙寅 己亥 辛卯 庚寅"));
  assert.ok(sun.chains.some((chain) => chain.includes("月支亥水 → 年支寅木 → 年干丙火 → 月干己土 → 日干辛金")), "孫中山逆流有情");
  assert.equal(sun.check.status, "pass");
  assert.equal(m.flowReport(c("癸酉 甲子 丙寅 戊戌")).check.status, "pass", "五行順生的上佳命造");
  const poor = m.flowReport(c("壬子 戊申 戊戌 辛酉"));
  assert.equal(poor.check.status, "fail", "貧命：身弱順流洩出");
  assert.match(poor.check.detail, /流錯方向/);
  // 年干辛（金）和月支巳不同柱也不同列：心訣不算相生。
  assert.ok(m.flowReport(c("辛丑 癸巳 甲子 丙寅")).blocked.length >= 1);
});

test("心訣第 92 訣：寒雖甚而暖有氣／無氣、暖雖至而寒有根／無根", () => {
  const climate = (chart) => m.climateReport(c(chart));
  assert.equal(climate("甲申 丙子 庚辰 戊寅").check.status, "pass");
  const noWarmth = climate("己酉 丙子 庚辰 甲申");
  assert.equal(noWarmth.check.status, "partial");
  assert.match(noWarmth.check.detail, /反以無暖為美/);
  assert.equal(climate("丁丑 丙午 丙午 壬辰").check.status, "pass");
  const noCold = climate("癸未 丁巳 丙午 癸巳");
  assert.equal(noCold.check.status, "partial");
  assert.match(noCold.check.detail, /反以無寒為美/);
  // 輕鬆學算命：火旺見丑辰濕土
  assert.ok(climate("丁丑 丙午 丙午 壬辰").soilNotes.some((note) => note.includes("丑辰")));
});

test("心訣第 99 訣：清濁三法", () => {
  assert.equal(m.clarityReport(c("癸亥 甲子 丙寅 乙未"), ["木"]).grade, "一清到底");
  const dry = m.clarityReport(c("癸巳 甲申 甲申 壬申"), ["火"]);
  assert.equal(status(dry.checks, "神枯"), "fail", "偏枯：巳火被申合去");
  assert.match(dry.checks.find((check) => check.label === "神枯").detail, /合住/);
  const muddy = m.clarityReport(c("癸丑 己未 丙午 己丑"), ["木"]);
  assert.equal(status(muddy.checks, "神枯"), "fail", "滿盤濁氣：缺木");
  assert.notEqual(muddy.grade, "一清到底");
});

test("妙算用神四品質：有力、有情", () => {
  const q = (chart, use) => m.useQuality(c(chart), use).checks;
  assert.equal(status(q("甲子 丁卯 己亥 戊辰", "火"), "有力"), "pass", "例二十四：丁火透干、得勢、不受剋");
  assert.equal(status(q("甲子 丁卯 己亥 戊辰", "火"), "有情"), "pass");
  assert.equal(status(q("丙辰 乙未 丁卯 癸卯", "水"), "有力"), "fail", "例二十七：癸水失時、根淺");
  assert.equal(status(q("庚辰 壬午 癸丑 乙卯", "金"), "有情"), "pass", "例二十五：庚→壬→癸順生");
  const cold = q("己巳 庚午 丙申 壬辰", "土");
  assert.equal(status(cold, "有情"), "fail", "例二十六：己土被庚金洩去轉生壬水");
  assert.match(cold.find((check) => check.label === "有情").detail, /洩去/);
});

test("妙算戀愛方式與限運", () => {
  const love = m.loveStyle(c("丁卯 丁未 丙午 己亥"));
  assert.deepEqual(love.hits.map((hit) => hit.pillar), ["月干", "時干", "年干"]);
  assert.equal(m.loveStyle(c("甲子 丙寅 甲寅 庚午")).selfExposed, true);
  assert.match(m.loveStyle(c("壬子 壬子 丙子 庚寅")).summary, /都沒有透出/);
  const rows = m.limitRows(c("甲子 丙寅 甲寅 庚午"), (element) => (element === "木" ? "good" : "neutral"));
  assert.equal(rows[2].miaoSuan, "37–55 歲");
  assert.equal(rows[2].stemFavor, "good");
});

test("轉盤／借盤：心訣第 46 訣的例子（太太命盤借看丈夫）", () => {
  const natal = c("辛卯 乙未 己未 戊辰");
  const options = m.relativeOptions("FEMALE");
  const husband = options.find((option) => option.key === "spouse");
  assert.deepEqual(m.findRelative(natal, husband), { pillar: 1, part: "干" }, "夫妻取月干乙木");
  const report = m.rotateChart(natal, { pillar: 1, part: "干" }, [{ ganZhi: "丙申" }, { ganZhi: "戊戌" }]);
  assert.equal(report.selfRole, "偏財", "太太（己土）是乙木的財");
  assert.equal(report.cells[2].stemGod, "偏財");
  assert.equal(report.luck[0].forThem, "傷官／正官");
  assert.equal(report.luck[1].forThem, "正財／正財");
  // 天干優於地支：妻星偏財壬透時干，勝過時支子（正財本氣）
  const wife = m.relativeOptions("MALE").find((option) => option.key === "spouse");
  assert.deepEqual(m.findRelative(c("甲寅 丙午 戊子 壬子"), wife), { pillar: 3, part: "干" });
  // 男命子女：官殺在地支也先於透出的食傷
  const child = m.relativeOptions("MALE").find((option) => option.key === "child");
  assert.deepEqual(m.findRelative(c("丙寅 庚寅 戊午 庚申"), child), { pillar: 1, part: "支" }, "時支申不是，月支寅七殺");
  // 原盤沒有就不能借盤：全盤沒有官殺時找不到丈夫。
  assert.equal(m.findRelative(c("甲寅 丙寅 甲午 丙寅"), husband), null);
});

test("五合三種化法、暗五合", () => {
  const notes = m.combineNotes(c("甲子 己巳 丁亥 壬寅"));
  assert.ok(notes.some((note) => note.includes("正化土") && note.includes("妻從夫化木")));
  assert.ok(notes.some((note) => note.includes("暗五合（丁合亥中壬）")));
});

test("大運流年標籤：蓋頭截腳、喜忌組合、合解沖、新見五行、四正全", () => {
  const favorWood = (element) => (element === "木" ? "good" : element === "金" ? "bad" : "neutral");
  const natal = c("癸卯 甲子 丙午 戊戌");
  const luck = (incoming, extra = {}) => m.fourBookLuckEvents(incoming, "運", { natal, favorOf: favorWood, ...extra }).map((e) => e.label);
  assert.ok(luck("庚寅").includes("蓋頭"), "喜木走庚寅");
  assert.ok(luck("甲申").includes("截腳"), "喜木走甲申");
  assert.ok(!luck("甲寅").includes("蓋頭"));
  // 子午沖，丑運合住子
  assert.ok(luck("乙丑").includes("合解沖"));
  // 原局（含藏干）沒有金水，庚申運新見金水
  const fresh = m.fourBookLuckEvents("庚申", "運", { natal: c("甲寅 丙寅 甲午 丙寅") }).map((e) => e.label);
  assert.ok(fresh.includes("新見金水"));
  assert.ok(!luck("庚申").some((label) => label.startsWith("新見")), "戌藏辛金，不算新見");
  // 卯子午已有，流年酉湊齊四正
  const annual = m.fourBookLuckEvents("己酉", "歲", { natal, favorOf: favorWood, luck: "甲寅" }).map((e) => e.label);
  assert.ok(annual.includes("四正全"));
  // 大運一喜一忌、流年全忌 → 凶禍立至（妙算 p322）
  const combo = m.fourBookLuckEvents("辛酉", "歲", { natal, favorOf: favorWood, luck: "甲申" }).map((e) => e.label);
  assert.ok(combo.includes("凶禍立至"));
  // 沒有喜忌時不出喜忌類標籤
  assert.ok(!m.fourBookLuckEvents("庚寅", "運", { natal }).some((e) => e.label === "蓋頭"));
});
