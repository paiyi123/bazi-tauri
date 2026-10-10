<template>
  <section class="lesson-three-step" aria-label="三步本命分析">
    <template v-if="analysis">
      <header class="lesson-summary">
        <h3>三步本命分析</h3>
        <p class="status-line">整體狀態：{{ statusLabel(analysis.status) }}</p>
        <p>{{ analysis.facts.conclusion }}</p>
      </header>

      <article class="lesson-step">
        <h4>{{ analysis.pattern.title }}</h4><p>{{ analysis.pattern.conclusion }}</p>
        <ul v-if="analysis.pattern.ordinaryCandidates.length" class="candidate-list">
          <li v-for="candidate in analysis.pattern.ordinaryCandidates" :key="candidate.pattern + candidate.monthHiddenStem">
            <strong>{{ candidate.pattern }}</strong>（{{ statusLabel(candidate.selection) }}）— 月令{{ candidate.monthHiddenStem }}{{ candidate.hiddenStemLayer }}；透干：{{ candidate.transparentLocations.join('、') || '未見' }}
          </li>
        </ul>
        <p v-else>普通格候選：依據不足。</p>
        <details><summary>特殊格與格局證據</summary>
          <p>特殊格審查：{{ analysis.pattern.specialReview.required ? '需要' : '目前未觸發' }}。</p><p v-if="analysis.pattern.specialReview.modelReferencePattern">App 參照：{{ analysis.pattern.specialReview.modelReferencePattern }}（{{ analysis.pattern.specialReview.modelReferenceCategory || '未分類' }}），只供比較。</p>
          <ul><li v-for="reason in analysis.pattern.specialReview.reasons" :key="reason">{{ reason }}</li><li v-for="version in analysis.pattern.specialReview.specializedUseVersions" :key="version.sourceDate">{{ version.sourceDate }}：{{ version.useDirection }}；{{ version.favorableDirection }}；{{ version.limitation }}</li><li>{{ analysis.pattern.specialReview.followChildPeerException }}</li></ul>
          <ul v-if="analysis.pattern.evidence.length"><li v-for="item in analysis.pattern.evidence" :key="item.id">{{ item.id }}：{{ item.description }}<small v-if="item.sourceDate">（{{ item.sourceDate }}）</small></li></ul><ul v-if="analysis.facts.roots.length"><li v-for="root in analysis.facts.roots" :key="root.evidenceId">根氣：{{ root.pillar }} {{ root.branch }} 藏{{ root.hiddenStem }}（{{ root.hiddenStemLayer }}，{{ root.rootKind }}）</li></ul>
        </details>
      </article>

      <article class="lesson-step">
        <h4>{{ analysis.strength.title }}</h4><p>{{ analysis.strength.conclusion }}</p>
        <div class="fact-grid"><span>分類：{{ statusLabel(analysis.strength.classification) }}</span><span>同氣得時：{{ yesNo(analysis.strength.sameSeasonElement) }}</span><span>印當令：{{ yesNo(analysis.strength.sealInSeason) }}</span><span>得勢暫估：{{ yesNo(analysis.strength.momentumProxy) }}</span><span>根氣：{{ analysis.strength.rootCount }} 處</span></div>
        <p v-if="analysis.strength.interactions.length" class="notice">互動存在，互動後有效力量未定。</p>
        <details><summary>強弱證據、互動與工程細節</summary>
          <ul><li v-for="interaction in analysis.strength.interactions" :key="interaction.id">{{ interaction.interactionType }}：{{ interaction.target }}（{{ interaction.outcome }}）— {{ statusLabel(interaction.assessment) }}</li></ul>
          <ul v-if="analysis.strength.evidence.length"><li v-for="item in analysis.strength.evidence" :key="item.id">{{ item.id }}：{{ item.description }}<small v-if="item.sourceDate">（{{ item.sourceDate }}）</small></li></ul><ul v-if="analysis.strength.limitations.length" class="limitations"><li v-for="item in analysis.strength.limitations" :key="item">限制：{{ item }}</li></ul>
        </details>
      </article>

      <article class="lesson-step">
        <h4>{{ analysis.preferences.title }}</h4><p>{{ analysis.preferences.conclusion }}</p>
        <div v-if="analysis.preferences.plans.length" class="plan-grid">
          <article v-for="plan in analysis.preferences.plans" :key="plan.ruleId" class="plan-card">
            <h5>{{ plan.title }} <small>（{{ statusLabel(plan.applicability) }}）</small></h5><p>{{ plan.condition }}</p>
            <template v-for="group in directionGroups(plan)" :key="group.label"><div v-if="group.items.length" class="direction-group"><strong>{{ group.label }}</strong><ul><li v-for="item in group.items" :key="item.family + item.element + item.role">{{ item.family }}（{{ item.element }}）：{{ item.reason }}</li></ul></div></template>
            <details><summary>方案證據與來源</summary><p>規則 ID：{{ plan.ruleId }}；來源：{{ plan.sources.join('、') || '未提供' }}</p><ul><li v-for="id in plan.evidenceIds" :key="id">{{ id }}：{{ analysis.facts.evidence.find(item => item.id === id)?.description || analysis.preferences.evidence.find(item => item.id === id)?.description || '證據詳情未提供' }}</li></ul><p v-if="plan.rejections.length">限制：{{ plan.rejections.join('；') }}</p></details>
          </article>
        </div><p v-else>目前沒有足夠依據形成方案：{{ analysis.preferences.noPlanReasons.join('；') || '依據不足' }}</p>
        <details><summary>通關、病藥、調候與來源</summary>
          <ul><li v-for="note in analysis.preferences.bridgePaths" :key="note.path.join('-')">通關候選 {{ note.path.join('→') }}：{{ statusLabel(note.status) }}；{{ note.explanation }}</li><li v-for="note in analysis.preferences.medicineNotes" :key="note.protectedPlanRuleId + note.protectedFamily">病藥（{{ note.protectedPlanRuleId }}）：{{ statusLabel(note.status) }}；{{ note.explanation }}</li><li v-for="note in analysis.preferences.seasonalCompatibility" :key="(note.planRuleId || 'none') + note.season">調候{{ seasonLabel(note.season) }}（方案 {{ note.planRuleId || '未指定' }}）：{{ statusLabel(note.status) }}；{{ note.explanation }}</li></ul>
          <ul v-if="analysis.preferences.evidence.length"><li v-for="item in analysis.preferences.evidence" :key="item.id">{{ item.id }}：{{ item.description }}<small v-if="item.sourceDate">（{{ item.sourceDate }}）</small></li></ul><ul v-if="analysis.preferences.limitations.length" class="limitations"><li v-for="item in analysis.preferences.limitations" :key="item">限制：{{ item }}</li></ul>
          <p>課程來源：{{ analysis.sources.map(source => `${source.date}（${source.ruleIds.join('、')}）`).join('；') }}</p>
        </details>
      </article>
      <details class="comparison"><summary>App 比較與整體限制</summary><p>格局：{{ analysis.comparison.pattern.appValue }}／{{ analysis.comparison.pattern.lessonValue }}（{{ statusLabel(analysis.comparison.pattern.agreementStatus) }}）— {{ analysis.comparison.pattern.note }}</p><p>強弱：{{ analysis.comparison.strength.appValue }}／{{ analysis.comparison.strength.lessonValue }}（{{ statusLabel(analysis.comparison.strength.agreementStatus) }}）— {{ analysis.comparison.strength.note }}</p><p>喜忌：{{ analysis.comparison.preferences.appValue }}／{{ analysis.comparison.preferences.lessonValue }}（{{ statusLabel(analysis.comparison.preferences.agreementStatus) }}）— {{ analysis.comparison.preferences.note }}</p><ul v-if="analysis.limitations.length" class="limitations"><li v-for="item in analysis.limitations" :key="item">限制：{{ item }}</li></ul></details>
    </template><p v-else class="empty-analysis">此結果尚無三步分析</p>
  </section>
</template>

<script setup lang="ts">
import type { LessonPreferencePlan, LessonThreeStepAnalysis } from '../types/bazi';
defineProps<{ analysis?: LessonThreeStepAnalysis | null }>();

const statusLabels: Record<string, string> = {
  candidate: '候選',
  fallback_candidate: '備選候選',
  selected: '選定候選',
  conditional: '有條件',
  insufficient: '依據不足',
  '依據不足': '依據不足',
  verified: '已核對（僅來源狀態）',
  implemented: '已實作',
  outside_bounded_scope: '本版範圍外',
  unassessed: '未評估',
  unresolved: '尚未解決',
  missing_intermediary: '缺少通關中介',
  conditional_compatible: '有條件相容',
  conflict: '有衝突',
  not_comparable: '不可直接比較',
  agree: '一致',
  partial: '部分一致',
  disagreement: '不一致',
  conditional_not_rescored: '有條件、未重算',
  strong: '強候選',
  '強候選': '強候選',
  '旺候選': '旺候選',
  '旺極候選': '旺極候選',
  '弱候選': '弱候選',
  '衰極候選': '衰極候選',
};
const seasonLabels: Record<string, string> = {
  winter: '冬季',
  summer: '夏季',
  outside_bounded_scope: '本版範圍外',
};
const statusLabel = (value: string) => statusLabels[value] || `未知狀態（${value || '未提供'}）`;
const seasonLabel = (value: string) => seasonLabels[value] || `未知季節（${value || '未提供'}）`;
const yesNo = (value: boolean) => value ? '是' : '否';
const directionGroups = (plan: LessonPreferencePlan) => [{ label: '取用方向', items: plan.useDirections }, { label: '喜神方向', items: plan.favorableDirections }, { label: '忌神方向', items: plan.unfavorableDirections }, { label: '有條件方向', items: plan.conditionalDirections }];
</script>

<style scoped>
.lesson-three-step { display: grid; gap: 12px; margin-top: 16px; line-height: 1.65; overflow-wrap: anywhere; }
.lesson-summary, .lesson-step, .comparison { padding: 14px 16px; border: 1px solid var(--el-border-color-light, #dcdfe6); border-radius: 8px; background: var(--el-bg-color, #fff); }
h3, h4, h5 { margin: 0 0 8px; } p { margin: 6px 0; } ul { margin: 6px 0; padding-left: 1.35rem; } li + li { margin-top: 4px; }
.status-line, .notice { color: var(--el-text-color-secondary, #606266); } .fact-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 6px; }
.plan-grid { display: grid; gap: 10px; } .plan-card { padding: 12px; border: 1px solid var(--el-border-color-lighter, #ebeef5); border-radius: 6px; background: var(--el-fill-color-lighter, #f5f7fa); } small { font-weight: normal; }
details { margin-top: 10px; } summary { cursor: pointer; font-weight: 600; } .empty-analysis { margin: 0; padding: 16px; color: var(--el-text-color-secondary, #606266); }
@media (max-width: 720px) { .lesson-three-step { margin-top: 12px; } }
</style>

