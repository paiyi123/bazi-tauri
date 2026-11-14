<template>
  <el-card shadow="hover" class="pillar-summary">
    <template #header>
      <div class="section-header">
        <h2>{{ hideConclusions ? "干支作用變化" : "作用變化與強弱喜忌" }}</h2>
      </div>
    </template>

    <template v-if="summary">
      <dl v-if="!hideConclusions" class="conclusions">
        <div class="row">
          <dt>日主</dt>
          <dd>
            <strong>{{ summary.strength.label }}</strong>
            <span class="sub">{{ summary.strength.detail }}</span>
            <span v-if="summary.strength.review" class="review">⚠ {{ summary.strength.review }}</span>
          </dd>
        </div>
        <div class="row">
          <dt>用神</dt>
          <dd>
            <template v-if="summary.useGods.length">
              <strong>{{ summary.useGods.join("／") }}</strong>
              <span v-if="summary.planCount > 1" class="sub">{{ summary.planCount }} 個條件方案，依條件而定</span>
            </template>
            <span v-else class="sub">{{ summary.noPlanReason }}</span>
          </dd>
        </div>
        <div class="row">
          <dt>喜</dt>
          <dd>
            <span v-for="item in summary.favorable" :key="item.element" class="wx-chip" :class="`wx-${item.element}`">{{ item.name }}</span>
            <span v-if="!summary.favorable.length" class="sub">—</span>
          </dd>
        </div>
        <div class="row">
          <dt>忌</dt>
          <dd>
            <span v-for="item in summary.unfavorable" :key="item.element" class="wx-chip" :class="`wx-${item.element}`">{{ item.name }}</span>
            <span v-if="!summary.unfavorable.length" class="sub">—</span>
          </dd>
        </div>
        <div v-if="summary.undecided.length" class="row">
          <dt>未定</dt>
          <dd>
            <span v-for="item in summary.undecided" :key="item.element" class="wx-chip" :class="`wx-${item.element}`">{{ item.name }}</span>
            <span class="sub">方案之間喜忌相反</span>
          </dd>
        </div>
      </dl>

      <section class="block">
        <h3>天干生剋合化</h3>
        <ul v-if="summary.stemLines.length" class="lines">
          <li v-for="line in summary.stemLines" :key="line.key" :class="{ muted: line.muted }">
            <span class="mark" :class="`v-${line.verdict}`">{{ mark(line.verdict) }}</span>
            <span class="label">{{ line.label }}</span>
            <span class="where">（{{ line.where }}）</span>
            <span class="detail">{{ line.detail }}</span>
          </li>
        </ul>
        <p v-else class="none">天干沒有五合。</p>
        <div v-if="summary.stemRelations.length" class="chips">
          <span v-for="chip in summary.stemRelations" :key="chip.key" class="rel-chip" :class="{ muted: chip.muted }">
            {{ chip.text }}<small>{{ chip.note }}</small>
          </span>
        </div>
      </section>

      <section class="block">
        <h3>地支沖合會刑害</h3>
        <ul v-if="summary.branchLines.length" class="lines">
          <li v-for="line in summary.branchLines" :key="line.key" :class="{ muted: line.muted }">
            <span class="mark" :class="`v-${line.verdict}`">{{ mark(line.verdict) }}</span>
            <span class="label">{{ line.label }}</span>
            <span class="where">（{{ line.where }}）</span>
            <span class="detail">{{ line.detail }}</span>
          </li>
        </ul>
        <p v-else class="none">地支沒有沖、合、會、刑、害。</p>
      </section>

      <p class="foot">依課堂三步法；▲有利 ▼不利 ◆依方案而定。詳細推論見「分析」頁。</p>
    </template>
    <p v-else class="none">此結果尚無課堂三步分析。</p>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { pillarSummary } from "../utils/pillarSummary";
import type { Favor } from "../utils/lessonPreference";

// hideConclusions：上方已有「本命結論」卡時，不再重複列強弱、用神、喜忌。
const props = defineProps<{ result: BaziResponse; hideConclusions?: boolean }>();

const summary = computed(() => pillarSummary(props.result.quantModel?.lessonContent?.threeStepAnalysis));

const mark = (verdict: Favor) => ({ good: "▲", bad: "▼", mixed: "◆", neutral: "・" })[verdict];
</script>

<style scoped>
.pillar-summary {
  min-width: 0;
  overflow-wrap: anywhere;
  font-size: calc(15px * var(--app-font-scale));
  line-height: 1.6;
}

.conclusions {
  display: grid;
  gap: 6px;
  margin: 0;
}

.row {
  display: grid;
  grid-template-columns: calc(2.6em) minmax(0, 1fr);
  gap: 8px;
  align-items: baseline;
}

dt {
  font-weight: 700;
  color: var(--el-text-color-secondary, #606266);
}

dd {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 8px;
  margin: 0;
  min-width: 0;
}

.sub {
  color: var(--el-text-color-secondary, #606266);
  font-size: 0.88em;
}

.review {
  flex-basis: 100%;
  color: var(--el-color-warning-dark-2, #b88230);
  font-size: 0.88em;
}

.wx-chip {
  display: inline-block;
  min-width: 1.9em;
  padding: 0 0.45em;
  border: 1px solid currentColor;
  border-radius: 999px;
  font-weight: 700;
  text-align: center;
}

.wx-wood { color: var(--wx-wood); }
.wx-fire { color: var(--wx-fire); }
.wx-earth { color: var(--wx-earth); }
.wx-metal { color: var(--wx-metal); }
.wx-water { color: var(--wx-water); }

.block {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px dashed var(--el-border-color-lighter, #ebeef5);
}

h3 {
  margin: 0 0 6px;
  font-size: 1em;
}

.lines {
  margin: 0;
  padding: 0;
  list-style: none;
}

.lines li + li {
  margin-top: 4px;
}

.mark {
  display: inline-block;
  width: 1.2em;
  font-weight: 700;
}

.v-good { color: var(--el-color-success, #2f9e44); }
.v-bad { color: var(--el-color-danger, #d9480f); }
.v-mixed, .v-neutral { color: var(--el-text-color-secondary, #909399); }

.label {
  font-weight: 600;
}

.where {
  color: var(--el-text-color-secondary, #606266);
  font-size: 0.88em;
}

.detail {
  display: block;
  padding-left: 1.2em;
  color: var(--el-text-color-regular, #4b5563);
  font-size: 0.9em;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 6px;
  margin-top: 6px;
}

.rel-chip {
  padding: 0 0.5em;
  border: 1px solid var(--el-border-color, #dcdfe6);
  border-radius: 6px;
  white-space: nowrap;
}

.rel-chip small {
  margin-left: 0.3em;
  color: var(--el-text-color-secondary, #909399);
  font-size: 0.8em;
}

.muted,
.muted .label {
  color: var(--el-text-color-secondary, #909399);
}

.none,
.foot {
  margin: 0;
  color: var(--el-text-color-secondary, #909399);
  font-size: 0.88em;
}

.foot {
  margin-top: 10px;
}
</style>
