<template>
  <div v-if="flow" :class="['efr', { compact }]">
    <div class="efr-chart">
      <svg class="efr-svg" viewBox="0 0 380 410" role="img" :aria-label="`${flow.dayStem}${flow.dayElement}日主的五行分布與相生相剋`">
        <!-- 內圈：相剋（淡虛線；兩邊都有兩個字以上才畫成紅色的「相戰」） -->
        <g v-for="edge in controlEdges" :key="`c${edge.from}`" :class="['efr-control', `is-${edge.state}`, byFavor ? `fav-${edge.effect}` : '', { dim: isEdgeDim(edge) }]">
          <line :x1="edge.x1" :y1="edge.y1" :x2="edge.x2" :y2="edge.y2" />
          <polygon :points="edge.head" />
          <text v-if="edge.state === 'clash' || (byFavor && edge.effect !== 'none')" :x="edge.lx" :y="edge.ly" class="efr-control-label">剋{{ favMark(edge) }}</text>
        </g>
        <!-- 外圈：相生（實線粗箭頭＝兩邊都有字；灰虛線打 ✕＝斷了） -->
        <g
          v-for="edge in generateEdges"
          :key="`g${edge.from}`"
          :class="['efr-generate', `is-${edge.state}`, byFavor ? `fav-${edge.effect}` : '', { dim: isEdgeDim(edge) }]"
          :style="{ '--c': byFavor ? FAVOR_COLOR[edge.effect] : `var(--wx-${flow.elements[edge.from].key})` }"
        >
          <line :x1="edge.x1" :y1="edge.y1" :x2="edge.x2" :y2="edge.y2" />
          <polygon v-if="edge.state === 'flow'" :points="edge.head" />
          <text :x="edge.lx" :y="edge.ly" class="efr-generate-label">{{ edge.state === 'flow' ? `生${favMark(edge)}` : '✕ 斷' }}</text>
        </g>

        <g class="efr-center">
          <circle cx="190" cy="215" r="40" />
          <text x="190" :y="flow.flipTo ? 201 : 207" class="efr-center-main" :style="{ fill: `var(--wx-${flow.elements[0].key})` }">{{ flow.dayStem }}{{ flow.dayElement }}日主</text>
          <text x="190" :y="flow.flipTo ? 217 : 224" class="efr-center-sub">{{ flow.classification || '—' }}</text>
          <text x="190" :y="flow.flipTo ? 230 : 238" class="efr-center-count">同 {{ flow.same }}：異 {{ flow.opp }}{{ flow.extraLabel ? (flow.extraLabel.includes('流年') ? '（含運年）' : '（含運）') : '' }}</text>
          <text v-if="flow.flipTo" x="190" y="243" class="efr-center-flip">合化後偏{{ flow.flipTo.replace('候選', '') }}</text>
        </g>

        <g
          v-for="node in nodes"
          :key="node.el.key"
          :class="['efr-node', { empty: !node.el.count, dim: isDim(node.el.index), day: node.el.index === 0 }]"
          :style="{ '--c': `var(--wx-${node.el.key})` }"
        >
          <circle :cx="node.x" :cy="node.y" :r="node.r" class="efr-node-disc" />
          <text :x="node.x" :y="node.el.count ? node.y - (node.lines.length > 1 ? 10 : 4) : node.y + 1" class="efr-node-name">{{ node.el.name }}</text>
          <text v-for="(line, li) in node.lines" :key="li" :x="node.x" :y="node.y + 15 + li * 15 - (node.lines.length > 1 ? 6 : 0)" class="efr-node-chars">
            <tspan v-for="char in line" :key="char.id" :class="{ dm: char.isDayMaster, mc: char.isMonthCommand, bonus: char.id.startsWith('bonus'), yun: char.extra === '運', nian: char.extra === '年' }" dx="1.5">{{ char.char }}<tspan v-if="char.isMonthCommand" class="efr-sup efr-ling">令</tspan><tspan v-if="char.transformed" class="efr-sup efr-hua">化</tspan><tspan v-if="char.extra" :class="['efr-sup', char.extra === '年' ? 'efr-nian' : 'efr-yun']">{{ char.extra }}</tspan></tspan>
          </text>
          <text v-if="!node.el.count" :x="node.x" :y="node.y + 17" class="efr-node-missing">缺</text>
          <g v-if="node.el.count">
            <circle :cx="node.x + node.r * 0.72" :cy="node.y - node.r * 0.72" r="10" class="efr-node-badge" />
            <text :x="node.x + node.r * 0.72" :y="node.y - node.r * 0.72 + 4" class="efr-node-badge-text">{{ node.el.count }}</text>
          </g>
          <g v-if="node.el.delta" :class="['efr-delta', node.el.delta > 0 ? 'up' : 'down']">
            <rect :x="node.x - node.r * 0.72 - 12" :y="node.y - node.r * 0.72 - 9" width="24" height="17" rx="8.5" />
            <text :x="node.x - node.r * 0.72" :y="node.y - node.r * 0.72 + 4">{{ node.el.delta > 0 ? '+' : '' }}{{ node.el.delta }}</text>
          </g>
          <text :x="node.x" :y="node.labelY" class="efr-node-role">
            {{ node.el.role }}<tspan class="efr-node-role-sub">·{{ node.el.roleText }}</tspan>
            <tspan v-if="node.el.favor === 'good'" class="efr-fav good" dx="4">{{ node.el.isUseGod ? '用神' : '喜' }}</tspan>
            <tspan v-else-if="node.el.favor === 'bad'" class="efr-fav bad" dx="4">忌</tspan>
          </text>
          <text v-if="!node.el.count && node.el.hidden.length" :x="node.x" :y="node.labelY + 13" class="efr-node-hidden">藏：{{ node.el.hidden.join('、') }}</text>
        </g>
      </svg>
      <div class="efr-line-switch" role="group" aria-label="線的顏色">
        <span>線的顏色</span>
        <button type="button" :class="{ on: byFavor }" @click="setLineMode('favor')">照喜忌</button>
        <button type="button" :class="{ on: !byFavor }" @click="setLineMode('element')">照五行</button>
      </div>
      <p v-if="flow.extraLabel" class="efr-extra">加上 <b>{{ flow.extraLabel }}</b>：圈裡標<b class="lg-yun">運</b>、<b class="lg-nian">年</b>的是大運、流年的字，圈左上角 ±n 是比原局多或少幾個字。強弱判定仍照原局。</p>
      <p v-if="byFavor" class="efr-legend">
        <span><i class="lg lg-good" />綠▲＝對日主有利（生喜、剋忌）</span>
        <span><i class="lg lg-bad" />紅▼＝對日主不利（生忌、剋喜）</span>
        <span><i class="lg lg-none" />灰＝那一行沒有明確喜忌</span>
        <span>外圈實線＝相生、內圈虛線＝相剋（粗的＝兩邊都有力）</span>
        <span>圈越大字越多；<b class="lg-ling">令</b>＝月令、<b class="lg-hua">化</b>＝合化過來的字</span>
      </p>
      <p v-else class="efr-legend">
        <span><i class="lg lg-gen" />外圈粗箭頭＝相生（有字才流得過去）</span>
        <span><i class="lg lg-ctl" />內圈虛線＝相剋，紅色＝兩邊都有力</span>
        <span>圈越大字越多；<b class="lg-ling">令</b>＝月令、<b class="lg-hua">化</b>＝合化過來的字</span>
      </p>
    </div>

    <ol class="efr-points">
      <li
        v-for="(point, index) in flow.points"
        :key="point.key"
        :class="['efr-point', `is-${point.tone}`, { on: active === point.key }]"
        @mouseenter="hover = point.key"
        @mouseleave="hover = null"
        @click="pinned = pinned === point.key ? null : point.key"
      >
        <span class="efr-point-no">{{ index + 1 }}</span>
        <div>
          <strong>{{ point.title }}</strong>
          <p><GlossaryText :text="point.text" /></p>
        </div>
      </li>
    </ol>
  </div>
  <p v-else class="efr-empty">這張盤沒有課堂三步法的資料，畫不出重點圖；請看「逐字」。</p>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { BaziResponse } from "../types/bazi";
import { elementFlow, type FlowChar, type FlowEdge, type FlowElement } from "../utils/elementFlow";
import GlossaryText from "./GlossaryText.vue";
import type { StrengthShift } from "../utils/strengthShift";

// 五行流通「重點圖」：五行五個圈繞一圈（日主在上，順時針相生），旁邊列出這張盤先看的幾句話。

// shift：四柱圖上點了大運時，帶大運的強弱變化（strengthShift）；不給就只看原局。
const props = defineProps<{ result: BaziResponse; compact?: boolean; shift?: StrengthShift | null }>();

const flow = computed(() => elementFlow(props.result, props.shift));

const CX = 190;
const CY = 215;
const R = 135;
const RADIUS = [26, 32, 38, 43, 48];

interface NodeView {
  el: FlowElement;
  x: number;
  y: number;
  r: number;
  labelY: number;
  lines: FlowChar[][];
}

const nodes = computed<NodeView[]>(() =>
  (flow.value?.elements ?? []).map((el) => {
    const angle = ((-90 + 72 * el.index) * Math.PI) / 180;
    const x = CX + R * Math.cos(angle);
    const y = CY + R * Math.sin(angle);
    const r = RADIUS[Math.min(el.count, 4)];
    const upper = el.index === 0 || el.index === 1 || el.index === 4;
    const chars: FlowChar[] = [
      ...[...el.chars].sort((a, b) => Number(b.row === "stem") - Number(a.row === "stem")),
      // 成局的合力當成一個「字」列在後面，例如「＋水局」。
      ...el.bonus.map((label, index) => ({ id: `bonus${index}`, char: `＋${label}`, row: "branch" as const, isDayMaster: false, isMonthCommand: false, transformed: false })),
    ];
    const width = chars.reduce((sum, char) => sum + char.char.length, 0);
    let lines: FlowChar[][] = chars.length ? [chars] : [];
    if (width > 4 && chars.length > 1) {
      // 一行放不下就拆兩行，盡量兩行一樣寬。
      let cut = 1;
      let acc = chars[0].char.length;
      while (cut < chars.length - 1 && acc + chars[cut].char.length <= Math.ceil(width / 2)) {
        acc += chars[cut].char.length;
        cut += 1;
      }
      lines = [chars.slice(0, cut), chars.slice(cut)];
    }
    return { el, x, y, r, labelY: upper ? y - r - 8 : y + r + 16, lines };
  }),
);

interface EdgeView extends FlowEdge {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  head: string;
  lx: number;
  ly: number;
}

function edgeView(edge: FlowEdge, headSize: number, labelAt: number, labelOffset: number): EdgeView {
  const a = nodes.value[edge.from];
  const b = nodes.value[edge.to];
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy) || 1;
  const ux = dx / len;
  const uy = dy / len;
  const x1 = a.x + ux * (a.r + 5);
  const y1 = a.y + uy * (a.r + 5);
  const tipX = b.x - ux * (b.r + 5);
  const tipY = b.y - uy * (b.r + 5);
  const x2 = tipX - ux * headSize;
  const y2 = tipY - uy * headSize;
  const px = -uy;
  const py = ux;
  const head = `${tipX},${tipY} ${x2 + px * headSize * 0.6},${y2 + py * headSize * 0.6} ${x2 - px * headSize * 0.6},${y2 - py * headSize * 0.6}`;
  // 標籤放在線外側（離圓心遠的那一邊）。
  const mx = x1 + (x2 - x1) * labelAt;
  const my = y1 + (y2 - y1) * labelAt;
  const out = Math.sign((mx - CX) * px + (my - CY) * py) || 1;
  return { ...edge, x1, y1, x2, y2, head, lx: mx + px * out * labelOffset, ly: my + py * out * labelOffset + 4 };
}

const generateEdges = computed(() => (nodes.value.length ? (flow.value?.generates ?? []).map((edge) => edgeView(edge, 9, 0.5, 13)) : []));
const controlEdges = computed(() =>
  nodes.value.length ? (flow.value?.controls ?? []).filter((edge) => edge.state !== "broken").map((edge) => edgeView(edge, 6, 0.3, 9)) : [],
);

// 線的顏色：照喜忌（預設；對日主有利綠、不利紅）或照五行（生的那一行的顏色）。
const LINE_KEY = "bazi:flow-line:v1";
const FAVOR_COLOR = { good: "#2f9e44", bad: "#c53030", none: "#b9b1a4" } as const;
function readLineMode(): "element" | "favor" {
  try {
    return localStorage.getItem(LINE_KEY) === "element" ? "element" : "favor";
  } catch {
    return "favor";
  }
}
const lineMode = ref<"element" | "favor">(readLineMode());
const byFavor = computed(() => lineMode.value === "favor");
function setLineMode(mode: "element" | "favor") {
  lineMode.value = mode;
  try {
    localStorage.setItem(LINE_KEY, mode);
  } catch {
    /* 存不了就只在這次有效 */
  }
}
const favMark = (edge: FlowEdge) => (!byFavor.value ? "" : edge.effect === "good" ? "▲" : edge.effect === "bad" ? "▼" : "");

const hover = ref<string | null>(null);
const pinned = ref<string | null>(null);
const active = computed(() => hover.value ?? pinned.value);
watch(flow, () => {
  hover.value = null;
  pinned.value = null;
});
const activeElements = computed(() => flow.value?.points.find((point) => point.key === active.value)?.elements ?? null);
const isDim = (index: number) => !!activeElements.value && !activeElements.value.includes(index);
const isEdgeDim = (edge: FlowEdge) => !!activeElements.value && !(activeElements.value.includes(edge.from) && activeElements.value.includes(edge.to));
</script>

<style scoped>
.efr {
  display: grid;
  grid-template-columns: minmax(280px, 440px) minmax(0, 1fr);
  gap: 8px 28px;
  align-items: center;
}
.efr.compact {
  grid-template-columns: minmax(0, 1fr);
}
@media (max-width: 860px) {
  .efr {
    grid-template-columns: minmax(0, 1fr);
  }
}
.efr-chart {
  min-width: 0;
}
.efr-svg {
  display: block;
  width: 100%;
  max-width: 440px;
  margin: 0 auto;
  height: auto;
  font-family: inherit;
}
.efr-svg text {
  text-anchor: middle;
}
.efr-svg .dim {
  opacity: 0.22;
}
.efr-svg g {
  transition: opacity 0.2s;
}

/* 相生 */
.efr-generate line {
  stroke: var(--c);
  stroke-width: 5;
  stroke-linecap: round;
  opacity: 0.85;
}
.efr-generate polygon {
  fill: var(--c);
}
.efr-generate-label {
  font-size: 10px;
  fill: var(--c);
  font-weight: 700;
}
.efr-generate.is-broken line {
  stroke: #c5bdb0;
  stroke-width: 2;
  stroke-dasharray: 3 6;
  opacity: 1;
}
.efr-generate.is-broken .efr-generate-label {
  fill: #a0522d;
  font-size: 11px;
}

/* 相剋 */
.efr-control line {
  stroke: #cfc7ba;
  stroke-width: 1.2;
  stroke-dasharray: 3 4;
}
.efr-control polygon {
  fill: #cfc7ba;
}
.efr-control.is-clash line {
  stroke: #c53030;
  stroke-width: 2.4;
  stroke-dasharray: 7 4;
}
.efr-control.is-clash polygon {
  fill: #c53030;
}
.efr-control-label {
  font-size: 11px;
  font-weight: 700;
  fill: #c53030;
}

/* 線上的「生／剋」字墊一圈底色，線交叉的地方才看得清楚 */
.efr-generate-label,
.efr-control-label {
  paint-order: stroke;
  stroke: #fffdf8;
  stroke-width: 3.5px;
  stroke-linejoin: round;
}

/* 線照喜忌上色 */
.efr-control.fav-good line {
  stroke: #2f9e44;
  stroke-width: 1.6;
}
.efr-control.fav-good polygon,
.efr-control.fav-good .efr-control-label {
  fill: #2f9e44;
}
.efr-control.fav-bad line {
  stroke: #c53030;
  stroke-width: 1.6;
}
.efr-control.fav-bad polygon,
.efr-control.fav-bad .efr-control-label {
  fill: #c53030;
}
.efr-control.fav-none.is-clash line {
  stroke: #b9b1a4;
}
.efr-control.fav-none.is-clash polygon,
.efr-control.fav-none .efr-control-label {
  fill: #9a9287;
}
.efr-control.is-clash.fav-good line,
.efr-control.is-clash.fav-bad line {
  stroke-width: 2.8;
}
.efr-control[class*="fav-"] .efr-control-label {
  font-size: 10px;
}

/* 中央 */
.efr-center circle {
  fill: #fffdf8;
  stroke: #eadfcc;
  stroke-width: 1;
}
.efr-center-main {
  font-size: 13px;
  font-weight: 800;
}
.efr-center-sub {
  font-size: 13px;
  font-weight: 700;
  fill: #3a3f45;
}
.efr-center-count {
  font-size: 10px;
  fill: #7a838d;
}
.efr-center-flip {
  font-size: 9.5px;
  font-weight: 700;
  fill: #b45309;
}

/* 五行圈 */
.efr-node-disc {
  fill: color-mix(in srgb, var(--c) 13%, #fff);
  stroke: var(--c);
  stroke-width: 2.5;
}
.efr-node.day .efr-node-disc {
  stroke-width: 4;
}
.efr-node.empty .efr-node-disc {
  fill: #faf8f4;
  stroke: #c5bdb0;
  stroke-width: 1.5;
  stroke-dasharray: 4 4;
}
.efr-node-name {
  font-size: 21px;
  font-weight: 800;
  fill: var(--c);
}
.efr-node.empty .efr-node-name {
  font-size: 17px;
  fill: #a8a095;
}
.efr-node-missing {
  font-size: 11px;
  font-weight: 700;
  fill: #a0522d;
}
.efr-node-chars {
  font-size: 13px;
  fill: #3a3f45;
}
.efr-node-chars .dm {
  font-weight: 800;
  text-decoration: underline;
}
.efr-node-chars .bonus {
  font-size: 11px;
  font-weight: 700;
  fill: var(--c);
}
.efr-sup {
  font-size: 8px;
  font-weight: 700;
  baseline-shift: 5px;
}
.efr-ling {
  fill: #7c3aed;
}
.efr-hua {
  fill: #b45309;
}
.efr-yun {
  fill: #0e7490;
}
.efr-node-chars .yun {
  fill: #0e7490;
  font-weight: 700;
}
.efr-nian {
  fill: #be185d;
}
.efr-node-chars .nian {
  fill: #be185d;
  font-weight: 700;
}
.lg-nian {
  color: #be185d;
}
.efr-delta rect {
  fill: #0e7490;
  stroke: #fffdf8;
  stroke-width: 2;
}
.efr-delta.down rect {
  fill: #8a8f95;
}
.efr-delta text {
  font-size: 10px;
  font-weight: 800;
  fill: #fff;
}
.efr-extra {
  margin: 0 0 6px;
  padding: 5px 10px;
  border-radius: 8px;
  background: #ecf7f9;
  color: #155e75;
  text-align: center;
  font-size: calc(12px * var(--app-font-scale));
}
.lg-yun {
  color: #0e7490;
}
.efr-node-badge {
  fill: var(--c);
  stroke: #fffdf8;
  stroke-width: 2;
}
.efr-node-badge-text {
  font-size: 11px;
  font-weight: 800;
  fill: #fff;
}
.efr-node-role {
  font-size: 12px;
  font-weight: 700;
  fill: #4a4f55;
}
.efr-node-role-sub {
  font-weight: 400;
  fill: #8a8f95;
}
.efr-fav {
  font-weight: 800;
}
.efr-fav.good {
  fill: #23703c;
}
.efr-fav.bad {
  fill: #c53030;
}
.efr-node-hidden {
  font-size: 10px;
  fill: #8a8f95;
}

.efr-line-switch {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin: 2px 0 6px;
  color: var(--el-text-color-secondary);
  font-size: calc(12px * var(--app-font-scale));
}
.efr-line-switch button {
  padding: 2px 10px;
  border: 1px solid #e0d6c4;
  border-radius: 999px;
  background: #fffdf8;
  color: #6b7178;
  font-size: inherit;
  cursor: pointer;
}
.efr-line-switch button.on {
  border-color: #b98a3c;
  background: #fbf0dc;
  color: #7a4a10;
  font-weight: 700;
}
.lg-good {
  border-top: 4px solid #2f9e44;
  border-radius: 2px;
}
.lg-bad {
  border-top: 4px solid #c53030;
  border-radius: 2px;
}
.lg-none {
  border-top: 4px solid #b9b1a4;
  border-radius: 2px;
}
.efr-legend {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 2px 14px;
  margin: 4px 0 0;
  color: var(--el-text-color-secondary);
  font-size: calc(11px * var(--app-font-scale));
}
.efr-legend span {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.lg {
  display: inline-block;
  width: 18px;
  height: 0;
}
.lg-gen {
  border-top: 4px solid #2f7d45;
  border-radius: 2px;
}
.lg-ctl {
  border-top: 2px dashed #c53030;
}
.lg-ling {
  color: #7c3aed;
}
.lg-hua {
  color: #b45309;
}

/* 重點 */
.efr-points {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 8px;
}
.efr-point {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 10px;
  padding: 9px 12px;
  border: 1px solid #efe6d6;
  border-left: 4px solid #b9c0c8;
  border-radius: 10px;
  background: #fffdf9;
  cursor: pointer;
  transition: box-shadow 0.2s, background 0.2s;
}
.efr-point.is-good {
  border-left-color: #2f9e44;
}
.efr-point.is-bad {
  border-left-color: #c53030;
}
.efr-point.on {
  background: #fff7e8;
  box-shadow: 0 2px 10px rgba(120, 80, 20, 0.12);
}
.efr-point-no {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: #b9c0c8;
  color: #fff;
  font-size: 12px;
  font-weight: 800;
}
.efr-point.is-good .efr-point-no {
  background: #2f9e44;
}
.efr-point.is-bad .efr-point-no {
  background: #c53030;
}
.efr-point strong {
  font-size: calc(14px * var(--app-font-scale));
}
.efr-point p {
  margin: 2px 0 0;
  color: #4a4f55;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}
.efr-empty {
  color: var(--el-text-color-secondary);
}
</style>
