<template>
  <div class="analysis-topics" :class="{ compact }">
    <!-- 桌面版：所有分區由上到下排，頂端一列可跳到各區。手機版：用選單一次看一區。 -->
    <nav class="topic-nav" :class="{ sticky: !compact, 'has-side-nav': sideNav }" aria-label="分析分區">
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
          <ul v-if="leadsOf(topic.key).length" class="topic-lead">
            <li v-for="(line, idx) in leadsOf(topic.key)" :key="idx" :class="`tone-${line.tone}`"><GlossaryText :text="line.text" /></li>
          </ul>
          <template v-if="topic.key === 'steps'">
            <ReadingSteps :result="result" @goto="goto" />
          </template>
          <template v-else-if="topic.key === 'learn'">
            <LearnPanel :result="result" />
          </template>
          <template v-else-if="topic.key === 'strength'">
            <DisagreementList :result="result" />
            <StrengthComparisonPanel :result="result" />
            <YongShenPanel v-if="hasLesson" :result="result" />
            <SourceFold :open="isResearch" label="詳細推論與各書細節" :hint="isResearch ? '課堂三步法完整推論、四本書清濁調候、量化用神' : '課堂三步法完整推論、四本書清濁調候'">
              <CourseContentPanel v-if="hasLesson" :result="result" section="core" />
              <FourBooksPanel :result="result" :only="['clarity', 'quality', 'climate', 'flow']" title="清濁・用神品質・調候（四本書）" />
              <FiveElementStrengthPanel v-if="isResearch" :result="result" />
            </SourceFold>
          </template>
          <template v-else-if="topic.key === 'pattern'">
            <TenGodComboPanel :result="result" />
            <SourceFold :open="isResearch" label="各書細節" :hint="isResearch ? '蘇民峰特別格局、陸致極結構圖' : '蘇民峰特別格局'">
              <SpecialPatternPanel :result="result" />
              <BasicStructurePanel v-if="isResearch" :result="result" />
            </SourceFold>
          </template>
          <template v-else-if="topic.key === 'interaction'">
            <CourseContentPanel v-if="hasLesson" :result="result" section="interactions" />
            <SourceFold v-if="isResearch" open label="各書細節" hint="量化模型合沖、四本書五合化法">
              <TenGodPanel :result="result" />
              <FourBooksPanel :result="result" :only="['combine']" title="五合的三種化法（四本書）" />
            </SourceFold>
          </template>
          <template v-else-if="topic.key === 'luck'">
            <LuckCyclePanel :result="result" :compact="compact" @preview-change="emit('preview-change', $event)" />
            <SourceFold :open="isResearch" label="各書細節" :hint="isResearch ? '大運流年要訣、限運' : '大運流年要訣（四本書）'">
              <FourBookTopicsPanel :result="result" :only="['luck']" title="大運流年要訣（四本書）" />
              <FourBooksPanel v-if="isResearch" :result="result" :only="['limit']" title="限運：每柱管哪段年齡（四本書）" />
            </SourceFold>
          </template>
          <template v-else-if="topic.key === 'family'">
            <SourceFold :open="isResearch" label="各書細節" :hint="isResearch ? '四本書六親婚姻、陸致極婚姻、轉盤' : '四本書六親婚姻、陸致極婚姻'">
              <FourBookTopicsPanel :result="result" :only="['relatives', 'family', 'marriage']" title="六親・婚姻（四本書）" />
              <LifeTopicsPanel :result="result" :only="['marriage']" title="婚姻（陸致極）" />
              <FourBooksPanel v-if="isResearch" :result="result" :only="['rotation', 'love']" title="轉盤看親人・戀愛方式（四本書）" />
            </SourceFold>
          </template>
          <template v-else-if="topic.key === 'health'">
            <SourceFold :open="isResearch" label="各書細節" hint="陸致極健康、四本書健康">
              <LifeTopicsPanel :result="result" :only="['health']" title="健康（陸致極）" />
              <FourBookTopicsPanel :result="result" :only="['health']" title="健康（四本書）" />
            </SourceFold>
          </template>
          <template v-else-if="topic.key === 'career'">
            <SourceFold :open="isResearch" label="各書細節" hint="四本書事業財富學業、陸致極財運">
              <FourBookTopicsPanel :result="result" :only="['career', 'wealth', 'study']" title="事業・財富・學業（四本書）" />
              <LifeTopicsPanel :result="result" :only="['wealth']" title="財運（陸致極）" />
            </SourceFold>
          </template>
          <template v-else-if="topic.key === 'reference'">
            <QuantFivePhasePanel v-if="!compact" :result="result" :compact="false" @preview-change="emit('preview-change', $event)" />
            <CourseContentPanel v-if="hasLessonContent" :result="result" section="reference" />
          </template>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { BaziResponse, LuckPreviewPillar } from "../types/bazi";
import { useViewMode } from "../composables/useViewMode";
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
import SourceFold from "./SourceFold.vue";
import GlossaryText from "./GlossaryText.vue";
import ReadingSteps from "./ReadingSteps.vue";
import LearnPanel from "./LearnPanel.vue";
import DisagreementList from "./DisagreementList.vue";
import { topicLeads, type LeadLine } from "../utils/topicLeads";
import { visibleTopicList, type TopicKey } from "../constants/analysisTopics";


const props = defineProps<{
  result: BaziResponse;
  compact: boolean;
  /** 手機版的大運已經在「運勢」分頁，這裡可以排除。 */
  exclude?: TopicKey[];
  /** 電腦版寬螢幕時左欄已有目錄，這一列就不用再顯示。 */
  sideNav?: boolean;
}>();

const emit = defineEmits<{
  "preview-change": [pillars: LuckPreviewPillar[] | null];
  /** 要看的分區不在這裡（例如手機版大運在「運勢」分頁）。 */
  "goto-outside": [topic: string];
}>();

const { isResearch } = useViewMode();
// 沒有課堂資料時，課堂卡片只會顯示「尚無分析」，不如不顯示。
const hasLessonContent = computed(() => !!props.result.quantModel?.lessonContent);
const hasLesson = computed(() => !!props.result.quantModel?.lessonContent?.threeStepAnalysis);
// 新手模式不顯示「量化計分與上課參考」分區，那些是研究用的原始資料。
const visibleTopics = computed(() => visibleTopicList(isResearch.value, props.exclude));
const current = ref<TopicKey>(visibleTopics.value[0]?.key ?? "strength");
const shownTopics = computed(() => {
  if (!props.compact) return visibleTopics.value;
  const chosen = visibleTopics.value.find((topic) => topic.key === current.value) ?? visibleTopics.value[0];
  return chosen ? [chosen] : [];
});

// 每個主題最上面的一句白話重點（大運、量化分區沒有）。
const leads = computed(() => {
  const map = new Map<TopicKey, LeadLine[]>();
  for (const key of ["strength", "pattern", "interaction", "family", "health", "career"] as const) {
    map.set(key, topicLeads(props.result, key));
  }
  return map;
});
const leadsOf = (key: TopicKey) => leads.value.get(key) ?? [];

function goto(topic: string) {
  const target = visibleTopics.value.find((item) => item.key === topic);
  if (target) choose(target.key);
  else emit("goto-outside", topic);
}

function choose(key: TopicKey) {
  if (props.compact) {
    current.value = key;
    return;
  }
  document.getElementById(`topic-${key}`)?.scrollIntoView({ behavior: "smooth", block: "start" });
}
</script>

<style scoped>
@media (min-width: 1100px) {
  .topic-nav.has-side-nav {
    display: none;
  }
}
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

.topic-lead {
  display: grid;
  gap: 6px;
  margin: 0;
  padding: 12px 16px;
  border-radius: 14px;
  background: var(--bazi-surface-strong);
  box-shadow: 0 1px 4px rgba(45, 33, 24, 0.08);
  list-style: none;
  font-size: calc(15px * var(--app-font-scale));
  line-height: 1.65;
}

.topic-lead li {
  padding-left: 12px;
  border-left: 4px solid var(--bazi-border-strong);
}

.topic-lead .tone-good { border-left-color: #2f9e44; }
.topic-lead .tone-bad { border-left-color: var(--bazi-danger); }
.topic-lead .tone-mixed { border-left-color: var(--bazi-gold); }

.topic-body {
  display: grid;
  gap: 18px;
  min-width: 0;
}

.compact .topic-body {
  gap: 10px;
}
</style>
