<template>
  <div class="app-shell" :class="{ 'compact-layout': isCompactLayout }" :style="appFontStyle">
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
        <el-card shadow="never" class="mode-card" @click="showInputPanel = true">
          <el-segmented v-model="inputMode" :options="modeOptions" block />
        </el-card>
        <el-card shadow="never" class="font-size-card">
          <div class="font-size-control">
            <span class="font-size-label">字級</span>
            <el-slider
              v-model="fontScale"
              :min="86"
              :max="118"
              :step="2"
              :show-tooltip="false"
              class="font-size-slider"
            />
            <span class="font-size-value">{{ fontScale }}%</span>
          </div>
        </el-card>
        <BirthForm
          v-if="showInputPanel && inputMode === 'birth'"
          :compact="isCompactLayout"
          :loading="loading"
          :error="error"
          :prefill-draft="prefillBirthDraft"
          @print-context="handlePrintContext"
          @submit="handleSubmit"
        />
        <PillarInputForm
          v-else-if="showInputPanel"
          :compact="isCompactLayout"
          :loading="loading"
          :error="error"
          @print-context="handlePrintContext"
          @submit="handleSubmitPillars"
          @apply-birth-draft="handleApplyBirthDraft"
        />
      </section>

      <section class="right-column">
        <div v-if="result" class="result-toolbar">
          <el-button type="primary" plain @click="handleOpenPrintPreview">列印命盤</el-button>
        </div>
        <div v-if="result && !isCompactLayout" class="result-stack">
          <SummaryPanel :result="result" :compact="false" />
          <PillarPanel :result="result" :preview-pillars="focusedLuckPillars" :compact="false" />
          <FiveElementFlowPanel :result="result" :compact="false" />
          <LuckCyclePanel :result="result" :compact="false" @preview-change="handlePreviewChange" />
          <FiveElementStrengthPanel :result="result" />
          <QuantFivePhasePanel :result="result" :compact="false" @preview-change="handlePreviewChange" />
          <TenGodPanel :result="result" />
        </div>

        <div v-else-if="result" class="compact-result-shell">
          <el-tabs v-model="activeCompactTab" class="compact-result-tabs">
            <el-tab-pane label="摘要" name="summary">
              <SummaryPanel :result="result" :compact="true" />
            </el-tab-pane>
            <el-tab-pane label="四柱" name="pillars" lazy>
              <PillarPanel
                v-if="activeCompactTab === 'pillars'"
                key="compact-pillars-panel"
                :result="result"
                :preview-pillars="focusedLuckPillars"
                :compact="true"
                :show-pillars="true"
                :show-relation-map="false"
                :show-interaction-section="false"
              />
            </el-tab-pane>
            <el-tab-pane label="運勢" name="luck" lazy>
              <div v-if="activeCompactTab === 'luck'" class="compact-section-stack">
                <LuckCyclePanel :result="result" :compact="true" @preview-change="handlePreviewChange" />
                <PillarPanel
                  key="compact-luck-relations-panel"
                  :result="result"
                  :preview-pillars="focusedLuckPillars"
                  :compact="true"
                  :show-pillars="false"
                  :show-relation-map="true"
                  :show-interaction-section="true"
                />
              </div>
            </el-tab-pane>
            <el-tab-pane label="流通" name="flow" lazy>
              <FiveElementFlowPanel v-if="activeCompactTab === 'flow'" :result="result" :compact="true" />
            </el-tab-pane>
            <el-tab-pane label="量化" name="quant" lazy>
              <QuantFivePhasePanel :result="result" :compact="true" @preview-change="handlePreviewChange" />
            </el-tab-pane>
            <el-tab-pane label="分析" name="analysis" lazy>
              <div class="compact-section-stack">
                <FiveElementStrengthPanel :result="result" />
                <TenGodPanel :result="result" />
              </div>
            </el-tab-pane>
          </el-tabs>
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

    <el-dialog
      v-model="showPrintPreview"
      title="列印預覽"
      width="1120px"
      top="2vh"
      class="print-preview-dialog"
      destroy-on-close
    >
      <div class="print-preview-shell">
        <iframe
          v-if="printHtml"
          ref="printFrame"
          class="print-preview-frame"
          :srcdoc="printHtml"
          title="列印預覽"
        />
        <el-empty v-else description="列印預覽載入中" />
      </div>
      <template #footer>
        <el-button @click="showPrintPreview = false">關閉</el-button>
        <el-button type="primary" :disabled="!printHtml" @click="handlePrintFromPreview">確認列印</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import FiveElementFlowPanel from "./components/FiveElementFlowPanel.vue";
import BirthForm from "./components/BirthForm.vue";
import FiveElementStrengthPanel from "./components/FiveElementStrengthPanel.vue";
import LuckCyclePanel from "./components/LuckCyclePanel.vue";
import PillarPanel from "./components/PillarPanel.vue";
import PillarInputForm from "./components/PillarInputForm.vue";
import QuantFivePhasePanel from "./components/QuantFivePhasePanel.vue";
import SummaryPanel from "./components/SummaryPanel.vue";
import TenGodPanel from "./components/TenGodPanel.vue";
import { analyzePillars, calculateBazi, renderPrintHtml } from "./services/bazi";
import type {
  BaziRequest,
  BaziResponse,
  LuckPreviewPillar,
  PillarAnalyzeRequest,
  PrintContext,
} from "./types/bazi";

const result = ref<BaziResponse | null>(null);
const loading = ref(false);
const error = ref("");
const inputMode = ref<"birth" | "pillars">("birth");
const focusedLuckPillars = ref<LuckPreviewPillar[]>([]);
const isCompactLayout = ref(false);
const showInputPanel = ref(true);
const FONT_SCALE_STORAGE_KEY = "bazi:font-scale:v1";
const fontScale = ref(loadFontScale());
const activeCompactTab = ref<"summary" | "pillars" | "luck" | "flow" | "quant" | "analysis">("summary");
const prefillBirthDraft = ref<BaziRequest | null>(null);
const printContext = ref<PrintContext | null>(null);
const showPrintPreview = ref(false);
const printHtml = ref<string | null>(null);
const printFrame = ref<HTMLIFrameElement | null>(null);
const modeOptions = [
  { label: "出生資料", value: "birth" },
  { label: "直接輸入四柱", value: "pillars" },
];
const appFontStyle = computed(() => {
  const scale = fontScale.value / 100;
  return {
    "--app-font-scale": String(scale),
    "--app-font-size-base": `${16 * scale}px`,
    "--el-font-size-extra-large": `${20 * scale}px`,
    "--el-font-size-large": `${18 * scale}px`,
    "--el-font-size-medium": `${16 * scale}px`,
    "--el-font-size-base": `${14 * scale}px`,
    "--el-font-size-small": `${13 * scale}px`,
    "--el-font-size-extra-small": `${12 * scale}px`,
  };
});
watch(fontScale, (value) => {
  window.localStorage.setItem(FONT_SCALE_STORAGE_KEY, String(value));
});

watch(result, () => {
  focusedLuckPillars.value = [];
  activeCompactTab.value = "summary";
});

let compactMediaQuery: MediaQueryList | null = null;
type LegacyMediaQueryList = MediaQueryList & {
  addListener?: (listener: (event: MediaQueryListEvent) => void) => void;
  removeListener?: (listener: (event: MediaQueryListEvent) => void) => void;
};

function loadFontScale() {
  if (typeof window === "undefined") {
    return 100;
  }
  const raw = Number(window.localStorage.getItem(FONT_SCALE_STORAGE_KEY));
  if (!Number.isFinite(raw)) {
    return 100;
  }
  return Math.min(118, Math.max(86, Math.round(raw / 2) * 2));
}

function syncCompactLayout() {
  isCompactLayout.value =
    compactMediaQuery?.matches ??
    (typeof window !== "undefined" ? window.innerWidth <= 768 : false);
  if (!isCompactLayout.value) {
    showInputPanel.value = true;
  }
}

function handleCompactLayoutChange(event: MediaQueryListEvent) {
  isCompactLayout.value = event.matches;
  if (!event.matches) {
    showInputPanel.value = true;
  }
}

onMounted(() => {
  compactMediaQuery = window.matchMedia("(max-width: 768px)");
  syncCompactLayout();
  const mediaQuery = compactMediaQuery as LegacyMediaQueryList;

  if (typeof mediaQuery.addEventListener === "function") {
    mediaQuery.addEventListener("change", handleCompactLayoutChange);
  } else {
    mediaQuery.addListener?.(handleCompactLayoutChange);
  }
});

onBeforeUnmount(() => {
  if (!compactMediaQuery) {
    return;
  }
  const mediaQuery = compactMediaQuery as LegacyMediaQueryList;

  if (typeof mediaQuery.removeEventListener === "function") {
    mediaQuery.removeEventListener("change", handleCompactLayoutChange);
  } else {
    mediaQuery.removeListener?.(handleCompactLayoutChange);
  }
});

function handlePreviewChange(pillars: LuckPreviewPillar[] | null) {
  focusedLuckPillars.value = pillars ?? [];
}

function handlePrintContext(payload: PrintContext) {
  printContext.value = payload;
}

async function handleOpenPrintPreview() {
  if (!result.value) {
    return;
  }

  showPrintPreview.value = true;
  printHtml.value = null;
  error.value = "";

  try {
    printHtml.value = await renderPrintHtml(result.value, printContext.value);
  } catch (err) {
    error.value = err instanceof Error ? err.message : "載入列印預覽失敗";
  }
}

function handlePrintFromPreview() {
  const frameWindow = printFrame.value?.contentWindow;
  if (!frameWindow) {
    return;
  }

  frameWindow.focus();
  frameWindow.print();
}

async function handleSubmit(payload: BaziRequest) {
  loading.value = true;
  error.value = "";

  try {
    result.value = await calculateBazi(payload);
    showInputPanel.value = false;
    await focusCompactResultTabs();
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
    showInputPanel.value = false;
    await focusCompactResultTabs();
  } catch (err) {
    error.value = err instanceof Error ? err.message : "四柱分析失敗";
  } finally {
    loading.value = false;
  }
}

function handleApplyBirthDraft(payload: BaziRequest) {
  prefillBirthDraft.value = { ...payload };
  inputMode.value = "birth";
  showInputPanel.value = true;
}

async function focusCompactResultTabs() {
  await nextTick();
  requestAnimationFrame(() => {
    window.scrollTo({
      top: 0,
      behavior: "smooth",
    });
  });
}
</script>
