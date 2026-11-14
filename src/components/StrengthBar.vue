<template>
  <div class="sbar" :class="{ compact }">
    <!-- 勢力條：同黨在左、異黨在右，分界線偏離中線多少，就是人數差多少 -->
    <div class="sbar-force" aria-label="同黨和異黨的人數">
      <span class="sbar-side t-same"><small>同黨</small><b>{{ now.same }}</b></span>
      <div class="sbar-track">
        <span class="sbar-center" aria-hidden="true"></span>
        <TransitionGroup name="tok" tag="div" class="sbar-tokens">
          <span
            v-for="token in tokens"
            :key="token.key"
            class="tok"
            :class="[`k-${token.kind}`, { dm: token.dayMaster, luck: token.luck, focus: token.focus }]"
            :title="token.title"
          >{{ token.text }}<i v-if="token.month" class="tok-badge">令</i><i v-else-if="token.luck" class="tok-badge luck">{{ token.luckMark }}</i></span>
        </TransitionGroup>
      </div>
      <span class="sbar-side t-opp"><b>{{ now.opp }}</b><small>異黨</small></span>
    </div>

    <!-- 三盞燈＋判定 -->
    <div class="sbar-lights">
      <span class="lamp" :class="{ on: shift.lights.season }" :title="shift.lights.season ? '月令和日主同五行' : '月令和日主不同五行'">{{ shift.lights.season ? "●" : "○" }} 得令</span>
      <span class="lamp" :class="{ on: natalNow.momentum, flip: natalNow.momentum !== shift.before.momentum }" :title="momentumText">{{ natalNow.momentum ? "●" : "○" }} 得勢</span>
      <span class="lamp" :class="{ on: shift.lights.rooted, warn: rootShaken.length > 0 }" :title="rootText">{{ shift.lights.rooted ? "●" : "○" }} 得地</span>
      <span class="verdict" :class="{ changed: verdictChanged }">
        <template v-if="verdictChanged"><s>{{ shift.before.classification }}</s> → </template>
        <b>{{ natalNow.classification || "…" }}</b>
      </span>
      <span v-if="shift.luck && luckEntered" class="lean" :class="leanClass(shift.luck)">大運{{ shift.luck.ganZhi }}：{{ leanShort(shift.luck) }}</span>
      <span v-if="shift.annual && annualEntered" class="lean" :class="leanClass(shift.annual)">流年{{ shift.annual.ganZhi }}：{{ leanShort(shift.annual) }}</span>
    </div>

    <!-- 步驟控制 -->
    <div class="sbar-stepper" role="group" aria-label="作用步驟">
      <button type="button" class="ctl" :class="{ on: applied === 0 }" title="回到還沒有任何作用的原局" @click="go(0)">原局</button>
      <button type="button" class="ctl icon" :disabled="applied === 0" aria-label="上一步" @click="go(applied - 1)">‹</button>
      <button type="button" class="step-now" :class="{ empty: !current }" @click="current && toggleList()">
        <template v-if="current">
          <span class="no">{{ focus + 1 }}/{{ total }}</span>
          <span class="tag">{{ current.tag }}</span>
          <span class="title">{{ current.title }}</span>
          <span class="kind" :class="kindOf(current).cls">{{ kindOf(current).text }}</span>
        </template>
        <template v-else>
          <span class="no">{{ applied }}/{{ total }}</span>
          <span class="title muted">{{ applied === 0 ? "原局：還沒套用任何作用" : applied === total ? "已套用全部作用" : `已套用前 ${applied} 步` }}</span>
        </template>
      </button>
      <button type="button" class="ctl icon" :disabled="applied === total" aria-label="下一步" @click="go(applied + 1)">›</button>
      <button type="button" class="ctl" :class="{ on: applied === total && focus < 0 }" @click="showAll">全部</button>
      <button v-if="total > 1" type="button" class="ctl" @click="togglePlay">{{ playing ? "■" : "▶ 播放" }}</button>
    </div>

    <div v-if="current" class="sbar-detail">
      <p><GlossaryText :text="current.detail" /></p>
      <p class="rule"><GlossaryText :text="current.rule" /></p>
    </div>
    <div v-else class="sbar-detail summary">
      <p v-for="(line, idx) in summaryLines" :key="idx">{{ line }}</p>
      <p v-if="shift.caution && applied === total" class="caution">⚠ {{ shift.caution }}</p>
    </div>

    <!-- 全部步驟 -->
    <button type="button" class="sbar-more" :aria-expanded="listOpen" @click="toggleList">
      {{ listOpen ? "收起步驟" : `看全部 ${total} 個步驟` }}
    </button>
    <div v-if="listOpen" class="sbar-list">
      <template v-for="group in groups" :key="group.key">
        <div v-if="group.items.length" class="sbar-group">
          <span class="group-title">{{ group.title }}</span>
          <button
            v-for="item in group.items"
            :key="item.step.id"
            type="button"
            class="chip"
            :class="[kindOf(item.step).cls, { current: focus === item.index, pending: item.index >= applied, off: item.step.inactive }]"
            @click="pick(item.index)"
          >
            <span class="chip-no">{{ item.index + 1 }}</span>{{ item.step.title }}
          </button>
        </div>
      </template>
    </div>

    <p class="sbar-legend">
      <span class="sw same"></span>同黨（印、比劫）
      <span class="sw opp"></span>異黨（食傷、財、官殺）
      <span class="lg same">↑↓</span>橘＝對日主這邊有利
      <span class="lg opp">↑↓</span>藍＝對異黨有利
      <span class="lg-note">只有合化與成局改人數，其他只標方向，不是分數。</span>
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import {
  bonusesAt,
  cellStatesAt,
  partyName,
  trendAt,
  type ShiftCount,
  type ShiftLuckInfo,
  type ShiftStep,
  type StrengthShift,
} from "../utils/strengthShift";
import GlossaryText from "./GlossaryText.vue";

// 四柱卡片下方的「強弱」列：勢力條、三盞燈、步驟控制。字上的標記由 PillarPanel 依同一個 applied 畫。

const props = defineProps<{ shift: StrengthShift; applied: number; focus: number; compact?: boolean }>();
const emit = defineEmits<{ "update:applied": [value: number]; "update:focus": [value: number] }>();

const total = computed(() => props.shift.steps.length);
const current = computed<ShiftStep | null>(() => props.shift.steps[props.focus] ?? null);
const listOpen = ref(false);
const playing = ref(false);
let timer: ReturnType<typeof setInterval> | null = null;

function set(applied: number, focus: number) {
  emit("update:applied", applied);
  emit("update:focus", focus);
}
function stop() {
  if (timer) clearInterval(timer);
  timer = null;
  playing.value = false;
}
function go(count: number, keepPlaying = false) {
  if (!keepPlaying) stop();
  const next = Math.min(Math.max(count, 0), total.value);
  set(next, next - 1);
}
function showAll() {
  stop();
  set(total.value, -1);
}
function pick(index: number) {
  stop();
  if (props.focus === index) set(props.applied, -1);
  else set(index + 1, index);
}
function toggleList() {
  listOpen.value = !listOpen.value;
}
function togglePlay() {
  if (playing.value) return stop();
  playing.value = true;
  go(0, true);
  let count = 0;
  timer = setInterval(() => {
    count += 1;
    if (count > total.value) return stop();
    go(count, true);
  }, 1700);
}
watch(() => props.shift, stop);
onBeforeUnmount(stop);

const now = computed<ShiftCount>(() => props.shift.timeline[props.applied] ?? props.shift.after);
/** 得勢、判定看的是原局：加了大運之後仍用原局作用後的結果。 */
const natalNow = computed<ShiftCount>(() => (now.value.withLuck ? props.shift.after : now.value));
const verdictChanged = computed(() => !!natalNow.value.classification && natalNow.value.classification !== props.shift.before.classification);
const luckEntered = computed(() => !!props.shift.luck && props.applied > props.shift.luck.firstStep);
const annualEntered = computed(() => !!props.shift.annual && props.applied > props.shift.annual.firstStep);
const rootShaken = computed(() => [
  ...props.shift.lights.rootShaken,
  ...(luckEntered.value ? props.shift.luck?.rootShaken ?? [] : []),
  ...(annualEntered.value ? props.shift.annual?.rootShaken ?? [] : []),
]);
const rootText = computed(() =>
  props.shift.lights.rooted ? (rootShaken.value.length ? `有根，但 ${rootShaken.value.join("、")} 被沖` : "地支有日主的根") : "地支沒有日主的根",
);
const momentumText = computed(() => {
  const count = natalNow.value;
  if (count.same !== count.opp) return `同黨 ${count.same}、異黨 ${count.opp}`;
  return `${count.same}：${count.opp} 平手，月令在${props.shift.monthCommandSameParty ? "同黨" : "異黨"}這邊`;
});
const leanClass = (luck: ShiftLuckInfo) => (luck.lean > 0 ? "l-same" : luck.lean < 0 ? "l-opp" : "l-none");
function leanShort(luck: ShiftLuckInfo) {
  if (props.applied < total.value) return "作用還沒看完";
  const dir = luck.lean > 0 ? "推向日主" : luck.lean < 0 ? "推向異黨" : "大致相抵";
  const advice = luck.advice?.match(/照扶抑的大方向是(.)的/)?.[1];
  return advice ? `${dir}，扶抑${advice}` : dir;
}
const summaryLines = computed(() => {
  if (props.applied === 0) return ["這是原局八個字的樣子。按「›」一步一步套上合、沖、刑、會，或按「播放」。"];
  if (props.applied < total.value) {
    const trend = trendAt(props.shift, props.applied);
    return [`已套用前 ${props.applied} 步：同黨 ${now.value.same}、異黨 ${now.value.opp}；只看方向的作用對日主有利 ${trend.sameUp + trend.oppDown} 項、對異黨有利 ${trend.sameDown + trend.oppUp} 項。`];
  }
  return props.shift.summary;
});

const states = computed(() => cellStatesAt(props.shift, props.applied));
const focusCells = computed(() => new Set(current.value?.cells ?? []));

interface Token {
  key: string;
  kind: string;
  text: string;
  title: string;
  dayMaster: boolean;
  month: boolean;
  luck: boolean;
  luckMark?: string;
  focus: boolean;
}
const tokens = computed<Token[]>(() => {
  const same: Token[] = [];
  const opp: Token[] = [];
  for (const cell of props.shift.cells) {
    if (cell.luck && cell.pillar >= 5 ? !annualEntered.value : cell.luck && !luckEntered.value) continue;
    const state = states.value.get(cell.id)!;
    const token: Token = {
      key: cell.id,
      kind: state.party,
      text: cell.char,
      title: `${cell.id}${cell.char}：${state.transformed ? `化${state.element}，` : ""}${cell.isDayMaster ? "日主" : `${partyName(state.party)}・${state.family}`}`,
      dayMaster: cell.isDayMaster,
      month: cell.isMonthCommand,
      luck: !!cell.luck,
      luckMark: cell.pillar >= 5 ? "年" : "運",
      focus: focusCells.value.has(cell.id),
    };
    (state.party === "same" ? same : opp).push(token);
  }
  for (const bonus of bonusesAt(props.shift, props.applied)) {
    const token: Token = {
      key: `bonus-${bonus.id}`,
      kind: `bonus-${bonus.party}`,
      text: `＋${bonus.label}`,
      title: `成局後這個五行力量大增，給${partyName(bonus.party)}加一`,
      dayMaster: false,
      month: false,
      luck: false,
      focus: false,
    };
    (bonus.party === "same" ? same : opp).push(token);
  }
  // 日主放最左；成局的合力貼著分界線
  same.sort((a, b) => Number(b.dayMaster) - Number(a.dayMaster) || Number(a.kind.startsWith("bonus")) - Number(b.kind.startsWith("bonus")));
  opp.sort((a, b) => Number(b.kind.startsWith("bonus")) - Number(a.kind.startsWith("bonus")));
  return [...same, { key: "divider", kind: "divider", text: "", title: "同黨和異黨的分界", dayMaster: false, month: false, luck: false, focus: false }, ...opp];
});

function kindOf(step: ShiftStep) {
  if (step.inactive) return { text: "不成立", cls: "k-off" };
  if (step.counted) {
    const parts: string[] = [];
    if (step.deltaSame) parts.push(`同${step.deltaSame > 0 ? "+" : "−"}${Math.abs(step.deltaSame)}`);
    if (step.deltaOpp) parts.push(`異${step.deltaOpp > 0 ? "+" : "−"}${Math.abs(step.deltaOpp)}`);
    return { text: parts.join(" ") || "人數不變", cls: "k-count" };
  }
  return { text: step.effects.length ? "方向" : "—", cls: "k-dir" };
}

const groups = computed(() => {
  const defs = [
    { key: "stem", title: "天干" },
    { key: "branch", title: "地支" },
    { key: "pillar", title: "同柱" },
    { key: "luck", title: `大運${props.shift.luck?.ganZhi ?? ""}` },
    { key: "annual", title: `流年${props.shift.annual?.ganZhi ?? ""}` },
  ];
  return defs.map((def) => ({
    ...def,
    items: props.shift.steps.map((step, index) => ({ step, index })).filter((item) => item.step.group === def.key),
  }));
});
</script>

<style scoped>
.sbar {
  --same: #c2620a;
  --same-bg: #fff1dc;
  --opp: #3d6a96;
  --opp-bg: #eaf1f8;
  display: grid;
  gap: 8px;
  margin-top: 4px;
  padding: 12px 14px;
  border: 1px solid var(--bazi-border, rgba(174, 128, 72, 0.25));
  border-radius: 14px;
  background: linear-gradient(180deg, #fffdf8, #fff8ec);
  font-size: calc(14px * var(--app-font-scale));
  line-height: 1.55;
}

.t-same { color: var(--same); }
.t-opp { color: var(--opp); }

/* 勢力條 */
.sbar-force {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
}
.sbar-side { display: inline-flex; align-items: baseline; gap: 4px; }
.sbar-side small { font-size: 0.8em; font-weight: 700; }
.sbar-side b { font-size: 1.5em; line-height: 1; font-variant-numeric: tabular-nums; }
.sbar-track { position: relative; padding: 4px 0; min-width: 0; }
.sbar-center {
  position: absolute;
  top: -2px;
  bottom: -2px;
  left: 50%;
  border-left: 2px dashed rgba(107, 114, 128, 0.45);
}
.sbar-tokens {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: stretch;
  gap: 3px;
}
.tok {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 1 1.9em;
  min-width: 0;
  height: 1.9em;
  border: 2px dashed var(--opp);
  border-radius: 6px;
  background: var(--opp-bg);
  color: var(--opp);
  font-weight: 800;
  transition: box-shadow 0.25s;
}
.tok.k-same { border: 2px solid var(--same); background: var(--same-bg); color: var(--same); }
.tok.dm { background: var(--same); color: #fff; }
.tok.luck { border-style: double; border-width: 4px; }
.tok.focus { box-shadow: 0 0 0 3px rgba(234, 179, 8, 0.55); }
.tok.k-bonus-same,
.tok.k-bonus-opp {
  flex: 0 0 auto;
  padding: 0 0.3em;
  font-size: 0.72em;
  white-space: nowrap;
  border-style: solid;
}
.tok.k-bonus-same {
  border-color: var(--same);
  color: var(--same);
  background: repeating-linear-gradient(45deg, #fff, #fff 4px, var(--same-bg) 4px, var(--same-bg) 8px);
}
.tok.k-bonus-opp { background: repeating-linear-gradient(45deg, #fff, #fff 4px, var(--opp-bg) 4px, var(--opp-bg) 8px); }
.tok.k-divider { flex: 0 0 4px; border: 0; border-radius: 2px; background: #1f2937; }
.tok-badge {
  position: absolute;
  top: -0.7em;
  right: -0.5em;
  padding: 0 0.25em;
  border-radius: 4px;
  background: #7c3aed;
  color: #fff;
  font-size: 0.58em;
  font-style: normal;
  line-height: 1.4;
}
.tok-badge.luck { background: #b45309; }
.tok-move, .tok-enter-active, .tok-leave-active { transition: transform 0.55s ease, opacity 0.35s ease; }
.tok-enter-from, .tok-leave-to { opacity: 0; transform: scale(0.6); }
.tok-leave-active { position: absolute; }

/* 燈與判定 */
.sbar-lights { display: flex; flex-wrap: wrap; align-items: center; gap: 6px; }
.lamp {
  padding: 1px 9px;
  border: 1px solid #d1d5db;
  border-radius: 999px;
  background: #f6f6f4;
  color: #6b7280;
  font-size: 0.88em;
  font-weight: 700;
  cursor: help;
}
.lamp.on { border-color: var(--same); background: var(--same-bg); color: var(--same); }
.lamp.flip { box-shadow: 0 0 0 2px rgba(202, 138, 4, 0.5); }
.lamp.warn { border-style: dashed; }
.verdict {
  padding: 1px 10px;
  border: 1.5px solid #1f2937;
  border-radius: 999px;
  background: #fff;
  font-size: 0.92em;
}
.verdict s { color: #9ca3af; }
.verdict.changed { border-color: #c0392b; color: #c0392b; }
.lean {
  padding: 1px 10px;
  border-radius: 999px;
  font-size: 0.86em;
  font-weight: 700;
  background: #f3f4f6;
  color: #4b5563;
}
.lean.l-same { background: var(--same-bg); color: var(--same); }
.lean.l-opp { background: var(--opp-bg); color: var(--opp); }

/* 步驟控制 */
.sbar-stepper {
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto auto auto;
  align-items: center;
  gap: 6px;
}
.ctl {
  padding: 0.2em 0.75em;
  border: 1px solid rgba(180, 83, 9, 0.32);
  border-radius: 999px;
  background: #fffdf8;
  color: var(--bazi-primary, #92400e);
  font: inherit;
  font-size: 0.86em;
  white-space: nowrap;
  cursor: pointer;
}
.ctl.icon { padding: 0.05em 0.6em; font-size: 1.1em; line-height: 1.2; }
.ctl:disabled { opacity: 0.35; cursor: default; }
.ctl.on { background: var(--bazi-primary, #92400e); color: #fffaf0; }
.ctl:focus-visible, .step-now:focus-visible, .chip:focus-visible, .sbar-more:focus-visible {
  outline: 2px solid var(--bazi-primary, #92400e);
  outline-offset: 2px;
}
.step-now {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  padding: 4px 10px;
  border: 1px solid rgba(146, 64, 14, 0.35);
  border-radius: 10px;
  background: #fff;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}
.step-now.empty { cursor: default; border-style: dashed; }
.no { flex: none; color: #6b7280; font-size: 0.8em; font-variant-numeric: tabular-nums; }
.tag { flex: none; padding: 0 0.4em; border-radius: 4px; background: #efe3cf; font-size: 0.8em; }
.title { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 700; }
.title.muted { font-weight: 500; color: #6b7280; }
.kind { flex: none; margin-left: auto; padding: 0 0.5em; border-radius: 999px; font-size: 0.76em; font-weight: 700; white-space: nowrap; }
.kind.k-count { background: #1f2937; color: #fff; }
.kind.k-dir { border: 1px solid #9ca3af; color: #4b5563; }
.kind.k-off { color: #9ca3af; }

.sbar-detail {
  padding: 6px 10px;
  border-left: 3px solid var(--bazi-primary, #92400e);
  border-radius: 0 8px 8px 0;
  background: #fffaf0;
}
.sbar-detail p { margin: 1px 0; }
.sbar-detail .rule { color: var(--el-text-color-secondary, #606266); font-size: 0.92em; }
.sbar-detail.summary { border-left-color: #9ca3af; background: #f8f3e9; }
.sbar-detail .caution { color: #9a6a15; font-weight: 700; }

.sbar-more {
  justify-self: start;
  padding: 0;
  border: 0;
  background: none;
  color: var(--bazi-primary, #92400e);
  font: inherit;
  font-size: 0.86em;
  text-decoration: underline dotted;
  cursor: pointer;
}
.sbar-list { display: grid; gap: 6px; }
.sbar-group { display: flex; flex-wrap: wrap; align-items: center; gap: 4px; }
.group-title { min-width: 3.2em; color: #6b7280; font-size: 0.82em; font-weight: 700; }
.chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 1px 8px 1px 2px;
  border: 1px solid rgba(174, 128, 72, 0.35);
  border-radius: 999px;
  background: #fffdf8;
  color: inherit;
  font: inherit;
  font-size: 0.84em;
  cursor: pointer;
}
.chip-no {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5em;
  height: 1.5em;
  border-radius: 50%;
  background: #efe3cf;
  font-size: 0.85em;
}
.chip.k-count .chip-no { background: #1f2937; color: #fff; }
.chip.pending { opacity: 0.5; }
.chip.off { text-decoration: line-through; color: #9ca3af; }
.chip.current { border-color: var(--bazi-primary, #92400e); background: #fff3dc; opacity: 1; }

.sbar-legend {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 2px 6px;
  margin: 0;
  color: var(--el-text-color-secondary, #606266);
  font-size: 0.76em;
}
.sw { display: inline-block; width: 0.95em; height: 0.95em; border-radius: 3px; }
.sw.same { border: 2px solid var(--same); background: var(--same-bg); }
.sw.opp { border: 2px dashed var(--opp); background: var(--opp-bg); margin-left: 6px; }
.lg { margin-left: 6px; padding: 0 0.3em; border: 1px solid currentColor; border-radius: 4px; font-weight: 700; }
.lg.same { color: var(--same); }
.lg.opp { color: var(--opp); }
.lg-note { flex-basis: 100%; }

.compact { padding: 10px 8px; }
.compact .sbar-stepper { grid-template-columns: auto auto minmax(0, 1fr) auto; }
.compact .sbar-stepper .ctl:nth-last-child(-n + 2) { display: none; }
.compact .tok:not(.k-divider) { flex-basis: 1.7em; height: 1.7em; }
.compact .sbar-force { grid-template-columns: 1fr 1fr; row-gap: 4px; }
.compact .sbar-force .t-opp { grid-column: 2; grid-row: 1; justify-self: end; }
.compact .sbar-force .t-same { grid-column: 1; grid-row: 1; }
.compact .sbar-track { grid-column: 1 / -1; grid-row: 2; }
.compact .tag { display: none; }

@media (prefers-reduced-motion: reduce) {
  .tok-move, .tok-enter-active, .tok-leave-active { transition: none; }
}
</style>
