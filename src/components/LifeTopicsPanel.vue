<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>{{ title ?? "婚姻・財運・健康" }}</h2>
      </div>
    </template>
    <p v-if="only" class="intro">
      依陸致極《八字命理學進階教程》第六、七、十一、十三章；喜忌用陸致極的強弱。只是書中的觀察方法，不是定論。
    </p>
    <p v-else class="intro">
      依陸致極《八字命理學進階教程》第六、七、十一、十三章的判讀要點整理。大運流年格子上的
      <span class="dot is-marriage">婚</span><span class="dot is-wealth">財</span><span class="dot is-health">健</span>
      標籤是對應的應期提示。這些只是書中的觀察方法，不是定論。
    </p>
    <el-collapse v-model="opened">
      <el-collapse-item v-for="section in sections" :key="section.key" :name="section.key">
        <template #title>
          <span :class="['section-title', `is-${section.key}`]">{{ section.title }}</span>
          <span class="section-summary">{{ section.summary }}</span>
        </template>
        <ul class="topic-list">
          <li v-for="item in section.items" :key="item.label" :class="`tone-${item.tone}`">
            <span class="topic-label">{{ item.label }}</span>
            <span class="topic-detail">{{ item.detail }}</span>
          </li>
        </ul>
      </el-collapse-item>
    </el-collapse>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { BaziResponse } from "../types/bazi";
import { lifeTopicSections } from "../utils/lifeTopics";

const props = defineProps<{
  result: BaziResponse;
  /** 只顯示某幾個主題（marriage、wealth、health）。 */
  only?: string[];
  title?: string;
}>();

const opened = ref(["marriage", "wealth", "health"]);

const sections = computed(() =>
  lifeTopicSections({
    natal: [props.result.yearPillar, props.result.monthPillar, props.result.dayPillar, props.result.hourPillar].map(
      (pillar) => `${pillar.stem}${pillar.branch}`,
    ),
    gender: props.result.gender,
    strength: props.result.quantModel?.luStrength?.category,
  }).filter((section) => !props.only || props.only.includes(section.key)),
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
  color: #fff;
  font-size: calc(11px * var(--app-font-scale));
  font-weight: 600;
}

.dot.is-marriage,
.section-title.is-marriage {
  background: #be185d;
}

.dot.is-wealth,
.section-title.is-wealth {
  background: #a16207;
}

.dot.is-health,
.section-title.is-health {
  background: #0f766e;
}

.section-title {
  flex-shrink: 0;
  margin-right: 10px;
  padding: 0 8px;
  border-radius: 8px;
  color: #fff;
  font-weight: 700;
  line-height: 1.8;
}

.section-summary {
  overflow: hidden;
  color: var(--bazi-text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
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
</style>
