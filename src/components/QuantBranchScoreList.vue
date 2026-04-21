<template>
  <div v-for="branch in displayBranchScores" :key="branch.pillar" class="branch-block">
    <div class="branch-title">
      <span>{{ displayBranchPillar(branch.pillar) }} {{ branch.target }}｜總分 {{ branch.finalScore }}</span>
      <el-tag v-if="branch.combineNote" size="small" type="warning" effect="light" class="branch-combine-tag">
        合化：{{ branch.combineNote }}
      </el-tag>
    </div>
    <el-table :data="branch.rows || []" stripe size="small" class="mb-3">
      <el-table-column prop="hiddenStem" label="藏干" :width="compact ? 68 : 80" />
      <el-table-column prop="tenGod" label="十神" :width="compact ? 76 : 90" />
      <el-table-column prop="ratio" label="比例" :width="compact ? 76 : 90">
        <template #default="{ row }">{{ formatRatio(row.ratio) }}</template>
      </el-table-column>
      <el-table-column prop="rawScore" label="本分" :width="compact ? 76 : 90" />
      <el-table-column prop="positionAdjustedScore" label="位調" :width="compact ? 76 : 90" />
      <el-table-column prop="interaction" label="互動" :width="compact ? 76 : 90" />
      <el-table-column prop="adjustmentScore" label="增減" :width="compact ? 76 : 90">
        <template #default="{ row }">{{ formatMaybe(row.adjustmentScore) }}</template>
      </el-table-column>
      <el-table-column prop="finalContribution" label="列分" :width="compact ? 76 : 90" />
      <el-table-column prop="note" label="說明" :min-width="compact ? 180 : 260" />
    </el-table>
    <div v-if="compact" class="branch-summary-row mb-4">
      <div class="branch-summary-item">
        <span class="branch-summary-label">合化</span>
        <span class="branch-summary-value">{{ branch.combineNote || "-" }}</span>
      </div>
      <div class="branch-summary-item">
        <span class="branch-summary-label">支基礎分</span>
        <span class="branch-summary-value">{{ branch.baseScore }}</span>
      </div>
      <div class="branch-summary-item">
        <span class="branch-summary-label">位調後</span>
        <span class="branch-summary-value">{{ formatMaybe(branch.positionAdjustedScore) }}</span>
      </div>
      <div class="branch-summary-item">
        <span class="branch-summary-label">透干/增值</span>
        <span class="branch-summary-value">{{ formatMaybe(branch.bonusScore) }}</span>
      </div>
      <div class="branch-summary-item">
        <span class="branch-summary-label">總分</span>
        <span class="branch-summary-value">{{ branch.finalScore }}</span>
      </div>
    </div>
    <el-descriptions v-else :column="5" border size="small" class="mb-4">
      <el-descriptions-item label="合化">{{ branch.combineNote || "-" }}</el-descriptions-item>
      <el-descriptions-item label="支基礎分">{{ branch.baseScore }}</el-descriptions-item>
      <el-descriptions-item label="位調後">{{ formatMaybe(branch.positionAdjustedScore) }}</el-descriptions-item>
      <el-descriptions-item label="透干/增值">{{ formatMaybe(branch.bonusScore) }}</el-descriptions-item>
      <el-descriptions-item label="總分">{{ branch.finalScore }}</el-descriptions-item>
    </el-descriptions>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { QuantModelPillarScore } from "../types/bazi";

const props = defineProps<{
  scores: QuantModelPillarScore[];
  compact?: boolean;
}>();

const displayBranchScores = computed(() => {
  const order = ["月支", "年支", "日支", "時支"];
  return [...props.scores].sort((a, b) => order.indexOf(a.pillar) - order.indexOf(b.pillar));
});

function displayBranchPillar(pillar: string) {
  return pillar === "月支" ? "月令" : pillar;
}

function formatMaybe(value?: number) {
  if (value === undefined || value === null) return "-";
  return value;
}

function formatRatio(value?: number) {
  if (value === undefined || value === null) return "-";
  return value.toFixed(2);
}
</script>

<style scoped>
.mb-3 {
  margin-bottom: 12px;
}

.mb-4 {
  margin-bottom: 16px;
}

.branch-block {
  margin-bottom: 20px;
}

.branch-title {
  font-weight: 700;
  margin: 8px 0;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.branch-combine-tag {
  font-weight: 500;
}

.branch-summary-row {
  display: grid;
  grid-template-columns: repeat(5, minmax(72px, 1fr));
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 10px;
  overflow: hidden;
  background: #fff;
}

.branch-summary-item {
  min-width: 0;
  padding: 8px 6px;
  border-right: 1px solid var(--el-border-color-lighter);
  display: grid;
  gap: 4px;
}

.branch-summary-item:last-child {
  border-right: none;
}

.branch-summary-label {
  color: var(--el-text-color-secondary);
  font-size: 11px;
  line-height: 1.2;
  white-space: nowrap;
}

.branch-summary-value {
  font-size: 12px;
  font-weight: 700;
  line-height: 1.25;
  word-break: break-word;
}

@media (max-width: 768px) {
  .branch-title {
    font-size: 13px;
    line-height: 1.45;
  }

  :deep(.el-table .cell),
  :deep(.el-descriptions__label),
  :deep(.el-descriptions__content) {
    font-size: 12px;
    line-height: 1.4;
  }

  .branch-summary-row {
    grid-template-columns: repeat(5, minmax(66px, 1fr));
  }

  .branch-summary-item {
    padding: 7px 4px;
  }

  .branch-summary-label {
    font-size: 10px;
  }

  .branch-summary-value {
    font-size: 11px;
  }
}
</style>
