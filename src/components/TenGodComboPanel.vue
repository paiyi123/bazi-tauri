<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>十神組合（十神洩天機）</h2>
      </div>
    </template>
    <p class="intro">
      依潘樂德《十神洩天機》整理：正官、正印、財、食神是正神，宜順用；七殺、偏印、傷官、比劫是偏神，宜有制。
      書中把喜忌當成已知，所以每項後面附上課堂三步法的喜忌供你判斷。大運流年格子上的
      <span class="dot">動</span>標籤是四大動神的變動提示。
    </p>
    <div v-if="overview" class="overview">
      <span class="chip">十神領袖：<b>{{ overview.leader }}</b>（{{ overview.leaderSource }}）</span>
      <span class="chip">正神 {{ overview.regular }}・偏神 {{ overview.irregular }}（三干四支本氣）</span>
    </div>
    <ul v-if="overview?.items.length" class="topic-list">
      <li v-for="item in overview.items" :key="item.key" :class="`tone-${item.tone}`">
        <span class="topic-label">{{ item.label }}</span>
        <span class="topic-detail">{{ item.detail }}</span>
        <span v-if="item.favorNote" class="topic-favor">{{ item.favorNote }}</span>
      </li>
    </ul>
    <el-empty v-else description="原局沒有書中所列的十神組合。" />
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { lessonPreferences } from "../utils/lessonPreference";
import { tenGodOverview } from "../utils/tenGodCombos";

const props = defineProps<{
  result: BaziResponse;
}>();

const overview = computed(() =>
  tenGodOverview(
    [props.result.yearPillar, props.result.monthPillar, props.result.dayPillar, props.result.hourPillar].map(
      (pillar) => `${pillar.stem}${pillar.branch}`,
    ),
    lessonPreferences(props.result.quantModel?.lessonContent?.threeStepAnalysis),
  ),
);
</script>

<style scoped>
.intro {
  margin: 0 0 8px;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.dot {
  display: inline-block;
  margin: 0 2px;
  padding: 0 5px;
  border-radius: 6px;
  color: #3730a3;
  background: #e0e7ff;
  font-size: calc(11px * var(--app-font-scale));
  font-weight: 700;
}

.overview {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.chip {
  padding: 2px 10px;
  border-radius: 10px;
  background: rgba(148, 163, 184, 0.14);
  font-size: calc(13px * var(--app-font-scale));
}

.topic-list {
  display: grid;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.topic-list li {
  display: grid;
  gap: 2px;
  padding: 6px 10px;
  border-radius: 10px;
  background: rgba(148, 163, 184, 0.08);
  box-shadow: inset 3px 0 0 #94a3b8;
}

.topic-list li.tone-good {
  background: rgba(47, 125, 69, 0.08);
  box-shadow: inset 3px 0 0 #2f7d45;
}

.topic-list li.tone-bad {
  background: rgba(180, 35, 24, 0.06);
  box-shadow: inset 3px 0 0 #b42318;
}

.topic-label {
  font-weight: 700;
}

.topic-detail {
  color: #44403c;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.topic-favor {
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
}
</style>
