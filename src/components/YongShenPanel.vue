<template>
  <el-card shadow="hover">
    <template #header>
      <div class="section-header">
        <h2>用神喜忌判斷（課堂三步法）</h2>
      </div>
    </template>

    <template v-if="guide">
      <div class="headline" :class="`status-${guide.status}`">{{ guide.headline }}</div>

      <div class="god-grid">
        <div v-for="row in guide.gods" :key="row.element" class="god-cell" :class="`role-${row.role}`">
          <div class="god-role">{{ row.roleLabel }}</div>
          <div class="god-element" :class="`wx-${WX[row.element]}`">{{ row.element }}</div>
          <div class="god-family">{{ row.family }}</div>
          <div class="god-reason">{{ row.reason }}</div>
        </div>
      </div>

      <div class="subsection-title">判斷步驟</div>
      <ol class="step-list">
        <li v-for="step in guide.steps" :key="step.label">
          <strong>{{ step.label }}</strong>：{{ step.text }}
        </li>
      </ol>

      <template v-if="guide.useElements.length">
        <div class="subsection-title">用神在原局有沒有力</div>
        <p class="verdict-line">{{ guide.strengthVerdict }}</p>
        <ul v-if="guide.positions.length" class="plain-list">
          <li v-for="item in guide.positions" :key="item.pillar + item.place + item.char">
            {{ item.pillar }}・{{ item.place }}：{{ item.char }}
          </li>
        </ul>

        <template v-if="guide.neighbors.length">
          <div class="subsection-title">相連檢查（旁邊誰生它、誰剋它）</div>
          <ul class="plain-list">
            <li v-for="item in guide.neighbors" :key="item.at">
              <strong>{{ item.at }}</strong>：
              <span v-if="item.supporters.length" class="good">得生 {{ item.supporters.join("、") }}</span>
              <span v-if="item.supporters.length && item.attackers.length">；</span>
              <span v-if="item.attackers.length" class="bad">受剋 {{ item.attackers.join("、") }}</span>
              <span v-if="!item.supporters.length && !item.attackers.length" class="muted">旁邊沒有生它或剋它的字</span>
            </li>
          </ul>
        </template>
      </template>

      <template v-if="guide.interactions.length">
        <div class="subsection-title">原局合沖碰到喜用神</div>
        <ul class="plain-list">
          <li v-for="item in guide.interactions" :key="item.label">
            <span class="mark" :class="`verdict-${item.verdict}`">{{ MARK[item.verdict] || "・" }}</span>
            <strong>{{ item.label }}</strong>
            <div class="reason">{{ item.reason }}</div>
          </li>
        </ul>
      </template>

      <div v-if="guide.specialWarnings.length" class="special-note">
        <div class="subsection-title">特別格局提醒</div>
        <ul class="plain-list">
          <li v-for="item in guide.specialWarnings" :key="item">{{ item }}</li>
        </ul>
      </div>

      <details class="glossary">
        <summary>名詞與規則</summary>
        <ul class="plain-list">
          <li><strong>用神</strong>：最能把命局拉回平衡的五行（身強抑、身弱扶，看局中什麼太多）。</li>
          <li><strong>喜神</strong>：生用神、或讓用神有去處的五行。</li>
          <li><strong>忌神</strong>：剋用神的五行；<strong>仇神</strong>：生忌神的五行。老師上課把兩者合稱「忌」。</li>
          <li><strong>閒神</strong>：方案沒列喜忌的五行，好壞看它和誰相連（09-02「土要帶金」、10-21 金與水相連就不忌）。</li>
          <li>合沖四句（10-14）：喜用合去損吉、忌神合來增凶、喜用合來增吉、忌神合去減凶；沖去忌神為好，沖去喜神為凶（09-30）。</li>
          <li>病藥、調候、通關不能違背扶抑；特別格局要順勢，不照扶抑。</li>
        </ul>
      </details>
    </template>
    <el-empty v-else description="此結果沒有課堂三步分析，無法判斷用神。" :image-size="60" />
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { yongShenGuide } from "../utils/yongShen";

const props = defineProps<{ result: BaziResponse }>();

const WX: Record<string, string> = { 木: "wood", 火: "fire", 土: "earth", 金: "metal", 水: "water" };
const MARK: Record<string, string> = { good: "▲有利", bad: "▼不利", mixed: "◆依方案而定", neutral: "・不判吉凶" };

const guide = computed(() => yongShenGuide(props.result.quantModel?.lessonContent?.threeStepAnalysis));
</script>

<style scoped>
.headline {
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--el-color-primary-light-9);
  color: var(--bazi-primary);
  font-weight: 600;
  font-size: calc(16px * var(--app-font-scale));
}
.headline.status-conflict,
.headline.status-pending,
.headline.status-insufficient {
  background: var(--el-color-warning-light-9);
  color: var(--el-color-warning-dark-2);
}
.god-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(118px, 1fr));
  gap: 8px;
  margin-top: 12px;
}
.god-cell {
  padding: 8px 10px;
  border: 1px solid var(--el-border-color-light);
  border-radius: 6px;
}
.god-cell.role-use { border-color: var(--el-color-success); background: var(--el-color-success-light-9); }
.god-cell.role-favorable { border-color: var(--el-color-success-light-5); }
.god-cell.role-unfavorable { border-color: var(--el-color-danger); background: var(--el-color-danger-light-9); }
.god-cell.role-enemy { border-color: var(--el-color-danger-light-5); }
.god-cell.role-mixed { border-color: var(--el-color-warning); }
.god-role { font-size: calc(12px * var(--app-font-scale)); color: var(--bazi-text-muted); }
.god-element { font-size: calc(22px * var(--app-font-scale)); font-weight: 700; }
.god-family { font-size: calc(13px * var(--app-font-scale)); }
.god-reason { margin-top: 2px; font-size: calc(12px * var(--app-font-scale)); color: var(--bazi-text-muted); }
.wx-wood { color: var(--wx-wood); }
.wx-fire { color: var(--wx-fire); }
.wx-earth { color: var(--wx-earth); }
.wx-metal { color: var(--wx-metal); }
.wx-water { color: var(--wx-water); }
.subsection-title { margin: 14px 0 6px; font-weight: 600; }
.step-list, .plain-list { margin: 0; padding-left: 1.3rem; line-height: 1.65; }
.plain-list { list-style: disc; }
.step-list li + li, .plain-list li + li { margin-top: 4px; }
.verdict-line { margin: 0 0 4px; font-weight: 600; }
.good { color: var(--el-color-success); }
.bad { color: var(--el-color-danger); }
.muted, .reason { color: var(--bazi-text-muted); font-size: calc(13px * var(--app-font-scale)); }
.mark { margin-right: 6px; font-weight: 600; font-size: calc(13px * var(--app-font-scale)); }
.verdict-good { color: var(--el-color-success); }
.verdict-bad { color: var(--el-color-danger); }
.verdict-mixed, .verdict-neutral { color: var(--bazi-text-muted); }
.special-note { margin-top: 4px; }
.glossary { margin-top: 14px; }
.glossary summary { cursor: pointer; font-weight: 600; }
</style>
