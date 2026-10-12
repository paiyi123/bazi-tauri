<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>結構分析圖解</h2>
      </div>
    </template>
    <p class="intro">
      照陸致極《八字命理學基礎教程》附錄的手算流程：五行字數不加權，括號內是地支附屬之氣；以十神計分找主導勢力定格局，再查表 7.1 取用神。只作對照參考。
    </p>

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

const signed = (value: number) => (value > 0 ? `+${value}` : `${value}`);

const climateFormula = computed(() => data.value.climate.parts.map((part) => `(${signed(part.score)})`).join("+"));
</script>

<style scoped>
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
