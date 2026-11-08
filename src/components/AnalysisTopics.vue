<template>
  <div class="analysis-topics" :class="{ compact }">
    <!-- 桌面版：所有分區由上到下排，頂端一列可跳到各區。手機版：用選單一次看一區。 -->
    <nav class="topic-nav" :class="{ sticky: !compact }" aria-label="分析分區">
      <button
        v-for="topic in visibleTopics"
        :key="topic.key"
        type="button"
        class="topic-chip"
        :class="{ active: compact && current === topic.key }"
        @click="choose(topic.key)"
      >
        {{ topic.short }}
      </button>
    </nav>

    <template v-for="topic in shownTopics" :key="topic.key">
      <section :id="`topic-${topic.key}`" class="topic-section">
        <header class="topic-header">
          <h2>{{ topic.title }}</h2>
          <p>{{ topic.desc }}</p>
        </header>
        <div class="topic-body">
          <template v-if="topic.key === 'strength'">
            <StrengthComparisonPanel :result="result" />
            <CourseContentPanel :result="result" section="core" />
            <YongShenPanel :result="result" />
            <FourBooksPanel :result="result" :only="['clarity', 'quality', 'climate', 'flow']" title="清濁・用神品質・調候（四本書）" />
            <FiveElementStrengthPanel :result="result" />
          </template>
          <template v-else-if="topic.key === 'pattern'">
            <BasicStructurePanel :result="result" />
            <SpecialPatternPanel :result="result" />
            <TenGodComboPanel :result="result" />
          </template>
          <template v-else-if="topic.key === 'interaction'">
            <CourseContentPanel :result="result" section="interactions" />
            <TenGodPanel :result="result" />
            <FourBooksPanel :result="result" :only="['combine']" title="五合的三種化法（四本書）" />
          </template>
          <template v-else-if="topic.key === 'luck'">
            <LuckCyclePanel :result="result" :compact="compact" @preview-change="emit('preview-change', $event)" />
            <FourBookTopicsPanel :result="result" :only="['luck']" title="大運流年要訣（四本書）" />
            <FourBooksPanel :result="result" :only="['limit']" title="限運：每柱管哪段年齡（四本書）" />
          </template>
          <template v-else-if="topic.key === 'family'">
            <FourBookTopicsPanel :result="result" :only="['relatives', 'family', 'marriage']" title="六親・婚姻（四本書）" />
            <LifeTopicsPanel :result="result" :only="['marriage']" title="婚姻（陸致極）" />
            <FourBooksPanel :result="result" :only="['rotation', 'love']" title="轉盤看親人・戀愛方式（四本書）" />
          </template>
          <template v-else-if="topic.key === 'health'">
            <LifeTopicsPanel :result="result" :only="['health']" title="健康（陸致極）" />
            <FourBookTopicsPanel :result="result" :only="['health']" title="健康（四本書）" />
          </template>
          <template v-else-if="topic.key === 'career'">
            <FourBookTopicsPanel :result="result" :only="['career', 'wealth', 'study']" title="事業・財富・學業（四本書）" />
            <LifeTopicsPanel :result="result" :only="['wealth']" title="財運（陸致極）" />
          </template>
          <template v-else-if="topic.key === 'reference'">
            <QuantFivePhasePanel v-if="!compact" :result="result" :compact="false" @preview-change="emit('preview-change', $event)" />
            <CourseContentPanel :result="result" section="reference" />
          </template>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { BaziResponse, LuckPreviewPillar } from "../types/bazi";
import BasicStructurePanel from "./BasicStructurePanel.vue";
import CourseContentPanel from "./CourseContentPanel.vue";
import FiveElementStrengthPanel from "./FiveElementStrengthPanel.vue";
import FourBookTopicsPanel from "./FourBookTopicsPanel.vue";
import FourBooksPanel from "./FourBooksPanel.vue";
import LifeTopicsPanel from "./LifeTopicsPanel.vue";
import LuckCyclePanel from "./LuckCyclePanel.vue";
import QuantFivePhasePanel from "./QuantFivePhasePanel.vue";
import SpecialPatternPanel from "./SpecialPatternPanel.vue";
import StrengthComparisonPanel from "./StrengthComparisonPanel.vue";
import TenGodComboPanel from "./TenGodComboPanel.vue";
import TenGodPanel from "./TenGodPanel.vue";
import YongShenPanel from "./YongShenPanel.vue";

type TopicKey = "strength" | "pattern" | "interaction" | "luck" | "family" | "health" | "career" | "reference";

const props = defineProps<{
  result: BaziResponse;
  compact: boolean;
  /** 手機版的大運已經在「運勢」分頁，這裡可以排除。 */
  exclude?: TopicKey[];
}>();

const emit = defineEmits<{
  "preview-change": [pillars: LuckPreviewPillar[] | null];
}>();

const TOPICS: { key: TopicKey; short: string; title: string; desc: string }[] = [
  { key: "luck", short: "大運流年", title: "大運流年", desc: "大運、流年、流月與各書的運歲標籤；看一年的步驟、限運年齡。" },
  { key: "strength", short: "強弱用神", title: "強弱・用神・喜忌", desc: "日主強弱、用神喜神忌神、清濁與調候。其他各區說的「喜／忌」都以課堂三步法為準。" },
  { key: "pattern", short: "格局", title: "格局・十神組合", desc: "結構與格局成敗、特別格局、十神組合。" },
  { key: "interaction", short: "合沖刑害", title: "干支作用（合沖刑害）", desc: "天干五合、地支沖合刑害、同柱干支，以及作用之後的強弱複核。" },
  { key: "family", short: "六親婚姻", title: "六親・婚姻", desc: "父母、兄弟、配偶、子女的星與宮，擇偶、婚期、轉盤看親人。" },
  { key: "health", short: "健康", title: "健康", desc: "五行太過不及、相剋的病症、需要留意的身體部位。" },
  { key: "career", short: "事業財富", title: "事業・財富・學業", desc: "適合的行業、財的多寡與來源、讀書與考運。" },
  { key: "reference", short: "量化・上課", title: "量化計分與上課參考", desc: "量化五行力的計分明細、課程流程與課堂命例比對。" },
];

const visibleTopics = computed(() => TOPICS.filter((topic) => !props.exclude?.includes(topic.key)));
const current = ref<TopicKey>(visibleTopics.value[0]?.key ?? "strength");
const shownTopics = computed(() =>
  props.compact ? visibleTopics.value.filter((topic) => topic.key === current.value) : visibleTopics.value,
);

function choose(key: TopicKey) {
  if (props.compact) {
    current.value = key;
    return;
  }
  document.getElementById(`topic-${key}`)?.scrollIntoView({ behavior: "smooth", block: "start" });
}
</script>

<style scoped>
.analysis-topics {
  display: grid;
  gap: 18px;
  min-width: 0;
}

.analysis-topics.compact {
  gap: 10px;
}

.topic-nav {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px 10px;
  border-radius: 12px;
  background: var(--bazi-surface-strong);
  box-shadow: 0 1px 4px rgba(45, 33, 24, 0.12);
}

.topic-nav.sticky {
  position: sticky;
  top: 0;
  z-index: 5;
}

.topic-chip {
  padding: 4px 12px;
  border: 1px solid rgba(115, 95, 76, 0.35);
  border-radius: 999px;
  background: transparent;
  color: var(--bazi-text);
  font-size: calc(13px * var(--app-font-scale));
  cursor: pointer;
}

.topic-chip:hover,
.topic-chip.active {
  border-color: #b45309;
  background: rgba(180, 83, 9, 0.12);
  color: #92400e;
  font-weight: 600;
}

.topic-section {
  display: grid;
  gap: 12px;
  min-width: 0;
  scroll-margin-top: 64px;
}

.compact .topic-section {
  gap: 10px;
}

.topic-header {
  padding: 4px 0 2px 12px;
  border-left: 5px solid #b45309;
}

.topic-header h2 {
  margin: 0;
  color: var(--bazi-text);
  font-size: calc(22px * var(--app-font-scale));
}

.compact .topic-header h2 {
  font-size: calc(19px * var(--app-font-scale));
}

.topic-header p {
  margin: 2px 0 0;
  color: var(--bazi-text-muted);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.5;
}

.topic-body {
  display: grid;
  gap: 18px;
  min-width: 0;
}

.compact .topic-body {
  gap: 10px;
}
</style>
