<template>
  <div class="app-shell" :class="{ 'compact-layout': isCompactLayout, 'has-bottom-tabs': isCompactLayout && !!result }" :style="appFontStyle">
    <header class="app-toolbar">
      <h1>八字排盤</h1>
      <div class="toolbar-actions">
        <el-segmented v-model="viewMode" :options="viewModeOptions" size="small" class="view-mode-switch" />
        <el-popover v-if="!isCompactLayout" trigger="click" placement="bottom-end" :width="300">
          <template #reference>
            <el-button size="small">設定</el-button>
          </template>
          <div class="settings-body">
            <div class="font-size-control">
              <span class="font-size-label">字級</span>
              <el-slider v-model="fontScale" :min="86" :max="200" :step="2" :show-tooltip="false" class="font-size-slider" />
              <span class="font-size-value">{{ fontScale }}%</span>
            </div>
            <p class="settings-note">新手模式：只看課堂三步法的主答案與重點。研究模式：多顯示量化計分、各書原始卡片、胎元命宮、神煞與分數。</p>
          </div>
        </el-popover>
        <el-button v-if="result && !isCompactLayout" size="small" type="primary" plain @click="handleOpenPrintPreview">列印</el-button>
      </div>
    </header>

    <main class="page-grid" :class="{ 'is-two-column': useSideColumn }">
      <section class="left-column">
        <div v-if="result && !showInputPanel" class="input-summary">
          <span>{{ result.solarDateTime }}<span class="input-summary-sep">·</span>{{ genderText }}</span>
          <el-button link type="primary" @click="showInputPanel = true">修改／新排盤</el-button>
        </div>
        <div v-if="useSideColumn && result" class="side-sticky">
          <ConclusionCard :result="result" />
          <TopicSideNav />
        </div>
        <template v-if="!result || showInputPanel">
          <div v-if="result" class="input-summary">
            <span>修改出生資料，或排一張新的命盤</span>
            <el-button link type="primary" @click="showInputPanel = false">收起</el-button>
          </div>
          <el-card shadow="never" class="mode-card">
            <el-segmented v-model="inputMode" :options="modeOptions" block />
          </el-card>
          <BirthForm
            v-if="inputMode === 'birth'"
            :compact="isCompactLayout"
            :loading="loading"
            :error="error"
            :prefill-draft="prefillBirthDraft"
            @print-context="handlePrintContext"
            @submit="handleSubmit"
          />
          <PillarInputForm
            v-else
            :compact="isCompactLayout"
            :loading="loading"
            :error="error"
            @print-context="handlePrintContext"
            @submit="handleSubmitPillars"
            @apply-birth-draft="handleApplyBirthDraft"
          />
        </template>
      </section>

      <section class="right-column">
        <div v-if="result && !isCompactLayout" class="result-stack">
          <ConclusionCard v-if="!useSideColumn" :result="result" />
          <PillarPanel :result="result" :preview-pillars="focusedLuckPillars" :compact="false" :show-relation-map="false" :show-interaction-section="isResearch" @preview-change="handlePreviewChange" />
          <SourceFold :open="isResearch" label="命盤摘要・五行流通圖" hint="農曆、起運時間、八個字之間的生剋流通">
            <SummaryPanel :result="result" :compact="false" />
            <FiveElementFlowPanel :result="result" :compact="false" />
          </SourceFold>
          <AnalysisTopics :result="result" :compact="false" :side-nav="useSideColumn" @preview-change="handlePreviewChange" />
        </div>

        <div v-else-if="result" class="compact-result-shell">
          <MiniChartBar v-if="activeCompactTab !== 'chart'" :result="result" class="mini-chart-sticky" />
          <div v-if="activeCompactTab === 'chart'" class="compact-section-stack">
            <ConclusionCard :result="result" compact />
            <PillarPanel
              key="compact-pillars-panel"
              :result="result"
              @preview-change="handlePreviewChange"
              :preview-pillars="focusedLuckPillars"
              :compact="true"
              :show-pillars="true"
              :show-relation-map="false"
              :show-interaction-section="false"
            />
            <PillarSummaryPanel :result="result" hide-conclusions />
            <SourceFold :open="isResearch" label="命盤摘要・五行流通圖" hint="農曆、起運時間、生剋流通">
              <FiveElementFlowPanel :result="result" :compact="true" />
              <SummaryPanel :result="result" :compact="true" />
            </SourceFold>
          </div>
          <div v-else-if="activeCompactTab === 'analysis'" class="compact-section-stack">
            <AnalysisTopics
              :result="result"
              :compact="true"
              :exclude="['luck']"
              @preview-change="handlePreviewChange"
              @goto-outside="selectCompactTab('luck')"
            />
          </div>
          <div v-else-if="activeCompactTab === 'luck'" class="compact-section-stack">
            <LuckCyclePanel :result="result" :compact="true" @preview-change="handlePreviewChange" />
            <FourBookTopicsPanel :result="result" :only="['luck']" title="大運流年要訣（四本書）" />
            <FourBooksPanel v-if="isResearch" :result="result" :only="['limit']" title="限運：每柱管哪段年齡（四本書）" />
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
          <div v-else class="compact-section-stack">
            <el-card shadow="never" class="more-card">
              <template #header>
                <div class="section-header"><h2>設定</h2></div>
              </template>
              <div class="more-row">
                <span class="more-label">顯示模式</span>
                <el-segmented v-model="viewMode" :options="viewModeOptions" />
              </div>
              <p class="settings-note">新手模式：只看課堂三步法的主答案與重點。研究模式：多顯示量化計分、各書原始卡片、胎元命宮、神煞與分數。</p>
              <div class="font-size-control">
                <span class="font-size-label">字級</span>
                <el-slider v-model="fontScale" :min="86" :max="200" :step="2" :show-tooltip="false" class="font-size-slider" />
                <span class="font-size-value">{{ fontScale }}%</span>
              </div>
              <div class="more-actions">
                <el-button @click="openInputPanel">修改／新排盤</el-button>
                <el-button type="primary" plain @click="handleOpenPrintPreview">列印命盤</el-button>
              </div>
            </el-card>
            <QuantFivePhasePanel v-if="isResearch" :result="result" :compact="true" @preview-change="handlePreviewChange" />
            <el-card v-else shadow="never" class="more-card">
              <p class="settings-note">量化五行力的計分表是研究資料，切到「研究」模式就會出現在這裡。</p>
            </el-card>
          </div>
        </div>

        <el-card v-else shadow="hover" class="empty-card">
          <el-empty description="輸入出生資料後，系統會以本機 Rust 引擎計算四柱、大運與起運資訊。">
            <template #image>
              <div class="empty-mark">命</div>
            </template>
          </el-empty>
        </el-card>
      </section>
    </main>

    <nav v-if="result && isCompactLayout" class="bottom-tabs" aria-label="結果分頁">
      <button
        v-for="tab in compactTabs"
        :key="tab.name"
        type="button"
        class="bottom-tab"
        :class="{ active: activeCompactTab === tab.name }"
        @click="selectCompactTab(tab.name)"
      >
        <span class="bottom-tab-icon">{{ tab.icon }}</span>
        <span>{{ tab.label }}</span>
      </button>
    </nav>

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
import AnalysisTopics from "./components/AnalysisTopics.vue";
import FiveElementFlowPanel from "./components/FiveElementFlowPanel.vue";
import BirthForm from "./components/BirthForm.vue";
import ConclusionCard from "./components/ConclusionCard.vue";
import MiniChartBar from "./components/MiniChartBar.vue";
import { useViewMode, type ViewMode } from "./composables/useViewMode";
import FourBooksPanel from "./components/FourBooksPanel.vue";
import FourBookTopicsPanel from "./components/FourBookTopicsPanel.vue";
import LuckCyclePanel from "./components/LuckCyclePanel.vue";
import PillarPanel from "./components/PillarPanel.vue";
import PillarSummaryPanel from "./components/PillarSummaryPanel.vue";
import PillarInputForm from "./components/PillarInputForm.vue";
import QuantFivePhasePanel from "./components/QuantFivePhasePanel.vue";
import SummaryPanel from "./components/SummaryPanel.vue";
import SourceFold from "./components/SourceFold.vue";
import TopicSideNav from "./components/TopicSideNav.vue";
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
type CompactTab = "chart" | "analysis" | "luck" | "more";
const activeCompactTab = ref<CompactTab>("chart");
const compactTabs: { name: CompactTab; label: string; icon: string }[] = [
  { name: "chart", label: "命盤", icon: "命" },
  { name: "analysis", label: "解盤", icon: "解" },
  { name: "luck", label: "運勢", icon: "運" },
  { name: "more", label: "更多", icon: "⋯" },
];
const { mode: viewMode, isResearch } = useViewMode();
// 電腦版排盤後：左欄固定結論卡，右欄放命盤與分析；打開輸入表單時回到單欄。
const useSideColumn = computed(() => !!result.value && !isCompactLayout.value && !showInputPanel.value);
const viewModeOptions: { label: string; value: ViewMode }[] = [
  { label: "新手", value: "beginner" },
  { label: "研究", value: "research" },
];
const genderText = computed(() =>
  result.value?.gender === "FEMALE" ? "女" : result.value?.gender === "MALE" ? "男" : "性別未填",
);
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
  activeCompactTab.value = "chart";
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
  const stored = window.localStorage.getItem(FONT_SCALE_STORAGE_KEY);
  const raw = Number(stored);
  // 沒存過時 Number(null) 會是 0，被夾成最小的 86%；改成預設 100%。
  if (stored === null || !Number.isFinite(raw) || raw <= 0) {
    return 100;
  }
  return Math.min(200, Math.max(86, Math.round(raw / 2) * 2));
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

function selectCompactTab(name: CompactTab) {
  activeCompactTab.value = name;
  window.scrollTo({ top: 0 });
}

function openInputPanel() {
  showInputPanel.value = true;
  window.scrollTo({ top: 0, behavior: "smooth" });
}

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
