<template>
  <el-table :data="scores" stripe class="mb-4" row-key="pillar">
    <el-table-column prop="pillar" label="天干" width="90" />
    <el-table-column prop="target" label="字" width="70" />
    <el-table-column prop="combineNote" label="合化" width="90">
      <template #default="{ row }">{{ row.combineNote || "-" }}</template>
    </el-table-column>
    <el-table-column prop="tenGod" label="十神" width="90" />
    <el-table-column prop="rawScore" label="本分" width="90" />
    <el-table-column prop="positionAdjustedScore" label="位調" width="90" />
    <el-table-column prop="clashAdjustment" label="沖合" width="90">
      <template #default="{ row }">{{ formatMaybe(row.clashAdjustment) }}</template>
    </el-table-column>
    <el-table-column prop="bonusScore" label="增值" width="90">
      <template #default="{ row }">{{ formatMaybe(row.bonusScore) }}</template>
    </el-table-column>
    <el-table-column prop="finalScore" label="總分" width="90" />
    <el-table-column label="明細" min-width="480">
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
</style>
