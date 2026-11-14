<template>
  <el-card shadow="never" class="reading-steps">
    <template #header>
      <div class="head">
        <p class="intro">
          照老師的順序：核對 → 定格 → 干支作用 → 強弱 → 特別格局 → 用神 → 喜忌 → 人生各項 → 大運流年 → 反驗。
          <template v-if="practiceMode">練習模式會先藏起第 4、6 步的答案，請自己判斷後再看答案。</template>
        </p>
        <label class="practice-toggle">
          <el-switch v-model="practiceMode" size="small" />
          <span>練習模式</span>
        </label>
      </div>
    </template>

    <ol class="steps">
      <li v-for="step in steps" :key="step.no" :class="['step', `is-${step.status}`, { locked: !learned(step) }]">
        <details :open="openSteps.has(step.no)" @toggle="onToggle(step.no, $event)">
          <summary>
            <span class="no">{{ step.no }}</span>
            <span class="title">{{ step.title }}</span>
            <span class="icon" :title="STATUS_TITLE[step.status]">{{ STATUS_ICON[step.status] }}</span>
            <span v-if="!learned(step)" class="summary muted">尚未學到（依課程進度），點開預覽</span>
            <span v-else-if="isQuiz(step) && !revealed.has(step.no)" class="summary muted">練習：先自己判斷</span>
            <span v-else class="summary">{{ step.summary }}</span>
          </summary>

          <div class="body">
            <!-- 練習：第 4 步強弱 -->
            <div v-if="isQuiz(step) && step.no === 4 && strengthQuiz" class="quiz">
              <div v-for="item in strengthQuiz.items" :key="item.key" class="quiz-row">
                <span class="quiz-label"><GlossaryText :text="item.label" /></span>
                <el-radio-group v-model="answers[item.key]" size="small" :disabled="revealed.has(4)">
                  <el-radio-button value="yes">有</el-radio-button>
                  <el-radio-button value="no">無</el-radio-button>
                </el-radio-group>
                <span v-if="revealed.has(4)" :class="answers[item.key] === item.answer ? 'right' : 'wrong'">
                  {{ answers[item.key] === item.answer ? "✓" : "✗" }} 答案：{{ item.answer === "yes" ? "有" : "無" }}
                </span>
              </div>
              <div class="quiz-row">
                <span class="quiz-label">判斷</span>
                <el-radio-group v-model="answers.side" size="small" :disabled="revealed.has(4)">
                  <el-radio-button value="強">身強這邊</el-radio-button>
                  <el-radio-button value="弱">身弱這邊</el-radio-button>
                </el-radio-group>
                <span v-if="revealed.has(4)" :class="answers.side === strengthQuiz.side ? 'right' : 'wrong'">
                  {{ answers.side === strengthQuiz.side ? "✓" : "✗" }} 答案：身{{ strengthQuiz.side }}
                </span>
              </div>
              <el-button v-if="!revealed.has(4)" size="small" type="primary" @click="reveal(4)">看答案</el-button>
            </div>

            <!-- 練習：第 6 步用神 -->
            <div v-if="isQuiz(step) && step.no === 6 && useQuiz.length" class="quiz">
              <div class="quiz-row">
                <span class="quiz-label">用神是？</span>
                <el-radio-group v-model="answers.use" size="small" :disabled="revealed.has(6)">
                  <el-radio-button v-for="name in ELEMENTS" :key="name" :value="name">{{ name }}</el-radio-button>
                </el-radio-group>
                <span v-if="revealed.has(6)" :class="useQuiz.includes(answers.use) ? 'right' : 'wrong'">
                  {{ useQuiz.includes(answers.use) ? "✓" : "✗" }} 答案：{{ useQuiz.join("、") }}
                </span>
              </div>
              <el-button v-if="!revealed.has(6)" size="small" type="primary" @click="reveal(6)">看答案</el-button>
            </div>

            <template v-if="!isQuiz(step) || revealed.has(step.no)">
              <p v-if="isQuiz(step)" class="summary-line"><GlossaryText :text="step.summary" /></p>
              <ul v-if="step.why.length" class="why">
                <li v-for="(line, idx) in step.why" :key="idx"><GlossaryText :text="line" /></li>
              </ul>
            </template>
            <el-button v-if="step.topic" link type="primary" @click="emit('goto', step.topic)">看詳細 →</el-button>
          </div>
        </details>
      </li>
    </ol>
  </el-card>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import type { BaziResponse } from "../types/bazi";
import { readingSteps, type ReadingStep, type StepStatus } from "../utils/readingSteps";
import { conclusionOf } from "../utils/conclusion";
import { isLearned, useLearning } from "../composables/useLearning";
import GlossaryText from "./GlossaryText.vue";

const props = defineProps<{ result: BaziResponse }>();
const emit = defineEmits<{ goto: [topic: string] }>();

const { courseStage, practiceMode } = useLearning();
const ELEMENTS = ["木", "火", "土", "金", "水"];
const STATUS_ICON: Record<StepStatus, string> = { ok: "✓", warn: "⚠", info: "・", none: "—" };
const STATUS_TITLE: Record<StepStatus, string> = { ok: "沒有特別狀況", warn: "這一步要特別注意", info: "參考", none: "沒有資料" };

const steps = computed(() => readingSteps(props.result));
const openSteps = ref(new Set<number>());
const revealed = ref(new Set<number>());
const answers = reactive<Record<string, string>>({});

const strengthQuiz = computed(() => {
  const strength = props.result.quantModel?.lessonContent?.threeStepAnalysis?.strength;
  const data = conclusionOf(props.result);
  if (!strength?.checks?.length || !data) return null;
  const side = data.strength.label.includes("強") || data.strength.label.includes("旺") ? "強" : "弱";
  return {
    items: strength.checks.map((check, index) => ({
      key: `check-${index}`,
      label: check.item.replace(/（.*?）/, ""),
      answer: check.hit ? "yes" : "no",
    })),
    side,
  };
});
const useQuiz = computed(() => conclusionOf(props.result)?.useElements.map((item) => item.name) ?? []);

// 換命盤或關掉練習模式時，答案重來。
watch(
  () => [props.result, practiceMode.value],
  () => {
    revealed.value = new Set();
    Object.keys(answers).forEach((key) => delete answers[key]);
  },
);

function learned(step: ReadingStep) {
  return isLearned(courseStage.value, step.needs);
}

function isQuiz(step: ReadingStep) {
  return practiceMode.value && learned(step) && ((step.no === 4 && !!strengthQuiz.value) || (step.no === 6 && useQuiz.value.length > 0));
}

function reveal(no: number) {
  revealed.value = new Set([...revealed.value, no]);
}

function onToggle(no: number, event: Event) {
  const open = (event.target as HTMLDetailsElement).open;
  const next = new Set(openSteps.value);
  if (open) next.add(no);
  else next.delete(no);
  openSteps.value = next;
}
</script>

<style scoped>
.reading-steps {
  min-width: 0;
  font-size: calc(15px * var(--app-font-scale));
  line-height: 1.6;
}
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 6px 12px;
}
.practice-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--bazi-text-muted);
  font-size: 0.9em;
  cursor: pointer;
}
.intro {
  flex: 1 1 240px;
  margin: 0;
  color: var(--bazi-text-muted);
  font-size: 0.88em;
}
.steps {
  display: grid;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}
.step details {
  border: 1px solid var(--bazi-border);
  border-left: 4px solid var(--bazi-border-strong);
  border-radius: 10px;
  background: var(--bazi-surface-strong);
}
.step.is-ok details { border-left-color: #2f9e44; }
.step.is-warn details { border-left-color: var(--bazi-gold); }
.step.is-none details { opacity: 0.7; }
.step.locked details { opacity: 0.6; }
summary {
  display: grid;
  grid-template-columns: 1.8em auto auto minmax(0, 1fr);
  align-items: baseline;
  gap: 2px 8px;
  padding: 8px 12px;
  cursor: pointer;
  list-style: none;
}
summary::-webkit-details-marker {
  display: none;
}
.no {
  display: inline-grid;
  place-items: center;
  width: 1.6em;
  height: 1.6em;
  border-radius: 50%;
  background: var(--bazi-primary);
  color: #fffaf0;
  font-size: 0.82em;
  font-weight: 700;
}
.title {
  font-weight: 700;
  white-space: nowrap;
}
.icon {
  color: var(--bazi-text-muted);
}
.step.is-ok .icon { color: #2f9e44; }
.step.is-warn .icon { color: #a16207; }
.summary {
  min-width: 0;
  color: var(--bazi-text);
  font-size: 0.92em;
}
.muted {
  color: var(--bazi-text-muted);
}
.body {
  display: grid;
  gap: 6px;
  padding: 0 14px 10px 2.9em;
}
.why {
  margin: 0;
  padding-left: 1.1em;
  color: var(--bazi-text);
  font-size: 0.92em;
}
.summary-line {
  margin: 0;
  font-weight: 600;
}
.quiz {
  display: grid;
  gap: 6px;
  padding: 8px 10px;
  border-radius: 8px;
  background: #fff6e6;
}
.quiz-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 4px 10px;
}
.quiz-label {
  min-width: 4em;
  font-weight: 600;
}
.right { color: #23703c; font-weight: 600; }
.wrong { color: var(--bazi-danger); font-weight: 600; }
.quiz .el-button {
  justify-self: start;
}
@media (max-width: 560px) {
  summary {
    grid-template-columns: 1.8em minmax(0, 1fr) auto;
  }
  .summary {
    grid-column: 2 / -1;
  }
  .title {
    white-space: normal;
  }
  .body {
    padding-left: 14px;
  }
}
</style>
