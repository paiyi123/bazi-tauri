<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>{{ title ?? "四本書檢核" }}</h2>
      </div>
    </template>
    <p v-if="only" class="intro">
      依《子平正源》《八字心訣》《妙算人生八字》《輕鬆學算命》。只當提示：不改分數，喜忌和用神沿用課堂三步法。✓ 清／符合、△ 有瑕疵、✗ 濁／不符。
    </p>
    <p v-else class="intro">
      依《子平正源》《八字心訣》《妙算人生八字》《輕鬆學算命》整理的檢查。只當提示：不改五行力分數，喜忌和用神都沿用課堂三步法。
      ✓ 清／符合、△ 有瑕疵、✗ 濁／不符。大運流年格子會另外標出蓋頭、截腳、喜忌組合、合解沖、新見五行、四正四庫全。
    </p>

    <details v-if="show('rotation')" open class="block">
      <summary>轉盤／借盤：從親人的角度看這張八字</summary>
      <p class="hint">
        以代表那位親人的字為新日主，重排十神，看「他」的旺衰和大運（子平正源第二章；心訣第 46 訣：天干優先，父母取年干、夫妻取月干、子女取時干）。也可以直接點下面任一個字。
      </p>
      <div class="relative-row">
        <el-radio-group v-model="relativeKey" size="small">
          <el-radio-button v-for="option in options" :key="option.key" :value="option.key">{{ option.label }}</el-radio-button>
        </el-radio-group>
      </div>
      <p v-if="relativeMissing" class="warn-line">原盤沒有代表{{ currentOption?.label }}的十神（{{ currentOption?.gods.join("、") }}），書說不能勉強借盤。</p>
      <p v-else-if="currentOption && relativeKey" class="hint">{{ currentOption.note }}</p>

      <table class="chart">
        <thead>
          <tr>
            <th></th>
            <th v-for="cell in displayCells" :key="cell.pillar">{{ cell.pillar }}</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <th>十神</th>
            <td v-for="cell in displayCells" :key="cell.pillar" class="god">{{ rotation ? cell.stemGod : "" }}</td>
          </tr>
          <tr>
            <th>天干</th>
            <td v-for="cell in displayCells" :key="cell.pillar">
              <button
                type="button"
                class="char"
                :class="[elementClass(cell.stem), { target: cell.isTarget === '干', self: rotation && cell.isSelf }]"
                @click="pick(cell.index, '干')"
              >
                {{ cell.stem }}
              </button>
            </td>
          </tr>
          <tr>
            <th>地支</th>
            <td v-for="cell in displayCells" :key="cell.pillar">
              <button
                type="button"
                class="char"
                :class="[elementClass(cell.branch), { target: cell.isTarget === '支' }]"
                @click="pick(cell.index, '支')"
              >
                {{ cell.branch }}
              </button>
            </td>
          </tr>
          <tr v-if="rotation">
            <th>藏干十神</th>
            <td v-for="cell in displayCells" :key="cell.pillar" class="hidden-gods">
              <span v-for="item in cell.hidden" :key="item.stem">{{ item.stem }}{{ shortGod(item.god) }}</span>
            </td>
          </tr>
        </tbody>
      </table>
      <template v-if="rotation">
        <p class="line">
          新日主：<b>{{ rotation.masterName }}</b>；命主（日干{{ dayStem }}）是他的<b>{{ rotation.selfRole }}</b>。
        </p>
        <p class="line">旺衰（心訣三選二）：{{ rotation.strength.detail }}</p>
        <p v-if="rotation.threats.length" class="line">在原局：{{ rotation.threats.join("；") }}。</p>
        <details v-if="rotation.luck.length" class="sub">
          <summary>大運對本人和對他（同一步運，十神不同）</summary>
          <table class="luck">
            <thead>
              <tr><th>大運</th><th>年份</th><th>對本人</th><th>對他</th><th>提示</th></tr>
            </thead>
            <tbody>
              <tr v-for="row in rotation.luck" :key="row.ganZhi + row.years" :class="`effect-${row.effect}`">
                <td>{{ row.ganZhi }}</td>
                <td>{{ row.years }}</td>
                <td>{{ row.forSelf }}</td>
                <td>{{ row.forThem }}</td>
                <td>{{ row.effectText }}</td>
              </tr>
            </tbody>
          </table>
        </details>
        <p class="hint">親人受剋＝那個字遇到它自己的官殺（比劫剋妻、食傷剋子、梟印傷孫）；那個字要強，才任得起財官（子平正源）。借盤後不論宮位和歲限（心訣）。</p>
      </template>
    </details>

    <details v-if="show('clarity')" open class="block">
      <summary>清濁三法（心訣第 99 訣）：{{ clarity.grade }}</summary>
      <ul class="check-list">
        <li v-for="check in clarity.checks" :key="check.label" :class="`check-${check.status}`">
          <span class="check-icon">{{ ICON[check.status] }}</span>
          <span class="check-label">{{ check.label }}</span>
          <span class="check-detail">{{ check.detail }}</span>
        </li>
      </ul>
      <p class="hint">{{ clarity.detail }} 日主旺衰在這裡用心訣三選二：{{ flow.strength.detail }}。</p>
    </details>

    <details v-if="show('flow')" :open="Boolean(only)" class="block">
      <summary>流通路線（心訣第 14、80 訣：只有同柱干支、鄰干、鄰支相生）</summary>
      <ul v-if="flow.chains.length" class="plain">
        <li v-for="chain in flow.chains" :key="chain">{{ chain }}</li>
      </ul>
      <p v-else class="line">沒有能相生的鄰位。</p>
      <p class="line">流進日主：{{ flow.inflow ?? "無" }}；從日主流出：{{ flow.outflow ?? "無" }}。</p>
      <p v-if="flow.blocked.length" class="hint">
        異柱干支（斜對角）心訣不算相生：{{ flow.blocked.join("、") }}。其他書和課堂的「生剋遠近」會把它們算成隔位、力量較弱。
      </p>
    </details>

    <details v-if="show('quality')" :open="Boolean(only)" class="block">
      <summary>用神四品質（妙算人生八字第 243–248 頁）</summary>
      <template v-if="qualities.length">
        <div v-for="quality in qualities" :key="quality.use" class="quality">
          <p class="line"><b>用神{{ quality.use }}</b>：{{ quality.summary }}</p>
          <ul class="check-list">
            <li v-for="check in quality.checks" :key="check.label" :class="`check-${check.status}`">
              <span class="check-icon">{{ ICON[check.status] }}</span>
              <span class="check-label">{{ check.label }}</span>
              <span class="check-detail">{{ check.detail }}</span>
            </li>
          </ul>
        </div>
      </template>
      <p v-else class="line">課堂三步法還沒有定出用神。</p>
    </details>

    <details v-if="show('climate')" :open="Boolean(only)" class="block">
      <summary>調候補充（心訣第 92 訣、輕鬆學算命）</summary>
      <ul class="plain">
        <li :class="`check-${climate.check.status}`">{{ ICON[climate.check.status] }} {{ climate.check.detail }}</li>
        <li v-for="note in climate.soilNotes" :key="note">{{ note }}</li>
        <li>{{ firstMethod }}</li>
      </ul>
    </details>

    <details v-if="show('love')" :open="Boolean(only)" class="block">
      <summary>戀愛方式（妙算第 273–275 頁）</summary>
      <p class="line">{{ love.summary }}</p>
      <ul v-if="love.hits.length" class="plain">
        <li v-for="hit in love.hits" :key="hit.pillar">透在{{ hit.pillar }}{{ hit.stem }}：吸引異性的方式是{{ hit.attract }}；多透過{{ hit.via }}認識。</li>
      </ul>
      <p v-if="love.selfExposed" class="hint">日支自透：書說人際關係好、重浪漫、對愛情主動。</p>
    </details>

    <details v-if="show('limit')" :open="Boolean(only)" class="block">
      <summary>限運：每一柱管哪段年齡（妙算第 319 頁、心訣第 42 訣）</summary>
      <table class="luck limit">
        <thead>
          <tr><th>柱</th><th>干支</th><th>妙算</th><th>心訣</th><th>天干</th><th>地支</th><th>說明</th></tr>
        </thead>
        <tbody>
          <tr v-for="row in limits" :key="row.pillar">
            <td>{{ row.pillar }}</td>
            <td>{{ row.ganZhi }}</td>
            <td>{{ row.miaoSuan }}</td>
            <td>{{ row.xinJue }}</td>
            <td :class="`favor-${row.stemFavor}`">{{ FAVOR[row.stemFavor] }}</td>
            <td :class="`favor-${row.branchFavor}`">{{ FAVOR[row.branchFavor] }}</td>
            <td>{{ row.note }}</td>
          </tr>
        </tbody>
      </table>
      <p class="hint">妙算：大運流年要配合限運；走到喜用那一柱的年齡，又不逢刑沖剋破，最能發揮。心訣以年柱為父母宮、月柱為兄弟宮，每柱 16 年。</p>
    </details>

    <p v-if="only && only.length === 1 && only[0] === 'combine' && !combines.length" class="line">原局沒有相鄰天干五合，也沒有暗五合的柱。</p>
    <details v-if="combines.length && show('combine')" :open="Boolean(only)" class="block">
      <summary>五合的三種化法、暗五合（心訣第 96 訣）</summary>
      <ul class="plain">
        <li v-for="note in combines" :key="note">{{ note }}</li>
      </ul>
    </details>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { BaziResponse } from "../types/bazi";
import { lessonPreferences } from "../utils/lessonPreference";
import { yongShenGuide } from "../utils/yongShen";
import {
  clarityReport,
  climateReport,
  combineNotes,
  findRelative,
  firstMethodHint,
  flowReport,
  limitRows,
  loveStyle,
  relativeOptions,
  rotateChart,
  useQuality,
  type CheckStatus,
  type Element,
  type Favor,
  type RotationTarget,
} from "../utils/fourBooks";

const ICON: Record<CheckStatus, string> = { pass: "✓", partial: "△", fail: "✗" };
const FAVOR: Record<Favor, string> = { good: "喜", bad: "忌", mixed: "依方案", neutral: "—" };
const ELEMENT_CLASS: Record<string, string> = {
  甲: "wood", 乙: "wood", 寅: "wood", 卯: "wood",
  丙: "fire", 丁: "fire", 巳: "fire", 午: "fire",
  戊: "earth", 己: "earth", 辰: "earth", 戌: "earth", 丑: "earth", 未: "earth",
  庚: "metal", 辛: "metal", 申: "metal", 酉: "metal",
  壬: "water", 癸: "water", 亥: "water", 子: "water",
};
const WUXING_KEY: Record<Element, "wood" | "fire" | "earth" | "metal" | "water"> = {
  木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water",
};

const props = defineProps<{
  result: BaziResponse;
  /** 只顯示某幾塊：rotation、clarity、flow、quality、climate、love、limit、combine。 */
  only?: string[];
  title?: string;
}>();

const show = (key: string) => !props.only || props.only.includes(key);

const natal = computed(() =>
  [props.result.yearPillar, props.result.monthPillar, props.result.dayPillar, props.result.hourPillar].map(
    (pillar) => `${pillar.stem}${pillar.branch}`,
  ),
);
const dayStem = computed(() => natal.value[2].charAt(0));
const analysis = computed(() => props.result.quantModel?.lessonContent?.threeStepAnalysis);
const preferences = computed(() => lessonPreferences(analysis.value));
const useElements = computed(() => (yongShenGuide(analysis.value)?.useElements ?? []) as Element[]);
const favorOf = (element: Element): Favor => preferences.value?.elements[WUXING_KEY[element]]?.favor ?? "neutral";

const options = computed(() => relativeOptions(props.result.gender === "FEMALE" ? "FEMALE" : "MALE"));
const relativeKey = ref("spouse");
const manualTarget = ref<RotationTarget | null>(null);
const currentOption = computed(() => options.value.find((option) => option.key === relativeKey.value));
const relativeTarget = computed(() => (currentOption.value ? findRelative(natal.value, currentOption.value) : null));
const relativeMissing = computed(() => !manualTarget.value && Boolean(relativeKey.value) && !relativeTarget.value);
const target = computed(() => manualTarget.value ?? relativeTarget.value);

watch(relativeKey, () => {
  manualTarget.value = null;
});
watch(natal, () => {
  manualTarget.value = null;
});

function pick(pillar: number, part: "干" | "支") {
  if (pillar === 2 && part === "干") {
    manualTarget.value = null;
    relativeKey.value = "";
    return;
  }
  manualTarget.value = { pillar, part };
  relativeKey.value = "";
}

const rotation = computed(() =>
  target.value
    ? rotateChart(
        natal.value,
        target.value,
        (props.result.daYun ?? []).filter((row) => row.ganZhi).map((row) => ({ ganZhi: row.ganZhi, startYear: row.startYear, endYear: row.endYear })),
      )
    : null,
);

// 依 App 排法由左到右：時、日、月、年。
const displayCells = computed(() => {
  const base = natal.value.map((pillar, index) => ({
    index,
    pillar: ["年", "月", "日", "時"][index],
    stem: pillar.charAt(0),
    branch: pillar.charAt(1),
    stemGod: index === 2 ? "日主" : "",
    hidden: [] as { stem: string; god: string }[],
    isTarget: null as "干" | "支" | null,
    isSelf: index === 2,
  }));
  const rotated = rotation.value;
  if (rotated) {
    rotated.cells.forEach((cell, index) => {
      base[index].stemGod = index === 2 ? `${cell.stemGod}（命主）` : cell.stemGod;
      base[index].hidden = cell.hidden;
      base[index].isTarget = cell.isTarget;
    });
  }
  return base.slice().reverse();
});

const flow = computed(() => flowReport(natal.value));
const clarity = computed(() => clarityReport(natal.value, useElements.value));
const qualities = computed(() => useElements.value.map((use) => useQuality(natal.value, use)));
const climate = computed(() => climateReport(natal.value));
const firstMethod = computed(() => firstMethodHint(dayStem.value));
const love = computed(() => loveStyle(natal.value));
const limits = computed(() => limitRows(natal.value, preferences.value ? favorOf : undefined));
const combines = computed(() => combineNotes(natal.value));

function elementClass(char: string) {
  return `wx-${ELEMENT_CLASS[char] ?? ""}`;
}

function shortGod(god: string) {
  return god.replace("比肩", "比").replace("劫財", "劫").replace("食神", "食").replace("傷官", "傷").replace("偏財", "才").replace("正財", "財").replace("七殺", "殺").replace("正官", "官").replace("偏印", "梟").replace("正印", "印");
}
</script>

<style scoped>
.intro,
.hint {
  margin: 0 0 8px;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.block {
  margin-top: 8px;
  padding: 6px 10px;
  border-radius: 10px;
  background: rgba(148, 163, 184, 0.08);
}

.block > summary {
  cursor: pointer;
  font-weight: 700;
  font-size: calc(14px * var(--app-font-scale));
  line-height: 1.8;
}

.sub > summary {
  cursor: pointer;
  font-size: calc(13px * var(--app-font-scale));
}

.line,
.warn-line {
  margin: 4px 0;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.warn-line {
  color: #b42318;
}

.relative-row {
  margin: 4px 0 6px;
  overflow-x: auto;
}

.chart {
  border-collapse: collapse;
  margin: 6px 0;
  font-size: calc(13px * var(--app-font-scale));
}

.chart th,
.chart td {
  padding: 2px 8px;
  text-align: center;
}

.chart th {
  color: var(--bazi-text-muted);
  font-weight: 500;
}

.char {
  min-width: 2em;
  padding: 2px 4px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: none;
  font-size: calc(18px * var(--app-font-scale));
  font-weight: 700;
  cursor: pointer;
}

.char.target {
  border-color: #ca8a04;
  background: rgba(202, 138, 4, 0.14);
}

.char.self {
  border-style: dashed;
  border-color: #94a3b8;
}

.god {
  font-size: calc(12px * var(--app-font-scale));
}

.hidden-gods {
  font-size: calc(11px * var(--app-font-scale));
  line-height: 1.4;
}

.hidden-gods span {
  display: block;
}

.wx-wood { color: var(--wx-wood); }
.wx-fire { color: var(--wx-fire); }
.wx-earth { color: var(--wx-earth); }
.wx-metal { color: var(--wx-metal); }
.wx-water { color: var(--wx-water); }

.luck {
  width: 100%;
  border-collapse: collapse;
  margin: 4px 0;
  font-size: calc(12px * var(--app-font-scale));
}

.luck th,
.luck td {
  padding: 2px 6px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.25);
  text-align: left;
}

.limit th,
.limit td:nth-child(-n + 6) {
  white-space: nowrap;
}

.effect-good td:last-child,
.favor-good {
  color: #2f7d45;
}

.effect-bad td:last-child,
.favor-bad {
  color: #b42318;
}

.plain {
  margin: 4px 0;
  padding-left: 1.2em;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.check-list {
  display: grid;
  gap: 2px;
  margin: 4px 0;
  padding: 0;
  list-style: none;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.55;
}

.check-list li {
  display: grid;
  grid-template-columns: 1.2em auto 1fr;
  column-gap: 6px;
}

.check-icon {
  font-weight: 700;
}

.check-pass .check-icon,
.plain .check-pass {
  color: #2f7d45;
}

.check-partial .check-icon,
.plain .check-partial {
  color: #a16207;
}

.check-fail .check-icon,
.plain .check-fail {
  color: #b42318;
}

.check-label {
  font-weight: 600;
  white-space: nowrap;
}

.quality + .quality {
  margin-top: 6px;
}
</style>
