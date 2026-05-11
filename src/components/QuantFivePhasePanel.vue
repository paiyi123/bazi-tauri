<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>量化五行力（書表對照版）</h2>
      </div>
    </template>

    <el-alert
      type="info"
      :closable="false"
      show-icon
      :title="result.quantModel?.summary || '尚無量化結果'"
      class="mb-4"
    />

    <el-descriptions :column="compact ? 1 : 2" :size="compact ? 'small' : 'default'" border class="mb-4" v-if="result.quantModel">
      <el-descriptions-item label="天干總分">{{ result.quantModel.stemScoreTotal }}</el-descriptions-item>
      <el-descriptions-item label="地支總分">{{ result.quantModel.branchScoreTotal }}</el-descriptions-item>
      <el-descriptions-item label="命局總分">{{ result.quantModel.totalScore }}</el-descriptions-item>
      <el-descriptions-item label="判定">{{ result.quantModel.strengthLabel }}</el-descriptions-item>
      <el-descriptions-item label="說明" :span="compact ? 1 : 2">{{ result.quantModel.note }}</el-descriptions-item>
    </el-descriptions>

    <h3 class="sub-title">天干計分（書表列法）</h3>
    <QuantStemScoreTable :scores="result.quantModel?.stemScores || []" :compact="compact" />

    <h3 class="sub-title">地支計分（書表列法）</h3>
    <QuantBranchScoreList :scores="result.quantModel?.branchScores || []" :compact="compact" />

    <h3 class="sub-title" v-if="(result.quantModel?.luckScores || []).length">大運量化</h3>
    <LuckQuantTable
      v-if="(result.quantModel?.luckScores || []).length"
      :scores="result.quantModel?.luckScores || []"
      @preview-change="emit('preview-change', $event)"
    />

    <p class="hint">目前為 Rust 版兼容量化輸出，規則可再逐步向 Java 原版收斂。</p>
  </el-card>
</template>

<script setup lang="ts">
import LuckQuantTable from "./LuckQuantTable.vue";
import QuantBranchScoreList from "./QuantBranchScoreList.vue";
import QuantStemScoreTable from "./QuantStemScoreTable.vue";
import type { BaziResponse, LuckPreviewPillar } from "../types/bazi";

const emit = defineEmits<{
  "preview-change": [LuckPreviewPillar[] | null];
}>();

defineProps<{
  result: BaziResponse;
  compact?: boolean;
}>();
</script>

<style scoped>
.mb-4 {
  margin-bottom: 16px;
}

.sub-title {
  margin: 12px 0 8px;
  font-size: calc(16px * var(--app-font-scale));
}

.hint {
  margin-top: 12px;
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
}

@media (max-width: 768px) {
  :deep(.el-descriptions__label),
  :deep(.el-descriptions__content) {
    font-size: calc(12px * var(--app-font-scale));
    line-height: 1.45;
  }

  .sub-title {
    font-size: calc(14px * var(--app-font-scale));
  }
}
</style>
