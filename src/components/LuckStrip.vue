<template>
  <div v-if="rows.length" class="luck-strip">
    <div class="luck-strip-head">
      <span class="luck-strip-title">大運</span>
      <span class="luck-strip-hint">{{ shownKey ? "上面已加入這步大運；再點一次收起，或在四柱的大運柱上點兩下換一步" : "點一步大運，就把它加到四柱旁邊，看合沖刑害" }}</span>
    </div>
    <div ref="trackRef" class="luck-strip-track">
      <button
        v-for="row in rows"
        :key="row.index"
        type="button"
        :class="['luck-strip-cell', { 'is-shown': shownKey === `luck-${row.index}`, 'is-selected': row.index === selectedIndex, 'is-current': row.isCurrent }]"
        :title="`${row.ganZhi}大運 ${row.ages}歲（${row.years}）`"
        @click="toggle(row)"
      >
        <span v-if="row.isCurrent" class="luck-strip-now">目前</span>
        <span class="luck-strip-god">{{ row.stemTenGod }}</span>
        <span class="luck-strip-gz">
          <span :class="['wuxing-char', `wuxing-${stemElement(row.ganZhi[0])}`]">{{ row.ganZhi[0] }}</span><span :class="['wuxing-char', `wuxing-${branchElement(row.ganZhi[1])}`]">{{ row.ganZhi[1] }}</span>
        </span>
        <span class="luck-strip-god">{{ row.branchTenGods?.[0] }}</span>
        <span class="luck-strip-age">{{ row.ages }}</span>
      </button>
    </div>

    <!-- 流年：加了大運之後，再點一年把流年也加到四柱 -->
    <template v-if="yearsLuck">
      <div class="luck-strip-head is-annual">
        <span class="luck-strip-title">流年</span>
        <span class="luck-strip-hint">{{ shownAnnualKey ? "上面已加入這個流年；再點一次收起，或在四柱的流年柱上點兩下換一年" : `${yearsLuck.ganZhi}大運的流年，點一年就把大運和流年一起加到四柱` }}</span>
      </div>
      <div ref="annualTrackRef" class="luck-strip-track">
        <button
          v-for="row in years"
          :key="row.index"
          type="button"
          :class="['luck-strip-cell', 'is-year', { 'is-shown': shownAnnualKey === annualKey(row.luckIndex, row.index), 'is-current': row.isCurrent }]"
          :title="`${row.year} ${row.ganZhi}流年（${row.age} 歲）`"
          @click="toggleAnnual(row)"
        >
          <span v-if="row.isCurrent" class="luck-strip-now">今年</span>
          <span class="luck-strip-god">{{ row.stemTenGod }}</span>
          <span class="luck-strip-gz">
            <span :class="['wuxing-char', `wuxing-${stemElement(row.ganZhi[0])}`]">{{ row.ganZhi[0] }}</span><span :class="['wuxing-char', `wuxing-${branchElement(row.ganZhi[1])}`]">{{ row.ganZhi[1] }}</span>
          </span>
          <span class="luck-strip-god">{{ row.branchTenGods?.[0] }}</span>
          <span class="luck-strip-age">{{ row.year }}</span>
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import type { BaziResponse, LuckPreviewPillar } from "../types/bazi";
import { annualKey, annualPreviewPillar, annualRows, luckPreviewPillar, luckRows, type AnnualRowView, type LuckRowView } from "../utils/luckPreview";
import { BRANCH_WUXING, STEM_WUXING } from "../constants/wuxing";
import { useLuckSelection } from "../composables/useLuckSelection";

// 四柱下方的精簡大運列：點一步大運，就把它當成預覽柱加到四柱，關係線一起畫出來。
// 選擇和正式的「大運流年」面板共用，兩邊同步。

const props = defineProps<{ result: BaziResponse; previewPillars?: LuckPreviewPillar[] }>();
const emit = defineEmits<{ "preview-change": [pillars: LuckPreviewPillar[] | null] }>();

const { luckIndex, annualYear, monthIndex, ensureFor } = useLuckSelection();
watch(
  () => props.result,
  (result) => ensureFor(result),
  { immediate: true },
);

const year = new Date().getFullYear();
// 和大運面板一樣由右到左（最早的在右邊）。
const rows = computed(() => luckRows(props.result, year).reverse());

const selectedIndex = computed(() => luckIndex.value);
const shownKey = computed(() => props.previewPillars?.find((item) => item.key.startsWith("luck-"))?.key ?? null);
const shownAnnualKey = computed(() => props.previewPillars?.find((item) => item.key.startsWith("annual-"))?.key ?? null);
const shownLuck = computed(() => rows.value.find((row) => `luck-${row.index}` === shownKey.value) ?? null);
// 流年列一開始就顯示：四柱上有大運就列那步大運的流年，沒有就列選到的（預設目前）大運。
const yearsLuck = computed(
  () => shownLuck.value ?? rows.value.find((row) => row.index === luckIndex.value) ?? rows.value.find((row) => row.isCurrent) ?? null,
);
// 流年也由右到左（最早的在右邊）
const years = computed(() => annualRows(yearsLuck.value, year).reverse());

const stemElement = (stem: string) => STEM_WUXING[stem] ?? "earth";
const branchElement = (branch: string) => BRANCH_WUXING[branch] ?? "earth";

// 手機上一排放不下：把選到的（沒有就目前）大運捲進畫面。
const trackRef = ref<HTMLElement | null>(null);
function revealSelected() {
  const track = trackRef.value;
  const cell = track?.querySelector<HTMLElement>(".is-shown, .is-selected, .is-current");
  if (!track || !cell) return;
  const left = cell.offsetLeft;
  if (left < track.scrollLeft || left + cell.offsetWidth > track.scrollLeft + track.clientWidth) {
    track.scrollLeft = left - (track.clientWidth - cell.offsetWidth) / 2;
  }
}
onMounted(revealSelected);
watch([() => props.result, luckIndex], () => nextTick(revealSelected));

function toggle(row: LuckRowView) {
  if (shownKey.value === `luck-${row.index}`) {
    emit("preview-change", null);
    return;
  }
  // 原本有流年柱：換到新大運裡同一個位置的流年（新大運有今年就用今年）
  const oldAnnual = props.previewPillars?.find((item) => item.key.startsWith("annual-"));
  const oldIndex = Number(oldAnnual?.key.split("-")[2]);
  if (oldAnnual) {
    const list = annualRows(row, year);
    const next = list.find((item) => item.isCurrent) ?? list.find((item) => item.index === oldIndex) ?? list[0];
    if (next) {
      luckIndex.value = row.index;
      annualYear.value = next.year;
      monthIndex.value = null;
      emit("preview-change", [luckPreviewPillar(row), annualPreviewPillar(next)]);
      return;
    }
  }
  luckIndex.value = row.index;
  annualYear.value = row.liuNian?.some((item) => item.year === year) ? year : null;
  monthIndex.value = null;
  emit("preview-change", [luckPreviewPillar(row)]);
}

function toggleAnnual(row: AnnualRowView) {
  const luck = yearsLuck.value;
  if (!luck) return;
  if (shownAnnualKey.value === annualKey(row.luckIndex, row.index)) {
    emit("preview-change", [luckPreviewPillar(luck)]);
    return;
  }
  luckIndex.value = luck.index;
  annualYear.value = row.year;
  monthIndex.value = null;
  emit("preview-change", [luckPreviewPillar(luck), annualPreviewPillar(row)]);
}

const annualTrackRef = ref<HTMLElement | null>(null);
onMounted(() => nextTick(revealAnnual));
watch(yearsLuck, () => nextTick(revealAnnual));
function revealAnnual() {
  const track = annualTrackRef.value;
  const cell = track?.querySelector<HTMLElement>(".is-shown, .is-current");
  if (track && cell) track.scrollLeft = cell.offsetLeft - (track.clientWidth - cell.offsetWidth) / 2;
}
watch(shownKey, () =>
  nextTick(() => {
    const track = annualTrackRef.value;
    const cell = track?.querySelector<HTMLElement>(".is-shown, .is-current");
    if (track && cell) track.scrollLeft = cell.offsetLeft - (track.clientWidth - cell.offsetWidth) / 2;
  }),
);
</script>

<style scoped>
.luck-strip {
  margin-top: 10px;
}
.luck-strip-head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 2px 10px;
  margin-bottom: 6px;
}
.luck-strip-title {
  font-weight: 700;
  font-size: calc(14px * var(--app-font-scale));
}
.luck-strip-hint {
  color: var(--el-text-color-secondary);
  font-size: calc(12px * var(--app-font-scale));
}
.luck-strip-track {
  position: relative;
  display: flex;
  gap: 6px;
  overflow-x: auto;
  padding: 8px 2px 4px;
  scrollbar-width: thin;
}
.luck-strip-cell {
  position: relative;
  flex: 1 0 calc(52px * var(--app-font-scale));
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 6px 4px 4px;
  border: 1px solid rgba(188, 195, 211, 0.8);
  border-radius: 10px;
  background: #fffdf8;
  font: inherit;
  cursor: pointer;
  transition: border-color 150ms ease, box-shadow 150ms ease, background 150ms ease;
}
.luck-strip-cell:hover {
  border-color: #d97706;
}
.luck-strip-cell.is-selected {
  border-color: rgba(217, 119, 6, 0.6);
}
.luck-strip-cell.is-shown {
  border-color: #d97706;
  background: #fff3d6;
  box-shadow: 0 0 0 2px rgba(217, 119, 6, 0.25);
}
.luck-strip-now {
  position: absolute;
  top: -8px;
  left: 50%;
  transform: translateX(-50%);
  padding: 0 5px;
  border-radius: 999px;
  background: #d97706;
  color: #fff;
  font-size: calc(10px * var(--app-font-scale));
  line-height: 1.5;
  white-space: nowrap;
}
/* 天干在上、地支在下，和四柱一樣直排。 */
.luck-strip-gz {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  font-size: calc(18px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.15;
}
.luck-strip-head.is-annual {
  margin-top: 8px;
  margin-bottom: 0;
}
.luck-strip-cell.is-year {
  flex-basis: calc(46px * var(--app-font-scale));
}
.luck-strip-cell.is-year .luck-strip-gz {
  font-size: calc(16px * var(--app-font-scale));
}
.luck-strip-god {
  min-height: 1.3em;
  color: #7a6a55;
  font-size: calc(10px * var(--app-font-scale));
  line-height: 1.3;
  white-space: nowrap;
}
.luck-strip-age {
  color: var(--el-text-color-secondary);
  font-size: calc(11px * var(--app-font-scale));
  white-space: nowrap;
}
</style>
