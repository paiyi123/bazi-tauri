<template>
  <details class="source-fold" :open="open">
    <summary>
      <span class="fold-title">{{ label }}</span>
      <span class="fold-hint">{{ hint }}</span>
    </summary>
    <div class="fold-body">
      <slot />
    </div>
  </details>
</template>

<script setup lang="ts">
// 「各書細節」摺疊區：新手模式預設收起，研究模式預設打開。卡片本身不變，只是收進來。
defineProps<{ label: string; hint?: string; open?: boolean }>();
</script>

<style scoped>
.source-fold {
  border: 1px dashed var(--bazi-border-strong);
  border-radius: 14px;
  background: rgba(255, 250, 240, 0.6);
}
summary {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 2px 10px;
  padding: 10px 14px;
  cursor: pointer;
  list-style: none;
}
summary::-webkit-details-marker {
  display: none;
}
summary::before {
  content: "▸";
  color: var(--bazi-accent);
  transition: transform var(--bazi-transition);
}
.source-fold[open] > summary::before {
  transform: rotate(90deg);
}
.fold-title {
  color: var(--bazi-primary);
  font-weight: 700;
}
.fold-hint {
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
}
.fold-body {
  display: grid;
  gap: 14px;
  padding: 0 10px 12px;
  min-width: 0;
}
</style>
