<template>
  <el-table :data="scores" stripe class="mb-4" row-key="pillar" size="small">
    <el-table-column prop="pillar" label="天干" :width="compact ? 74 : 90" />
    <el-table-column prop="target" label="字" :width="compact ? 58 : 70" />
    <el-table-column prop="combineNote" label="合化" :width="compact ? 76 : 90">
      <template #default="{ row }">{{ row.combineNote || "-" }}</template>
    </el-table-column>
    <el-table-column prop="tenGod" label="十神" :width="compact ? 76 : 90" />
    <el-table-column prop="rawScore" label="本分" :width="compact ? 76 : 90" />
    <el-table-column prop="positionAdjustedScore" label="位調" :width="compact ? 76 : 90" />
    <el-table-column prop="clashAdjustment" label="沖合" :width="compact ? 76 : 90">
      <template #default="{ row }">{{ formatMaybe(row.clashAdjustment) }}</template>
    </el-table-column>
    <el-table-column prop="bonusScore" label="增值" :width="compact ? 76 : 90">
      <template #default="{ row }">{{ formatMaybe(row.bonusScore) }}</template>
    </el-table-column>
    <el-table-column prop="finalScore" label="總分" :width="compact ? 76 : 90" />
    <el-table-column label="明細" :min-width="compact ? 240 : 480">
      <template #default="{ row }">
        <div v-if="row.rows?.length" class="sub-rows detail-cell">
          <div v-for="(sub, idx) in row.rows" :key="idx" class="sub-row">
            {{ sub.interaction }} {{ formatMaybe(sub.adjustmentScore) }}
          </div>
        </div>
        <div v-else class="detail-cell">{{ row.details }}</div>
      </template>
    </el-table-column>
  </el-table>
</template>

<script setup lang="ts">
import type { QuantModelPillarScore } from "../types/bazi";

defineProps<{
  scores: QuantModelPillarScore[];
  compact?: boolean;
}>();

function formatMaybe(value?: number) {
  if (value === undefined || value === null) return "-";
  return value;
}
</script>

<style scoped>
.mb-4 {
  margin-bottom: 16px;
}

.sub-rows {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sub-row {
  font-size: 12px;
  color: var(--el-text-color-regular);
}

.detail-cell {
  white-space: normal;
  word-break: break-word;
  line-height: 1.5;
}

@media (max-width: 768px) {
  :deep(.el-table .cell) {
    font-size: 12px;
    line-height: 1.35;
  }
}
</style>
