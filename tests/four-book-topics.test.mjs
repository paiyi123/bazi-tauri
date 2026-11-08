import test from "node:test";
import assert from "node:assert/strict";
import { createServer } from "vite";

const server = await createServer({ server: { middlewareMode: true }, appType: "custom", logLevel: "silent" });
const m = await server.ssrLoadModule("/src/utils/fourBookTopics.ts");
test.after(() => server.close());

const c = (chart) => chart.split(" ");
const labels = (natal, gender, key, extra = {}) =>
  m.fourBookTopicSections({ natal: c(natal), gender, ...extra }).find((section) => section.key === key).items.map((item) => item.label);

// 命例出自 docs/books/八字心訣/命例.tsv（依年 月 日 時）；沒有命例的規則用書上的片段條件自組命盤。
test("心訣第 74、75 訣：健康案例", () => {
  assert.ok(labels("癸卯 癸亥 壬戌 庚戌", "MALE", "health").includes("火只藏不現"), "05-34 先天性心臟病：火只藏不現");
  const infertile = labels("丁亥 癸丑 己未 癸酉", "FEMALE", "health");
  assert.ok(infertile.includes("土水相戰"), "05-49 不孕：土水相戰（腎膀胱）");
  assert.ok(infertile.includes("土多無木"), "05-49：土多無木");
  assert.ok(labels("辛丑 丙申 丁丑 乙巳", "FEMALE", "health").includes("火金相戰"), "05-32 肝膽病：火金相戰");
  assert.ok(labels("辛丑 戊戌 戊子 壬戌", "FEMALE", "health").includes("土多無木"), "05-51 輕生：土多無木");
});

test("六親一覽：找星、位置、距離", () => {
  const rows = m.relativeRows({ natal: c("辛亥 庚寅 丁卯 戊申"), gender: "FEMALE" });
  const husband = rows.find((row) => row.relative === "丈夫");
  assert.equal(husband.star, "正官（水）", "04-01 坤造：丁火的夫星是壬水正官");
  assert.equal(husband.distance, "遠離", "壬水只在年支亥、時支申中");
  const father = rows.find((row) => row.relative === "父親");
  assert.match(father.where, /年干辛/, "父親先找偏財");
  const male = m.relativeRows({ natal: c("丙申 辛卯 己亥 戊辰"), gender: "MALE" });
  assert.equal(male.find((row) => row.relative === "妻子").distance, "緊貼", "04-07：日支亥藏壬正財，緊貼日主");
});

test("姻緣婚嫁", () => {
  assert.ok(labels("丙申 辛卯 己亥 戊辰", "MALE", "marriage").includes("妻填妻位"), "04-07：日支坐正財");
  assert.ok(labels("甲午 乙亥 庚辰 己卯", "MALE", "marriage").includes("正財合日主"), "宋子文：月干乙正財合庚");
  const both = labels("甲辰 甲戌 癸丑 丙辰", "FEMALE", "marriage");
  assert.ok(both.includes("官殺並見"), "04-05 坤造：戊己官殺都透出");
  // 日支被兩字爭合：日支卯，年支戌、時支戌都合卯。
  assert.ok(labels("甲戌 丙寅 丁卯 庚戌", "FEMALE", "marriage").includes("日支被兩字合"));
});

test("事業、財富、學業", () => {
  assert.ok(labels("丁卯 辛亥 甲子 丙寅", "MALE", "career").includes("傷官見官"), "丁傷官、辛正官相鄰透干");
  const horses = labels("甲申 甲戌 甲寅 甲戌", "MALE", "career");
  assert.ok(horses.includes("驛馬在年支申") && horses.includes("驛馬在日支寅"), "05-23：申寅互為驛馬");
  assert.ok(labels("甲子 丙寅 丙戌 甲午", "MALE", "wealth").includes("財藏於庫"), "丙日金財只藏戌庫");
  assert.ok(labels("庚子 乙酉 辛酉 壬辰", "FEMALE", "study").includes("金水洩秀"), "05-43：辛日見壬透");
  assert.ok(labels("己卯 庚午 甲子 丙寅", "MALE", "study").includes("命中無印") === false, "子藏癸印");
  assert.ok(labels("甲寅 丙寅 丙午 甲午", "MALE", "study").includes("印透干"));
});

test("課堂強弱優先於三選二", () => {
  assert.ok(labels("丙申 辛卯 己亥 戊辰", "MALE", "marriage", { strong: true }).includes("日支坐正財（身強）"));
  assert.ok(labels("丙申 辛卯 己亥 戊辰", "MALE", "marriage", { strong: false }).includes("日支坐正財（身弱）"));
});

test("六神與行運表依課堂喜忌選說法", () => {
  const favorOf = (element) => (element === "金" ? "good" : element === "木" ? "bad" : "neutral");
  const rows = m.luckGodRows({ natal: c("甲辰 甲戌 甲寅 甲戌"), gender: "MALE", favorOf });
  assert.equal(rows.length, 5);
  const officer = rows.find((row) => row.family === "官殺");
  assert.equal(officer.element, "金");
  assert.match(officer.meaning, /升遷/);
  assert.match(rows.find((row) => row.family === "比劫").meaning, /連累/);
});

test("運歲標籤：轉角運、運剋歲、天地合日柱、雙沖用神、驛馬動", () => {
  const natal = c("甲寅 丁卯 丙午 戊子");
  const kinds = (incoming, who, extra) => m.fourBookTopicEvents(incoming, who, { natal, gender: "MALE", ...extra }).map((event) => event.label);
  assert.ok(kinds("乙巳", "運", { previousLuck: "甲辰" }).includes("轉角運"), "辰（東）→巳（南）");
  assert.ok(!kinds("丙午", "運", { previousLuck: "乙巳" }).includes("轉角運"), "同在南方");
  assert.ok(kinds("戊申", "歲", { luck: "甲辰" }).includes("運剋歲"), "甲木剋戊土");
  assert.ok(!kinds("庚申", "歲", { luck: "甲辰" }).includes("運剋歲"), "歲剋運（太歲剋大運）不標");
  assert.ok(kinds("辛未", "歲", {}).includes("天地合日柱"), "丙辛合、午未合");
  const double = m.fourBookTopicEvents("辛酉", "歲", { natal, gender: "MALE", useElements: ["木"], luck: "癸酉" });
  assert.equal(double.find((event) => event.label === "雙沖用神")?.level, "warn", "大運、流年酉同沖月支卯（木用神）");
  const horse = m.fourBookTopicEvents("庚申", "歲", { natal: c("甲申 甲戌 甲寅 甲戌"), gender: "MALE", favorOf: () => "good" });
  assert.match(horse.find((event) => event.label === "驛馬動")?.detail ?? "", /吉馬逢沖/);
});
