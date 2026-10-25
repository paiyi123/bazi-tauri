<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>特別格局（蘇民峰）</h2>
      </div>
    </template>
    <p class="intro">
      依蘇民峰《八字論命》第十四章逐條檢查從格、從旺五格、化氣格，以及魁罡、金神、兩神成象。
      這裡只列「候選」和每一條件是否符合（✓ 符合、△ 有瑕疵、✗ 不符），不改五行力分數與課堂喜忌。
      大運流年格子會標出化神還原、沖魁罡。
    </p>
    <template v-if="report">
      <ul v-if="shown.length" class="pattern-list">
        <li v-for="item in shown" :key="item.key" :class="`verdict-${item.verdict}`">
          <div class="pattern-head">
            <span class="pattern-name">{{ item.name }}</span>
            <span class="verdict">{{ item.verdictLabel }}</span>
            <span class="page">{{ item.page }}</span>
          </div>
          <p class="pattern-summary">{{ item.summary }}</p>
          <ul class="check-list">
            <li v-for="check in item.checks" :key="check.label" :class="`check-${check.status}`">
              <span class="check-icon">{{ ICON[check.status] }}</span>
              <span class="check-label">{{ check.label }}</span>
              <span class="check-detail">{{ check.detail }}</span>
            </li>
          </ul>
          <p v-if="item.advice" class="advice">{{ item.advice }}</p>
        </li>
      </ul>
      <el-empty v-else description="原局沒有符合書中特別格局條件的候選，照正格（扶抑）論。" :image-size="60" />
      <p v-if="hidden.length" class="hidden-list">
        不成立：
        <template v-for="(item, index) in hidden" :key="item.key">
          <el-tooltip placement="top" :content="failedText(item)">
            <span class="hidden-name">{{ item.name }}</span>
          </el-tooltip>
          <span v-if="index < hidden.length - 1">、</span>
        </template>
      </p>
      <ul v-if="report.notes.length" class="note-list">
        <li v-for="note in report.notes" :key="note">{{ note }}</li>
      </ul>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { specialPatternReport, type CheckStatus, type PatternCandidate } from "../utils/specialPatterns";

const ICON: Record<CheckStatus, string> = { pass: "✓", partial: "△", fail: "✗" };

const props = defineProps<{
  result: BaziResponse;
}>();

const report = computed(() =>
  specialPatternReport(
    [props.result.yearPillar, props.result.monthPillar, props.result.dayPillar, props.result.hourPillar].map(
      (pillar) => `${pillar.stem}${pillar.branch}`,
    ),
    { female: props.result.gender === "FEMALE" },
  ),
);

const all = computed<PatternCandidate[]>(() => {
  const value = report.value;
  if (!value) return [];
  return [...value.follow, value.prosperous, value.transform, ...value.marks].filter(
    (item): item is PatternCandidate => item !== null,
  );
});

const shown = computed(() => all.value.filter((item) => item.verdict !== "none" || isMark(item)));
const hidden = computed(() => all.value.filter((item) => item.verdict === "none" && !isMark(item)));

// 魁罡、金神被破也值得看（破格本身是書中的論點），所以一律展開。
function isMark(item: PatternCandidate) {
  return report.value?.marks.includes(item) ?? false;
}

function failedText(item: PatternCandidate) {
  const failed = item.checks.filter((check) => check.status === "fail");
  return failed.length ? failed.map((check) => `✗ ${check.label}：${check.detail}`).join("　") : item.summary;
}
</script>

<style scoped>
.intro {
  margin: 0 0 8px;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.pattern-list {
  display: grid;
  gap: 10px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.pattern-list > li {
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(148, 163, 184, 0.08);
  box-shadow: inset 3px 0 0 #94a3b8;
}

.pattern-list > li.verdict-formed {
  background: rgba(47, 125, 69, 0.08);
  box-shadow: inset 3px 0 0 #2f7d45;
}

.pattern-list > li.verdict-partial {
  background: rgba(202, 138, 4, 0.08);
  box-shadow: inset 3px 0 0 #ca8a04;
}

.pattern-list > li.verdict-none {
  background: rgba(180, 35, 24, 0.05);
  box-shadow: inset 3px 0 0 #b42318;
}

.pattern-head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 8px;
}

.pattern-name {
  font-weight: 700;
}

.verdict {
  padding: 0 8px;
  border-radius: 8px;
  background: rgba(148, 163, 184, 0.18);
  font-size: calc(12px * var(--app-font-scale));
}

.page {
  color: var(--bazi-text-muted);
  font-size: calc(11px * var(--app-font-scale));
}

.pattern-summary,
.advice {
  margin: 4px 0;
  color: #44403c;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.advice {
  color: var(--bazi-text-muted);
}

.check-list {
  display: grid;
  gap: 2px;
  margin: 4px 0 0;
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

.check-pass .check-icon {
  color: #2f7d45;
}

.check-partial .check-icon {
  color: #ca8a04;
}

.check-fail .check-icon {
  color: #b42318;
}

.check-label {
  font-weight: 600;
  white-space: nowrap;
}

.check-detail {
  color: #57534e;
  min-width: 0;
}

.hidden-list,
.note-list {
  margin: 8px 0 0;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.6;
}

.hidden-name {
  border-bottom: 1px dotted currentColor;
  cursor: help;
}

.note-list {
  padding-left: 18px;
}

@media (max-width: 640px) {
  .check-list li {
    grid-template-columns: 1.2em 1fr;
  }

  .check-detail {
    grid-column: 2;
  }

  .check-label {
    white-space: normal;
  }
}
</style>
