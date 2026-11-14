<template>
  <div class="mini-chart" role="group" aria-label="命盤速覽">
    <div class="pillars">
      <span v-for="item in pillars" :key="item.key" class="pillar" :class="{ day: item.key === 'day' }">
        <span :class="`wx-${item.stemElement}`">{{ item.stem }}</span><span :class="`wx-${item.branchElement}`">{{ item.branch }}</span>
      </span>
    </div>
    <div v-if="data" class="facts">
      <strong>{{ data.strength.label }}</strong>
      <span v-if="data.useElements.length" class="group">
        用<span v-for="item in data.useElements" :key="item.element" class="dot" :class="`bg-${item.element}`">{{ item.name }}</span>
      </span>
      <span v-if="data.unfavorable.length" class="group">
        忌<span v-for="item in data.unfavorable" :key="item.element" class="dot ghost" :class="`wx-${item.element}`">{{ item.name }}</span>
      </span>
      <span v-if="data.annual" class="group year" :class="`v-${data.annual.overall}`">
        今年{{ data.annual.ganZhi }}{{ ({ good: "▲", bad: "▼", mixed: "◆", neutral: "" })[data.annual.overall] }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { BRANCH_WUXING, STEM_WUXING } from "../constants/wuxing";
import { conclusionOf } from "../utils/conclusion";

// 手機版頂端固定的一條命盤速覽：四柱（時日月年）＋強弱、用神、忌神、今年。切到解盤、運勢時不用回頭找命盤。
const props = defineProps<{ result: BaziResponse }>();

const data = computed(() => conclusionOf(props.result));
const pillars = computed(() =>
  [
    { key: "hour", pillar: props.result.hourPillar },
    { key: "day", pillar: props.result.dayPillar },
    { key: "month", pillar: props.result.monthPillar },
    { key: "year", pillar: props.result.yearPillar },
  ].map(({ key, pillar }) => ({
    key,
    stem: pillar?.stem ?? "",
    branch: pillar?.branch ?? "",
    stemElement: STEM_WUXING[pillar?.stem ?? ""] ?? "earth",
    branchElement: BRANCH_WUXING[pillar?.branch ?? ""] ?? "earth",
  })),
);
</script>

<style scoped>
.mini-chart {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 12px;
  padding: 8px 12px;
  border: 1px solid var(--bazi-border);
  border-radius: 14px;
  background: rgba(255, 252, 246, 0.97);
  box-shadow: var(--bazi-shadow-sm);
  font-size: calc(14px * var(--app-font-scale));
  line-height: 1.4;
}
.pillars {
  display: flex;
  gap: 8px;
  font-size: 1.14em;
  font-weight: 700;
  letter-spacing: 0.04em;
}
.pillar.day {
  padding: 0 3px;
  border-bottom: 2px solid var(--bazi-accent);
}
.facts {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 2px 10px;
}
.group {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  color: var(--bazi-text-muted);
}
.dot {
  display: inline-block;
  min-width: 1.5em;
  padding: 0 0.3em;
  border: 1px solid currentColor;
  border-radius: 999px;
  color: #fff;
  font-weight: 700;
  text-align: center;
}
.dot.ghost {
  background: transparent;
}
.bg-wood { background: var(--wx-wood); border-color: var(--wx-wood); }
.bg-fire { background: var(--wx-fire); border-color: var(--wx-fire); }
.bg-earth { background: var(--wx-earth); border-color: var(--wx-earth); }
.bg-metal { background: var(--wx-metal); border-color: var(--wx-metal); }
.bg-water { background: var(--wx-water); border-color: var(--wx-water); }
.wx-wood { color: var(--wx-wood); }
.wx-fire { color: var(--wx-fire); }
.wx-earth { color: var(--wx-earth); }
.wx-metal { color: var(--wx-metal); }
.wx-water { color: var(--wx-water); }
.year { font-weight: 600; }
.v-good { color: #23703c; }
.v-bad { color: var(--bazi-danger); }
</style>
