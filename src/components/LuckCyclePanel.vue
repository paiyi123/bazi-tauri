<template>
  <el-card shadow="hover" :class="['luck-panel', { 'is-compact': compact }]">
    <template #header>
      <div class="section-header">
        <div>
          <h2>大運</h2>
          <p class="luck-start-line">{{ luckStartLine }}</p>
        </div>
      </div>
    </template>

    <div class="compact-lane-toggle-bar">
      <button
        type="button"
        :class="['compact-lane-toggle', { 'is-active': expandAnnualLane }]"
        @click="toggleAnnualLane()"
      >
        {{ expandAnnualLane ? "收合流年" : "展開流年" }}
      </button>
      <button
        type="button"
        :class="['compact-lane-toggle', { 'is-active': expandMonthLane }]"
        @click="toggleMonthLane()"
      >
        {{ expandMonthLane ? "收合流月" : "展開流月" }}
      </button>
    </div>

    <div class="luck-board">
      <div class="luck-lane">
        <div class="lane-label">大<br />運</div>
        <el-scrollbar class="lane-scrollbar" always>
          <div class="lane-track">
            <button
              v-for="row in rows"
              :key="`luck-${row.index}`"
              type="button"
              :class="[
                'fate-cell',
                'fate-cell-major',
                {
                  'is-selected': row.index === selectedLuckIndex,
                  'is-current': isCurrentLuckRow(row),
                },
              ]"
              @click="selectLuckRow(row)"
            >
              <div class="fate-cell-year">{{ formatLuckRangeLabel(row) }}</div>
              <div class="fate-cell-main">
                <div class="fate-char-row">
                  <span :class="['wuxing-char', `wuxing-${stemElement(ganZhiStem(row.ganZhi))}`]">
                    {{ ganZhiStem(row.ganZhi) }}
                  </span>
                  <span class="ten-god-text">{{ row.stemTenGod || "-" }}</span>
                </div>
                <div class="fate-char-row">
                  <span :class="['wuxing-char', `wuxing-${branchElement(ganZhiBranch(row.ganZhi))}`]">
                    {{ ganZhiBranch(row.ganZhi) }}
                  </span>
                  <span class="ten-god-text">{{ primaryBranchTenGod(row.branchTenGods) }}</span>
                </div>
              </div>
              <div v-if="displayCellHiddenTenGodPairs(row.branchHiddenStems, row.branchTenGods).length" class="fate-hidden-list">
                <span
                  v-for="(item, index) in displayCellHiddenTenGodPairs(row.branchHiddenStems, row.branchTenGods)"
                  :key="`luck-hidden-${row.index}-${item.stem}-${index}`"
                  class="fate-hidden-item"
                >
                  <span :class="['wuxing-char', `wuxing-${stemElement(item.stem)}`]">{{ item.stem }}</span>
                  <span class="hidden-ten-god-text">{{ item.tenGod }}</span>
                </span>
              </div>
            </button>
          </div>
        </el-scrollbar>
      </div>

      <div v-if="showAnnualLane" class="luck-lane">
        <div class="lane-label">流<br />年</div>
        <el-scrollbar v-if="selectedLuckRow?.liuNian?.length" class="lane-scrollbar" always>
          <div class="lane-track">
            <button
              v-for="annual in selectedLuckRow.liuNian"
              :key="`annual-${selectedLuckRow.index}-${annual.year}`"
              type="button"
              :class="[
                'fate-cell',
                'fate-cell-annual',
                {
                  'is-selected': annual.year === selectedAnnualYear,
                  'is-current': isCurrentAnnualRow(annual),
                },
              ]"
              @click="selectAnnualRow(annual)"
            >
              <div class="fate-cell-year">{{ annual.year }}</div>
              <div class="fate-cell-main">
                <div class="fate-char-row">
                  <span :class="['wuxing-char', `wuxing-${stemElement(ganZhiStem(annual.ganZhi))}`]">
                    {{ ganZhiStem(annual.ganZhi) }}
                  </span>
                  <span class="ten-god-text">{{ annual.stemTenGod || "-" }}</span>
                </div>
                <div class="fate-char-row">
                  <span :class="['wuxing-char', `wuxing-${branchElement(ganZhiBranch(annual.ganZhi))}`]">
                    {{ ganZhiBranch(annual.ganZhi) }}
                  </span>
                  <span class="ten-god-text">{{ primaryBranchTenGod(annual.branchTenGods) }}</span>
                </div>
              </div>
              <div
                v-if="displayCellHiddenTenGodPairs(annual.branchHiddenStems, annual.branchTenGods).length"
                class="fate-hidden-list"
              >
                <span
                  v-for="(item, index) in displayCellHiddenTenGodPairs(annual.branchHiddenStems, annual.branchTenGods)"
                  :key="`annual-hidden-${selectedLuckRow.index}-${annual.year}-${item.stem}-${index}`"
                  class="fate-hidden-item"
                >
                  <span :class="['wuxing-char', `wuxing-${stemElement(item.stem)}`]">{{ item.stem }}</span>
                  <span class="hidden-ten-god-text">{{ item.tenGod }}</span>
                </span>
              </div>
              <div class="fate-cell-meta">{{ annual.age }}歲</div>
            </button>
          </div>
        </el-scrollbar>
        <div v-else class="lane-empty">這一步大運尚無流年資料</div>
      </div>

      <div v-if="showMonthLane" class="luck-lane">
        <div class="lane-label">流<br />月</div>
        <el-scrollbar v-if="selectedAnnualRow?.liuYue?.length" class="lane-scrollbar" always>
          <div class="lane-track">
            <button
              v-for="month in selectedAnnualRow.liuYue"
              :key="`month-${selectedAnnualRow.year}-${month.index}`"
              type="button"
              :class="[
                'fate-cell',
                'fate-cell-month',
                { 'is-selected': month.index === selectedMonthIndex },
              ]"
              @click="selectMonthRow(month)"
            >
              <div class="fate-cell-topline">
                <span class="fate-cell-term">{{ month.jieQi || "-" }}</span>
                <span class="fate-cell-term-date">{{ month.jieQiDate || "-" }}</span>
              </div>
              <div class="fate-cell-year">{{ formatLiuYueLabel(month.month) }}</div>
              <div class="fate-cell-main">
                <div class="fate-char-row">
                  <span :class="['wuxing-char', `wuxing-${stemElement(ganZhiStem(month.ganZhi))}`]">
                    {{ ganZhiStem(month.ganZhi) }}
                  </span>
                  <span class="ten-god-text">{{ month.stemTenGod || "-" }}</span>
                </div>
                <div class="fate-char-row">
                  <span :class="['wuxing-char', `wuxing-${branchElement(ganZhiBranch(month.ganZhi))}`]">
                    {{ ganZhiBranch(month.ganZhi) }}
                  </span>
                  <span class="ten-god-text">{{ primaryBranchTenGod(month.branchTenGods) }}</span>
                </div>
              </div>
              <div
                v-if="displayCellHiddenTenGodPairs(month.branchHiddenStems, month.branchTenGods).length"
                class="fate-hidden-list fate-hidden-list-month"
              >
                <span
                  v-for="(item, index) in displayCellHiddenTenGodPairs(month.branchHiddenStems, month.branchTenGods)"
                  :key="`month-hidden-${selectedAnnualRow.year}-${month.index}-${item.stem}-${index}`"
                  class="fate-hidden-item"
                >
                  <span :class="['wuxing-char', `wuxing-${stemElement(item.stem)}`]">{{ item.stem }}</span>
                  <span class="hidden-ten-god-text">{{ item.tenGod }}</span>
                </span>
              </div>
              <div class="fate-cell-meta">{{ month.xunKong ? `旬空 ${month.xunKong}` : month.xun || "-" }}</div>
            </button>
          </div>
        </el-scrollbar>
        <div v-else class="lane-empty">這一年尚無流月資料</div>
      </div>
    </div>

    <div v-if="selectedLuckRow" class="luck-detail-bar">
      <span>目前大運：{{ selectedLuckRow.ganZhi }}（{{ selectedLuckRow.ageRange }}歲 / {{ selectedLuckRow.yearRange }}）</span>
      <span v-if="showAnnualLane && selectedAnnualRow">目前流年：{{ selectedAnnualRow.year }} {{ selectedAnnualRow.ganZhi }}（{{ selectedAnnualRow.age }}歲）</span>
      <span v-if="showMonthLane && selectedMonthRow">目前流月：{{ formatLiuYueLabel(selectedMonthRow.month) }} {{ selectedMonthRow.ganZhi }}</span>
    </div>

    <div class="interaction-matrix-shell">
      <div class="interaction-matrix" :style="interactionMatrixStyle">
        <div class="interaction-summary-stack">
          <div v-if="stemSummaryStrips.length" class="interaction-summary-group">
            <div class="interaction-summary-grid" :style="interactionGridStyle">
              <div
                v-for="item in stemSummaryStrips"
                :key="item.key"
                :class="['interaction-summary-strip', item.tone]"
                :style="summaryStripStyle(item)"
              >
                {{ item.label }}: {{ item.value }}
              </div>
            </div>
          </div>
        </div>

        <div class="interaction-pillar-grid" :style="interactionGridStyle">
          <div v-for="item in interactionColumns" :key="item.key" class="interaction-pillar-card">
            <div class="interaction-pillar-label">{{ item.label }}</div>
            <div class="interaction-pillar-main">
              <span :class="['wuxing-char', `wuxing-${stemElement(item.stem)}`]">{{ item.stem }}</span>
              <span class="ten-god-text">{{ item.stemTenGod || "-" }}</span>
            </div>
            <div class="interaction-pillar-main">
              <span :class="['wuxing-char', `wuxing-${branchElement(item.branch)}`]">{{ item.branch }}</span>
              <span class="ten-god-text">{{ primaryBranchTenGod(item.branchTenGods) }}</span>
            </div>
            <div
              v-if="hiddenTenGodPairs(item.branchHiddenStems, item.branchTenGods).length"
              class="interaction-pillar-hidden-list"
            >
              <span
                v-for="(pair, index) in hiddenTenGodPairs(item.branchHiddenStems, item.branchTenGods)"
                :key="`${item.key}-${pair.stem}-${pair.tenGod}-${index}`"
                class="interaction-pillar-hidden-item"
              >
                <span :class="['wuxing-char', `wuxing-${stemElement(pair.stem)}`]">{{ pair.stem }}</span>
                <span class="hidden-ten-god-text">{{ pair.tenGod }}</span>
              </span>
            </div>
          </div>
        </div>

        <div v-if="branchSummaryStrips.length" class="interaction-summary-group">
          <div class="interaction-summary-grid" :style="interactionGridStyle">
            <div
              v-for="item in branchSummaryStrips"
              :key="item.key"
              :class="['interaction-summary-strip', item.tone]"
              :style="summaryStripStyle(item)"
            >
              {{ item.label }}: {{ item.value }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { BaziResponse, DaYun, LiuNian, LiuYue, LuckPreviewPillar } from "../types/bazi";

const props = defineProps<{
  result: BaziResponse;
  compact?: boolean;
}>();

const emit = defineEmits<{
  "preview-change": [LuckPreviewPillar[] | null];
}>();

type WuXing = "wood" | "fire" | "earth" | "metal" | "water";
type DisplayDaYun = DaYun & {
  ageRange: string;
  yearRange: string;
  rangeLabel: string;
};
type HiddenTenGodPair = {
  stem: string;
  tenGod: string;
};
type InteractionPillarItem = {
  key: string;
  label: string;
  stem: string;
  branch: string;
  stemTenGod?: string;
  branchHiddenStems?: string[];
  branchTenGods?: string[];
};
type InteractionSummary = {
  key: string;
  label: string;
  value: string;
  tone: string;
  start: number;
  end: number;
};

const currentYear = new Date().getFullYear();
const selectedLuckIndex = ref<number | null>(null);
const selectedAnnualYear = ref<number | null>(null);
const selectedMonthIndex = ref<number | null>(null);
const expandAnnualLane = ref(!props.compact);
const expandMonthLane = ref(!props.compact);

const STEM_WUXING: Record<string, WuXing> = {
  甲: "wood",
  乙: "wood",
  丙: "fire",
  丁: "fire",
  戊: "earth",
  己: "earth",
  庚: "metal",
  辛: "metal",
  壬: "water",
  癸: "water",
};

const BRANCH_WUXING: Record<string, WuXing> = {
  子: "water",
  丑: "earth",
  寅: "wood",
  卯: "wood",
  辰: "earth",
  巳: "fire",
  午: "fire",
  未: "earth",
  申: "metal",
  酉: "metal",
  戌: "earth",
  亥: "water",
};

const STEM_COMBINATION_ELEMENTS: Record<string, string> = {
  甲己: "土",
  乙庚: "金",
  丙辛: "水",
  丁壬: "木",
  戊癸: "火",
};

const STEM_CLASH_PAIRS = new Set(["甲庚", "乙辛", "丙壬", "丁癸"]);

const BRANCH_SIX_COMBINATION_ELEMENTS: Record<string, string> = {
  子丑: "土",
  寅亥: "木",
  卯戌: "火",
  辰酉: "金",
  巳申: "水",
  午未: "土",
};

const BRANCH_HALF_COMBINATIONS: Record<string, string> = {
  寅午: "火",
  午戌: "火",
  亥卯: "木",
  卯未: "木",
  申子: "水",
  子辰: "水",
  巳酉: "金",
  酉丑: "金",
};

const BRANCH_CLASH_PAIRS = new Set(["子午", "丑未", "寅申", "卯酉", "辰戌", "巳亥"]);

const BRANCH_PUNISH_RULES: Array<{ pair: string; detail: string }> = [
  { pair: "子卯", detail: "子卯相刑" },
  { pair: "寅巳", detail: "寅巳相刑" },
  { pair: "巳申", detail: "巳申相刑" },
  { pair: "寅申", detail: "寅申相刑" },
  { pair: "丑戌", detail: "丑戌相刑" },
  { pair: "戌未", detail: "戌未相刑" },
  { pair: "丑未", detail: "丑未相刑" },
];

const BRANCH_BREAK_RULES: Array<{ pair: string; detail: string }> = [
  { pair: "子酉", detail: "子酉相破" },
  { pair: "卯午", detail: "卯午相破" },
  { pair: "辰丑", detail: "辰丑相破" },
  { pair: "未戌", detail: "未戌相破" },
  { pair: "寅亥", detail: "寅亥相破" },
  { pair: "巳申", detail: "巳申相破" },
];

const BRANCH_HARM_RULES: Array<{ pair: string; detail: string }> = [
  { pair: "子未", detail: "子未相害" },
  { pair: "丑午", detail: "丑午相害" },
  { pair: "寅巳", detail: "寅巳相害" },
  { pair: "卯辰", detail: "卯辰相害" },
  { pair: "申亥", detail: "申亥相害" },
  { pair: "酉戌", detail: "酉戌相害" },
];

const BRANCH_DARK_MEETING_RULES: Array<{ pair: string; element: string }> = [
  { pair: "寅辰", element: "木" },
  { pair: "巳未", element: "火" },
  { pair: "申戌", element: "金" },
  { pair: "亥丑", element: "水" },
];

const SELF_PUNISH_BRANCHES = new Set(["辰", "午", "酉", "亥"]);
const SUMMARY_TONES: Record<string, string> = {
  monthAnnual: "tone-1",
  annualNatal: "tone-2",
  luckNatal: "tone-3",
  annualLuck: "tone-5",
  natal: "tone-4",
};

function stemElement(stem: string): WuXing {
  return STEM_WUXING[stem] ?? "earth";
}

function branchElement(branch: string): WuXing {
  return BRANCH_WUXING[branch] ?? "earth";
}

function ganZhiStem(ganZhi: string) {
  return ganZhi.charAt(0);
}

function ganZhiBranch(ganZhi: string) {
  return ganZhi.charAt(1);
}

function formatRange(start: number | null, end: number | null) {
  return start != null && end != null ? `${start}-${end}` : "未定";
}

function formatAgeRange(start: number | null, end: number | null) {
  return start != null && end != null ? `${start}-${end + 1}` : "未定";
}

function formatLiuYueLabel(month: string | undefined) {
  if (!month) {
    return "未定";
  }
  return month.endsWith("月") ? month : `${month}月`;
}

function primaryBranchTenGod(tenGods?: string[]) {
  return tenGods?.[0] || "-";
}

function hiddenTenGodPairs(hiddenStems?: string[], tenGods?: string[]): HiddenTenGodPair[] {
  if (!hiddenStems?.length || !tenGods?.length) {
    return [];
  }
  return hiddenStems.map((stem, index) => ({
    stem,
    tenGod: tenGods[index] || "-",
  }));
}

function displayCellHiddenTenGodPairs(hiddenStems?: string[], tenGods?: string[]) {
  return hiddenTenGodPairs(hiddenStems, tenGods);
}

function formatLuckRangeLabel(row: DisplayDaYun) {
  return props.compact ? `${row.ageRange}歲\n${row.yearRange}` : row.rangeLabel;
}

function unorderedPairKey(a: string, b: string) {
  return [a, b].sort().join("");
}

function findPairEntry(record: Record<string, string>, a: string, b: string) {
  const target = unorderedPairKey(a, b);
  return Object.entries(record).find(([key]) => unorderedPairKey(key.charAt(0), key.charAt(1)) === target);
}

function hasPairMatch(values: Set<string>, a: string, b: string) {
  const target = unorderedPairKey(a, b);
  return [...values].some((key) => unorderedPairKey(key.charAt(0), key.charAt(1)) === target);
}

function findPairRule(rules: Array<{ pair: string; detail: string }>, a: string, b: string) {
  const target = unorderedPairKey(a, b);
  return rules.find((rule) => unorderedPairKey(rule.pair.charAt(0), rule.pair.charAt(1)) === target);
}

function hasHiddenStemCombination(left: InteractionPillarItem, right: InteractionPillarItem) {
  const leftHidden = left.branchHiddenStems ?? [];
  const rightHidden = right.branchHiddenStems ?? [];
  return leftHidden.some((leftStem) =>
    rightHidden.some((rightStem) => !!findPairEntry(STEM_COMBINATION_ELEMENTS, leftStem, rightStem)),
  );
}

function buildPreviewPillar(
  key: string,
  label: string,
  ganZhi: string,
  stemTenGod?: string,
  branchHiddenStems?: string[],
  branchTenGods?: string[],
  xun?: string,
  xunKong?: string,
  isCurrentYear = false,
): LuckPreviewPillar {
  return {
    key,
    label,
    ganZhi,
    stemTenGod,
    branchHiddenStems: branchHiddenStems ?? [],
    branchTenGods: branchTenGods ?? [],
    xun,
    xunKong,
    isCurrentYear,
  };
}

function isCurrentLuckRow(row: DaYun): boolean {
  return !!row.liuNian?.some((item) => item.year === currentYear);
}

function isCurrentAnnualRow(row: LiuNian): boolean {
  return row.year === currentYear;
}

function emitPreview(items: LuckPreviewPillar[]) {
  emit("preview-change", items);
}

function emitLuckPreview(row: DaYun) {
  emitPreview([
    buildPreviewPillar(
      `luck-${row.index}`,
      "大運",
      row.ganZhi,
      row.stemTenGod,
      row.branchHiddenStems,
      row.branchTenGods,
      row.xun,
      row.xunKong,
      isCurrentLuckRow(row),
    ),
  ]);
}

function emitAnnualPreview(luckRow: DaYun, annualRow: LiuNian) {
  emitPreview([
    buildPreviewPillar(
      `luck-${luckRow.index}`,
      "大運",
      luckRow.ganZhi,
      luckRow.stemTenGod,
      luckRow.branchHiddenStems,
      luckRow.branchTenGods,
      luckRow.xun,
      luckRow.xunKong,
      isCurrentLuckRow(luckRow),
    ),
    buildPreviewPillar(
      `annual-${luckRow.index}-${annualRow.index}`,
      "流年",
      annualRow.ganZhi,
      annualRow.stemTenGod,
      annualRow.branchHiddenStems,
      annualRow.branchTenGods,
      annualRow.xun,
      annualRow.xunKong,
      isCurrentAnnualRow(annualRow),
    ),
  ]);
}

function emitMonthPreview(luckRow: DaYun, annualRow: LiuNian, monthRow: LiuYue) {
  emitPreview([
    buildPreviewPillar(
      `luck-${luckRow.index}`,
      "大運",
      luckRow.ganZhi,
      luckRow.stemTenGod,
      luckRow.branchHiddenStems,
      luckRow.branchTenGods,
      luckRow.xun,
      luckRow.xunKong,
      isCurrentLuckRow(luckRow),
    ),
    buildPreviewPillar(
      `annual-${luckRow.index}-${annualRow.index}`,
      "流年",
      annualRow.ganZhi,
      annualRow.stemTenGod,
      annualRow.branchHiddenStems,
      annualRow.branchTenGods,
      annualRow.xun,
      annualRow.xunKong,
      isCurrentAnnualRow(annualRow),
    ),
    buildPreviewPillar(
      `month-${annualRow.year}-${monthRow.index}`,
      "流月",
      monthRow.ganZhi,
      monthRow.stemTenGod,
      monthRow.branchHiddenStems,
      monthRow.branchTenGods,
      monthRow.xun,
      monthRow.xunKong,
    ),
  ]);
}

const rows = computed<DisplayDaYun[]>(() =>
  props.result.daYun.map((item) => ({
    ...item,
    ageRange: formatAgeRange(item.startAge, item.endAge),
    yearRange: formatRange(item.startYear, item.endYear),
    rangeLabel: `（${formatAgeRange(item.startAge, item.endAge)}歲 / ${formatRange(item.startYear, item.endYear)}）`,
  })),
);

const natalItems = computed<InteractionPillarItem[]>(() => [
  {
    key: "year",
    label: "年柱",
    stem: props.result.yearPillar.stem,
    branch: props.result.yearPillar.branch,
    stemTenGod: props.result.yearStemTenGod,
    branchHiddenStems: props.result.yearHiddenStems,
    branchTenGods: props.result.yearBranchTenGods,
  },
  {
    key: "month",
    label: "月柱",
    stem: props.result.monthPillar.stem,
    branch: props.result.monthPillar.branch,
    stemTenGod: props.result.monthStemTenGod,
    branchHiddenStems: props.result.monthHiddenStems,
    branchTenGods: props.result.monthBranchTenGods,
  },
  {
    key: "day",
    label: "日柱",
    stem: props.result.dayPillar.stem,
    branch: props.result.dayPillar.branch,
    stemTenGod: props.result.dayStemTenGod,
    branchHiddenStems: props.result.dayHiddenStems,
    branchTenGods: props.result.dayBranchTenGods,
  },
  {
    key: "hour",
    label: "時柱",
    stem: props.result.hourPillar.stem,
    branch: props.result.hourPillar.branch,
    stemTenGod: props.result.hourStemTenGod,
    branchHiddenStems: props.result.hourHiddenStems,
    branchTenGods: props.result.hourBranchTenGods,
  },
]);

const selectedLuckRow = computed(
  () =>
    rows.value.find((row) => row.index === selectedLuckIndex.value) ??
    rows.value.find((row) => isCurrentLuckRow(row)) ??
    rows.value[0] ??
    null,
);

const selectedAnnualRow = computed(
  () =>
    selectedLuckRow.value?.liuNian?.find((annual) => annual.year === selectedAnnualYear.value) ??
    selectedLuckRow.value?.liuNian?.find((annual) => isCurrentAnnualRow(annual)) ??
    selectedLuckRow.value?.liuNian?.[0] ??
    null,
);

const selectedMonthRow = computed(
  () =>
    (showMonthLane.value
      ? selectedAnnualRow.value?.liuYue?.find((month) => month.index === selectedMonthIndex.value) ??
        selectedAnnualRow.value?.liuYue?.[0]
      : null) ??
    null,
);

const showAnnualLane = computed(() => expandAnnualLane.value);
const showMonthLane = computed(() => expandMonthLane.value);

const selectedLuckItem = computed<InteractionPillarItem | null>(() => {
  if (!selectedLuckRow.value) {
    return null;
  }
  return {
    key: `luck-${selectedLuckRow.value.index}`,
    label: `大運${selectedLuckRow.value.startYear ?? ""}`,
    stem: ganZhiStem(selectedLuckRow.value.ganZhi),
    branch: ganZhiBranch(selectedLuckRow.value.ganZhi),
    stemTenGod: selectedLuckRow.value.stemTenGod,
    branchHiddenStems: selectedLuckRow.value.branchHiddenStems,
    branchTenGods: selectedLuckRow.value.branchTenGods,
  };
});

const selectedAnnualItem = computed<InteractionPillarItem | null>(() => {
  if (!showAnnualLane.value || !selectedAnnualRow.value) {
    return null;
  }
  return {
    key: `annual-${selectedAnnualRow.value.year}`,
    label: `流年${selectedAnnualRow.value.year}`,
    stem: ganZhiStem(selectedAnnualRow.value.ganZhi),
    branch: ganZhiBranch(selectedAnnualRow.value.ganZhi),
    stemTenGod: selectedAnnualRow.value.stemTenGod,
    branchHiddenStems: selectedAnnualRow.value.branchHiddenStems,
    branchTenGods: selectedAnnualRow.value.branchTenGods,
  };
});

const interactionColumns = computed<InteractionPillarItem[]>(() => [
  ...(selectedAnnualItem.value ? [selectedAnnualItem.value] : []),
  ...(selectedLuckItem.value ? [selectedLuckItem.value] : []),
  ...natalItems.value,
]);

const interactionColumnIndexMap = computed(
  () => new Map(interactionColumns.value.map((item, index) => [item.key, index + 1])),
);

const interactionGridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${Math.max(interactionColumns.value.length, 1)}, minmax(${props.compact ? 92 : 120}px, 1fr))`,
}));

const interactionMatrixStyle = computed(() => ({
  minWidth: `${Math.max(interactionColumns.value.length, 1) * (props.compact ? 92 : 120)}px`,
}));

const luckStartLine = computed(() => {
  const luckStart = props.result.luckStart;
  if (luckStart.startSummary) {
    return `起運：${luckStart.startSummary}`;
  }
  if (
    luckStart.startYear != null &&
    luckStart.startMonth != null &&
    luckStart.startDay != null &&
    luckStart.startHour != null
  ) {
    return `起運：${luckStart.startYear}年${luckStart.startMonth}月${luckStart.startDay}日${luckStart.startHour}時`;
  }
  return "起運資料未提供";
});

function summarizeStemInteractions(left: InteractionPillarItem, right: InteractionPillarItem) {
  const parts: string[] = [];
  if (left.stem === right.stem) {
    parts.push("伏吟");
  }
  const stemEntry = findPairEntry(STEM_COMBINATION_ELEMENTS, left.stem, right.stem);
  if (stemEntry) {
    parts.push(`${stemEntry[0]}合${stemEntry[1]}`);
  }
  if (hasPairMatch(STEM_CLASH_PAIRS, left.stem, right.stem)) {
    parts.push(`${left.stem}${right.stem}相沖`);
  }
  return [...new Set(parts)];
}

function summarizeBranchInteractions(left: InteractionPillarItem, right: InteractionPillarItem) {
  const parts: string[] = [];
  if (left.branch === right.branch) {
    parts.push("伏吟");
    if (SELF_PUNISH_BRANCHES.has(left.branch)) {
      parts.push("自刑");
    }
  }
  const sixCombinationEntry = findPairEntry(BRANCH_SIX_COMBINATION_ELEMENTS, left.branch, right.branch);
  if (sixCombinationEntry) {
    parts.push(`${sixCombinationEntry[0]}合${sixCombinationEntry[1]}`);
  }
  const halfCombinationEntry = findPairEntry(BRANCH_HALF_COMBINATIONS, left.branch, right.branch);
  if (halfCombinationEntry) {
    parts.push(`${halfCombinationEntry[0]}半合${halfCombinationEntry[1]}`);
  }
  if (hasPairMatch(BRANCH_CLASH_PAIRS, left.branch, right.branch)) {
    parts.push(`${left.branch}${right.branch}相沖`);
  }
  const punishRule = findPairRule(BRANCH_PUNISH_RULES, left.branch, right.branch);
  if (punishRule) {
    parts.push(punishRule.detail);
  }
  const breakRule = findPairRule(BRANCH_BREAK_RULES, left.branch, right.branch);
  if (breakRule) {
    parts.push(breakRule.detail);
  }
  const harmRule = findPairRule(BRANCH_HARM_RULES, left.branch, right.branch);
  if (harmRule) {
    parts.push(harmRule.detail);
  }
  const darkMeetingRule = findPairEntry(
    Object.fromEntries(BRANCH_DARK_MEETING_RULES.map((item) => [item.pair, item.element])),
    left.branch,
    right.branch,
  );
  if (darkMeetingRule) {
    parts.push(`${darkMeetingRule[0]}暗會`);
  }
  if (hasHiddenStemCombination(left, right)) {
    parts.push("暗合");
  }
  return [...new Set(parts)];
}

function summarizeStemInteractionsCompact(left: InteractionPillarItem, right: InteractionPillarItem) {
  const parts: string[] = [];
  if (left.stem === right.stem) {
    parts.push("伏吟");
  }
  const stemEntry = findPairEntry(STEM_COMBINATION_ELEMENTS, left.stem, right.stem);
  if (stemEntry) {
    parts.push(`合${stemEntry[1]}`);
  }
  if (hasPairMatch(STEM_CLASH_PAIRS, left.stem, right.stem)) {
    parts.push("相沖");
  }
  return [...new Set(parts)];
}

function summarizeBranchInteractionsCompact(left: InteractionPillarItem, right: InteractionPillarItem) {
  const parts: string[] = [];
  if (left.branch === right.branch) {
    parts.push("伏吟");
    if (SELF_PUNISH_BRANCHES.has(left.branch)) {
      parts.push("自刑");
    }
  }
  const sixCombinationEntry = findPairEntry(BRANCH_SIX_COMBINATION_ELEMENTS, left.branch, right.branch);
  if (sixCombinationEntry) {
    parts.push(`合${sixCombinationEntry[1]}`);
  }
  const halfCombinationEntry = findPairEntry(BRANCH_HALF_COMBINATIONS, left.branch, right.branch);
  if (halfCombinationEntry) {
    parts.push(`半合${halfCombinationEntry[1]}`);
  }
  if (hasPairMatch(BRANCH_CLASH_PAIRS, left.branch, right.branch)) {
    parts.push("相沖");
  }
  if (findPairRule(BRANCH_PUNISH_RULES, left.branch, right.branch)) {
    parts.push("相刑");
  }
  if (findPairRule(BRANCH_BREAK_RULES, left.branch, right.branch)) {
    parts.push("相破");
  }
  if (findPairRule(BRANCH_HARM_RULES, left.branch, right.branch)) {
    parts.push("相害");
  }
  const darkMeetingRule = findPairEntry(
    Object.fromEntries(BRANCH_DARK_MEETING_RULES.map((item) => [item.pair, item.element])),
    left.branch,
    right.branch,
  );
  if (darkMeetingRule) {
    parts.push("暗會");
  }
  if (hasHiddenStemCombination(left, right)) {
    parts.push("暗合");
  }
  return [...new Set(parts)];
}

function summarizeGroupInteractions(
  primary: InteractionPillarItem[],
  secondary: InteractionPillarItem[],
  summarizePair: (left: InteractionPillarItem, right: InteractionPillarItem) => string[],
) {
  const result = new Set<string>();
  for (const left of primary) {
    for (const right of secondary) {
      if (left.key === right.key) {
        continue;
      }
      for (const part of summarizePair(left, right)) {
        result.add(part);
      }
    }
  }
  return [...result];
}

function summarizeWithinGroup(
  items: InteractionPillarItem[],
  summarizePair: (left: InteractionPillarItem, right: InteractionPillarItem) => string[],
) {
  const result = new Set<string>();
  for (let i = 0; i < items.length; i += 1) {
    for (let j = i + 1; j < items.length; j += 1) {
      for (const part of summarizePair(items[i], items[j])) {
        result.add(part);
      }
    }
  }
  return [...result];
}

function buildSummaryItem(
  key: string,
  label: string,
  participants: InteractionPillarItem[],
  values: string[],
): InteractionSummary | null {
  if (key.startsWith("month")) {
    return {
      key,
      label,
      value: values.length ? values.join("，") : "無沖合關係",
      tone: SUMMARY_TONES[key] ?? "tone-1",
      start: 1,
      end: Math.max(interactionColumns.value.length, 1),
    };
  }

  const indices = participants
    .map((item) => interactionColumnIndexMap.value.get(item.key))
    .filter((index): index is number => typeof index === "number");
  if (!indices.length) {
    return null;
  }
  return {
    key,
    label,
    value: values.length ? values.join("，") : "無沖合關係",
    tone: SUMMARY_TONES[key] ?? "tone-1",
    start: Math.min(...indices),
    end: Math.max(...indices),
  };
}

function buildSummaryRows(
  summarizePair: (left: InteractionPillarItem, right: InteractionPillarItem) => string[],
) {
  const summaries: InteractionSummary[] = [];

  if (selectedAnnualItem.value) {
    const summary = buildSummaryItem(
      "annualNatal",
      "年.局",
      [selectedAnnualItem.value, ...natalItems.value],
      summarizeGroupInteractions([selectedAnnualItem.value], natalItems.value, summarizePair),
    );
    if (summary) {
      summaries.push(summary);
    }
  }

  if (selectedLuckItem.value) {
    const summary = buildSummaryItem(
      "luckNatal",
      "運.局",
      [selectedLuckItem.value, ...natalItems.value],
      summarizeGroupInteractions([selectedLuckItem.value], natalItems.value, summarizePair),
    );
    if (summary) {
      summaries.push(summary);
    }
  }

  if (selectedAnnualItem.value && selectedLuckItem.value) {
    const summary = buildSummaryItem(
      "annualLuck",
      "年.運",
      [selectedAnnualItem.value, selectedLuckItem.value],
      summarizeGroupInteractions([selectedAnnualItem.value], [selectedLuckItem.value], summarizePair),
    );
    if (summary) {
      summaries.push(summary);
    }
  }

  const natalSummary = buildSummaryItem(
    "natal",
    "命局",
    natalItems.value,
    summarizeWithinGroup(natalItems.value, summarizePair),
  );
  if (natalSummary) {
    summaries.push(natalSummary);
  }

  return summaries;
}

function buildMonthAnnualSummary(
  summarizePair: (left: InteractionPillarItem, right: InteractionPillarItem) => string[],
) {
  if (!showMonthLane.value || !selectedAnnualItem.value || !selectedAnnualRow.value?.liuYue?.length) {
    return [];
  }

  const parts: string[] = [];
  for (const month of selectedAnnualRow.value.liuYue) {
    const monthItem: InteractionPillarItem = {
      key: `month-summary-${selectedAnnualRow.value.year}-${month.index}`,
      label: `${month.index + 1}月`,
      stem: ganZhiStem(month.ganZhi),
      branch: ganZhiBranch(month.ganZhi),
      stemTenGod: month.stemTenGod,
      branchHiddenStems: month.branchHiddenStems,
      branchTenGods: month.branchTenGods,
    };
    const monthParts = summarizePair(monthItem, selectedAnnualItem.value);
    if (monthParts.length) {
      parts.push(`${month.index + 1}月${monthParts.join("/")}`);
    }
  }

  const summary = buildSummaryItem("monthAnnual", "農曆月.年", [selectedAnnualItem.value], parts);
  return summary ? [summary] : [];
}

function toggleAnnualLane() {
  const next = !expandAnnualLane.value;
  expandAnnualLane.value = next;
  if (!next) {
    expandMonthLane.value = false;
  }
}

function toggleMonthLane() {
  const next = !expandMonthLane.value;
  expandMonthLane.value = next;
  if (next) {
    expandAnnualLane.value = true;
  }
}

const stemSummaryStrips = computed(() => [
  ...buildMonthAnnualSummary(summarizeStemInteractionsCompact),
  ...buildSummaryRows(summarizeStemInteractions),
]);

const branchSummaryStrips = computed(() => {
  const items = [
    ...buildMonthAnnualSummary(summarizeBranchInteractionsCompact),
    ...buildSummaryRows(summarizeBranchInteractions),
  ];
  const orderedKeys = ["annualLuck", "natal", "luckNatal", "annualNatal", "monthAnnual"];
  const keySet = new Set(orderedKeys);

  return [
    ...orderedKeys
      .map((key) => items.find((item) => item.key === key))
      .filter((item): item is InteractionSummary => !!item),
    ...items.filter((item) => !keySet.has(item.key)),
  ];
});

function summaryStripStyle(item: InteractionSummary) {
  return {
    gridColumn: `${item.start} / ${item.end + 1}`,
  };
}

function initializeSelection() {
  const defaultLuck = rows.value.find((row) => isCurrentLuckRow(row)) ?? rows.value[0] ?? null;
  selectedLuckIndex.value = defaultLuck?.index ?? null;
  selectedAnnualYear.value =
    defaultLuck?.liuNian?.find((annual) => isCurrentAnnualRow(annual))?.year ??
    defaultLuck?.liuNian?.[0]?.year ??
    null;
  selectedMonthIndex.value =
    defaultLuck?.liuNian?.find((annual) => annual.year === selectedAnnualYear.value)?.liuYue?.[0]?.index ??
    defaultLuck?.liuNian?.[0]?.liuYue?.[0]?.index ??
    null;
}

function selectLuckRow(row: DisplayDaYun) {
  selectedLuckIndex.value = row.index;
  selectedAnnualYear.value =
    row.liuNian?.find((annual) => isCurrentAnnualRow(annual))?.year ?? row.liuNian?.[0]?.year ?? null;
  selectedMonthIndex.value =
    row.liuNian?.find((annual) => annual.year === selectedAnnualYear.value)?.liuYue?.[0]?.index ??
    row.liuNian?.[0]?.liuYue?.[0]?.index ??
    null;
  emitLuckPreview(row);
}

function selectAnnualRow(row: LiuNian) {
  selectedAnnualYear.value = row.year;
  selectedMonthIndex.value = row.liuYue?.[0]?.index ?? null;
  if (selectedLuckRow.value) {
    emitAnnualPreview(selectedLuckRow.value, row);
  }
}

function selectMonthRow(row: LiuYue) {
  selectedMonthIndex.value = row.index;
  if (selectedLuckRow.value && selectedAnnualRow.value) {
    emitMonthPreview(selectedLuckRow.value, selectedAnnualRow.value, row);
  }
}

watch(
  rows,
  () => {
    initializeSelection();
  },
  { immediate: true },
);

watch(selectedLuckRow, (row) => {
  if (!row) {
    selectedAnnualYear.value = null;
    selectedMonthIndex.value = null;
    return;
  }
  if (!row.liuNian?.some((annual) => annual.year === selectedAnnualYear.value)) {
    selectedAnnualYear.value =
      row.liuNian?.find((annual) => isCurrentAnnualRow(annual))?.year ?? row.liuNian?.[0]?.year ?? null;
  }
});

watch(selectedAnnualRow, (row) => {
  if (!row) {
    selectedMonthIndex.value = null;
    return;
  }
  if (!row.liuYue?.some((month) => month.index === selectedMonthIndex.value)) {
    selectedMonthIndex.value = row.liuYue?.[0]?.index ?? null;
  }
});

watch(
  () => props.compact,
  (compact) => {
    expandAnnualLane.value = !compact;
    expandMonthLane.value = !compact;
  },
  { immediate: true },
);

watch(expandAnnualLane, (expanded) => {
  if (!expanded) {
    expandMonthLane.value = false;
    if (selectedLuckRow.value) {
      emitLuckPreview(selectedLuckRow.value);
    }
    return;
  }
  if (expandMonthLane.value && selectedLuckRow.value && selectedAnnualRow.value && selectedMonthRow.value) {
    emitMonthPreview(selectedLuckRow.value, selectedAnnualRow.value, selectedMonthRow.value);
    return;
  }
  if (selectedLuckRow.value && selectedAnnualRow.value) {
    emitAnnualPreview(selectedLuckRow.value, selectedAnnualRow.value);
  }
});

watch(expandMonthLane, (expanded) => {
  if (!selectedLuckRow.value || !selectedAnnualRow.value) {
    return;
  }
  if (expanded && selectedMonthRow.value) {
    emitMonthPreview(selectedLuckRow.value, selectedAnnualRow.value, selectedMonthRow.value);
    return;
  }
  emitAnnualPreview(selectedLuckRow.value, selectedAnnualRow.value);
});
</script>

<style scoped>
.luck-panel {
  overflow: hidden;
}

.luck-start-line {
  margin: 8px 0 0;
  color: #dc2626;
  font-size: 14px;
}

.compact-lane-toggle-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
}

.compact-lane-toggle {
  border: 1px solid rgba(203, 213, 225, 0.9);
  background: rgba(248, 250, 252, 0.96);
  color: #334155;
  border-radius: 999px;
  padding: 6px 12px;
  font-size: 12px;
  line-height: 1.1;
  cursor: pointer;
  transition: background 0.18s ease, color 0.18s ease, border-color 0.18s ease;
}

.compact-lane-toggle.is-active {
  background: #1d4ed8;
  border-color: #1d4ed8;
  color: #fff;
}

.luck-board {
  display: grid;
  gap: 16px;
}

.luck-lane {
  display: grid;
  grid-template-columns: 56px minmax(0, 1fr);
  border: 1px solid rgba(203, 213, 225, 0.82);
  background: rgba(255, 255, 255, 0.62);
}

.lane-label {
  display: grid;
  place-items: center;
  border-right: 1px solid rgba(203, 213, 225, 0.82);
  background: rgba(248, 250, 252, 0.96);
  color: #0f172a;
  font-size: 20px;
  line-height: 1.35;
}

.lane-track {
  display: grid;
  grid-auto-flow: column;
  grid-auto-columns: 112px;
  width: max-content;
  min-width: max-content;
}

.lane-scrollbar {
  width: 100%;
}

.lane-scrollbar :deep(.el-scrollbar__wrap) {
  overflow-y: hidden;
}

.lane-scrollbar :deep(.el-scrollbar__bar.is-horizontal) {
  height: 10px;
}

.lane-scrollbar :deep(.el-scrollbar__thumb) {
  background: rgba(148, 163, 184, 0.72);
}

.lane-empty {
  display: flex;
  align-items: center;
  min-height: 132px;
  padding: 0 18px;
  color: #64748b;
}

.fate-cell {
  position: relative;
  display: grid;
  grid-template-rows: minmax(20px, auto) minmax(72px, auto) minmax(72px, auto) minmax(18px, auto);
  gap: 10px;
  height: 232px;
  padding: 10px 12px 12px;
  border: 0;
  border-right: 1px solid rgba(226, 232, 240, 0.95);
  background: rgba(255, 255, 255, 0.84);
  text-align: left;
  cursor: pointer;
  transition: background 0.2s ease, box-shadow 0.2s ease;
  overflow: hidden;
}

.fate-cell-annual {
  height: 240px;
}

.fate-cell-month {
  grid-template-rows: minmax(30px, auto) minmax(20px, auto) minmax(72px, auto) minmax(72px, auto) minmax(18px, auto);
  height: 284px;
  gap: 6px;
}

.fate-cell:last-child {
  border-right: 0;
}

.fate-cell:hover {
  background: rgba(248, 250, 252, 0.98);
}

.fate-cell.is-selected {
  background: rgba(203, 200, 200, 0.88);
}

.fate-cell.is-current:not(.is-selected) {
  box-shadow: inset 0 -3px 0 #d97706;
}

.fate-cell-major {
  height: 248px;
}

.fate-cell-year {
  color: #0f172a;
  font-size: 14px;
  min-height: 20px;
}

.fate-cell-topline {
  display: grid;
  gap: 1px;
  color: #0f172a;
  font-size: 12px;
  line-height: 1.15;
  min-height: 30px;
}

.fate-cell-term {
  font-weight: 500;
}

.fate-cell-term-date {
  color: #334155;
}

.fate-cell-main {
  display: grid;
  gap: 8px;
  min-height: 72px;
}

.fate-hidden-list {
  display: grid;
  gap: 4px;
  min-height: 72px;
  max-height: 72px;
  overflow: hidden;
}

.fate-hidden-list-month {
  min-height: 72px;
  max-height: 72px;
  padding-top: 2px;
}

.fate-hidden-item {
  display: flex;
  align-items: baseline;
  gap: 4px;
  min-height: 18px;
}

.fate-char-row {
  display: flex;
  align-items: baseline;
  gap: 4px;
  min-height: 32px;
}

.fate-char-row .wuxing-char {
  min-width: 1em;
  font-size: 22px;
  font-weight: 700;
  line-height: 1;
}

.fate-cell-major .fate-char-row .wuxing-char {
  font-size: 24px;
}

.ten-god-text {
  color: #111827;
  font-size: 12px;
}

.hidden-ten-god-text {
  color: #475569;
  font-size: 11px;
}

.fate-cell-meta {
  color: #64748b;
  font-size: 12px;
  min-height: 18px;
  line-height: 18px;
  align-self: end;
}

.luck-detail-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 18px;
  padding-top: 4px;
  color: #475569;
  font-size: 13px;
}

.interaction-matrix-shell {
  margin-top: 10px;
  overflow-x: auto;
}

.interaction-matrix {
  display: grid;
  gap: 0;
}

.interaction-summary-stack {
  display: grid;
  gap: 10px;
}

.interaction-summary-group {
  display: grid;
  gap: 0;
}

.interaction-summary-title {
  padding: 6px 10px;
  background: rgba(248, 250, 252, 0.96);
  color: #334155;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.interaction-summary-grid {
  display: grid;
  gap: 0;
  background: rgba(255, 255, 255, 0.94);
  border: 1px solid rgba(203, 213, 225, 0.82);
  border-top: 0;
}

.interaction-summary-strip {
  min-height: 32px;
  padding: 6px 10px;
  font-size: 14px;
  line-height: 1.35;
}

.tone-1 {
  background: #f9a03f;
  color: #111827;
}

.tone-2 {
  background: #ef4b34;
  color: #111827;
}

.tone-3 {
  background: #38a84c;
  color: #111827;
}

.tone-4 {
  background: #4c84f0;
  color: #111827;
}

.tone-5 {
  background: #f6d743;
  color: #111827;
}

.interaction-pillar-grid {
  display: grid;
  border: 1px solid rgba(203, 213, 225, 0.82);
  border-top: 0;
}

.interaction-pillar-card {
  min-height: 96px;
  padding: 8px 10px 10px;
  border-right: 1px solid rgba(203, 213, 225, 0.82);
  background: rgba(255, 255, 255, 0.84);
}

.interaction-pillar-card:last-child {
  border-right: 0;
}

.interaction-pillar-label {
  margin-bottom: 8px;
  color: #0f172a;
  font-size: 14px;
}

.interaction-pillar-main {
  display: flex;
  align-items: baseline;
  gap: 4px;
  min-height: 28px;
}

.interaction-pillar-main .wuxing-char {
  font-size: 22px;
  font-weight: 700;
  line-height: 1;
}

.interaction-pillar-main .ten-god-text {
  font-size: 13px;
}

.interaction-pillar-hidden-list {
  display: grid;
  gap: 4px;
  margin-top: 6px;
  min-height: 54px;
}

.interaction-pillar-hidden-item {
  display: flex;
  align-items: baseline;
  gap: 4px;
  min-height: 16px;
}

.interaction-pillar-hidden-item .wuxing-char {
  font-size: 14px;
  font-weight: 500;
}

.interaction-pillar-hidden-item .hidden-ten-god-text {
  font-size: 10px;
}

.wuxing-char {
  display: inline-block;
  min-width: 1.2em;
  text-align: center;
}

.wuxing-wood {
  color: #16a34a;
}

.wuxing-fire {
  color: #ff2a1a;
}

.wuxing-earth {
  color: #a16207;
}

.wuxing-metal {
  color: #f59e0b;
}

.wuxing-water {
  color: #1d4ed8;
}

.luck-panel.is-compact .luck-start-line {
  font-size: 10px;
  line-height: 1.35;
}

.luck-panel.is-compact .luck-board {
  gap: 8px;
}

.luck-panel.is-compact .luck-lane {
  grid-template-columns: 34px minmax(0, 1fr);
}

.luck-panel.is-compact .lane-label {
  font-size: 11px;
  line-height: 1.2;
}

.luck-panel.is-compact .lane-track {
  grid-auto-columns: minmax(88px, 88px);
}

.luck-panel.is-compact .fate-cell,
.luck-panel.is-compact .fate-cell-major,
.luck-panel.is-compact .fate-cell-annual {
  height: 286px;
  padding: 7px 8px 9px;
  gap: 2px;
}

.luck-panel.is-compact .fate-cell-month {
  height: 334px;
  grid-template-rows: minmax(24px, auto) minmax(22px, auto) minmax(56px, auto) minmax(112px, auto) minmax(14px, auto);
}

.luck-panel.is-compact .fate-cell-year {
  font-size: 10px;
  min-height: 28px;
  line-height: 1.15;
  white-space: pre-line;
}

.luck-panel.is-compact .fate-cell-topline,
.luck-panel.is-compact .fate-cell-meta {
  font-size: 9px;
  line-height: 1.15;
}

.luck-panel.is-compact .fate-cell-main {
  min-height: 62px;
  gap: 4px;
}

.luck-panel.is-compact .fate-char-row {
  min-height: 24px;
  gap: 4px;
  align-items: baseline;
}

.luck-panel.is-compact .fate-char-row .wuxing-char,
.luck-panel.is-compact .fate-cell-major .fate-char-row .wuxing-char {
  font-size: 20px;
  font-weight: 700;
  line-height: 1;
}

.luck-panel.is-compact .ten-god-text {
  font-size: 11px;
  line-height: 1.15;
  letter-spacing: 0;
}

.luck-panel.is-compact .hidden-ten-god-text {
  font-size: 10px;
  line-height: 1.15;
  letter-spacing: 0;
}

.luck-panel.is-compact .fate-hidden-list,
.luck-panel.is-compact .fate-hidden-list-month {
  min-height: 104px;
  max-height: 104px;
  gap: 4px;
}

.luck-panel.is-compact .fate-hidden-item {
  min-height: 18px;
  gap: 4px;
  align-items: baseline;
}

.luck-panel.is-compact .fate-hidden-item .wuxing-char {
  font-size: 14px;
  font-weight: 500;
  line-height: 1;
}

.luck-panel.is-compact .luck-detail-bar {
  gap: 6px 12px;
  font-size: 10px;
}

.luck-panel.is-compact .interaction-summary-title,
.luck-panel.is-compact .interaction-summary-strip,
.luck-panel.is-compact .interaction-pillar-label {
  font-size: 10px;
}

.luck-panel.is-compact .interaction-summary-strip {
  min-height: 24px;
  padding: 4px 5px;
  line-height: 1.3;
}

.luck-panel.is-compact .interaction-pillar-card {
  min-height: 68px;
  padding: 5px 5px 6px;
}

.luck-panel.is-compact .interaction-pillar-main {
  min-height: 20px;
}

.luck-panel.is-compact .interaction-pillar-main .wuxing-char {
  font-size: 15px;
}

.luck-panel.is-compact .interaction-pillar-main .ten-god-text,
.luck-panel.is-compact .interaction-pillar-hidden-item .wuxing-char {
  font-size: 9px;
}

.luck-panel.is-compact .interaction-pillar-hidden-item .hidden-ten-god-text {
  font-size: 8px;
}

@media (max-width: 960px) {
  .luck-lane {
    grid-template-columns: 44px minmax(0, 1fr);
  }

  .lane-label {
    font-size: 16px;
  }

  .lane-track {
    grid-auto-columns: minmax(92px, 92px);
  }

  .fate-cell,
  .fate-cell-major,
  .fate-cell-annual {
    height: 248px;
  }

  .fate-cell-month {
    height: 292px;
  }

  .interaction-summary-strip {
    font-size: 13px;
  }
}

@media (max-width: 768px) {
  .luck-panel.is-compact :deep(.el-card__body) {
    padding: 12px;
  }

  .luck-panel.is-compact .lane-scrollbar :deep(.el-scrollbar__bar.is-horizontal) {
    height: 8px;
  }
}

@media (max-width: 480px) {
  .luck-panel.is-compact .luck-lane {
    grid-template-columns: 30px minmax(0, 1fr);
  }

  .luck-panel.is-compact .lane-track {
    grid-auto-columns: minmax(80px, 80px);
  }

  .luck-panel.is-compact .fate-cell,
  .luck-panel.is-compact .fate-cell-major,
  .luck-panel.is-compact .fate-cell-annual {
    height: 270px;
    padding: 6px 7px 8px;
  }

  .luck-panel.is-compact .fate-cell-month {
    height: 312px;
  }

  .luck-panel.is-compact .fate-char-row .wuxing-char,
  .luck-panel.is-compact .fate-cell-major .fate-char-row .wuxing-char,
  .luck-panel.is-compact .interaction-pillar-main .wuxing-char {
    font-size: 18px;
  }

  .luck-panel.is-compact .ten-god-text {
    font-size: 10px;
  }

  .luck-panel.is-compact .hidden-ten-god-text {
    font-size: 9px;
  }

  .luck-panel.is-compact .fate-hidden-item .wuxing-char {
    font-size: 13px;
  }

  .luck-panel.is-compact .interaction-summary-strip,
  .luck-panel.is-compact .interaction-pillar-label,
  .luck-panel.is-compact .interaction-summary-title {
    font-size: 9px;
  }
}
</style>
