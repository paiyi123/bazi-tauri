<template>
  <div :class="['hs-cols', { compact }]">
    <div
      v-for="(pair, idx) in pairs"
      :key="`${pair.stem}-${idx}`"
      :class="['hs-col', { 'is-root': !!rootOf(pair.stem), 'is-shaken': rootOf(pair.stem)?.shaken }]"
      :style="rootOf(pair.stem) ? { '--root-c': rootOf(pair.stem)!.color } : undefined"
      :data-sv-hidden="`${col}-${pair.stem}`"
      :title="`${pair.stem}（${pair.tenGod}）`"
    >
      <span :class="['hs-stem', 'wuxing-char', `wuxing-${STEM_WUXING[pair.stem] ?? 'earth'}`]">{{ pair.stem }}</span>
      <span class="hs-god">
        <span v-for="(ch, i) in Array.from(pair.tenGod)" :key="i">{{ ch }}</span>
      </span>
      <span v-if="rootOf(pair.stem)" class="hs-root">{{ compact ? rootOf(pair.stem)!.level.charAt(0) : `${rootOf(pair.stem)!.level}根` }}{{ rootOf(pair.stem)!.shaken ? (compact ? '✕' : '·沖') : '' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { STEM_WUXING } from "../constants/wuxing";

// 地支藏干畫成直的小欄：幾個藏干就幾欄，十神直寫在字下面（照老師黑板的畫法）。
// 選到某個天干看根時，它的根落在哪一欄，那一欄就用該天干的五行色框起來，根的連線也接到這一欄。

export interface HiddenRootMark {
  /** 藏干字 */
  stem: string;
  level: string;
  color: string;
  shaken: boolean;
}

const props = defineProps<{
  pairs: Array<{ stem: string; tenGod: string }>;
  col: number;
  roots?: HiddenRootMark[] | null;
  compact?: boolean;
}>();

function rootOf(stem: string) {
  return props.roots?.find((root) => root.stem === stem) ?? null;
}
</script>

<style scoped>
.hs-cols {
  display: flex;
  justify-content: center;
  align-items: stretch;
  gap: 6px;
  margin-top: 4px;
}
.hs-col {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 2.1em;
  padding: 4px 3px 6px;
  border: 1px solid var(--bazi-border, #e5dccf);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.7);
  transition: border-color 0.25s, background 0.25s, box-shadow 0.25s;
}
.hs-stem {
  font-size: calc(18px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.2;
}
.hs-god {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-top: 2px;
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.2;
  color: var(--el-text-color-regular, #555);
}
.hs-col.is-root {
  border: 2px solid var(--root-c);
  background: color-mix(in srgb, var(--root-c) 12%, #fff);
  box-shadow: 0 2px 6px color-mix(in srgb, var(--root-c) 25%, transparent);
}
.hs-col.is-root.is-shaken {
  border-style: dashed;
  background: #fff;
  box-shadow: none;
}
.hs-root {
  margin-top: 4px;
  padding: 0 4px;
  border-radius: 999px;
  background: var(--root-c);
  color: #fff;
  font-size: calc(10px * var(--app-font-scale));
  line-height: 1.5;
  font-weight: 700;
  white-space: nowrap;
}
.hs-col.is-shaken .hs-root {
  background: #fff;
  color: var(--root-c);
  box-shadow: inset 0 0 0 1px var(--root-c);
}
.compact.hs-cols {
  gap: 2px;
}
.compact.hs-cols {
  width: 100%;
  min-width: 0;
  max-width: 100%;
  gap: 1px;
}
.compact .hs-col {
  min-width: 0;
  flex: 1 1 0;
  max-width: 1.9em;
  padding: 3px 0 4px;
  padding: 3px 1px 4px;
  border-radius: 6px;
}
.compact .hs-stem {
  font-size: calc(13px * var(--app-font-scale));
}
.compact .hs-god {
  font-size: calc(10px * var(--app-font-scale));
}
.compact .hs-col.is-root {
  border-width: 1.5px;
}
.compact .hs-root {
  padding: 0 3px;
  font-size: calc(9px * var(--app-font-scale));
  white-space: normal;
  text-align: center;
}
</style>
