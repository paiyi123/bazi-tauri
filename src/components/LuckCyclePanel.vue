<template>
  <el-card shadow="hover" :class="['luck-panel', { 'is-compact': compact }]">
    <template #header>
      <div class="section-header">
        <div>
          <h2>大運</h2>
          <p class="luck-start-line">{{ luckStartLine }}</p>
          <p class="luck-favor-line" :title="favorLineTitle">{{ favorLine }}</p>
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
              <span v-if="isCurrentLuckRow(row)" class="current-badge is-tab">目前</span>
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
              <div v-if="luckNatalRelations.get(row.index)?.length" class="fate-relation-list" title="此步大運與原局四柱的合沖刑破">
                <span
                  v-for="tag in luckNatalRelations.get(row.index)"
                  :key="`luck-relation-${row.index}-${tag.text}`"
                  :class="['fate-relation-tag', `is-${tag.tone}`, tag.judgement && `favor-${tag.judgement.favor}`]"
                  :title="tag.judgement?.reason"
                >
                  {{ tag.text }}<span v-if="tag.judgement?.favor === 'good'" class="favor-mark">▲</span
                  ><span v-else-if="tag.judgement?.favor === 'bad'" class="favor-mark">▼</span>
                </span>
              </div>
              <div v-if="luckEventsByRow.get(row.index)?.length" class="fate-event-list">
                <span
                  v-for="event in luckEventsByRow.get(row.index)"
                  :key="`luck-event-${row.index}-${event.kind}`"
                  :class="['fate-event-tag', `is-${event.level}`, event.topic && `is-topic-${event.topic}`]"
                  :title="event.detail"
                >
                  <b v-if="event.topic" class="topic-mark">{{ TOPIC_MARK[event.topic] }}</b>{{ event.label }}
                </span>
              </div>
            </button>
          </div>
        </el-scrollbar>
      </div>

      <div v-if="showAnnualLane" class="luck-lane">
        <div class="lane-label">流<br />年</div>
        <el-scrollbar v-if="displayedAnnualRows.length" class="lane-scrollbar" always>
          <div class="lane-track">
            <button
              v-for="annual in displayedAnnualRows"
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
              <span v-if="isCurrentAnnualRow(annual)" class="current-badge">今年</span>
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
              <div v-if="annualEventsByYear.get(annual.year)?.length" class="fate-event-list">
                <span
                  v-for="event in annualEventsByYear.get(annual.year)"
                  :key="`annual-event-${annual.year}-${event.kind}`"
                  :class="['fate-event-tag', `is-${event.level}`, event.topic && `is-topic-${event.topic}`]"
                  :title="event.detail"
                >
                  <b v-if="event.topic" class="topic-mark">{{ TOPIC_MARK[event.topic] }}</b>{{ event.label }}
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
        <el-scrollbar v-if="displayedMonthRows.length" class="lane-scrollbar" always>
          <div class="lane-track">
            <button
              v-for="month in displayedMonthRows"
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

    <div v-if="selectedEventNotes.length" class="luck-event-notes">
      <div class="luck-event-notes-title">事件提示（依陸致極兩本教程與《十神洩天機》，只標出容易有事的時機，不代表吉凶）</div>
      <div v-for="note in selectedEventNotes" :key="note.key" class="luck-event-note">
        <span :class="['fate-event-tag', `is-${note.level}`, note.topic && `is-topic-${note.topic}`]"
          ><b v-if="note.topic" class="topic-mark">{{ TOPIC_MARK[note.topic] }}</b>{{ note.scope }}・{{ note.label }}</span
        >
        <span>{{ note.detail }}</span>
      </div>
    </div>

    <div class="interaction-matrix-shell">
      <div class="interaction-matrix" :style="interactionMatrixStyle">
        <div class="interaction-summary-stack">
          <div v-if="stemSummaryStrips.length" class="interaction-summary-group">
            <div class="interaction-summary-grid" :style="interactionGridStyle">
              <div
                v-for="item in stemSummaryStrips"
                :key="item.key"
                :class="['interaction-summary-strip', item.tone, { 'is-empty': item.value === '無沖合關係' }]"
                :style="summaryStripStyle(item)"
              >
                <span class="strip-label">{{ item.label }}</span>{{ item.value }}
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
              :class="['interaction-summary-strip', item.tone, { 'is-empty': item.value === '無沖合關係' }]"
              :style="summaryStripStyle(item)"
            >
              <span class="strip-label">{{ item.label }}</span>{{ item.value }}
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
import { BRANCH_WUXING, STEM_WUXING, type WuXing } from "../constants/wuxing";
import {
  combineJudgements,
  judgeLuckRelation,
  lessonPreferences,
  type RelationJudgement,
  type RelationSide,
} from "../utils/lessonPreference";
import { annualEvents, majorLuckEvents, type LuckEvent, type LuckEventContext } from "../utils/luckEvents";
import { spouseStorageYears, topicEvents, type TopicContext } from "../utils/lifeTopics";
import { movingGodEvents } from "../utils/tenGodCombos";
import { basicStructure, patternLuckEvents } from "../utils/basicStructure";

const TOPIC_MARK: Record<string, string> = { marriage: "婚", wealth: "財", health: "健", move: "動" };

const props = defineProps<{
  result: BaziResponse;
  compact?: boolean;
}>();

const emit = defineEmits<{
  "preview-change": [LuckPreviewPillar[] | null];
}>();

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

function descendingDisplayRows<T>(items?: T[]) {
  return [...(items ?? [])].reverse();
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
  props.result.daYun
    .map((item) => ({
      ...item,
      ageRange: formatAgeRange(item.startAge, item.endAge),
      yearRange: formatRange(item.startYear, item.endYear),
      rangeLabel: `（${formatAgeRange(item.startAge, item.endAge)}歲 / ${formatRange(item.startYear, item.endYear)}）`,
    }))
    .reverse(),
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

type RelationTag = {
  text: string;
  tone: "combine" | "clash" | "punish" | "break" | "echo";
  judgement?: RelationJudgement;
};

function relationTone(text: string): RelationTag["tone"] {
  if (text.includes("沖")) {
    return "clash";
  }
  if (text.includes("刑")) {
    return "punish";
  }
  if (text.includes("破") || text.includes("害")) {
    return "break";
  }
  if (text.includes("伏吟")) {
    return "echo";
  }
  return "combine";
}

const lessonFavor = computed(() => lessonPreferences(props.result.quantModel?.lessonContent?.threeStepAnalysis));

const favorLine = computed(() => {
  const preferences = lessonFavor.value;
  if (!preferences) {
    return "關係標籤：課堂三步分析沒有喜忌方案，暫不判有利不利";
  }
  const names = (favor: string) =>
    Object.values(preferences.elements)
      .filter((item) => item.favor === favor)
      .map((item) => item.name)
      .join("、") || "無";
  const mixed = names("mixed");
  return `關係標籤：▲有利 ▼不利（依課堂三步分析：喜用 ${names("good")}；忌 ${names("bad")}${
    mixed === "無" ? "" : `；依方案而定 ${mixed}`
  }）`;
});

const favorLineTitle = computed(() => {
  const preferences = lessonFavor.value;
  if (!preferences) {
    return undefined;
  }
  return [
    `方案：${preferences.planTitles.join("、")}`,
    ...Object.values(preferences.elements).flatMap((item) => item.notes),
  ].join("\n");
});

// Each 大運 against the four natal pillars, same rules as the 「大運 × 原局」 strip.
const luckNatalRelations = computed(
  () =>
    new Map(
      rows.value.map((row) => {
        const item: InteractionPillarItem = {
          key: `luck-${row.index}`,
          label: "大運",
          stem: ganZhiStem(row.ganZhi),
          branch: ganZhiBranch(row.ganZhi),
          branchHiddenStems: row.branchHiddenStems,
          branchTenGods: row.branchTenGods,
        };
        // Keep which characters each tag touches, so it can be judged against 喜忌.
        type Touch = { scope: "stem" | "branch"; luck: RelationSide; natal: RelationSide };
        const touched = new Map<string, Touch[]>();
        const collect = (text: string, scope: Touch["scope"], luckChar: string, natalChar: string) => {
          const table = scope === "stem" ? STEM_WUXING : BRANCH_WUXING;
          const list = touched.get(text) ?? [];
          const luckElement = table[luckChar];
          const natalElement = table[natalChar];
          if (luckElement && natalElement) {
            list.push({
              scope,
              luck: { char: luckChar, element: luckElement },
              natal: { char: natalChar, element: natalElement },
            });
          }
          touched.set(text, list);
        };
        natalItems.value.forEach((natal) => {
          summarizeStemInteractions(item, natal).forEach((text) => collect(text, "stem", item.stem, natal.stem));
          summarizeBranchInteractions(item, natal).forEach((text) => collect(text, "branch", item.branch, natal.branch));
        });
        const preferences = lessonFavor.value;
        return [
          row.index,
          [...touched.entries()].map(([text, touches]): RelationTag => ({
            text,
            tone: relationTone(text),
            judgement:
              preferences && touches.length
                ? combineJudgements(
                    touches.map((touch) =>
                      judgeLuckRelation(text, touch.scope, touch.luck, touch.natal, preferences),
                    ),
                  )
                : undefined,
          })),
        ];
      }),
    ),
);

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
    descendingDisplayRows(selectedLuckRow.value?.liuNian)[0] ??
    null,
);

const selectedMonthRow = computed(
  () =>
    (showMonthLane.value
      ? selectedAnnualRow.value?.liuYue?.find((month) => month.index === selectedMonthIndex.value) ??
        descendingDisplayRows(selectedAnnualRow.value?.liuYue)[0]
      : null) ??
    null,
);

const displayedAnnualRows = computed(() => descendingDisplayRows(selectedLuckRow.value?.liuNian));

const basicPattern = computed(() => {
  const natal = natalItems.value.slice(0, 4).map((item) => `${item.stem}${item.branch}`);
  return natal.length === 4 ? basicStructure(natal, props.result.quantModel?.luStrength?.verdict).pattern.name : "";
});

const luckEventContext = computed<LuckEventContext>(() => {
  const lu = props.result.quantModel?.luStrength;
  return {
    natal: natalItems.value.slice(0, 4).map((item) => `${item.stem}${item.branch}`),
    strength: lu?.category,
    rootless: lu ? lu.roots.length === 0 : undefined,
    verdict: lu?.verdict,
    pattern: basicPattern.value,
  };
});

const topicContext = computed<TopicContext>(() => ({
  natal: luckEventContext.value.natal,
  gender: props.result.gender,
  strength: luckEventContext.value.strength,
}));

// 配偶星只藏在庫中時，沖開該庫的流年依時間順序分成婚期候選與婚變留意（命例 98）。
const spouseStorageByYear = computed(() =>
  spouseStorageYears(
    topicContext.value,
    rows.value.flatMap((row) => row.liuNian ?? []).map((annual) => ({ year: annual.year, ganZhi: annual.ganZhi, age: annual.age })),
  ),
);

const luckEventsByRow = computed(
  () =>
    new Map(
      rows.value.map((row) => [
        row.index,
        [
          ...majorLuckEvents(row.ganZhi, luckEventContext.value),
          ...topicEvents(row.ganZhi, "運", topicContext.value),
          ...movingGodEvents(row.ganZhi, "運", luckEventContext.value.natal),
          ...patternLuckEvents(row.ganZhi, "運", luckEventContext.value.natal, basicPattern.value),
        ],
      ]),
    ),
);

function spouseStorageEvent(year: number): LuckEvent[] {
  const state = spouseStorageByYear.value.get(year);
  if (!state) return [];
  const star = props.result.gender === "FEMALE" ? "夫星" : "妻星";
  return [
    {
      kind: "marriage-open-storage",
      label: state === "first" ? `沖開${star}庫` : `再沖${star}庫`,
      level: state === "first" ? "info" : "warn",
      topic: "marriage",
      detail:
        state === "first"
          ? `${star}只藏在庫中，這是二十歲後第一次被流年沖開：作者觀察常是結婚時機（命例 98）。`
          : `${star}庫再次被沖：作者觀察已婚者此時往往婚變（命例 98）。`,
    },
  ];
}

const annualEventsByYear = computed(
  () =>
    new Map(
      (selectedLuckRow.value?.liuNian ?? []).map((annual) => {
        const luck = selectedLuckRow.value?.ganZhi ?? "";
        return [
          annual.year,
          [
            ...annualEvents(annual.ganZhi, luck, luckEventContext.value),
            ...spouseStorageEvent(annual.year),
            ...topicEvents(annual.ganZhi, "歲", topicContext.value, luck),
            ...movingGodEvents(annual.ganZhi, "歲", luckEventContext.value.natal, luck),
            ...patternLuckEvents(annual.ganZhi, "歲", luckEventContext.value.natal, basicPattern.value),
          ],
        ];
      }),
    ),
);

const selectedEventNotes = computed(() => {
  const notes: { key: string; scope: string; label: string; level: string; detail: string; topic?: string }[] = [];
  const luck = selectedLuckRow.value;
  if (luck) {
    (luckEventsByRow.value.get(luck.index) ?? []).forEach((event) =>
      notes.push({ key: `luck-${event.kind}`, scope: `大運${luck.ganZhi}`, ...event }),
    );
  }
  const annual = showAnnualLane.value ? selectedAnnualRow.value : null;
  if (annual) {
    (annualEventsByYear.value.get(annual.year) ?? []).forEach((event) =>
      notes.push({ key: `annual-${event.kind}`, scope: `${annual.year}${annual.ganZhi}`, ...event }),
    );
  }
  return notes;
});

const displayedMonthRows = computed(() => descendingDisplayRows(selectedAnnualRow.value?.liuYue));

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
      "流年 × 原局",
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
      "大運 × 原局",
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
      "流年 × 大運",
      [selectedAnnualItem.value, selectedLuckItem.value],
      summarizeGroupInteractions([selectedAnnualItem.value], [selectedLuckItem.value], summarizePair),
    );
    if (summary) {
      summaries.push(summary);
    }
  }

  const natalSummary = buildSummaryItem(
    "natal",
    "原局本身",
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

  const summary = buildSummaryItem("monthAnnual", "流月 × 流年", [selectedAnnualItem.value], parts);
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
  const defaultAnnuals = descendingDisplayRows(defaultLuck?.liuNian);
  const defaultAnnual =
    defaultLuck?.liuNian?.find((annual) => isCurrentAnnualRow(annual)) ?? defaultAnnuals[0] ?? null;
  const defaultMonths = descendingDisplayRows(defaultAnnual?.liuYue);
  selectedLuckIndex.value = defaultLuck?.index ?? null;
  selectedAnnualYear.value = defaultAnnual?.year ?? null;
  selectedMonthIndex.value = defaultMonths[0]?.index ?? null;
}

function selectLuckRow(row: DisplayDaYun) {
  const defaultAnnuals = descendingDisplayRows(row.liuNian);
  const defaultAnnual = row.liuNian?.find((annual) => isCurrentAnnualRow(annual)) ?? defaultAnnuals[0] ?? null;
  const defaultMonths = descendingDisplayRows(defaultAnnual?.liuYue);
  selectedLuckIndex.value = row.index;
  selectedAnnualYear.value = defaultAnnual?.year ?? null;
  selectedMonthIndex.value = defaultMonths[0]?.index ?? null;
  emitLuckPreview(row);
}

function selectAnnualRow(row: LiuNian) {
  selectedAnnualYear.value = row.year;
  selectedMonthIndex.value = descendingDisplayRows(row.liuYue)[0]?.index ?? null;
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
      row.liuNian?.find((annual) => isCurrentAnnualRow(annual))?.year ??
      descendingDisplayRows(row.liuNian)[0]?.year ??
      null;
  }
});

watch(selectedAnnualRow, (row) => {
  if (!row) {
    selectedMonthIndex.value = null;
    return;
  }
  if (!row.liuYue?.some((month) => month.index === selectedMonthIndex.value)) {
    selectedMonthIndex.value = descendingDisplayRows(row.liuYue)[0]?.index ?? null;
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
  font-size: calc(14px * var(--app-font-scale));
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
  font-size: calc(12px * var(--app-font-scale));
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
  font-size: calc(20px * var(--app-font-scale));
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
  grid-template-rows: auto minmax(72px, auto) minmax(72px, auto) auto;
  height: auto;
  min-height: 248px;
  padding-top: 18px;
  align-content: start;
}

.fate-relation-list {
  display: flex;
  flex-wrap: wrap;
  align-content: flex-start;
  gap: 3px;
}

.fate-relation-tag {
  padding: 0 5px;
  border: 1px solid currentColor;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.85);
  font-size: calc(11px * var(--app-font-scale));
  font-weight: 600;
  line-height: 1.55;
  white-space: nowrap;
}

.fate-relation-tag.is-combine {
  color: #2f7d45;
}

.fate-relation-tag.is-clash {
  color: #b42318;
}

.fate-relation-tag.is-punish {
  color: #c2410c;
}

.fate-relation-tag.is-break {
  color: #64748b;
}

.fate-relation-tag.is-echo {
  color: #7c3aed;
}

/* 喜忌 verdict: keep the relation color, mark the result with a filled corner and an arrow. */
.fate-relation-tag.favor-good {
  background: rgba(47, 125, 69, 0.12);
  box-shadow: inset 3px 0 0 #2f7d45;
}

.fate-relation-tag.favor-bad {
  background: rgba(180, 35, 24, 0.1);
  box-shadow: inset 3px 0 0 #b42318;
}

.favor-mark {
  margin-left: 2px;
  font-size: 0.85em;
}

.favor-good .favor-mark {
  color: #2f7d45;
}

.favor-bad .favor-mark {
  color: #b42318;
}

.luck-favor-line {
  margin: 2px 0 0;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
  cursor: help;
}

.fate-cell-year {
  color: #0f172a;
  font-size: calc(14px * var(--app-font-scale));
  min-height: 20px;
}

.fate-cell-topline {
  display: grid;
  gap: 1px;
  color: #0f172a;
  font-size: calc(12px * var(--app-font-scale));
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
  font-size: calc(22px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1;
}

.fate-cell-major .fate-char-row .wuxing-char {
  font-size: calc(24px * var(--app-font-scale));
}

.ten-god-text {
  color: #111827;
  font-size: calc(12px * var(--app-font-scale));
}

.hidden-ten-god-text {
  color: #475569;
  font-size: calc(11px * var(--app-font-scale));
}

.fate-cell-meta {
  color: #64748b;
  font-size: calc(12px * var(--app-font-scale));
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
  font-size: calc(13px * var(--app-font-scale));
}

.fate-event-list {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
  margin-top: 3px;
}

.fate-event-tag {
  padding: 0 5px;
  border-radius: 6px;
  font-size: calc(11px * var(--app-font-scale));
  font-weight: 600;
  line-height: 1.6;
  white-space: nowrap;
}

.fate-event-tag.is-warn {
  color: #fff;
  background: #b45309;
}

.fate-event-tag.is-info {
  color: #44403c;
  background: #e7e5e4;
}

.luck-event-notes {
  display: grid;
  gap: 6px;
  margin-top: 8px;
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(180, 83, 9, 0.06);
  font-size: calc(13px * var(--app-font-scale));
  color: #44403c;
}

.luck-event-notes-title {
  color: #78716c;
  font-size: calc(12px * var(--app-font-scale));
}

.luck-event-note {
  display: flex;
  gap: 8px;
  align-items: baseline;
  line-height: 1.55;
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
  font-size: calc(12px * var(--app-font-scale));
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
  font-size: calc(14px * var(--app-font-scale));
  line-height: 1.35;
}

.interaction-summary-strip {
  border-left: 4px solid transparent;
  color: var(--bazi-text);
}

.strip-label {
  margin-right: 8px;
  font-weight: 700;
}

.interaction-summary-strip.is-empty {
  color: var(--bazi-text-subtle);
}

/* 流月 × 流年 */
.tone-1 {
  background: #f5f0fb;
  border-left-color: #7c3aed;
}

/* 流年 × 原局 */
.tone-2 {
  background: #fdf0f3;
  border-left-color: #be185d;
}

/* 大運 × 原局 */
.tone-3 {
  background: #fff4e5;
  border-left-color: var(--bazi-accent);
}

/* 原局本身 */
.tone-4 {
  background: #f6efe6;
  border-left-color: var(--bazi-primary);
}

/* 流年 × 大運 */
.tone-5 {
  background: #fdf7e3;
  border-left-color: var(--bazi-gold);
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
  font-size: calc(14px * var(--app-font-scale));
}

.interaction-pillar-main {
  display: flex;
  align-items: baseline;
  gap: 4px;
  min-height: 28px;
}

.interaction-pillar-main .wuxing-char {
  font-size: calc(22px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1;
}

.interaction-pillar-main .ten-god-text {
  font-size: calc(13px * var(--app-font-scale));
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
  font-size: calc(14px * var(--app-font-scale));
  font-weight: 500;
}

.interaction-pillar-hidden-item .hidden-ten-god-text {
  font-size: calc(10px * var(--app-font-scale));
}

.wuxing-char {
  display: inline-block;
  min-width: 1.2em;
  text-align: center;
}

.wuxing-wood {
  color: var(--wx-wood);
}

.wuxing-fire {
  color: var(--wx-fire);
}

.wuxing-earth {
  color: var(--wx-earth);
}

.wuxing-metal {
  color: var(--wx-metal);
}

.wuxing-water {
  color: var(--wx-water);
}

.luck-panel.is-compact .luck-start-line {
  font-size: calc(10px * var(--app-font-scale));
  line-height: 1.35;
}

.luck-panel.is-compact .luck-board {
  gap: 8px;
}

.luck-panel.is-compact .luck-lane {
  grid-template-columns: 34px minmax(0, 1fr);
}

.luck-panel.is-compact .lane-label {
  font-size: calc(11px * var(--app-font-scale));
  line-height: 1.2;
}

.luck-panel.is-compact .lane-track {
  grid-auto-columns: minmax(88px, 88px);
}

.luck-panel.is-compact .fate-cell,
.luck-panel.is-compact .fate-relation-tag {
  padding: 0 3px;
  font-size: calc(9px * var(--app-font-scale));
}

.luck-panel.is-compact .fate-cell-major,
.luck-panel.is-compact .fate-cell-annual {
  height: 286px;
  padding: 7px 8px 9px;
  gap: 2px;
}

.luck-panel.is-compact .fate-cell-major {
  height: auto;
  min-height: 286px;
}

.luck-panel.is-compact .fate-cell-month {
  height: 334px;
  grid-template-rows: minmax(24px, auto) minmax(22px, auto) minmax(56px, auto) minmax(112px, auto) minmax(14px, auto);
}

.luck-panel.is-compact .fate-cell-year {
  font-size: calc(10px * var(--app-font-scale));
  min-height: 28px;
  line-height: 1.15;
  white-space: pre-line;
}

.luck-panel.is-compact .fate-cell-topline,
.luck-panel.is-compact .fate-cell-meta {
  font-size: calc(9px * var(--app-font-scale));
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
  font-size: calc(20px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1;
}

.luck-panel.is-compact .ten-god-text {
  font-size: calc(11px * var(--app-font-scale));
  line-height: 1.15;
  letter-spacing: 0;
}

.luck-panel.is-compact .hidden-ten-god-text {
  font-size: calc(10px * var(--app-font-scale));
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
  font-size: calc(14px * var(--app-font-scale));
  font-weight: 500;
  line-height: 1;
}

.luck-panel.is-compact .luck-detail-bar {
  gap: 6px 12px;
  font-size: calc(10px * var(--app-font-scale));
}

.luck-panel.is-compact .interaction-summary-title,
.luck-panel.is-compact .interaction-summary-strip,
.luck-panel.is-compact .interaction-pillar-label {
  font-size: calc(10px * var(--app-font-scale));
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
  font-size: calc(15px * var(--app-font-scale));
}

.luck-panel.is-compact .interaction-pillar-main .ten-god-text,
.luck-panel.is-compact .interaction-pillar-hidden-item .wuxing-char {
  font-size: calc(9px * var(--app-font-scale));
}

.luck-panel.is-compact .interaction-pillar-hidden-item .hidden-ten-god-text {
  font-size: calc(8px * var(--app-font-scale));
}

@media (max-width: 960px) {
  .luck-lane {
    grid-template-columns: 44px minmax(0, 1fr);
  }

  .lane-label {
    font-size: calc(16px * var(--app-font-scale));
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
    font-size: calc(13px * var(--app-font-scale));
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
    font-size: calc(18px * var(--app-font-scale));
  }

  .luck-panel.is-compact .ten-god-text {
    font-size: calc(10px * var(--app-font-scale));
  }

  .luck-panel.is-compact .hidden-ten-god-text {
    font-size: calc(9px * var(--app-font-scale));
  }

  .luck-panel.is-compact .fate-hidden-item .wuxing-char {
    font-size: calc(13px * var(--app-font-scale));
  }

  .luck-panel.is-compact .interaction-summary-strip,
  .luck-panel.is-compact .interaction-pillar-label,
  .luck-panel.is-compact .interaction-summary-title {
    font-size: calc(9px * var(--app-font-scale));
  }
}


.compact-lane-toggle {
  border-color: var(--bazi-border);
  background: rgba(255, 253, 248, 0.94);
  color: var(--bazi-primary);
  font-weight: 800;
}

.compact-lane-toggle:hover,
.compact-lane-toggle:focus-visible {
  border-color: var(--bazi-border-strong);
  box-shadow: 0 0 0 3px var(--bazi-ring);
}

.compact-lane-toggle.is-active {
  background: var(--bazi-primary);
  border-color: var(--bazi-primary);
  color: #fffaf0;
}

.luck-lane {
  border-color: var(--bazi-border);
  border-radius: 18px;
  background: rgba(255, 253, 248, 0.78);
  overflow: hidden;
}

.lane-label {
  border-right-color: var(--bazi-border);
  background: linear-gradient(180deg, #fff6e8, #f5dfbd);
  color: var(--bazi-primary);
  font-family: "Noto Serif TC", "PMingLiU", "MingLiU", serif;
  font-weight: 800;
}

.fate-cell {
  border-right-color: rgba(174, 128, 72, 0.2);
  background: rgba(255, 253, 248, 0.88);
  transition: background var(--bazi-transition), box-shadow var(--bazi-transition), transform var(--bazi-transition);
}

.fate-cell:hover {
  background: #fff5e3;
  box-shadow: inset 0 0 0 1px rgba(180, 83, 9, 0.16);
}

.fate-cell.is-selected {
  background: linear-gradient(180deg, #f8e7bc, #fff7e8);
  box-shadow: inset 0 0 0 2px var(--bazi-gold);
}

.fate-cell.is-current:not(.is-selected) {
  background: #fff1dc;
  box-shadow: inset 0 -3px 0 var(--bazi-accent);
}

.current-badge {
  position: absolute;
  right: 8px;
  bottom: 8px;
  padding: 1px 8px;
  border-radius: 999px;
  background: var(--bazi-accent);
  color: #fffaf0;
  font-size: calc(11px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.6;
  pointer-events: none;
}

/* 大運格底部放關係標籤，所以「目前」改成貼在上緣的小頁籤 */
.current-badge.is-tab {
  top: 0;
  right: auto;
  bottom: auto;
  left: 50%;
  padding: 0 10px;
  border-radius: 0 0 8px 8px;
  font-size: calc(10px * var(--app-font-scale));
  line-height: 1.5;
  transform: translateX(-50%);
}

.fate-cell-year,
.fate-cell-topline {
  color: var(--bazi-primary-strong);
  font-weight: 800;
}

.ten-god-text {
  color: var(--bazi-text);
  font-weight: 700;
}

.hidden-ten-god-text,
.fate-cell-meta,
.lane-empty {
  color: var(--bazi-text-muted);
}

.luck-detail-bar,
.interaction-matrix-shell {
  border-color: var(--bazi-border);
  background: rgba(255, 250, 243, 0.82);
}

/* 流年格子會多出事件提示，高度隨內容延伸（同一列的格子會一起拉齊）。 */
.fate-cell.fate-cell-annual {
  height: auto;
  min-height: 240px;
  grid-template-rows: auto minmax(72px, auto) minmax(72px, auto);
  grid-auto-rows: auto;
  align-content: start;
}

.luck-panel.is-compact .fate-cell.fate-cell-annual {
  height: auto;
  min-height: 286px;
}

.luck-panel.is-compact .fate-event-tag {
  padding: 0 3px;
  font-size: calc(9px * var(--app-font-scale));
}

.luck-event-note .fate-event-tag {
  flex-shrink: 0;
}

.topic-mark {
  margin-right: 2px;
  font-weight: 800;
}

.fate-event-tag.is-topic-marriage {
  color: #9d174d;
  background: #fce7f3;
}

.fate-event-tag.is-topic-marriage.is-warn {
  color: #fff;
  background: #be185d;
}

.fate-event-tag.is-topic-wealth {
  color: #854d0e;
  background: #fef3c7;
}

.fate-event-tag.is-topic-wealth.is-warn {
  color: #fff;
  background: #a16207;
}

.fate-event-tag.is-topic-health {
  color: #115e59;
  background: #ccfbf1;
}

.fate-event-tag.is-topic-health.is-warn {
  color: #fff;
  background: #0f766e;
}

.fate-event-tag.is-topic-move {
  color: #3730a3;
  background: #e0e7ff;
}
</style>
