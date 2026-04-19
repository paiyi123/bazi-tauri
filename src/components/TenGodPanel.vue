<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>合沖刑破分析</h2>
      </div>
    </template>

    <el-table v-if="rows.length > 0" :data="rows" stripe>
      <el-table-column prop="scope" label="部位" width="90" />
      <el-table-column prop="type" label="類型" width="90" />
      <el-table-column prop="target" label="組合" width="120" />
      <el-table-column prop="outcome" label="合化結果" width="120" />
      <el-table-column prop="pillars" label="對應柱位" min-width="200" />
      <el-table-column prop="detail" label="說明" min-width="220" />
    </el-table>
    <el-empty v-else description="四柱中未檢出明顯的合、沖、刑、破組合。" />
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";

const props = defineProps<{
  result: BaziResponse;
}>();

const rows = computed(() => props.result.quantModel?.interactions || []);
</script>
