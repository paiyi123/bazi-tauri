<template>
  <el-card v-if="shift" shadow="hover" class="strength-shift" :class="{ compact }">
    <template #header>
      <div class="section-header">
        <h2>強弱變化圖</h2>
        <span class="header-sub">合、沖、刑、會一步一步看：哪個字增力、減力、換邊</span>
      </div>
    </template>

    <p class="intro">
      先數原局八個字裡，幫日主的<b class="t-same">同黨</b>（印、比劫）和耗日主的<b class="t-opp">異黨</b>（食傷、財、官殺）各有幾個，再把干支之間的作用一項一項套上去。
      按「下一步」或直接點下面的步驟，圖上會標出那一步動到哪幾個字。
    </p>

    <div v-if="luckRows.length" class="luck-pick" role="group" aria-label="加上大運">
      <span class="luck-pick-label">加上大運</span>
      <div class="luck-pick-track">
        <button type="button" class="luck-chip" :class="{ on: luckPick === null }" @click="luckPick = null">不加</button>
        <button
          v-for="row in luckRows"
          :key="row.index"
          type="button"
          class="luck-chip"
          :class="{ on: luckPick === row.index }"
          :aria-pressed="luckPick === row.index"
          @click="luckPick = luckPick === row.index ? null : row.index"
        >
          <b>{{ row.ganZhi }}</b><small v-if="row.ages">{{ row.ages }}歲</small><i v-if="row.isCurrent">目前</i>
        </button>
      </div>
    </div>

    <div class="controls" role="group" aria-label="步驟控制">
      <button type="button" class="ctl" :class="{ on: applied === 0 }" @click="go(0)">原局</button>
      <button v-if="shift.luck" type="button" class="ctl" :class="{ on: applied === shift.luck.firstStep && focusIndex < 0 }" @click="showNatal">原局作用後</button>
      <button type="button" class="ctl" :disabled="applied === 0" @click="go(applied - 1)">◀ 上一步</button>
      <span class="progress">{{ applied }} / {{ total }}</span>
      <button type="button" class="ctl" :disabled="applied === total" @click="go(applied + 1)">下一步 ▶</button>
      <button type="button" class="ctl" :class="{ on: applied === total && focusIndex < 0 }" @click="showAll">全部作用</button>
      <button v-if="total > 1" type="button" class="ctl play" @click="togglePlay">{{ playing ? "■ 停止" : "▶ 播放" }}</button>
    </div>

    <!-- 四柱：時 日 月 年，上排天干、下排地支 -->
    <div class="board" :class="{ 'has-luck': !!shift.luck }" aria-label="四柱與作用標記">
      <div v-for="column in columns" :key="column.name" class="column" :class="{ 'luck-column': column.luck, ghost: column.ghost }">
        <div class="column-name">{{ column.luck ? "大運" : `${column.name}柱` }}</div>
        <div
          v-for="tile in column.tiles"
          :key="tile.cell.id"
          class="tile"
          :class="[`p-${tile.state.party}`, { dm: tile.cell.isDayMaster, focus: tile.focused, dim: focusStep && !tile.focused }]"
        >
          <div class="tile-top">
            <span class="ten">{{ tile.cell.isDayMaster ? "日主" : tile.cell.tenGod || tile.cell.family }}</span>
            <span v-if="tile.cell.isMonthCommand" class="badge month">月令</span>
            <span v-if="tile.cell.isRoot" class="badge root" title="藏干有日主同五行，是日主的根">根</span>
          </div>
          <div class="glyph">
            <span class="char" :class="[`wx-${wx(tile.cell.element)}`, { was: tile.state.transformed }]">{{ tile.cell.char }}</span>
            <span v-if="tile.state.transformed" class="now" :class="`wx-${wx(tile.state.element)}`">→{{ tile.state.element }}</span>
          </div>
          <div class="side">{{ tile.cell.isDayMaster ? "　" : `${partyName(tile.state.party)}・${tile.state.family}` }}</div>
          <div class="marks">
            <span
              v-for="(mark, idx) in tile.marks"
              :key="idx"
              class="mark"
              :class="[`h-${mark.helps ?? 'none'}`, { light: mark.effect.light, current: mark.current }]"
              :title="mark.effect.light ? '隔位或力量較輕，僅供參考' : ''"
            >{{ glyph(mark.effect.dir) }}{{ mark.effect.text }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 勢力條：同黨在左、異黨在右，分界線偏離中線多少，就是人數差多少 -->
    <div class="force">
      <div class="force-head">
        <span class="t-same">同黨 {{ now.same }}</span>
        <span class="force-title">勢力條（數人頭{{ now.withLuck ? "，含大運" : "" }}）</span>
        <span class="t-opp">異黨 {{ now.opp }}</span>
      </div>
      <div class="force-track">
        <span class="center-line" aria-hidden="true"></span>
        <TransitionGroup name="token" tag="div" class="tokens">
          <span
            v-for="token in tokens"
            :key="token.key"
            class="token"
            :class="[`k-${token.kind}`, { dm: token.dayMaster, month: token.month, luck: token.luck }]"
            :title="token.title"
          >{{ token.text }}<i v-if="token.month" class="crown">令</i><i v-if="token.luck" class="crown luck-crown">運</i></span>
        </TransitionGroup>
      </div>
      <div class="force-foot">
        <span>有利日主這邊 {{ trendNow.sameUp + trendNow.oppDown }} 項</span>
        <span class="mid">▲ 中線</span>
        <span>有利異黨那邊 {{ trendNow.sameDown + trendNow.oppUp }} 項</span>
      </div>
    </div>

    <!-- 三盞燈：得令、得勢、得地 -->
    <div class="lights">
      <span class="light" :class="{ on: shift.lights.season }">
        <b>{{ shift.lights.season ? "✓" : "✗" }} 得令</b>
        <small>月令和日主{{ shift.lights.season ? "同" : "不同" }}五行</small>
      </span>
      <span class="light" :class="{ on: natalNow.momentum, flip: natalNow.momentum !== shift.before.momentum }">
        <b>{{ natalNow.momentum ? "✓" : "✗" }} 得勢</b>
        <small>{{ momentumText }}</small>
      </span>
      <span class="light" :class="{ on: shift.lights.rooted, warn: shift.lights.rootShaken.length > 0 }">
        <b>{{ shift.lights.rooted ? "✓" : "✗" }} 得地</b>
        <small>{{ shift.lights.rooted ? (shift.lights.rootShaken.length ? `有根，但 ${shift.lights.rootShaken.join("、")} 被沖` : "地支有根") : "地支沒有根" }}</small>
      </span>
      <span class="verdict" :class="{ changed: now.classification && now.classification !== shift.before.classification }">
        <small>{{ now.withLuck ? "原局判定（大運不改判）" : applied === 0 ? "原局判定" : applied === natalTotal ? "作用後判定" : `做到第 ${applied} 步` }}</small>
        <b>{{ now.classification || "（看完全部作用再判）" }}</b>
      </span>
    </div>

    <div v-if="shift.luck && now.withLuck" class="luck-lean" :class="applied === total ? leanClass : 'l-none'">
      <b>大運{{ shift.luck.ganZhi }}<small v-if="shift.luck.label">{{ shift.luck.label }}</small></b>
      <span v-if="applied === total">這十年{{ shift.luck.leanText }}。{{ shift.luck.advice ?? "原局是特別格局候選或依據不足，要先確認格局。" }}</span>
      <span v-else>大運的作用還沒看完，按「下一步」或「全部作用」。</span>
    </div>

    <!-- 步驟清單 -->
    <div class="steps">
      <template v-for="group in groups" :key="group.key">
        <h3 v-if="group.items.length">{{ group.title }}<small>{{ group.hint }}</small></h3>
        <ol v-if="group.items.length" class="step-list">
          <li v-for="item in group.items" :key="item.step.id">
            <button
              type="button"
              class="step"
              :class="{ current: focusIndex === item.index, pending: item.index >= applied, inactive: item.step.inactive }"
              :aria-expanded="focusIndex === item.index"
              @click="pick(item.index)"
            >
              <span class="no">{{ item.index + 1 }}</span>
              <span class="tag">{{ item.step.tag }}</span>
              <span class="title">{{ item.step.title }}</span>
              <span class="kind" :class="item.kindClass">{{ item.kindText }}</span>
            </button>
            <div v-if="focusIndex === item.index" class="step-detail">
              <p v-if="item.effectLines.length" class="effects">
                <span v-for="(line, idx) in item.effectLines" :key="idx" class="mark" :class="[`h-${line.helps ?? 'none'}`, { light: line.light }]">{{ line.text }}</span>
              </p>
              <p><GlossaryText :text="item.step.detail" /></p>
              <p class="rule"><b>通則</b>　<GlossaryText :text="item.step.rule" /></p>
            </div>
          </li>
        </ol>
      </template>
      <p v-if="!shift.steps.length" class="empty">這張盤沒有會改變強弱的合、沖、刑、會。</p>
    </div>

    <div class="summary">
      <p v-for="(line, idx) in shift.summary" :key="idx">{{ line }}</p>
      <p v-if="shift.caution" class="caution">⚠ {{ shift.caution }}</p>
    </div>

    <ul class="legend">
      <li><span class="swatch p-same"></span>同黨（印、比劫）</li>
      <li><span class="swatch p-opp"></span>異黨（食傷、財、官殺）</li>
      <li><span class="mark h-same">↑增力</span><span class="mark h-same">↓減力</span>橘色＝對日主這邊有利</li>
      <li><span class="mark h-opp">↑增力</span><span class="mark h-opp">↓減力</span>藍色＝對異黨那邊有利</li>
      <li><span class="mark h-none light">淡色</span>隔位或力量較輕</li>
      <li v-if="shift.luck" class="wide">加上大運：原局的強弱不因大運改判，只看這十年把力量推向哪一邊。先看人數（大運兩個字、合化換邊、填實成局），人數相抵才看方向。</li>
      <li class="wide">只有「合而能化」和「成局」會改人數（和下方的作用變化後強弱複核相同）；其他作用只標方向，不給分數。老師說五行不能量化，方向相反的作用多時，要用過去的大運反驗。</li>
    </ul>
  </el-card>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import type { BaziResponse } from "../types/bazi";
import { currentLuckIndex } from "../utils/baziNow";
import {
  bonusesAt,
  cellStatesAt,
  helpsParty,
  partyName,
  strengthShift,
  trendAt,
  type ShiftCell,
  type ShiftCellState,
  type ShiftCount,
  type ShiftDir,
  type ShiftEffect,
  type ShiftParty,
  type ShiftStep,
} from "../utils/strengthShift";
import GlossaryText from "./GlossaryText.vue";

const props = defineProps<{ result: BaziResponse; compact?: boolean }>();

// 可以選一步大運加進來看（預設不加）。
const currentLuck = currentLuckIndex(props.result);
const luckRows = computed(() =>
  (props.result.daYun ?? [])
    .filter((row) => row.ganZhi?.length === 2)
    .map((row) => ({
      index: row.index,
      ganZhi: row.ganZhi,
      // 歲數寫法和大運面板相同（結束歲數 +1）。
      ages: row.startAge == null ? "" : row.endAge == null ? `${row.startAge}` : `${row.startAge}–${row.endAge + 1}`,
      isCurrent: row.index === currentLuck,
    })),
);
const luckPick = ref<number | null>(null);
watch(
  () => props.result,
  () => {
    luckPick.value = null;
  },
);

const shift = computed(() => {
  const row = luckRows.value.find((item) => item.index === luckPick.value);
  return strengthShift(
    props.result.quantModel?.lessonContent?.threeStepAnalysis,
    row ? { ganZhi: row.ganZhi, label: row.ages ? `${row.ages} 歲` : "" } : null,
  );
});
const total = computed(() => shift.value?.steps.length ?? 0);
/** 原局的作用有幾步（之後是大運的步驟）。 */
const natalTotal = computed(() => shift.value?.luck?.firstStep ?? total.value);

/** 已經套用到第幾步（0 = 原局）。預設先給答案：全部作用。 */
const applied = ref(0);
/** 目前展開說明的步驟（-1 = 沒有）。 */
const focusIndex = ref(-1);
const playing = ref(false);
let timer: ReturnType<typeof setInterval> | null = null;

function stop() {
  if (timer) clearInterval(timer);
  timer = null;
  playing.value = false;
}

watch(
  shift,
  (value) => {
    stop();
    applied.value = value?.steps.length ?? 0;
    focusIndex.value = -1;
  },
  { immediate: true },
);
onBeforeUnmount(stop);

function go(count: number, keepPlaying = false) {
  if (!keepPlaying) stop();
  applied.value = Math.min(Math.max(count, 0), total.value);
  focusIndex.value = applied.value - 1;
}

function showNatal() {
  stop();
  applied.value = natalTotal.value;
  focusIndex.value = -1;
}

function showAll() {
  stop();
  applied.value = total.value;
  focusIndex.value = -1;
}

function pick(index: number) {
  stop();
  if (focusIndex.value === index) {
    focusIndex.value = -1;
    return;
  }
  applied.value = index + 1;
  focusIndex.value = index;
}

function togglePlay() {
  if (playing.value) {
    stop();
    return;
  }
  playing.value = true;
  go(0, true);
  timer = setInterval(() => {
    if (applied.value >= total.value) {
      stop();
      return;
    }
    go(applied.value + 1, true);
  }, 1800);
}

const focusStep = computed<ShiftStep | null>(() => shift.value?.steps[focusIndex.value] ?? null);
const states = computed(() => (shift.value ? cellStatesAt(shift.value, applied.value) : new Map<string, ShiftCellState>()));
const now = computed<ShiftCount>(() => shift.value!.timeline[applied.value] ?? shift.value!.after);
/** 得勢看的是原局八個字：加了大運之後仍用原局作用後的人數。 */
const natalNow = computed<ShiftCount>(() => (now.value.withLuck ? shift.value!.after : now.value));
const luckEntered = computed(() => !!shift.value?.luck && applied.value > shift.value.luck.firstStep);
const leanClass = computed(() => {
  const lean = shift.value?.luck?.lean ?? 0;
  return lean > 0 ? "l-same" : lean < 0 ? "l-opp" : "l-none";
});
const trendNow = computed(() => trendAt(shift.value!, applied.value));

const momentumText = computed(() => {
  const count = natalNow.value;
  if (count.same > count.opp) return `同黨 ${count.same} 多於異黨 ${count.opp}`;
  if (count.same < count.opp) return `同黨 ${count.same} 少於異黨 ${count.opp}`;
  return `${count.same}：${count.opp} 平手，月令在${shift.value!.monthCommandSameParty ? "同黨" : "異黨"}這邊`;
});

const WX: Record<string, string> = { 木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water" };
const wx = (element: string) => WX[element] ?? "earth";
const glyph = (dir: ShiftDir) => (dir === "up" ? "↑" : dir === "down" ? "↓" : dir === "change" ? "⇄" : "∞");

interface TileMark {
  effect: ShiftEffect;
  helps: ShiftParty | null;
  current: boolean;
}
interface Tile {
  cell: ShiftCell;
  state: ShiftCellState;
  focused: boolean;
  marks: TileMark[];
}

/** 這個字在各步驟得到的標記，連同「對哪一邊有利」。 */
function marksOf(cell: ShiftCell): TileMark[] {
  const marks: TileMark[] = [];
  let party = cell.party;
  shift.value!.steps.slice(0, applied.value).forEach((step, index) => {
    for (const effect of step.effects) {
      if (effect.cell !== cell.id) continue;
      if (effect.dir === "change" && effect.toParty) {
        marks.push({ effect, helps: effect.toParty, current: index === focusIndex.value });
        party = effect.toParty;
      } else {
        marks.push({ effect, helps: helpsParty(effect, party), current: index === focusIndex.value });
      }
    }
  });
  return marks;
}

const columns = computed(() => {
  if (!shift.value) return [];
  const focused = new Set(focusStep.value?.cells ?? []);
  // 畫面順序：時、日、月、年（和排盤一致）
  return [...(shift.value.luck ? [4] : []), 3, 2, 1, 0].map((pillar) => {
    const cells = shift.value!.cells.filter((cell) => cell.pillar === pillar);
    const tiles: Tile[] = ["stem", "branch"]
      .map((row) => cells.find((cell) => cell.row === row))
      .filter((cell): cell is ShiftCell => !!cell)
      .map((cell) => ({ cell, state: states.value.get(cell.id)!, focused: focused.has(cell.id), marks: marksOf(cell) }));
    return { name: "年月日時運"[pillar], tiles, luck: pillar === 4, ghost: pillar === 4 && !luckEntered.value };
  });
});

interface Token {
  key: string;
  kind: "same" | "opp" | "bonus-same" | "bonus-opp" | "divider";
  text: string;
  title: string;
  dayMaster: boolean;
  month: boolean;
  luck?: boolean;
}

const tokens = computed<Token[]>(() => {
  if (!shift.value) return [];
  const same: Token[] = [];
  const opp: Token[] = [];
  for (const cell of shift.value.cells) {
    if (cell.luck && !luckEntered.value) continue;
    const state = states.value.get(cell.id)!;
    const token: Token = {
      key: cell.id,
      kind: state.party,
      text: cell.char,
      title: `${cell.id}${cell.char}：${state.transformed ? `化${state.element}，` : ""}${cell.isDayMaster ? "日主" : `${partyName(state.party)}・${state.family}`}`,
      dayMaster: cell.isDayMaster,
      month: cell.isMonthCommand,
      luck: !!cell.luck,
    };
    (state.party === "same" ? same : opp).push(token);
  }
  for (const bonus of bonusesAt(shift.value, applied.value)) {
    const token: Token = {
      key: `bonus-${bonus.id}`,
      kind: bonus.party === "same" ? "bonus-same" : "bonus-opp",
      text: `＋${bonus.label}`,
      title: `成局後這個五行力量大增，給${partyName(bonus.party)}加一`,
      dayMaster: false,
      month: false,
    };
    (bonus.party === "same" ? same : opp).unshift(token);
  }
  // 同黨從中間往左排、異黨從中間往右排：日主放最左，成局的合力貼著分界線
  same.sort((a, b) => Number(b.dayMaster) - Number(a.dayMaster) || Number(a.kind.startsWith("bonus")) - Number(b.kind.startsWith("bonus")));
  const divider: Token = { key: "divider", kind: "divider", text: "", title: "同黨和異黨的分界", dayMaster: false, month: false };
  return [...same, divider, ...opp];
});

const GROUPS = computed(() => [
  { key: "stem", title: "天干", hint: "五合、相鄰的生剋" },
  { key: "branch", title: "地支", hint: "三會、三合、半合、六沖、六合、刑" },
  { key: "pillar", title: "同柱干支", hint: "每一柱的天干坐在什麼地支上" },
  { key: "luck", title: `大運 ${shift.value?.luck?.ganZhi ?? ""}`, hint: `${shift.value?.luck?.label ?? ""}　這步大運和原局八個字的作用` },
]);

const groups = computed(() => {
  if (!shift.value) return [];
  const cellMap = new Map(shift.value.cells.map((cell) => [cell.id, cell]));
  const finalStates = cellStatesAt(shift.value, shift.value.steps.length);
  return GROUPS.value.map((group) => ({
    ...group,
    items: shift.value!.steps
      .map((step, index) => ({ step, index }))
      .filter((item) => item.step.group === group.key)
      .map(({ step, index }) => {
        const deltas: string[] = [];
        if (step.deltaSame) deltas.push(`同黨 ${step.deltaSame > 0 ? "+" : "−"}${Math.abs(step.deltaSame)}`);
        if (step.deltaOpp) deltas.push(`異黨 ${step.deltaOpp > 0 ? "+" : "−"}${Math.abs(step.deltaOpp)}`);
        const kindText = step.inactive ? "不成立" : step.counted ? deltas.join("、") || "人數不變" : step.effects.length ? "只看方向" : "不影響強弱";
        const kindClass = step.inactive ? "k-off" : step.counted ? "k-count" : "k-dir";
        const effectLines = step.effects.map((effect) => {
          const cell = cellMap.get(effect.cell)!;
          const party = effect.dir === "change" ? cell.party : finalStates.get(effect.cell)?.party ?? cell.party;
          return {
            text: `${effect.cell}${cell.char} ${glyph(effect.dir)}${effect.text}`,
            helps: effect.dir === "change" ? effect.toParty ?? null : helpsParty(effect, party),
            light: !!effect.light,
          };
        });
        return { step, index, kindText, kindClass, effectLines };
      }),
  }));
});
</script>

<style scoped>
.strength-shift {
  --same: #c2620a;
  --same-bg: #fff1dc;
  --opp: #3d6a96;
  --opp-bg: #eaf1f8;
  min-width: 0;
  overflow-wrap: anywhere;
  font-size: calc(15px * var(--app-font-scale));
  line-height: 1.6;
}

.section-header {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 12px;
}

.header-sub {
  color: var(--el-text-color-secondary, #606266);
  font-size: 0.86em;
}

.intro {
  margin: 0 0 10px;
  color: var(--el-text-color-regular, #4a4a4a);
  font-size: 0.93em;
}

.t-same { color: var(--same); }
.t-opp { color: var(--opp); }

.luck-pick {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.luck-pick-label {
  flex: none;
  font-weight: 700;
  font-size: 0.88em;
}

.luck-pick-track {
  display: flex;
  gap: 6px;
  min-width: 0;
  overflow-x: auto;
  padding: 9px 4px 6px 2px;
}

.luck-chip {
  position: relative;
  display: inline-flex;
  flex: none;
  flex-direction: column;
  align-items: center;
  min-width: 3.4em;
  padding: 0.2em 0.6em;
  border: 1px solid rgba(174, 128, 72, 0.35);
  border-radius: 8px;
  background: #fffdf8;
  color: inherit;
  font: inherit;
  font-size: 0.88em;
  line-height: 1.3;
  cursor: pointer;
}

.luck-chip small { color: var(--el-text-color-secondary, #606266); font-size: 0.78em; white-space: nowrap; }
.luck-chip i {
  position: absolute;
  top: -0.6em;
  right: -0.3em;
  padding: 0 0.3em;
  border-radius: 4px;
  background: #b45309;
  color: #fff;
  font-size: 0.68em;
  font-style: normal;
}
.luck-chip.on { border-color: var(--bazi-primary, #92400e); background: var(--bazi-primary, #92400e); color: #fffaf0; }
.luck-chip.on small { color: #fde8c8; }
.luck-chip:focus-visible { outline: 2px solid var(--bazi-primary, #92400e); outline-offset: 2px; }

.controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  margin-bottom: 12px;
}

.ctl {
  padding: 0.3em 0.8em;
  border: 1px solid rgba(180, 83, 9, 0.32);
  border-radius: 999px;
  background: #fffdf8;
  color: var(--bazi-primary, #92400e);
  font: inherit;
  font-size: 0.88em;
  cursor: pointer;
}

.ctl:disabled { opacity: 0.4; cursor: default; }
.ctl.on { background: var(--bazi-primary, #92400e); color: #fffaf0; }
.ctl.play { margin-left: auto; }
.ctl:focus-visible, .step:focus-visible { outline: 2px solid var(--bazi-primary, #92400e); outline-offset: 2px; }

.progress {
  min-width: 3.6em;
  text-align: center;
  font-variant-numeric: tabular-nums;
  color: var(--el-text-color-secondary, #606266);
  font-size: 0.88em;
}

/* ── 四柱 ── */
.board {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}

.board.has-luck { grid-template-columns: repeat(5, minmax(0, 1fr)); }

.luck-column {
  padding-right: 8px;
  border-right: 2px dotted rgba(146, 64, 14, 0.45);
}

.luck-column .column-name { color: var(--bazi-primary, #92400e); font-weight: 700; }
.luck-column .tile { border-style: double; border-width: 4px; }
.luck-column.ghost .tile { opacity: 0.3; }

.column {
  display: grid;
  grid-template-rows: auto 1fr 1fr;
  gap: 6px;
  min-width: 0;
}

.column-name {
  text-align: center;
  color: var(--el-text-color-secondary, #606266);
  font-size: 0.82em;
}

.tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  min-width: 0;
  padding: 6px 4px 8px;
  border: 2px solid var(--opp);
  border-radius: 10px;
  background: var(--opp-bg);
  transition: background-color 0.4s, border-color 0.4s, opacity 0.3s, box-shadow 0.3s;
}

.tile.p-opp { border-style: dashed; }
.tile.p-same { border-color: var(--same); background: var(--same-bg); }
.tile.dm { border-width: 3px; background: #ffe2b8; }
.tile.dim { opacity: 0.55; }
.tile.focus { box-shadow: 0 0 0 3px rgba(146, 64, 14, 0.28); }

.tile-top {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 2px 4px;
  font-size: 0.74em;
  color: var(--el-text-color-secondary, #606266);
}

.badge {
  padding: 0 0.35em;
  border-radius: 4px;
  color: #fff;
  font-weight: 700;
}

.badge.month { background: #7c3aed; }
.badge.root { background: #5b7a3a; }

.glyph {
  display: flex;
  align-items: baseline;
  gap: 2px;
  font-weight: 800;
  line-height: 1.15;
}

.char { font-size: 1.9em; }
.char.was { text-decoration: line-through; text-decoration-thickness: 2px; opacity: 0.55; }
.now { font-size: 1.15em; }

.wx-wood { color: var(--wx-wood); }
.wx-fire { color: var(--wx-fire); }
.wx-earth { color: var(--wx-earth); }
.wx-metal { color: var(--wx-metal); }
.wx-water { color: var(--wx-water); }

.side {
  font-size: 0.74em;
  font-weight: 700;
}

.p-same .side { color: var(--same); }
.p-opp .side { color: var(--opp); }

.marks {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 3px;
  min-height: 1.5em;
}

.mark {
  display: inline-block;
  padding: 0 0.4em;
  border: 1px solid currentColor;
  border-radius: 6px;
  background: #fff;
  font-size: 0.74em;
  font-weight: 700;
  line-height: 1.5;
  white-space: nowrap;
}

.mark.h-same { color: var(--same); }
.mark.h-opp { color: var(--opp); }
.mark.h-none { color: #6b7280; }
.mark.light { border-style: dashed; font-weight: 500; opacity: 0.7; }
.mark.current { background: #fff7c2; box-shadow: 0 0 0 2px rgba(202, 138, 4, 0.45); }

/* ── 勢力條 ── */
.force {
  margin-top: 14px;
  padding: 10px 10px 8px;
  border: 1px solid rgba(174, 128, 72, 0.25);
  border-radius: 10px;
  background: #fffdf8;
}

.force-head,
.force-foot {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
  font-weight: 700;
}

.force-title,
.force-foot {
  color: var(--el-text-color-secondary, #606266);
  font-weight: 500;
  font-size: 0.8em;
}

.force-foot .mid { flex: none; }
.force-foot span:last-child { text-align: right; }

.force-track {
  position: relative;
  margin: 6px 0 4px;
  padding: 6px 0;
}

.center-line {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  border-left: 2px dashed rgba(107, 114, 128, 0.6);
}

.tokens {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: stretch;
  gap: 3px;
}

.token {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 1 2.3em;
  min-width: 0;
  height: 2.3em;
  border: 2px solid var(--opp);
  border-radius: 6px;
  background: var(--opp-bg);
  color: var(--opp);
  font-weight: 800;
}

.token.k-opp { border-style: dashed; }
.token.k-same { border-color: var(--same); background: var(--same-bg); color: var(--same); }
.token.dm { background: var(--same); color: #fff; }

.token.k-bonus-same,
.token.k-bonus-opp {
  flex: 0 0 auto;
  padding: 0 0.3em;
  font-size: 0.74em;
  white-space: nowrap;
  border-style: solid;
}

.token.k-bonus-same {
  border-color: var(--same);
  color: var(--same);
  background: repeating-linear-gradient(45deg, #fff, #fff 4px, var(--same-bg) 4px, var(--same-bg) 8px);
}

.token.k-bonus-opp {
  background: repeating-linear-gradient(45deg, #fff, #fff 4px, var(--opp-bg) 4px, var(--opp-bg) 8px);
}

.token.k-divider {
  flex: 0 0 4px;
  border: 0;
  border-radius: 2px;
  background: #1f2937;
}

.crown {
  position: absolute;
  top: -0.75em;
  right: -0.5em;
  padding: 0 0.25em;
  border-radius: 4px;
  background: #7c3aed;
  color: #fff;
  font-size: 0.6em;
  font-style: normal;
  line-height: 1.4;
}

.token.luck { border-style: double; border-width: 4px; }
.crown.luck-crown { background: #b45309; }

.token-move,
.token-enter-active,
.token-leave-active {
  transition: transform 0.6s ease, opacity 0.4s ease;
}

.token-enter-from,
.token-leave-to {
  opacity: 0;
  transform: scale(0.6);
}

.token-leave-active { position: absolute; }

/* ── 三盞燈 ── */
.lights {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr)) minmax(0, 1.25fr);
  gap: 8px;
  margin-top: 10px;
}

.light,
.verdict {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  padding: 6px 8px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  background: #f6f6f4;
  color: #6b7280;
}

.light.on { border-color: var(--same); background: var(--same-bg); color: var(--same); }
.light.flip { box-shadow: 0 0 0 2px rgba(202, 138, 4, 0.5); }
.light.warn { border-style: dashed; }
.light small,
.verdict small { font-size: 0.76em; color: var(--el-text-color-secondary, #606266); }

.verdict { background: #fff; border-color: #1f2937; color: #1f2937; }
.verdict b { font-size: 1.12em; }
.verdict.changed { border-color: var(--el-color-danger, #c0392b); color: var(--el-color-danger, #c0392b); }

.luck-lean {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 10px;
  margin-top: 8px;
  padding: 8px 10px;
  border: 2px solid #9ca3af;
  border-radius: 8px;
  background: #f6f6f4;
}

.luck-lean b small { margin-left: 6px; font-weight: 400; color: var(--el-text-color-secondary, #606266); }
.luck-lean.l-same { border-color: var(--same); background: var(--same-bg); }
.luck-lean.l-opp { border-color: var(--opp); background: var(--opp-bg); }

/* ── 步驟 ── */
.steps { margin-top: 14px; }

.steps h3 {
  margin: 12px 0 4px;
  font-size: 0.95em;
}

.steps h3 small {
  margin-left: 8px;
  color: var(--el-text-color-secondary, #606266);
  font-weight: 400;
}

.step-list {
  display: grid;
  gap: 4px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.step {
  display: grid;
  grid-template-columns: 1.7em auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 5px 8px;
  border: 1px solid rgba(174, 128, 72, 0.22);
  border-radius: 8px;
  background: #fffdf8;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.step:hover { background: #fff3dc; }
.step.pending { opacity: 0.5; }
.step.inactive .title { text-decoration: line-through; text-decoration-color: rgba(107, 114, 128, 0.6); }
.step.current { border-color: var(--bazi-primary, #92400e); background: #fff3dc; opacity: 1; }

.no {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.7em;
  height: 1.7em;
  border-radius: 50%;
  background: #efe3cf;
  font-size: 0.8em;
  font-variant-numeric: tabular-nums;
}

.tag {
  padding: 0 0.45em;
  border-radius: 4px;
  background: #efe3cf;
  font-size: 0.8em;
  white-space: nowrap;
}

.title { min-width: 0; font-weight: 700; }

.kind {
  padding: 0 0.5em;
  border-radius: 999px;
  font-size: 0.76em;
  font-weight: 700;
  white-space: nowrap;
}

.kind.k-count { background: #1f2937; color: #fff; }
.kind.k-dir { border: 1px solid #9ca3af; color: #4b5563; }
.kind.k-off { color: #9ca3af; }

.step-detail {
  margin: 2px 0 6px calc(1.7em + 6px);
  padding: 6px 10px;
  border-left: 3px solid var(--bazi-primary, #92400e);
  background: #fffaf0;
  font-size: 0.92em;
}

.step-detail p { margin: 2px 0; }
.step-detail .effects { display: flex; flex-wrap: wrap; gap: 4px; }
.step-detail .effects .mark { font-size: 0.86em; }
.step-detail .rule { color: var(--el-text-color-secondary, #606266); }

.empty { color: var(--el-text-color-secondary, #606266); }

.summary {
  margin-top: 12px;
  padding: 8px 10px;
  border-radius: 8px;
  background: #f8f3e9;
  font-size: 0.93em;
}

.summary p { margin: 2px 0; }
.summary .caution { color: var(--el-color-warning-dark-2, #9a6a15); font-weight: 700; }

.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 14px;
  margin: 10px 0 0;
  padding: 0;
  list-style: none;
  color: var(--el-text-color-secondary, #606266);
  font-size: 0.8em;
}

.legend li { display: inline-flex; flex-wrap: wrap; align-items: center; gap: 4px; }
.legend .wide { flex-basis: 100%; }
.legend .mark { font-size: 0.92em; }

.swatch {
  display: inline-block;
  width: 1.1em;
  height: 1.1em;
  border: 2px dashed var(--opp);
  border-radius: 4px;
  background: var(--opp-bg);
}

.swatch.p-same { border: 2px solid var(--same); background: var(--same-bg); }

/* ── 手機 ── */
.compact .board { gap: 4px; }
.compact .luck-column { padding-right: 4px; }
.compact .board.has-luck .char { font-size: 1.4em; }
.compact .board.has-luck .mark { font-size: 0.68em; padding: 0 0.25em; white-space: normal; }
.compact .luck-pick { flex-direction: column; align-items: stretch; gap: 2px; }
.compact .tile { padding: 4px 2px 6px; border-radius: 8px; }
.compact .char { font-size: 1.6em; }
.compact .lights { grid-template-columns: repeat(3, minmax(0, 1fr)); }
.compact .verdict { grid-column: 1 / -1; flex-direction: row; align-items: baseline; gap: 8px; }
.compact .step { gap: 4px; padding: 5px 6px; }
.compact .ctl.play { margin-left: 0; }
.compact .force { padding: 8px 6px 6px; }
.compact .tokens { gap: 2px; }
.compact .force-foot .mid { display: none; }
.compact .step-detail { margin-left: 0; }

@media (prefers-reduced-motion: reduce) {
  .tile, .token-move, .token-enter-active, .token-leave-active { transition: none; }
}
</style>
