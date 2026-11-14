<template>
  <span class="glossary-text">
    <template v-for="(piece, index) in pieces" :key="index">
      <el-popover v-if="piece.entry" trigger="click" placement="top" :width="280">
        <template #reference>
          <span class="glossary-term" role="button" tabindex="0">{{ piece.text }}</span>
        </template>
        <div class="glossary-pop">
          <strong>{{ piece.entry.term }}</strong>
          <p>{{ piece.entry.text }}</p>
          <small>出處：{{ piece.entry.source }}</small>
        </div>
      </el-popover>
      <template v-else>{{ piece.text }}</template>
    </template>
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { splitGlossary } from "../constants/glossary";

// 自動把文字裡的名詞加上虛線底線，點一下看白話解釋（名詞庫在 src/constants/glossary.ts）。
const props = defineProps<{ text: string }>();
const pieces = computed(() => splitGlossary(props.text ?? ""));
</script>

<style scoped>
.glossary-term {
  border-bottom: 1px dashed var(--bazi-accent);
  cursor: help;
}
.glossary-term:focus-visible {
  outline: 2px solid var(--bazi-ring);
}
.glossary-pop {
  display: grid;
  gap: 4px;
  font-size: calc(14px * var(--app-font-scale));
  line-height: 1.6;
}
.glossary-pop p {
  margin: 0;
}
.glossary-pop small {
  color: var(--bazi-text-muted);
}
</style>
