<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>{{ title ?? "四本書：六親、婚姻與人生各項" }}</h2>
      </div>
    </template>
    <p class="intro">
      依《妙算人生八字》《八字心訣》《子平正源》。只當提示：不改分數，強弱用課堂三步法（{{ strengthText }}），喜忌、用神也用課堂結果。
      <span class="tone-good">綠</span>＝書上說有利、<span class="tone-bad">紅</span>＝書上說要留意、灰＝說明。
    </p>

    <details v-if="show('relatives')" open class="block">
      <summary>六親一覽（心訣第 45、61 訣：先找星、看在哪、量距離、論旺衰）</summary>
      <div class="table-wrap">
        <table class="grid relatives">
          <thead>
            <tr><th>親人</th><th>代表的星</th><th>在哪裡</th><th>距離</th><th>星喜忌</th><th>宮位</th><th>宮喜忌</th><th>那位親人自己</th></tr>
          </thead>
          <tbody>
            <tr v-for="row in relatives" :key="row.relative">
              <td class="nowrap"><b>{{ row.relative }}</b></td>
              <td class="nowrap">{{ row.star }}</td>
              <td>{{ row.where }}</td>
              <td class="nowrap">{{ row.distance }}</td>
              <td :class="`favor-${row.favor}`">{{ FAVOR[row.favor] }}</td>
              <td>{{ row.palace }}</td>
              <td :class="`favor-${row.palaceFavor}`">{{ FAVOR[row.palaceFavor] }}</td>
              <td>{{ row.vigor }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p class="hint">
        年柱妙算看祖上父母、心訣稱父母宮；月柱看兄弟。星看「他對我的實質幫助」，宮看「我對他的感覺」（心訣）；緊貼日主（月干、時干、日支）關係密切，遠離則緣薄或少見面。
        星不現時書說可以用宮代替；想看那位親人自己的運，用上面「四本書檢核」卡片的轉盤。
      </p>
    </details>

    <details v-for="section in sections" :key="section.key" :open="Boolean(only) || section.key === 'marriage'" class="block">
      <summary>{{ section.title }}</summary>
      <ul class="items">
        <li v-for="item in section.items" :key="item.label" :class="`tone-${item.tone}`">
          <span class="label">{{ item.label }}</span>
          <span class="detail">{{ item.detail }}<small class="source">（{{ item.source }}）</small></span>
        </li>
      </ul>
      <template v-if="section.key === 'luck'">
        <div class="table-wrap">
          <table class="grid luck-gods">
            <thead>
              <tr><th>走到</th><th>五行</th><th>課堂</th><th>動到哪方面（妙算）</th><th>這次的意思</th><th>心訣：錢</th><th>名</th><th>事業</th><th>感情</th></tr>
            </thead>
            <tbody>
              <tr v-for="row in luckGods" :key="row.family">
                <td class="nowrap"><b>{{ row.family }}運</b></td>
                <td>{{ row.element }}</td>
                <td :class="`favor-${row.favor}`">{{ FAVOR[row.favor] }}</td>
                <td>{{ row.domain }}</td>
                <td :class="`favor-${row.favor}`">{{ row.meaning }}</td>
                <td>{{ row.money }}</td>
                <td>{{ row.fame }}</td>
                <td>{{ row.career }}</td>
                <td>{{ row.love }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <p class="hint">
          大運流年格子另外標：轉角運（大運跨方位）、運剋歲（犯上）、天地合日柱（婚期）、雙沖用神（健康）、驛馬動。
        </p>
      </template>
    </details>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { lessonPreferences } from "../utils/lessonPreference";
import { yongShenGuide } from "../utils/yongShen";
import type { Element, Favor } from "../utils/fourBooks";
import { fourBookTopicSections, luckGodRows, relativeRows, type BookTopicContext } from "../utils/fourBookTopics";

const FAVOR: Record<Favor, string> = { good: "喜", bad: "忌", mixed: "依方案", neutral: "—" };
const WUXING_KEY: Record<Element, "wood" | "fire" | "earth" | "metal" | "water"> = {
  木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water",
};

const props = defineProps<{
  result: BaziResponse;
  /** 只顯示某幾塊：relatives（六親表）、family、marriage、health、career、wealth、study、luck。 */
  only?: string[];
  title?: string;
}>();

const show = (key: string) => !props.only || props.only.includes(key);

const analysis = computed(() => props.result.quantModel?.lessonContent?.threeStepAnalysis);
const preferences = computed(() => lessonPreferences(analysis.value));
const classification = computed(() => analysis.value?.strength?.classification ?? "");
const strong = computed(() => {
  if (["強候選", "旺候選", "旺極候選"].includes(classification.value)) return true;
  if (["弱候選", "衰極候選"].includes(classification.value)) return false;
  return undefined;
});
const strengthText = computed(() =>
  strong.value === undefined ? "課堂沒有定出強弱，改用心訣三選二" : `課堂判「${classification.value}」`,
);

const context = computed<BookTopicContext>(() => ({
  natal: [props.result.yearPillar, props.result.monthPillar, props.result.dayPillar, props.result.hourPillar].map(
    (pillar) => `${pillar.stem}${pillar.branch}`,
  ),
  gender: props.result.gender === "FEMALE" ? "FEMALE" : "MALE",
  strong: strong.value,
  favorOf: preferences.value
    ? (element: Element): Favor => preferences.value?.elements[WUXING_KEY[element]]?.favor ?? "neutral"
    : undefined,
  useElements: (yongShenGuide(analysis.value)?.useElements ?? []) as Element[],
  luck: (props.result.daYun ?? []).map((row) => row.ganZhi).filter(Boolean),
}));

const relatives = computed(() => relativeRows(context.value));
const sections = computed(() => fourBookTopicSections(context.value).filter((section) => show(section.key)));
const luckGods = computed(() => luckGodRows(context.value));
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

.table-wrap {
  overflow-x: auto;
}

.grid {
  width: 100%;
  border-collapse: collapse;
  margin: 4px 0;
  font-size: calc(12px * var(--app-font-scale));
}

.grid th,
.grid td {
  padding: 3px 6px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.25);
  text-align: left;
  vertical-align: top;
}

.relatives {
  min-width: 600px;
}

.luck-gods {
  min-width: 760px;
}

.grid th {
  color: var(--bazi-text-muted);
  font-weight: 500;
  white-space: nowrap;
}

.nowrap,
.favor-good,
.favor-bad,
.favor-mixed,
.favor-neutral {
  white-space: nowrap;
}

.grid td.favor-good:not(:nth-child(3)),
.grid td.favor-bad:not(:nth-child(3)) {
  white-space: normal;
}

.favor-good,
.tone-good {
  color: #2f7d45;
}

.favor-bad,
.tone-bad {
  color: #b42318;
}

.items {
  display: grid;
  gap: 4px;
  margin: 4px 0;
  padding: 0;
  list-style: none;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.55;
}

.items li {
  display: grid;
  grid-template-columns: minmax(6em, max-content) 1fr;
  column-gap: 8px;
  padding-left: 6px;
  border-left: 3px solid rgba(148, 163, 184, 0.5);
}

.items li.tone-good {
  border-left-color: #2f7d45;
}

.items li.tone-bad {
  border-left-color: #b42318;
}

.items .label {
  font-weight: 600;
}

.items .detail {
  color: var(--bazi-text, inherit);
}

.source {
  color: var(--bazi-text-muted);
  font-size: calc(11px * var(--app-font-scale));
}

@media (max-width: 640px) {
  .items li {
    grid-template-columns: 1fr;
  }
}
</style>
