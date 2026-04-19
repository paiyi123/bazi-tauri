<template>
  <div class="app-shell">
    <header class="app-header">
      <div>
        <p class="eyebrow">Tauri + tyme4rs</p>
        <h1>八字排盤</h1>
        <p class="subtitle">
          參考 `bazi-sources` 的核心流程改寫為本機應用，現在同時可往 Windows 與 Android 版本延伸，並保留大運、流年、流月展示。
        </p>
      </div>
    </header>

    <main class="page-grid">
      <section class="left-column">
        <el-card shadow="never" class="mode-card">
          <el-segmented v-model="inputMode" :options="modeOptions" block />
        </el-card>
        <BirthForm
          v-if="inputMode === 'birth'"
          :loading="loading"
          :error="error"
          @submit="handleSubmit"
        />
        <PillarInputForm
          v-else
          :loading="loading"
          :error="error"
          @submit="handleSubmitPillars"
        />
      </section>

      <section class="right-column">
        <div v-if="result" class="result-stack">
          <SummaryPanel :result="result" />
          <PillarPanel :result="result" :preview-pillars="focusedLuckPillars" />
          <LuckCyclePanel :result="result" @preview-change="handlePreviewChange" />
          <FiveElementStrengthPanel :result="result" />
          <QuantFivePhasePanel :result="result" @preview-change="handlePreviewChange" />
          <TenGodPanel :result="result" />
        </div>

        <el-card v-else shadow="hover" class="empty-card">
          <el-empty description="左側輸入出生資料後，系統會以本機 Rust 引擎計算四柱、大運與起運資訊。">
            <template #image>
              <div class="empty-mark">命</div>
            </template>
          </el-empty>
        </el-card>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import BirthForm from "./components/BirthForm.vue";
import FiveElementStrengthPanel from "./components/FiveElementStrengthPanel.vue";
import LuckCyclePanel from "./components/LuckCyclePanel.vue";
import PillarPanel from "./components/PillarPanel.vue";
import PillarInputForm from "./components/PillarInputForm.vue";
import QuantFivePhasePanel from "./components/QuantFivePhasePanel.vue";
import SummaryPanel from "./components/SummaryPanel.vue";
import TenGodPanel from "./components/TenGodPanel.vue";
import { analyzePillars, calculateBazi } from "./services/bazi";
import type {
  BaziRequest,
  BaziResponse,
  LuckPreviewPillar,
  PillarAnalyzeRequest,
} from "./types/bazi";

const result = ref<BaziResponse | null>(null);
const loading = ref(false);
const error = ref("");
const inputMode = ref<"birth" | "pillars">("birth");
const focusedLuckPillars = ref<LuckPreviewPillar[]>([]);
const modeOptions = [
  { label: "出生資料", value: "birth" },
  { label: "直接輸入四柱", value: "pillars" },
];

watch(result, () => {
  focusedLuckPillars.value = [];
});

function handlePreviewChange(pillars: LuckPreviewPillar[] | null) {
  focusedLuckPillars.value = pillars ?? [];
}

async function handleSubmit(payload: BaziRequest) {
  loading.value = true;
  error.value = "";

  try {
    result.value = await calculateBazi(payload);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "排盤失敗";
  } finally {
    loading.value = false;
  }
}

async function handleSubmitPillars(payload: PillarAnalyzeRequest) {
  loading.value = true;
  error.value = "";

  try {
    result.value = await analyzePillars(payload);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "四柱分析失敗";
  } finally {
    loading.value = false;
  }
}
</script>
