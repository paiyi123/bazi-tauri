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
      sub: "得令、得勢、得地",
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
