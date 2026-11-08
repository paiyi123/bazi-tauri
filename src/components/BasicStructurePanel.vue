<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>結構分析圖解（陸致極）</h2>
      </div>
    </template>
    <p class="intro">
      照陸致極《八字命理學基礎教程》附錄的手算流程：五行字數不加權，括號內是地支附屬之氣；以十神計分找主導勢力定格局，再查表 7.1 取用神。只作對照參考。
    </p>

    <svg class="wheel" viewBox="0 0 560 470" role="img" :aria-label="`五行生剋圖：${wheelLabel}`">
      <defs>
        <marker id="wheel-gen" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M0 0 L10 5 L0 10 z" fill="#57534e" />
        </marker>
        <marker id="wheel-ctl" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M0 0 L10 5 L0 10 z" fill="#b42318" />
        </marker>
      </defs>
      <line :x1="divider.x1" :y1="divider.y1" :x2="divider.x2" :y2="divider.y2" class="divider" />
      <text x="40" y="44" class="party-label">日主同方 {{ data.same.main }}({{ data.same.attached }})</text>
      <text x="520" y="452" text-anchor="end" class="party-label">日主異方 {{ data.opposite.main }}({{ data.opposite.attached }})</text>
      <path v-for="arc in genArcs" :key="arc.key" :d="arc.d" class="gen-arc" marker-end="url(#wheel-gen)" />
      <line
        v-for="ctl in ctlLines"
        :key="ctl.key"
        :x1="ctl.x1"
        :y1="ctl.y1"
        :x2="ctl.x2"
        :y2="ctl.y2"
        class="ctl-line"
        marker-end="url(#wheel-ctl)"
      />
      <g v-for="node in nodes" :key="node.element">
        <circle :cx="node.x" :cy="node.y" r="30" :class="['node', `node-${WX_CLASS[node.element]}`]" />
        <text :x="node.x" :y="node.y - 3" text-anchor="middle" :class="['node-element', `wx-${WX_CLASS[node.element]}`]">{{ node.element }}</text>
        <text :x="node.x" :y="node.y + 15" text-anchor="middle" class="node-relation">{{ node.relationLabel }}</text>
        <text :x="node.lx" :y="node.ly" :text-anchor="node.anchor" class="node-info">
          <tspan class="state-box">{{ node.state }}</tspan>
          <tspan dx="6">{{ node.main }}({{ node.attached }})</tspan>
        </text>
        <text :x="node.lx" :y="node.ly + 18" :text-anchor="node.anchor" class="node-gods">
          <tspan v-for="(god, i) in node.tenGods" :key="god.name" :dx="i ? 8 : 0" :class="{ lead: god.name === data.pattern.tenGod }">{{ god.name }} {{ god.count }}</tspan>
        </text>
      </g>
    </svg>
    <div class="wheel-legend">實線箭頭＝相生（順時針），紅色虛線＝相剋；斜線左上為日主同方（日主、生我），右下為日主異方（我生、我剋、剋我）。書圖 3.7、附錄 p297。</div>

    <el-table :data="data.elements" size="small" border class="element-table">
      <el-table-column label="五行" width="64">
        <template #default="{ row }">
          <span :class="['wx', `wx-${WX_CLASS[row.element]}`]">{{ row.element }}</span>
          <span class="relation">{{ row.relation }}</span>
        </template>
      </el-table-column>
      <el-table-column label="方" width="52" prop="party" />
      <el-table-column label="月令" width="52" prop="state" />
      <el-table-column label="字數" width="64" align="center">
        <template #default="{ row }">{{ row.main }}({{ row.attached }})</template>
      </el-table-column>
      <el-table-column label="十神計分" min-width="120">
        <template #default="{ row }">
          <span v-for="god in row.tenGods" :key="god.name" :class="['god', { lead: god.name === data.pattern.tenGod }]">
            {{ god.name }} {{ god.count }}
          </span>
        </template>
      </el-table-column>
    </el-table>

    <div class="subline">
      同方 {{ data.same.main }}({{ data.same.attached }})：異方 {{ data.opposite.main }}({{ data.opposite.attached }})；日主強弱取陸致極檢核的「{{ verdict || data.useGod.strength }}」。
    </div>
    <div class="subline">
      寒暖燥濕 {{ climateFormula }} = {{ signed(data.climate.total) }}，{{ data.climate.level }}（±6 以內為中和）。
    </div>

    <div class="result-grid">
      <div class="result-cell">
        <div class="result-label">格局（主導勢力）</div>
        <div class="result-value">{{ data.pattern.name }}</div>
        <div class="result-sub">{{ data.pattern.reason }}</div>
      </div>
      <div class="result-cell">
        <div class="result-label">用神（表 7.1）</div>
        <div class="result-value">{{ data.useGod.useGod ?? "—" }}</div>
        <div class="result-sub">{{ data.useGod.note }}</div>
      </div>
    </div>

    <ul v-if="data.checks.length" class="check-list">
      <li v-for="check in data.checks" :key="check.label" :class="`tone-${check.tone}`">
        <span class="check-label">{{ check.label }}</span>
        <span class="check-detail">{{ check.detail }}</span>
      </li>
    </ul>
    <p class="footnote">
      書上沒有給字數對應強弱等級的門檻，也有取格不依月令的命例（如教授申月取正印格），所以這裡的格局是程式依書上步驟推出的候選，請和課堂取格對照。大運流年格子上的「破格／成格」標籤依表 7.2、7.3 標出。
    </p>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { basicStructure } from "../utils/basicStructure";

const props = defineProps<{
  result: BaziResponse;
}>();

const WX_CLASS: Record<string, string> = { 木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water" };

const verdict = computed(() => props.result.quantModel?.luStrength?.verdict ?? "");

const data = computed(() =>
  basicStructure(
    [props.result.yearPillar, props.result.monthPillar, props.result.dayPillar, props.result.hourPillar].map(
      (pillar) => `${pillar.stem}${pillar.branch}`,
    ),
    verdict.value || undefined,
  ),
);

const RELATION_LABEL: Record<string, string> = { 日主: "日主・同我", 食傷: "我生", 財: "我剋", 官殺: "剋我", 印: "生我" };
const CX = 280;
const CY = 245;
const R = 150;
const angleOf = (index: number) => ((-90 + index * 72) * Math.PI) / 180;
const point = (angle: number, radius: number) => ({ x: CX + radius * Math.cos(angle), y: CY + radius * Math.sin(angle) });
const round = (value: number) => Math.round(value * 10) / 10;

const nodes = computed(() =>
  data.value.elements.map((row, index) => {
    const angle = angleOf(index);
    const center = point(angle, R);
    const label = point(angle, R + 48);
    const cos = Math.cos(angle);
    return {
      ...row,
      relationLabel: RELATION_LABEL[row.relation],
      x: round(center.x),
      y: round(center.y),
      lx: round(index === 0 ? center.x + 42 : label.x),
      ly: round(index === 0 ? center.y - 10 : label.y + (Math.sin(angle) > 0.5 ? 6 : -4)),
      anchor: index === 0 ? "start" : Math.abs(cos) < 0.2 ? "middle" : cos > 0 ? "start" : "end",
    };
  }),
);

// 相生：沿圓周順時針，避開節點圓。
const genArcs = computed(() =>
  [0, 1, 2, 3, 4].map((index) => {
    const pad = (14 * Math.PI) / 180;
    const from = point(angleOf(index) + pad, R);
    const to = point(angleOf(index + 1) - pad, R);
    return { key: `gen-${index}`, d: `M ${round(from.x)} ${round(from.y)} A ${R} ${R} 0 0 1 ${round(to.x)} ${round(to.y)}` };
  }),
);

// 相剋：隔一位的五角星虛線，兩端縮進節點圓外。
const ctlLines = computed(() =>
  [0, 1, 2, 3, 4].map((index) => {
    const a = point(angleOf(index), R);
    const b = point(angleOf(index + 2), R);
    const length = Math.hypot(b.x - a.x, b.y - a.y);
    const ux = (b.x - a.x) / length;
    const uy = (b.y - a.y) / length;
    return { key: `ctl-${index}`, x1: round(a.x + ux * 36), y1: round(a.y + uy * 36), x2: round(b.x - ux * 36), y2: round(b.y - uy * 36) };
  }),
);

// 分隔線：日主與我生之間、生我與剋我之間。
const divider = computed(() => {
  const start = point(((-54) * Math.PI) / 180, R + 70);
  const end = point((162 * Math.PI) / 180, R + 70);
  return { x1: round(start.x), y1: round(start.y), x2: round(end.x), y2: round(end.y) };
});

const wheelLabel = computed(() => nodes.value.map((node) => `${node.relationLabel}${node.element}${node.state}${node.main}(${node.attached})`).join("、"));

const signed = (value: number) => (value > 0 ? `+${value}` : `${value}`);

const climateFormula = computed(() => data.value.climate.parts.map((part) => `(${signed(part.score)})`).join("+"));
</script>

<style scoped>
.wheel {
  display: block;
  width: 100%;
  max-width: 560px;
  margin: 0 auto;
}

.divider {
  stroke: #78716c;
  stroke-width: 1.2;
}

.party-label {
  fill: #57534e;
  font-size: 15px;
  font-weight: 700;
}

.gen-arc {
  fill: none;
  stroke: #57534e;
  stroke-width: 1.6;
}

.ctl-line {
  stroke: #b42318;
  stroke-width: 1.3;
  stroke-dasharray: 3 4;
}

.node {
  fill: #fffdf8;
  stroke-width: 2.5;
}

.node-wood {
  stroke: var(--wx-wood);
}

.node-fire {
  stroke: var(--wx-fire);
}

.node-earth {
  stroke: var(--wx-earth);
}

.node-metal {
  stroke: var(--wx-metal);
}

.node-water {
  stroke: var(--wx-water);
}

.node-element {
  font-size: 20px;
  font-weight: 700;
  fill: currentColor;
}

.node-relation {
  fill: #57534e;
  font-size: 10px;
}

.node-info {
  fill: #292524;
  font-size: 15px;
  font-weight: 700;
}

.state-box {
  fill: #92400e;
}

.node-gods {
  fill: #57534e;
  font-size: 12.5px;
}

.node-gods .lead {
  fill: #292524;
  font-weight: 700;
  text-decoration: underline;
}

.wheel-legend {
  margin: 2px 0 10px;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.5;
}

.intro,
.footnote {
  margin: 0 0 8px;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}

.footnote {
  margin: 10px 0 0;
  font-size: calc(12px * var(--app-font-scale));
}

.wx {
  font-weight: 700;
}

.wx-wood {
  color: var(--wx-wood);
}

.wx-fire {
  color: var(--wx-fire);
}

.wx-earth {
  color: var(--wx-earth);
}

.wx-metal {
  color: var(--wx-metal);
}

.wx-water {
  color: var(--wx-water);
}

.relation {
  margin-left: 4px;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
}

.god {
  margin-right: 10px;
  white-space: nowrap;
}

.god.lead {
  font-weight: 700;
  text-decoration: underline;
}

.subline {
  margin-top: 6px;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
  overflow-wrap: anywhere;
}

.result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 8px;
  margin-top: 10px;
}

.result-cell {
  padding: 8px 10px;
  border: 1px solid rgba(148, 163, 184, 0.4);
  border-radius: 10px;
}

.result-label {
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
}

.result-value {
  font-size: calc(18px * var(--app-font-scale));
  font-weight: 700;
}

.result-sub {
  color: #44403c;
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.5;
}

.check-list {
  display: grid;
  gap: 6px;
  margin: 10px 0 0;
  padding: 0;
  list-style: none;
}

.check-list li {
  display: grid;
  gap: 2px;
  padding: 6px 10px;
  border-radius: 10px;
  background: rgba(148, 163, 184, 0.08);
  box-shadow: inset 3px 0 0 #94a3b8;
}

.check-list li.tone-good {
  background: rgba(47, 125, 69, 0.08);
  box-shadow: inset 3px 0 0 #2f7d45;
}

.check-list li.tone-bad {
  background: rgba(180, 35, 24, 0.06);
  box-shadow: inset 3px 0 0 #b42318;
}

.check-label {
  font-weight: 700;
}

.check-detail {
  color: #44403c;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}
</style>
