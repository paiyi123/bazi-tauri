<template>
  <div v-for="branch in displayBranchScores" :key="branch.pillar" class="branch-block">
    <div class="branch-title">
      <span>{{ displayBranchPillar(branch.pillar) }} {{ branch.target }}｜總分 {{ branch.finalScore }}</span>
      <el-tag v-if="branch.combineNote" size="small" type="warning" effect="light" class="branch-combine-tag">
        合化：{{ branch.combineNote }}
      </el-tag>
    </div>
    <el-table :data="branch.rows || []" stripe size="small" class="mb-3">
      <el-table-column prop="hiddenStem" label="藏干" width="80" />
      <el-table-column prop="tenGod" label="十神" width="90" />
      <el-table-column prop="ratio" label="比例" width="90">
        <template #default="{ row }">{{ formatRatio(row.ratio) }}</template>
      </el-table-column>
      <el-table-column prop="rawScore" label="本分" width="90" />
      <el-table-column prop="positionAdjustedScore" label="位調" width="90" />
      <el-table-column prop="interaction" label="互動" width="90" />
      <el-table-column prop="adjustmentScore" label="增減" width="90">
        <template #default="{ row }">{{ formatMaybe(row.adjustmentScore) }}</template>
      </el-table-column>
      <el-table-column prop="finalContribution" label="列分" width="90" />
      <el-table-column prop="note" label="說明" min-width="260" />
    </el-table>
    <el-descriptions :column="5" border size="small" class="mb-4">
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
</style>
