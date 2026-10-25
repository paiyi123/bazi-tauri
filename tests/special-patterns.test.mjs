import test from "node:test";
import assert from "node:assert/strict";
import { createServer } from "vite";

const server = await createServer({ server: { middlewareMode: true }, appType: "custom", logLevel: "silent" });
const { specialPatternReport, specialPatternLuckEvents } = await server.ssrLoadModule("/src/utils/specialPatterns.ts");
test.after(() => server.close());

const report = (chart, options) => specialPatternReport(chart.split(" "), options);
const followOf = (chart) =>
  report(chart)
    .follow.filter((item) => item.verdict !== "none")
    .map((item) => `${item.name}:${item.verdict}`);

// 書上命例（蘇民峰《八字論命》第十四章，頁碼見 docs/books/蘇民峰-八字論命/06）
test("從格候選：從財、從殺、從兒、從強、從勢", () => {
  assert.deepEqual(followOf("乙卯 己卯 庚午 丁亥"), ["從財格:formed"]);
  assert.deepEqual(followOf("戊辰 甲寅 壬戌 丙午"), ["從財格:partial"]);
  assert.deepEqual(followOf("庚戌 乙酉 乙酉 乙酉"), ["從殺格:formed"]);
  assert.deepEqual(followOf("壬寅 丁未 己卯 乙亥"), ["從殺格:formed"]);
  assert.deepEqual(followOf("戊子 辛酉 戊申 壬子"), ["從兒格:formed"]);
  assert.deepEqual(followOf("庚辰 乙酉 癸卯 庚申"), ["從強格:formed"]);
  assert.deepEqual(followOf("辛巳 甲午 甲午 己巳"), ["從勢格:formed"]);
  // 財與官殺一樣多時不硬判從財，改看從勢（p245）
  assert.deepEqual(followOf("壬申 己酉 丙子 戊子"), ["從勢格:formed"]);
  // 一般命不出現候選
  assert.deepEqual(followOf("丁亥 丙午 壬寅 己酉"), []);
  assert.deepEqual(followOf("丁卯 癸卯 丙子 己亥"), []);
});

test("從旺五格", () => {
  const prosperous = (chart) => report(chart).prosperous;
  assert.equal(prosperous("癸未 甲寅 乙亥 己卯").verdict, "formed");
  assert.equal(prosperous("戊戌 甲寅 丙午 甲午").verdict, "formed");
  assert.equal(prosperous("戊辰 己未 己未 庚午").verdict, "formed");
  assert.equal(prosperous("壬辰 辛亥 癸丑 壬子").verdict, "formed", "水庫辰不算官殺");
  // 寒土不成稼穡（p250）
  assert.equal(prosperous("癸未 壬戌 己丑 庚午").verdict, "none");
  // 白虎格：丁火被戊土隔開引化，不破格，算次一等（p254）
  const baiHu = prosperous("丁酉 戊申 辛丑 己丑");
  assert.equal(baiHu.verdict, "partial");
  assert.equal(baiHu.checks.find((check) => check.label === "不見官殺").status, "partial");
});

test("化氣格六條件", () => {
  assert.equal(report("戊辰 壬戌 甲辰 己巳").transform.verdict, "formed");
  assert.equal(report("乙卯 丁亥 壬寅 甲辰").transform.verdict, "formed");
  assert.equal(report("壬辰 辛亥 辛亥 丙申").transform.verdict, "formed", "清太宗：得辰字不算剋水");
  assert.equal(report("壬子 癸丑 辛亥 丙申").transform.verdict, "formed", "亥子丑會水，月支已成化神");
  assert.equal(report("癸丑 丙辰 辛亥 戊子").transform.verdict, "partial");
  assert.equal(report("辛巳 甲午 甲午 己巳").transform.verdict, "none");
  assert.equal(report("丁亥 丙午 壬寅 己酉").transform, null);
});

test("魁罡、金神、兩神成象", () => {
  const mark = (chart, key) => report(chart).marks.find((item) => item.key === key);
  assert.equal(mark("己未 庚午 庚戌 壬午", "kui-gang").verdict, "formed");
  assert.equal(mark("辛巳 庚寅 庚戌 丙戌", "kui-gang").verdict, "none", "庚日見丙火破格");
  assert.equal(mark("戊戌 辛酉 戊戌 丙辰", "kui-gang").verdict, "none", "時支辰沖日支戌");
  assert.match(report("戊戌 辛酉 戊戌 辛酉", { female: true }).marks.find((m) => m.key === "kui-gang").advice, /女命/);
  assert.equal(mark("庚子 辛巳 癸酉 辛酉", "jin-shen").verdict, "formed");
  assert.equal(mark("癸卯 辛酉 乙丑 癸未", "jin-shen").verdict, "partial");
  assert.ok(report("戊戌 辛酉 戊戌 辛酉").marks.some((item) => item.name.startsWith("兩神成象")));
});

test("大運流年：化神還原、沖魁罡", () => {
  const labels = (incoming, chart) => specialPatternLuckEvents(incoming, "運", chart.split(" ")).map((e) => `${e.label}:${e.level}`);
  assert.ok(labels("庚辰", "戊辰 壬戌 甲辰 己巳").some((label) => label.startsWith("化神還原") && label.endsWith("warn")));
  assert.ok(labels("乙卯", "戊辰 壬戌 甲辰 己巳").some((label) => label.startsWith("化神受剋")));
  assert.ok(labels("丙辰", "己未 庚午 庚戌 壬午").some((label) => label.startsWith("沖魁罡")));
  assert.deepEqual(labels("丙辰", "辛巳 庚寅 庚戌 丙戌"), [], "魁罡已破就不再提示");
  assert.deepEqual(labels("庚辰", "丁亥 丙午 壬寅 己酉"), []);
});
