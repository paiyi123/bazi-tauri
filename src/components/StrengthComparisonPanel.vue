<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>日主強弱對照</h2>
      </div>
    </template>

    <div class="view-grid">
      <div v-for="view in views" :key="view.key" class="view-cell" :class="{ conflict: view.conflict }">
        <div class="view-source">{{ view.source }}</div>
        <div class="view-value">{{ view.value }}</div>
        <div class="view-sub">{{ view.sub }}</div>
      </div>
    </div>
    <div v-if="hasConflict" class="conflict-note">三種看法不一致，請展開各自的依據比對差在哪裡。</div>

    <template v-if="lesson?.checks?.length">
      <div class="subsection-title">課堂三步法強弱分析</div>
      <el-table :data="lesson.checks" size="small" border class="check-table">
        <el-table-column label="項目" width="84">
          <template #default="{ row }">
            <div>{{ row.item }}</div>
            <el-tag :type="row.hit ? 'success' : 'info'" effect="plain" size="small">{{ row.hit ? "✓ 有" : "✗ 無" }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="本命盤的情況與課堂定義" min-width="180">
          <template #default="{ row }">
            <div>{{ row.finding }}</div>
            <div class="rule-text">課堂定義（{{ row.sourceDate }}）：{{ row.rule }}</div>
          </template>
        </el-table-column>
      </el-table>

      <div class="party-list">
        <div class="party-row">
          <span class="party-label">同黨（印、比劫）{{ lesson.samePartyCount }} 個</span>
          <el-tag v-for="item in lesson.samePartyCharacters" :key="item" type="success" effect="plain" size="small">{{ item }}</el-tag>
        </div>
        <div class="party-row">
          <span class="party-label">異黨（食傷、財、官殺）{{ lesson.opposingPartyCount }} 個</span>
          <el-tag v-for="item in lesson.opposingPartyCharacters" :key="item" type="danger" effect="plain" size="small">{{ item }}</el-tag>
        </div>
      </div>

      <div class="conclusion">{{ lesson.verdictReason }}</div>
      <div v-if="lesson.balanceHint" class="conclusion balance-hint">{{ lesson.balanceHint }}</div>
      <el-collapse class="note-collapse">
        <el-collapse-item title="課堂判定表與限制" name="lesson-notes">
          <ul class="reason-list">
            <li>三者兼備、不見剋洩耗 → 旺極（專旺格候選）；三者兼備、見剋洩耗 → 旺（2026-07-22、07-29）。</li>
            <li>三者得其二 → 強（2026-07-22）；不得時但得勢得地 → 衰中轉強（2026-07-29）。</li>
            <li>三者只得其一 → 弱；三者皆無且滿盤剋洩耗 → 衰極，可能入從格。</li>
            <li>同黨異黨四比四時，看月令屬哪一黨：得令者為旺（2026-09-02、09-09）。</li>
            <li>被相鄰地支刑、沖的地支只剩本氣，裡面的中氣、餘氣根不算得地；自刑、或同時被合住的不算（2026-11-04）。</li>
            <li>月令是印、其餘七字印佔 4 個以上、不見比劫、日主沒有本氣根 → 母慈滅子，不論身強（2026-11-04 講義例一）。</li>
            <li v-for="item in lesson.limitations" :key="item">{{ item }}</li>
          </ul>
        </el-collapse-item>
      </el-collapse>
    </template>

    <template v-if="lu">
      <div class="subsection-title">陸致極強弱檢核</div>
      <el-table :data="lu.checks" size="small" border class="check-table">
        <el-table-column prop="item" label="項目" width="72" />
        <el-table-column prop="result" label="結果" min-width="160" />
        <el-table-column label="點數" width="72" align="right">
          <template #default="{ row }">{{ formatPoints(row.points) }}</template>
        </el-table-column>
      </el-table>
      <div class="subline">
        合計 {{ lu.score.toFixed(2) }} 分，門檻 {{ lu.threshold.toFixed(2) }}；同方（比劫＋印）{{ lu.samePartyCount }} / 8。
      </div>

      <div v-if="lu.roots.length" class="root-list">
        <el-tag
          v-for="root in lu.roots"
          :key="`${root.pillar}-${root.branch}`"
          :type="root.grade === '強根' ? 'success' : root.grade === '中根' ? 'warning' : 'info'"
          effect="plain"
        >
          {{ root.pillar }}{{ root.branch }}{{ root.hiddenStem !== "-" ? `藏${root.hiddenStem}` : "" }}・{{ root.grade }}
          <span v-if="root.note">（{{ root.note }}）</span>
        </el-tag>
      </div>

      <div v-if="lu.flags.length" class="flag-list">
        <el-alert
          v-for="flag in lu.flags"
          :key="flag.kind"
          :title="flag.title"
          :description="flag.detail"
          type="warning"
          :closable="false"
          show-icon
        />
      </div>

      <div class="conclusion">{{ lu.conclusion }}</div>
      <el-collapse class="note-collapse">
        <el-collapse-item title="來源與限制" name="notes">
          <ul class="reason-list">
            <li v-for="item in lu.sources" :key="item">{{ item }}</li>
            <li v-for="item in lu.limitations" :key="item">{{ item }}</li>
          </ul>
        </el-collapse-item>
      </el-collapse>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";

const props = defineProps<{
  result: BaziResponse;
}>();

const quant = computed(() => props.result.quantModel);
const lu = computed(() => quant.value?.luStrength);
const lesson = computed(() => quant.value?.lessonContent?.threeStepAnalysis?.strength);

function category(value: string | undefined): "強" | "弱" | null {
  if (!value) return null;
  if (value.includes("強") || value.includes("旺")) return "強";
  if (value.includes("弱") || value.includes("衰")) return "弱";
  return null;
}

const views = computed(() => {
  const quantLabel = quant.value?.strengthLabel ?? "—";
  const lessonLabel = lesson.value?.classification ?? "—";
  const luLabel = lu.value?.verdict ?? "—";
  const items = [
    {
      key: "quant",
      source: "量化五行力",
      value: quantLabel,
      sub: quant.value ? `總分 ${quant.value.totalScore.toFixed(1)}` : "",
      category: category(quantLabel),
    },
    {
      key: "lesson",
      source: "課堂三步法",
      value: lessonLabel,
      sub: lesson.value?.checks?.length
        ? lesson.value.checks.map((check) => `${check.item.replace("（得時）", "")}${check.hit ? "✓" : "✗"}`).join(" ")
        : "得令、得勢、得地",
      category: category(lessonLabel),
    },
    {
      key: "lu",
      source: "陸致極檢核",
      value: luLabel,
      sub: lu.value ? `得令「${lu.value.monthState}」` : "",
      category: category(luLabel),
    },
  ];
  const known = items.map((item) => item.category).filter((item) => item !== null);
  const majority = known.filter((item) => item === "強").length * 2 > known.length ? "強" : "弱";
  const split = new Set(known).size > 1;
  return items.map((item) => ({
    ...item,
    conflict: split && item.category !== null && item.category !== majority,
  }));
});

const hasConflict = computed(() => views.value.some((view) => view.conflict));

function formatPoints(value: number): string {
  return value > 0 ? `+${value.toFixed(2)}` : value.toFixed(2);
}
</script>

<style scoped>
.view-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.view-cell {
  border: 1px solid var(--el-border-color);
  border-radius: 12px;
  padding: 10px 12px;
  text-align: center;
}

.view-cell.conflict {
  border-color: var(--el-color-danger);
  background: var(--el-color-danger-light-9);
}

.view-source {
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
}

.view-value {
  margin-top: 4px;
  font-weight: 700;
  font-size: calc(18px * var(--app-font-scale));
  color: var(--bazi-primary);
}

.view-sub {
  margin-top: 2px;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
}

.conflict-note {
  margin-top: 8px;
  color: var(--el-color-danger);
  font-size: calc(13px * var(--app-font-scale));
}

.subsection-title {
  margin-top: 18px;
  font-weight: 700;
  font-size: calc(16px * var(--app-font-scale));
  color: var(--bazi-primary);
  letter-spacing: 0.03em;
}

.check-table {
  margin-top: 10px;
}

.subline {
  margin-top: 8px;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
}

.root-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.flag-list {
  display: grid;
  gap: 8px;
  margin-top: 12px;
}

.rule-text {
  margin-top: 2px;
  color: var(--bazi-text-muted);
  font-size: calc(12px * var(--app-font-scale));
}

.party-list {
  display: grid;
  gap: 6px;
  margin-top: 10px;
}

.party-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.party-label {
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  margin-right: 2px;
}

.conclusion {
  margin-top: 12px;
  line-height: 1.6;
}

.note-collapse {
  margin-top: 8px;
}

.reason-list {
  margin: 0;
  padding-left: 18px;
}

@media (max-width: 640px) {
  .view-grid {
    grid-template-columns: 1fr;
  }
}
</style>
