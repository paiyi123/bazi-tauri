<template>
  <el-table
    :data="scores"
    stripe
    size="small"
    class="luck-quant-table"
    :row-class-name="luckRowClassName"
    @row-click="handleLuckRowClick"
    @expand-change="handleLuckExpandChange"
  >
    <el-table-column type="expand" width="50">
      <template #default="{ row }">
        <div class="annual-block" v-if="row.annualScores?.length">
          <div class="annual-title">流年／歲運量化</div>
          <el-radio-group v-model="annualCombinationMode" size="small" class="annual-mode-switch">
            <el-radio-button label="active">當值半運合參</el-radio-button>
            <el-radio-button label="full">整個大運合參</el-radio-button>
          </el-radio-group>
          <el-table
            :data="row.annualScores"
            size="small"
            stripe
            class="luck-annual-table"
            :row-class-name="annualRowClassName"
          >
            <el-table-column prop="year" label="年份" width="76" />
            <el-table-column prop="age" label="歲數" width="64" />
            <el-table-column prop="annualStemTenGod" label="流年干十神" width="96" />
            <el-table-column label="流年" width="72">
              <template #default="{ row: annual }">
                <span v-if="annual.ganZhi" class="luck-ganzhi">
                  <span :class="['wuxing-char', `wuxing-${stemElement(ganZhiStem(annual.ganZhi))}`]">
                    {{ ganZhiStem(annual.ganZhi) }}
                  </span>
                  <span :class="['wuxing-char', `wuxing-${branchElement(ganZhiBranch(annual.ganZhi))}`]">
                    {{ ganZhiBranch(annual.ganZhi) }}
                  </span>
                </span>
                <span v-else>未定</span>
              </template>
            </el-table-column>
            <el-table-column label="流年支十神" width="120">
              <template #default="{ row: annual }">
                <span
                  v-if="hiddenTenGodPairs(annual.annualBranchHiddenStems, annual.annualBranchTenGods).length"
                  class="hidden-ten-gods hidden-ten-gods-stacked"
                >
                  <template
                    v-for="item in hiddenTenGodPairs(annual.annualBranchHiddenStems, annual.annualBranchTenGods)"
                    :key="`annual-${item.stem}-${item.tenGod}`"
                  >
                    <span :class="['hidden-ten-god-item', `wuxing-${stemElement(item.stem)}`]">
                      <span class="wuxing-char">{{ item.stem }}</span>
                      <span>{{ item.tenGod }}</span>
                    </span>
                  </template>
                </span>
                <span v-else>未定</span>
              </template>
            </el-table-column>
            <el-table-column :label="annualCombinationMode === 'active' ? '當值大運' : '整個大運'" width="84">
              <template #default="{ row: annual }">
                {{ annualCombinationMode === "active" ? annual.activeLuckScore : annual.fullLuckScore }}
              </template>
            </el-table-column>
            <el-table-column prop="annualStemScore" label="流年干" width="78" />
            <el-table-column prop="annualBranchScore" label="流年支" width="78" />
            <el-table-column prop="annualTotalScore" label="流年整柱" width="90" />
            <el-table-column :label="annualCombinationMode === 'active' ? '半運合參' : '整運合參'" width="84">
              <template #default="{ row: annual }">
                {{ annualCombinationMode === "active" ? annual.combinedScore : annual.fullCombinedScore }}
              </template>
            </el-table-column>
            <el-table-column label="R值" width="68">
              <template #default="{ row: annual }">
                {{ annualCombinationMode === "active" ? annual.impactRatio : annual.fullImpactRatio }}
              </template>
            </el-table-column>
            <el-table-column label="作用後命局" width="94">
              <template #default="{ row: annual }">
                {{ annualCombinationMode === "active" ? annual.effectiveNatalScore : annual.fullEffectiveNatalScore }}
              </template>
            </el-table-column>
            <el-table-column label="傾向" width="84">
              <template #default="{ row: annual }">
                {{ annualCombinationMode === "active" ? annual.tendency : annual.fullTendency }}
              </template>
            </el-table-column>
            <el-table-column label="說明" width="78">
              <template #default="{ row: annual }">
                <el-button link type="primary" @click.stop="handleAnnualRowClick(row, annual, null, $event)">
                  查看
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
        <div v-else class="annual-empty">尚無流年量化資料</div>
      </template>
    </el-table-column>
    <el-table-column prop="index" label="#" width="48" />
    <el-table-column prop="stemTenGod" label="運干十神" width="84" />
    <el-table-column label="大運" width="72">
      <template #default="{ row }">
        <span v-if="row.ganZhi" class="luck-ganzhi luck-ganzhi-major">
          <span :class="['wuxing-char', `wuxing-${stemElement(ganZhiStem(row.ganZhi))}`]">
            {{ ganZhiStem(row.ganZhi) }}
          </span>
          <span :class="['wuxing-char', `wuxing-${branchElement(ganZhiBranch(row.ganZhi))}`]">
            {{ ganZhiBranch(row.ganZhi) }}
          </span>
        </span>
        <span v-else>未定</span>
      </template>
    </el-table-column>
    <el-table-column label="運支十神" width="120">
      <template #default="{ row }">
        <span
          v-if="hiddenTenGodPairs(row.branchHiddenStems, row.branchTenGods).length"
          class="hidden-ten-gods hidden-ten-gods-stacked"
        >
          <template
            v-for="item in hiddenTenGodPairs(row.branchHiddenStems, row.branchTenGods)"
            :key="`luck-${item.stem}-${item.tenGod}`"
          >
            <span :class="['hidden-ten-god-item', `wuxing-${stemElement(item.stem)}`]">
              <span class="wuxing-char">{{ item.stem }}</span>
              <span>{{ item.tenGod }}</span>
            </span>
          </template>
        </span>
        <span v-else>未定</span>
      </template>
    </el-table-column>
    <el-table-column label="歲數" width="110">
      <template #default="{ row }">{{ formatAgeRange(row.startAge, row.endAge) }}</template>
    </el-table-column>
    <el-table-column prop="firstHalfScore" label="前五年(干)" width="96" />
    <el-table-column prop="secondHalfScore" label="後五年(支)" width="96" />
    <el-table-column prop="totalScore" label="整柱參考" width="92" />
    <el-table-column prop="impactRatio" label="R值" width="64" />
    <el-table-column prop="effectiveNatalScore" label="作用後命局" width="96" />
    <el-table-column prop="tendency" label="傾向" width="84" />
    <el-table-column label="說明" width="78">
      <template #default="{ row }">
        <el-button link type="primary" @click.stop="openLuckDetail(row, null, $event)">查看</el-button>
      </template>
    </el-table-column>
  </el-table>

  <el-popover
    v-model:visible="luckDetailPopoverVisible"
    :virtual-ref="luckDetailVirtualRef"
    virtual-triggering
    trigger="manual"
    placement="left-start"
    :width="520"
    popper-class="luck-detail-popover"
  >
    <div class="luck-detail-title">{{ luckDetailTitle }}</div>
    <div class="luck-detail-sections">
      <div v-for="section in luckDetailSections" :key="section.title" class="luck-detail-section">
        <div class="luck-detail-section-title">{{ section.title }}</div>
        <div class="luck-detail-lines">
          <div v-for="(line, idx) in section.lines" :key="`${section.title}-${idx}`" class="luck-detail-line">
            {{ line }}
          </div>
        </div>
      </div>
    </div>
  </el-popover>
</template>

<script setup lang="ts">
import { useLuckDetailPopover } from "../composables/useLuckDetailPopover";
import type { LuckPreviewPillar, QuantAnnualLuckScore, QuantLuckScore } from "../types/bazi";

defineProps<{
  scores: QuantLuckScore[];
}>();

const emit = defineEmits<{
  "preview-change": [LuckPreviewPillar[] | null];
}>();

type WuXing = "wood" | "fire" | "earth" | "metal" | "water";
type HiddenTenGodPair = { stem: string; tenGod: string };

const currentYear = new Date().getFullYear();

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

const {
  annualCombinationMode,
  luckDetailPopoverVisible,
  luckDetailTitle,
  luckDetailSections,
  luckDetailVirtualRef,
  openLuckDetail,
  openAnnualDetail,
} = useLuckDetailPopover();

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

function formatAgeRange(startAge: number | null, endAge: number | null) {
  return startAge != null && endAge != null ? `${startAge}-${endAge + 1}` : "未定";
}

function hiddenTenGodPairs(hiddenStems?: string[], tenGods?: string[]): HiddenTenGodPair[] {
  if (!hiddenStems?.length || !tenGods?.length) {
    return [];
  }
  return hiddenStems.map((stem, index) => ({ stem, tenGod: tenGods[index] || "" }));
}

function buildPreviewPillar(
  key: string,
  label: string,
  ganZhi: string,
  stemTenGod?: string,
  branchHiddenStems?: string[],
  branchTenGods?: string[],
  isCurrentYear = false,
): LuckPreviewPillar {
  return {
    key,
    label,
    ganZhi,
    stemTenGod,
    branchHiddenStems: branchHiddenStems ?? [],
    branchTenGods: branchTenGods ?? [],
    isCurrentYear,
  };
}

function isCurrentLuckRow(row: QuantLuckScore) {
  return !!row.annualScores?.some((item) => item.year === currentYear);
}

function isCurrentAnnualRow(row: QuantAnnualLuckScore) {
  return row.year === currentYear;
}

function luckRowClassName({ row }: { row: QuantLuckScore }) {
  return isCurrentLuckRow(row) ? "current-year-row" : "";
}

function annualRowClassName({ row }: { row: QuantAnnualLuckScore }) {
  return isCurrentAnnualRow(row) ? "current-year-row" : "";
}

function emitLuckPreview(row: QuantLuckScore) {
  emit("preview-change", [
    buildPreviewPillar(
      `quant-luck-${row.index}`,
      "大運",
      row.ganZhi,
      row.stemTenGod,
      row.branchHiddenStems,
      row.branchTenGods,
      isCurrentLuckRow(row),
    ),
  ]);
}

function emitAnnualPreview(luckRow: QuantLuckScore, annualRow: QuantAnnualLuckScore) {
  emit("preview-change", [
    buildPreviewPillar(
      `quant-luck-${luckRow.index}`,
      "大運",
      luckRow.ganZhi,
      luckRow.stemTenGod,
      luckRow.branchHiddenStems,
      luckRow.branchTenGods,
      isCurrentLuckRow(luckRow),
    ),
    buildPreviewPillar(
      `quant-annual-${luckRow.index}-${annualRow.index}`,
      "流年",
      annualRow.ganZhi,
      annualRow.annualStemTenGod,
      annualRow.annualBranchHiddenStems,
      annualRow.annualBranchTenGods,
      isCurrentAnnualRow(annualRow),
    ),
  ]);
}

function handleLuckRowClick(row: QuantLuckScore, column?: unknown, event?: Event) {
  emitLuckPreview(row);
  openLuckDetail(row, (column as { type?: string } | null | undefined) ?? undefined, event as MouseEvent | undefined);
}

function handleAnnualRowClick(
  luckRow: QuantLuckScore,
  annualRow: QuantAnnualLuckScore,
  column?: unknown,
  event?: Event,
) {
  emitAnnualPreview(luckRow, annualRow);
  openAnnualDetail(
    annualRow,
    (column as { type?: string } | null | undefined) ?? undefined,
    event as MouseEvent | undefined,
  );
}

function handleLuckExpandChange(row: QuantLuckScore) {
  emitLuckPreview(row);
}
</script>

<style scoped>
.annual-block {
  padding: 4px 8px 8px;
}

.annual-title {
  font-weight: 700;
  margin-bottom: 4px;
  line-height: 1.2;
}

.annual-mode-switch {
  margin-bottom: 6px;
}

.annual-empty {
  padding: 8px;
  color: var(--el-text-color-secondary);
}

.luck-ganzhi {
  display: inline-flex;
  align-items: center;
  font-size: calc(18px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.2;
}

.luck-ganzhi-major {
  font-size: calc(20px * var(--app-font-scale));
}

.hidden-ten-gods {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  row-gap: 2px;
  font-weight: 600;
}

.hidden-ten-gods-stacked {
  display: inline-flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.hidden-ten-god-item {
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
}

:deep(.current-year-row > td.el-table__cell) {
  background: #fff3d6;
}

:deep(.current-year-row > td.el-table__cell:first-child) {
  box-shadow: inset 4px 0 0 #d97706;
}

.wuxing-char {
  display: inline-block;
  min-width: 1.2em;
  text-align: center;
}

.wuxing-wood {
  color: #2e7d32;
}

.wuxing-fire {
  color: #c62828;
}

.wuxing-earth {
  color: #8d6e63;
}

.wuxing-metal {
  color: #546e7a;
}

.wuxing-water {
  color: #1565c0;
}

.luck-detail-sections {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.luck-detail-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.luck-detail-section-title {
  font-weight: 700;
  color: #6a4f1f;
}

.luck-detail-lines {
  display: flex;
  flex-direction: column;
  gap: 6px;
  line-height: 1.55;
}

.luck-detail-title {
  font-weight: 700;
  color: #172033;
  margin-bottom: 8px;
}

.luck-detail-line {
  white-space: normal;
  word-break: break-word;
}

:deep(.luck-detail-popover.el-popover) {
  background: #fffdf9;
  border: 1px solid rgba(196, 203, 214, 0.8);
  box-shadow: 0 18px 42px rgba(23, 32, 51, 0.16);
  color: #253046;
  line-height: 1.55;
  padding: 14px 16px;
}

.luck-quant-table :deep(.el-table__header th.el-table__cell),
.luck-quant-table :deep(.el-table__body td.el-table__cell),
.luck-annual-table :deep(.el-table__header th.el-table__cell),
.luck-annual-table :deep(.el-table__body td.el-table__cell) {
  padding-top: 3px;
  padding-bottom: 3px;
  vertical-align: middle;
}

.luck-quant-table :deep(.cell),
.luck-annual-table :deep(.cell) {
  padding-top: 0;
  padding-bottom: 0;
  line-height: 1.15;
}

.luck-quant-table :deep(.el-table__expanded-cell) {
  padding: 4px 8px 8px;
}

.luck-quant-table :deep(.el-table__row),
.luck-annual-table :deep(.el-table__row) {
  height: auto;
}

.luck-quant-table :deep(.el-table__body tr),
.luck-annual-table :deep(.el-table__body tr) {
  cursor: pointer;
}
</style>
