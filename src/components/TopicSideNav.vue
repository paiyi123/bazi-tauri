<template>
  <nav class="side-nav" aria-label="目錄">
    <button type="button" :class="['side-nav-item', { active: current === 'top' }]" @click="go('top')">四柱</button>
    <button
      v-for="topic in topics"
      :key="topic.key"
      type="button"
      :class="['side-nav-item', { active: current === topic.key }]"
      @click="go(topic.key)"
    >
      {{ topic.short }}
    </button>
  </nav>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { visibleTopicList } from "../constants/analysisTopics";
import { useViewMode } from "../composables/useViewMode";

// 電腦版左欄的目錄：點一下跳到那一區，捲動時標出目前看到哪一區。
const { isResearch } = useViewMode();
const topics = computed(() => visibleTopicList(isResearch.value));
const current = ref("top");

function go(key: string) {
  if (key === "top") {
    window.scrollTo({ top: 0, behavior: "smooth" });
    return;
  }
  document.getElementById(`topic-${key}`)?.scrollIntoView({ behavior: "smooth", block: "start" });
}

function update() {
  let found = "top";
  for (const topic of topics.value) {
    const el = document.getElementById(`topic-${topic.key}`);
    if (el && el.getBoundingClientRect().top <= 140) found = topic.key;
  }
  current.value = found;
}

let frame = 0;
function onScroll() {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = 0;
    update();
  });
}
onMounted(() => {
  window.addEventListener("scroll", onScroll, { passive: true });
  update();
});
onBeforeUnmount(() => {
  window.removeEventListener("scroll", onScroll);
  if (frame) cancelAnimationFrame(frame);
});
</script>

<style scoped>
.side-nav {
  display: none;
}
@media (min-width: 1100px) {
  .side-nav {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 10px 12px;
    border: 1px solid var(--bazi-border);
    border-radius: 16px;
    background: var(--bazi-surface-strong);
  }
}
.side-nav-item {
  padding: 4px 12px;
  border: 1px solid var(--bazi-border);
  border-radius: 999px;
  background: transparent;
  color: var(--bazi-text);
  font: inherit;
  font-size: calc(13px * var(--app-font-scale));
  cursor: pointer;
}
.side-nav-item:hover {
  border-color: var(--bazi-primary);
}
.side-nav-item.active {
  border-color: var(--bazi-primary);
  background: var(--bazi-primary);
  color: #fffaf0;
  font-weight: 700;
}
</style>
