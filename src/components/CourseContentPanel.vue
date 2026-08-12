<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>上課內容</h2>
      </div>
    </template>

    <template v-if="lessonContent">
      <el-alert type="info" :closable="false" show-icon :title="lessonContent.status" />
      <el-descriptions :column="1" border class="course-content-box">
        <el-descriptions-item label="課程流程">
          <ol class="analysis-order">
            <li v-for="step in lessonContent.analysisOrder" :key="step">{{ step }}</li>
          </ol>
        </el-descriptions-item>
        <el-descriptions-item label="規則庫說明">
          {{ lessonContent.futureRuleNotice }}
        </el-descriptions-item>
      </el-descriptions>

      <section v-if="visibleInferences.length" class="inference-section">
        <h3>本命套用結果</h3>
        <article
          v-for="inference in visibleInferences"
          :key="inference.ruleId"
          class="inference-item"
        >
          <div class="inference-heading">
            <strong>{{ inference.title }}</strong>
            <div class="inference-tags">
              <el-tag size="small" type="success" effect="plain">{{ inference.status }}</el-tag>
              <el-tag size="small" type="info" effect="plain">
                {{ inference.algorithmStatus }}／{{ inference.outputMode }}
              </el-tag>
            </div>
          </div>
          <p>{{ inference.conclusion }}</p>
          <div class="comparison-grid">
            <div>
              <strong>App 原值</strong>
              <span>格局：{{ inference.appReference.geJu }}</span>
              <span>強弱：{{ inference.appReference.strengthLabel }}</span>
              <span>特殊格：{{ inference.appReference.specialPattern }}</span>
            </div>
            <div v-if="inference.lessonCandidate">
              <strong>課堂候選值</strong>
              <span>格局：{{ inference.lessonCandidate.pattern }}</span>
              <span>強弱：{{ inference.lessonCandidate.strength }}</span>
              <span>用神候選：{{ inference.lessonCandidate.useElements.join("、") }}</span>
            </div>
          </div>
          <div class="policy-line">
            比對：{{ inference.agreementStatus }}；fixture：{{ inference.fixtureScope }}；策略：{{ inference.fallbackPolicy }}
          </div>
          <div class="inference-columns">
            <div>
              <strong>套用證據</strong>
              <ul>
                <li v-for="evidence in inference.evidence" :key="evidence">{{ evidence }}</li>
              </ul>
            </div>
            <div>
              <strong>排除／保留</strong>
              <ul>
                <li
                  v-for="alternative in inference.rejectedAlternatives"
                  :key="alternative"
                >
                  {{ alternative }}
                </li>
              </ul>
            </div>
          </div>
        </article>
      </section>

      <h3 v-if="lessonContent.lessons.length" class="lesson-library-title">課程規則庫</h3>
      <div v-if="lessonContent.lessons.length" class="lesson-list">
        <article v-for="lesson in lessonContent.lessons" :key="lesson.id" class="lesson-item">
          <div class="lesson-heading">
            <div>
              <span class="lesson-date">{{ lesson.sourceDate }}</span>
              <h3>{{ lesson.title }}</h3>
            </div>
            <el-tag size="small" effect="plain">{{ lesson.category }}</el-tag>
          </div>
          <p class="lesson-summary">{{ lesson.summary }}</p>
          <el-alert
            class="lesson-status"
            type="warning"
            :closable="false"
            :title="`演算法狀態：${lesson.algorithmStatus}`"
          />
          <div class="lesson-meta">
            <el-tag size="small" type="info" effect="plain">驗證：{{ lesson.verification }}</el-tag>
            <el-tag
              size="small"
              :type="lesson.fixtureAllowed ? 'success' : 'danger'"
              effect="plain"
            >
              {{ lesson.fixtureAllowed ? "可作 fixture" : "禁止作 fixture" }}
            </el-tag>
          </div>
          <div class="lesson-evidence">
            <strong>規則／證據</strong>
            <ul>
              <li v-for="item in lesson.evidence" :key="item">{{ item }}</li>
            </ul>
          </div>
        </article>
      </div>
    </template>

    <el-empty v-else description="尚未提供上課內容規則庫。" :image-size="72" />
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";

const props = defineProps<{
  result: BaziResponse;
}>();

const lessonContent = computed(() => props.result.quantModel?.lessonContent);
const visibleInferences = computed(
  () => lessonContent.value?.inferences.filter((inference) => inference.matchStatus !== "not_matched") ?? [],
);
</script>

<style scoped>
.course-content-box {
  margin-top: 12px;
}

.analysis-order {
  margin: 0;
  padding-left: 1.35rem;
}

.analysis-order li + li {
  margin-top: 4px;
}

.inference-section {
  margin-top: 16px;
}

.inference-section > h3,
.lesson-library-title {
  margin: 0 0 10px;
  font-size: 1.05rem;
}

.lesson-library-title {
  margin-top: 18px;
}

.inference-item {
  padding: 14px 16px;
  border: 1px solid var(--el-color-success-light-5);
  border-radius: 8px;
  background: var(--el-color-success-light-9);
}

.inference-item + .inference-item {
  margin-top: 10px;
}

.inference-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.inference-tags {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
}

.inference-item p {
  margin: 10px 0;
  line-height: 1.65;
}

.comparison-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  margin: 10px 0;
}

.comparison-grid > div {
  display: grid;
  gap: 4px;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--el-fill-color-blank);
}

.policy-line {
  margin: 8px 0 12px;
  color: var(--el-text-color-secondary);
  font-size: 0.82rem;
}

.inference-columns {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.inference-columns ul {
  margin: 6px 0 0;
  padding-left: 1.25rem;
}

.inference-columns li + li {
  margin-top: 4px;
}

.lesson-list {
  display: grid;
  gap: 12px;
  margin-top: 16px;
}

.lesson-item {
  padding: 14px 16px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  background: var(--el-fill-color-blank);
}

.lesson-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.lesson-heading h3 {
  margin: 3px 0 0;
  font-size: 1rem;
}

.lesson-date {
  color: var(--el-text-color-secondary);
  font-size: 0.8rem;
}

.lesson-summary {
  margin: 10px 0;
  line-height: 1.65;
}

.lesson-status {
  margin: 8px 0;
}

.lesson-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 8px 0;
}

.lesson-evidence strong {
  font-size: 0.9rem;
}

.lesson-evidence ul {
  margin: 6px 0 0;
  padding-left: 1.25rem;
}

.lesson-evidence li + li {
  margin-top: 4px;
}

@media (max-width: 720px) {
  .comparison-grid,
  .inference-columns {
    grid-template-columns: 1fr;
  }
}
</style>
