<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>命盤摘要</h2>
        <el-tag v-if="luckStart?.forward != null" :type="luckStart.forward ? 'success' : 'warning'" effect="light">
          {{ luckStart.forward ? "順排" : "逆排" }}
        </el-tag>
      </div>
    </template>

    <el-descriptions :column="descriptionColumns" border :size="compact ? 'small' : 'default'">
      <el-descriptions-item label="輸入曆別">{{ result.inputCalendarType }}</el-descriptions-item>
      <el-descriptions-item label="輸入日期">{{ result.inputDateTime }}</el-descriptions-item>
      <el-descriptions-item label="公曆">{{ result.solarDateTime }}</el-descriptions-item>
      <el-descriptions-item label="農曆">{{ result.lunarDateTime }}</el-descriptions-item>
      <el-descriptions-item label="八字">
        <span class="bazi-colored">
          <template v-for="(pillar, idx) in baziPillars" :key="idx">
            <span v-if="pillar.prefix" class="pillar-prefix">{{ pillar.prefix }}</span>
            <span :class="['wuxing-char', `wuxing-${pillar.stemElement}`]">{{ pillar.stem }}</span>
            <span :class="['wuxing-char', `wuxing-${pillar.branchElement}`]">{{ pillar.branch }}</span>
            <span v-if="idx < baziPillars.length - 1" class="pillar-gap"> </span>
          </template>
        </span>
      </el-descriptions-item>
      <el-descriptions-item label="日主">{{ result.dayMaster }}</el-descriptions-item>
      <el-descriptions-item label="格局">{{ result.geJu }}</el-descriptions-item>
      <el-descriptions-item label="格局依據">{{ result.geJuBasis }}</el-descriptions-item>
      <el-descriptions-item label="胎元">{{ result.taiYuan }}</el-descriptions-item>
      <el-descriptions-item label="命宮">{{ result.mingGong }}</el-descriptions-item>
      <el-descriptions-item label="身宮">{{ result.shenGong }}</el-descriptions-item>
      <el-descriptions-item v-if="directPillarYearSummary" label="可能西元年" :span="fullSpan">
        <span class="shen-sha-text">{{ directPillarYearSummary }}</span>
      </el-descriptions-item>
      <template v-if="hasExactLuckTiming">
        <el-descriptions-item label="起運日期">{{ luckStart?.startSolar }}</el-descriptions-item>
        <el-descriptions-item label="起運偏移" :span="fullSpan">
          {{ luckStart?.startYear }}年 {{ luckStart?.startMonth }}月 {{ luckStart?.startDay }}日 {{ luckStart?.startHour }}時
        </el-descriptions-item>
        <el-descriptions-item label="上大運" :span="fullSpan">
          {{ luckStart?.startSummary || `出生後${luckStart?.startYear}年${luckStart?.startMonth}月${luckStart?.startDay}日上大運` }}
        </el-descriptions-item>
      </template>
      <el-descriptions-item v-else label="起運資訊" :span="fullSpan">
        {{ luckStartSummary }}
      </el-descriptions-item>
      <el-descriptions-item v-if="luckTransitionSummary" label="交脫大運" :span="fullSpan">
        {{ luckTransitionSummary }}
      </el-descriptions-item>
      <el-descriptions-item v-if="luckStart?.transitionSummaryExperimental" label="交脫大運（研究值）" :span="fullSpan">
        {{ luckStart.transitionSummaryExperimental }}
      </el-descriptions-item>
    </el-descriptions>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";

const props = defineProps<{
  result: BaziResponse;
  compact?: boolean;
}>();

type WuXing = "wood" | "fire" | "earth" | "metal" | "water";

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

const luckStart = computed(() => props.result.luckStart);
const descriptionColumns = computed(() => (props.compact ? 1 : 2));
const fullSpan = computed(() => (props.compact ? 1 : 2));
const hasExactLuckTiming = computed(
  () =>
    !!luckStart.value &&
    !!luckStart.value.startSolar &&
    luckStart.value.startYear != null &&
    luckStart.value.startMonth != null &&
    luckStart.value.startDay != null &&
    luckStart.value.startHour != null,
);
const luckStartSummary = computed(() => luckStart.value?.startSummary || "未提供起運資訊");
const luckTransitionSummary = computed(() => luckStart.value?.transitionSummary || "");
const directPillarYearSummary = computed(() => {
  const hint = props.result.directPillarYearHint;
  if (!hint) {
    return "";
  }
  const parts: string[] = [];
  if (hint.selectedYear != null) {
    parts.push(`已選：${hint.selectedYear}`);
  }
  if (hint.candidateYears?.length) {
    parts.push(`候選：${hint.candidateYears.join("、")}`);
  }
  if (hint.note) {
    parts.push(hint.note);
  }
  return parts.join("；");
});

const baziPillars = computed(() => {
  const pillars = [
    { pillar: props.result.hourPillar, prefix: "" },
    { pillar: props.result.dayPillar, prefix: "日:" },
    { pillar: props.result.monthPillar, prefix: "" },
    { pillar: props.result.yearPillar, prefix: "" },
  ];

  return pillars.map(({ pillar, prefix }) => ({
    prefix,
    stem: pillar.stem,
    branch: pillar.branch,
    stemElement: STEM_WUXING[pillar.stem] ?? "earth",
    branchElement: BRANCH_WUXING[pillar.branch] ?? "earth",
  }));
});
</script>

<style scoped>
.section-header {
  align-items: center;
}

.bazi-colored {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0 0.16em;
  font-size: calc(18px * var(--app-font-scale));
  font-weight: 700;
  letter-spacing: 0.03em;
}

.pillar-prefix {
  margin-right: 0.08em;
  color: #253046;
  font-weight: 600;
}

.wuxing-char {
  display: inline-block;
  min-width: 1.2em;
  text-align: center;
}

.pillar-gap {
  display: inline-block;
  width: 0.45em;
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

.shen-sha-text {
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.6;
  color: #526071;
}

@media (max-width: 768px) {
  :deep(.el-descriptions__label),
  :deep(.el-descriptions__content) {
    font-size: calc(12px * var(--app-font-scale));
    line-height: 1.45;
  }

  :deep(.el-descriptions__label.el-descriptions__cell.is-bordered-label) {
    width: 92px;
  }

  .bazi-colored {
    font-size: calc(16px * var(--app-font-scale));
    line-height: 1.35;
  }

  .pillar-gap {
    width: 0.18em;
  }
}

@media (max-width: 480px) {
  :deep(.el-descriptions__cell) {
    padding: 8px 10px;
  }

  .bazi-colored {
    font-size: calc(15px * var(--app-font-scale));
    gap: 0 0.1em;
  }
}


:deep(.el-card__header) {
  background: linear-gradient(180deg, rgba(255, 247, 234, 0.92), rgba(255, 247, 234, 0));
}

.pillar-prefix {
  color: var(--bazi-primary-strong);
}

.shen-sha-text {
  color: var(--bazi-text-muted);
}
</style>
