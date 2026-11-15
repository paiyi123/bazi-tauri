<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>格局高低總評</h2>
      </div>
    </template>
    <template v-if="grade">
      <div class="verdict" :class="`level-${levelKey}`">
        <div class="scale" aria-label="等級">
          <span v-for="item in LEVELS" :key="item" class="step" :class="{ on: item === grade.level }">{{ item }}</span>
        </div>
        <div class="verdict-text">
          <b>{{ grade.levelText }}</b>
          <span class="total">合計 {{ signed(grade.total) }}（滿分 ±{{ grade.max }}）</span>
          <span v-if="grade.headline" class="headline">{{ grade.headline }}。</span>
        </div>
      </div>
      <p class="intro">
        照《命理新論》下冊看富貴的三個步驟：先看格局成不成，再看喜用有沒有力，最後看行運向背；中間補上清濁、中和、流通（蘇民峰「格局高低六項」、陸致極「單純與有力」）。
        每一項最多加減 2 分，每一條都註明出處。強弱本身不分好壞，只看平衡；喜忌一律用課堂三步法。
      </p>

      <details v-for="dim in grade.dimensions" :key="dim.key" class="dim" :open="isResearch">
        <summary>
          <span class="dim-title">{{ dim.title }}</span>
          <span class="dim-score" :class="scoreClass(dim)">{{ dim.skipped ? "不計分" : signed(dim.score) }}</span>
          <span class="dim-question">{{ dim.question }}</span>
        </summary>
        <p v-if="dim.skipped" class="skipped">{{ dim.skipped }}</p>
        <ul class="reasons">
          <li v-for="(reason, idx) in dim.reasons" :key="idx" :class="reasonClass(reason.delta)">
            <span class="delta">{{ reason.delta === 0 ? "提醒" : signed(reason.delta) }}</span>
            <span class="reason-body">
              <span class="reason-text">{{ reason.text }}</span>
              <span class="reason-source">出處：{{ reason.source }}</span>
            </span>
          </li>
        </ul>
        <p v-if="!dim.skipped && dim.raw !== dim.score" class="capped">這一項原始合計 {{ signed(dim.raw) }}，以 ±2 為上限計。</p>
      </details>

      <ul class="caveats">
        <li v-for="(line, idx) in grade.caveats" :key="idx">{{ line }}</li>
      </ul>
    </template>
    <el-empty v-else description="這張命盤沒有課堂三步分析，無法評格局高低。" />
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { useViewMode } from "../composables/useViewMode";
import { patternGrade, type GradeDimension, type GradeLevel } from "../utils/patternGrade";

const props = defineProps<{ result: BaziResponse }>();
const { isResearch } = useViewMode();

const LEVELS: GradeLevel[] = ["上", "中上", "中", "中下", "下"];
const grade = computed(() => patternGrade(props.result));
const levelKey = computed(() => (grade.value ? LEVELS.indexOf(grade.value.level) : 2));

const signed = (value: number) => (value > 0 ? `+${value}` : `${value}`);
const scoreClass = (dim: GradeDimension) => (dim.skipped ? "muted" : dim.score > 0 ? "plus" : dim.score < 0 ? "minus" : "zero");
const reasonClass = (delta: number) => (delta > 0 ? "plus" : delta < 0 ? "minus" : "note");
</script>

<style scoped>
.verdict {
  display: grid;
  gap: 8px;
  margin-bottom: 10px;
  padding: 10px 12px;
  border-radius: 12px;
  background: rgba(148, 163, 184, 0.1);
}

.verdict.level-0,
.verdict.level-1 {
  background: rgba(47, 125, 69, 0.09);
}

.verdict.level-3,
.verdict.level-4 {
  background: rgba(180, 35, 24, 0.07);
}

.scale {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.step {
  min-width: 2.6em;
  padding: 2px 8px;
  border-radius: 8px;
  text-align: center;
  color: var(--bazi-text-muted);
  background: rgba(148, 163, 184, 0.16);
  font-size: calc(13px * var(--app-font-scale));
}

.step.on {
  color: #fff;
  background: #7c5a2e;
  font-weight: 700;
}

.level-0 .step.on,
.level-1 .step.on {
  background: #2f7d45;
}

.level-3 .step.on,
.level-4 .step.on {
  background: #b42318;
}

.verdict-text {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 2px 10px;
  line-height: 1.6;
}

.total,
.headline {
  color: #44403c;
  font-size: calc(13px * var(--app-font-scale));
}

.intro {
  margin: 0 0 8px;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.dim {
  margin-bottom: 6px;
  border-radius: 10px;
  background: rgba(148, 163, 184, 0.06);
}

.dim summary {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 2px 8px;
  padding: 6px 10px;
  cursor: pointer;
}

.dim-title {
  font-weight: 700;
}

.dim-score {
  padding: 0 8px;
  border-radius: 8px;
  font-size: calc(12px * var(--app-font-scale));
  font-weight: 700;
}

.dim-score.plus {
  color: #2f7d45;
  background: rgba(47, 125, 69, 0.12);
}

.dim-score.minus {
  color: #b42318;
  background: rgba(180, 35, 24, 0.1);
}

.dim-score.zero,
.dim-score.muted {
  color: var(--bazi-text-muted);
  background: rgba(148, 163, 184, 0.16);
}

.dim-question {
  flex-basis: 100%;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
}

.skipped,
.capped {
  margin: 0 10px 6px;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
}

.reasons {
  display: grid;
  gap: 6px;
  margin: 0;
  padding: 0 10px 8px;
  list-style: none;
}

.reasons li {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 8px;
  background: rgba(148, 163, 184, 0.08);
  box-shadow: inset 3px 0 0 #94a3b8;
}

.reasons li.plus {
  background: rgba(47, 125, 69, 0.07);
  box-shadow: inset 3px 0 0 #2f7d45;
}

.reasons li.minus {
  background: rgba(180, 35, 24, 0.05);
  box-shadow: inset 3px 0 0 #b42318;
}

.delta {
  min-width: 2.6em;
  font-size: calc(12px * var(--app-font-scale));
  font-weight: 700;
  text-align: center;
}

.plus .delta {
  color: #2f7d45;
}

.minus .delta {
  color: #b42318;
}

.note .delta {
  color: var(--bazi-text-muted);
}

.reason-body {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.reason-text {
  color: #44403c;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.reason-source {
  color: var(--bazi-text-muted);
  font-size: calc(11.5px * var(--app-font-scale));
}

.caveats {
  margin: 8px 0 0;
  padding-left: 1.2em;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.6;
}
</style>
