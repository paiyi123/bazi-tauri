<template>
  <span v-if="info && (info.tags.length || info.toElement || info.root)" class="sv-tags">
    <span v-if="info.root" :class="['sv-tag', 'sv-tag-root', { 'is-shaken': info.root.shaken }]">{{ info.root.text }}</span>
    <span v-if="info.toElement" class="sv-tag sv-tag-to" :title="`合化後以${info.toElement}論`">化{{ info.toElement }}</span>
    <span
      v-for="(tag, idx) in info.tags"
      :key="idx"
      :class="['sv-tag', `is-${tag.help ?? 'none'}`, { 'is-light': tag.light, 'is-short': !tag.current }]"
      :title="tag.text"
    >{{ tag.symbol }}<template v-if="tag.current">{{ tag.text }}</template></span>
  </span>
</template>

<script setup lang="ts">
// 強弱模式下貼在四柱字下方的小標記：↑增力 ↓減力 ⇄化 ∞絆。目前這一步的寫出文字，其餘只留符號。
export interface StrengthCharTag {
  symbol: string;
  text: string;
  help: "same" | "opp" | null;
  light?: boolean;
  current: boolean;
}
export interface StrengthCharInfo {
  /** 這個地支是目前選的天干的根 */
  root?: { text: string; shaken: boolean } | null;
  party: "same" | "opp";
  dayMaster: boolean;
  focus: boolean;
  dim: boolean;
  toElement: string | null;
  tags: StrengthCharTag[];
}
defineProps<{ info: StrengthCharInfo | null }>();
</script>

<style scoped>
.sv-tags {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 2px;
  margin-top: 3px;
  min-height: 0;
}
.sv-tag {
  display: inline-block;
  padding: 0 5px;
  border-radius: 999px;
  font-size: calc(11px * var(--app-font-scale));
  line-height: 1.55;
  font-weight: 600;
  white-space: nowrap;
  background: #f1f3f5;
  color: #5c6773;
}
.sv-tag.is-short {
  padding: 0 4px;
}
.sv-tag.is-same {
  background: #fff1dc;
  color: #b4560a;
}
.sv-tag.is-opp {
  background: #eaf1f8;
  color: #2f5c86;
}
.sv-tag.is-light {
  opacity: 0.6;
}
.sv-tag-root {
  background: #2f9e44;
  color: #fff;
}
.sv-tag-root.is-shaken {
  background: #c53030;
}
.sv-tag-to {
  background: #fff;
  color: #6b4fa0;
  border: 1px dashed #9b84c9;
}
</style>
