<template>
  <el-card shadow="hover" :class="{ 'pillar-panel-compact': compact }">
    <template #header>
      <div class="section-header pillar-section-header">
        <h2>{{ panelTitle }}</h2>
        <div v-if="showPillars && shift" class="view-switch" role="group" aria-label="四柱顯示方式">
          <button type="button" :class="{ 'is-on': !strengthOn && !flowOn }" :aria-pressed="!strengthOn && !flowOn" @click="setView('relation')">關係</button>
          <button type="button" :class="{ 'is-on': strengthOn }" :aria-pressed="strengthOn" @click="setView('strength')">強弱</button>
          <button type="button" :class="{ 'is-on': flowOn }" :aria-pressed="flowOn" @click="setView('flow')">流通</button>
        </div>
      </div>
    </template>

    <RelationBrackets v-if="showPillars" :active="bracketActive" :links="flowOn ? [] : bracketLinks" @update:active="onBracketActive">
    <div
      :class="['pillar-card-grid', { 'is-dense': pillarItems.length > 4, 'is-strength': strengthOn }]"
      :data-pillar-grid="panelUid"
      :style="{ gridTemplateColumns: `repeat(${pillarItems.length}, minmax(0, 1fr))` }"
    >
      <svg v-if="strengthOn && rootLines.length" class="sv-root-lines" aria-hidden="true">
        <g v-for="line in rootLines" :key="line.key" :class="['sv-root-line', `lv-${line.level}`, { shaken: line.shaken }]" :style="{ '--root-c': line.color }">
          <path :d="line.d" />
          <circle :cx="line.ex" :cy="line.ey" r="3.5" />
          <text v-if="line.shaken" :x="line.ex + 7" :y="line.ey - 4" class="sv-root-x">✕</text>
        </g>
      </svg>
      <div v-if="luckPickerFor" class="luck-picker" role="listbox" :aria-label="`換一個${pickerKind}`" @click.stop @pointerdown.stop>
        <div class="luck-picker-head">
          <span>換一個{{ pickerKind }}</span>
          <span class="luck-picker-tip">點一格就換上去</span>
          <button type="button" class="luck-picker-close" aria-label="關閉" @click="luckPickerFor = null">✕</button>
        </div>
        <div class="luck-picker-track">
          <button
            v-for="cell in pickerCells"
            :key="cell.key"
            type="button"
            role="option"
            :aria-selected="cell.key === luckPickerFor"
            :title="cell.title"
            :class="['luck-picker-cell', { 'is-on': cell.key === luckPickerFor, 'is-current': cell.isCurrent }]"
            @click="cell.pick()"
          >
            <span v-if="cell.isCurrent" class="luck-picker-now">{{ pickerKind === '流年' ? '今年' : '目前' }}</span>
            <span class="luck-picker-god">{{ cell.stemGod }}</span>
            <span :class="['luck-picker-char', 'wuxing-char', `wuxing-${stemElement(cell.ganZhi[0])}`]">{{ cell.ganZhi[0] }}</span>
            <span :class="['luck-picker-char', 'wuxing-char', `wuxing-${branchElement(cell.ganZhi[1])}`]">{{ cell.ganZhi[1] }}</span>
            <span class="luck-picker-god">{{ cell.branchGod }}</span>
            <span class="luck-picker-age">{{ cell.sub }}</span>
          </button>
        </div>
      </div>
      <div v-for="(item, index) in pillarItems" :key="item.key" class="pillar-cell">
        <div
          :data-relation-col="index"
          :class="[
            'pillar-box',
            { 'is-preview': item.isPreview, 'is-current-year-preview': item.isCurrentYear, 'is-compact': compact, 'is-luck-switchable': isLuckItem(item) }
          ]"
          :title="isLuckItem(item) ? `點兩下可以換一個${item.label}` : undefined"
          @click="onPillarTap(item, $event)"
        >
          <template v-if="compact">
            <div class="compact-pillar-header">
              <p class="pillar-label">{{ item.label }}<span v-if="isLuckItem(item)" class="luck-switch-mark" aria-hidden="true">⇅</span></p>
              <span v-if="isResearch" class="compact-score-chip">{{ item.topScoreValue }}</span>
            </div>
            <div :class="['compact-ten-god-chip', tenGodClass(item.stemTenGod)]" data-align="chip">{{ tenGodText(item.stemTenGod) }}</div>
            <div class="compact-pillar-ganzhi">
              <div class="compact-pillar-char-block">
                <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.stemElement}`, { 'is-related': isRelated(index, 'stem') }, svClass(index, 'stem')]" :data-sv="`${index}-stem`">{{ item.value.stem }}</div>
                <StrengthCharTags v-if="strengthOn" :info="svInfo(index, 'stem')" />
                <button
                  v-if="strengthOn && rootChip(index)"
                  type="button"
                  :class="['sv-root-chip', rootChip(index)!.cls]"
                  :style="{ '--root-c': rootColor(index) }"
                  :data-sv-root="index"
                  :title="rootChip(index)!.title"
                  @click="toggleRootFocus(index)"
                >{{ rootChip(index)!.text }}</button>
              </div>
              <div class="compact-pillar-char-block" data-align="branch">
                <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.branchElement}`, { 'is-related': isRelated(index, 'branch') }, svClass(index, 'branch')]" :data-sv="`${index}-branch`">{{ item.value.branch }}</div>
                <div v-if="luTagsOf(index).length" class="lu-tags">
                  <span v-for="tag in luTagsOf(index)" :key="tag.text" :class="['lu-tag', { 'is-hidden-combo': tag.kind === 'hidden' }]" :title="tag.title">{{ tag.text }}</span>
                </div>
                <StrengthCharTags v-if="strengthOn" :info="svInfo(index, 'branch')" />
              </div>
            </div>
            <div class="compact-pillar-line" data-align="hidden">
              <HiddenStemColumns :pairs="hiddenStemTenGodPairs(item)" :col="index" :roots="hiddenRootsOf(index)" compact />
            </div>
            <div v-if="isResearch" class="compact-pillar-line">
              <span class="compact-line-label">神煞</span>
              <span class="compact-line-value">{{ item.shenSha.length ? item.shenSha.join('、') : '—' }}</span>
            </div>
            <div v-if="isResearch" class="compact-score-row compact-score-row-bottom">
              <span class="compact-score-label">{{ item.bottomScoreLabel }}</span>
              <span class="compact-score-value">{{ item.bottomScoreValue }}</span>
            </div>
            <div v-if="isResearch" class="compact-pillar-foot">
              <span>{{ item.value.naYin }}</span>
              <span>{{ item.value.xunKong }}</span>
            </div>
          </template>
          <template v-else>
            <div v-if="isResearch" class="score-band score-band-top">
              <span class="score-band-label">{{ item.topScoreLabel }}</span>
              <span class="score-band-value">{{ item.topScoreValue }}</span>
            </div>
            <p class="pillar-label">{{ item.label }}<span v-if="isLuckItem(item)" class="luck-switch-mark" aria-hidden="true">⇅</span></p>
            <div class="meta-row" data-align="chip">
              <div class="meta-value">{{ tenGodText(item.stemTenGod) }}</div>
            </div>
            <div class="pillar-main">
              <div class="pillar-main-row">
                <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.stemElement}`, { 'is-related': isRelated(index, 'stem') }, svClass(index, 'stem')]" :data-sv="`${index}-stem`">{{ item.value.stem }}</div>
                <StrengthCharTags v-if="strengthOn" :info="svInfo(index, 'stem')" />
                <button
                  v-if="strengthOn && rootChip(index)"
                  type="button"
                  :class="['sv-root-chip', rootChip(index)!.cls]"
                  :style="{ '--root-c': rootColor(index) }"
                  :data-sv-root="index"
                  :title="rootChip(index)!.title"
                  @click="toggleRootFocus(index)"
                >{{ rootChip(index)!.text }}</button>
              </div>
              <div class="pillar-main-row" data-align="branch">
                <!-- 祿浮在地支右邊，不佔位置，地支保持置中 -->
                <span class="branch-anchor">
                  <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.branchElement}`, { 'is-related': isRelated(index, 'branch') }, svClass(index, 'branch')]" :data-sv="`${index}-branch`">{{ item.value.branch }}</div>
                  <span v-if="luTagsOf(index).length" class="lu-float">
                    <span v-for="tag in luTagsOf(index)" :key="tag.text" :class="['lu-tag', { 'is-hidden-combo': tag.kind === 'hidden' }]" :title="tag.title">{{ tag.text }}</span>
                  </span>
                </span>
                <StrengthCharTags v-if="strengthOn" :info="svInfo(index, 'branch')" />
              </div>
            </div>
            <div class="meta-row" data-align="hidden">
              <HiddenStemColumns :pairs="hiddenStemTenGodPairs(item)" :col="index" :roots="hiddenRootsOf(index)" />
            </div>
            <div v-if="isResearch" class="meta-row">
              <div class="meta-label">神煞</div>
              <div class="meta-value meta-value-shen-sha">{{ item.shenSha.length ? item.shenSha.join('、') : '—' }}</div>
            </div>
            <el-descriptions v-if="isResearch" :column="1" size="small">
              <el-descriptions-item label="納音">{{ item.value.naYin }}</el-descriptions-item>
              <el-descriptions-item label="五行">{{ item.value.wuXing }}</el-descriptions-item>
              <el-descriptions-item label="地勢">{{ item.value.diShi }}</el-descriptions-item>
              <el-descriptions-item label="旬 / 空亡">{{ item.value.xun }} / {{ item.value.xunKong }}</el-descriptions-item>
            </el-descriptions>
            <div v-if="isResearch" class="score-band score-band-bottom">
              <span class="score-band-label">{{ item.bottomScoreLabel }}</span>
              <span class="score-band-value">{{ item.bottomScoreValue }}</span>
            </div>
          </template>
        </div>
      </div>
    </div>
    </RelationBrackets>
    <p v-if="showPillars && !strengthOn && !flowOn && activeLink" :class="['relation-explain', `is-${activeLink.favor}`]">
      <strong>{{ activeLink.label }}</strong>
      <span class="relation-explain-favor">{{ FAVOR_TEXT[activeLink.favor ?? 'neutral'] }}</span>
      <span v-if="activeLink.reason">{{ activeLink.reason }}</span>
    </p>
    <p v-else-if="showPillars && !strengthOn && !flowOn && bracketLinks.length" class="relation-hint">天干的關係畫在上方，地支的畫在下方。箭頭 ▲ 對日主有利、▼ 不利（依課堂喜忌）；點名稱看原因。</p>
    <p v-if="showPillars && strengthOn && rootNote" class="sv-root-note" :style="{ '--root-c': rootFocusCol == null ? undefined : rootColor(rootFocusCol) }" v-html="rootNote" />
    <StrengthBar
      v-if="showPillars && strengthOn && shift"
      v-model:applied="applied"
      v-model:focus="focus"
      :shift="shift"
      :compact="compact"
    />
    <div v-if="showPillars && flowOn" class="pillar-flow">
      <ElementFlowRing :result="result" :compact="compact" :shift="shift" />
    </div>
    <LuckStrip
      v-if="showPillars"
      :result="result"
      :preview-pillars="previewPillars"
      @preview-change="emit('preview-change', $event)"
    />

    <div v-if="showRelationMap" class="relation-map-section">
      <div class="relation-map-header">
        <h3>柱位關係圖</h3>
      </div>
      <RelationBrackets v-model:active="activeRelation" class="relation-map" :links="bracketLinks">
        <div class="relation-node-grid" :style="{ gridTemplateColumns: `repeat(${diagramNodes.length}, minmax(0, 1fr))` }">
          <div
            v-for="(node, index) in diagramNodes"
            :key="node.key"
            :data-relation-col="index"
            :class="['relation-node', { 'is-preview': node.isPreview, 'is-current-year': node.isCurrentYear }]"
          >
            <div class="relation-node-label">{{ node.label }}</div>
            <div :class="['relation-node-ten-god', tenGodClass(node.stemTenGod)]">{{ tenGodText(node.stemTenGod) }}</div>
            <div class="relation-node-pillar">
              <div :class="['relation-node-stem', 'wuxing-char', `wuxing-${stemElement(node.stem)}`, { 'is-related': isRelated(index, 'stem') }]">{{ node.stem }}</div>
              <div :class="['relation-node-branch', 'wuxing-char', `wuxing-${branchElement(node.branch)}`, { 'is-related': isRelated(index, 'branch') }]">{{ node.branch }}</div>
              <div v-if="luTagsOf(index).length" class="lu-tags">
                <span v-for="tag in luTagsOf(index)" :key="tag.text" :class="['lu-tag', { 'is-hidden-combo': tag.kind === 'hidden' }]" :title="tag.title">{{ tag.text }}</span>
              </div>
            </div>
            <div :class="['relation-node-hidden-stems', { 'is-empty': !node.hiddenStemPairs.length }]">
              <span
                v-for="(pair, pairIndex) in node.hiddenStemPairs"
                :key="`${node.key}-${pair.stem}-${pair.tenGod}-${pairIndex}`"
                class="relation-node-hidden-item"
              >
                <span :class="['relation-node-hidden-stem', 'wuxing-char', `wuxing-${stemElement(pair.stem)}`]">{{ pair.stem }}</span>
                <span :class="['relation-node-hidden-ten-god', tenGodClass(pair.tenGod)]">{{ pair.tenGod }}</span>
              </span>
            </div>
          </div>
        </div>
      </RelationBrackets>
      <p v-if="activeLink" :class="['relation-explain', `is-${activeLink.favor}`]">
        <strong>{{ activeLink.label }}</strong>
        <span class="relation-explain-favor">{{ FAVOR_TEXT[activeLink.favor ?? 'neutral'] }}</span>
        <span v-if="activeLink.reason">{{ activeLink.reason }}</span>
      </p>
      <p v-else-if="bracketLinks.length" class="relation-hint">箭頭 ▲ 對日主有利、▼ 不利（依課堂喜忌）；點名稱看原因。</p>
    </div>

    <div v-if="showInteractionSection" class="interaction-section">
      <div class="interaction-header">
        <h3>目前盤面合沖刑破</h3>
        <span class="interaction-count">{{ pillarItems.length }}柱</span>
      </div>
      <p class="interaction-hint">
        原局四柱優先採用量化模型互動摘要；大運、流年預覽段落則依目前盤面即時補出新增的合、沖、刑、破。
      </p>
      <template v-if="interactionRows.length">
        <div v-if="groupedInteractionSections.annual" class="interaction-group">
          <div class="interaction-group-title">流年加入後生成的</div>
          <el-table :data="groupedInteractionSections.annual.rows" size="small" stripe>
            <el-table-column prop="scope" label="部位" width="82" />
            <el-table-column prop="type" label="類型" width="92" />
            <el-table-column prop="target" label="組合" width="110" />
            <el-table-column prop="pillars" label="對應柱位" min-width="200" />
            <el-table-column prop="detail" label="說明" min-width="220" />
          </el-table>
        </div>

        <div
          v-if="groupedInteractionSections.luck"
          :class="['interaction-group', { 'with-divider': !!groupedInteractionSections.annual }]"
        >
          <div class="interaction-group-title">大運加入後生成的</div>
          <el-table :data="groupedInteractionSections.luck.rows" size="small" stripe>
            <el-table-column prop="scope" label="部位" width="82" />
            <el-table-column prop="type" label="類型" width="92" />
            <el-table-column prop="target" label="組合" width="110" />
            <el-table-column prop="pillars" label="對應柱位" min-width="200" />
            <el-table-column prop="detail" label="說明" min-width="220" />
          </el-table>
        </div>

        <div
          class="interaction-group"
          :class="{ 'with-divider': !!groupedInteractionSections.annual || !!groupedInteractionSections.luck }"
        >
          <div class="interaction-group-title">原局四柱本來有的</div>
          <el-table :data="groupedInteractionSections.natal.rows" size="small" stripe>
            <el-table-column prop="scope" label="部位" width="82" />
            <el-table-column prop="type" label="類型" width="92" />
            <el-table-column prop="target" label="組合" width="110" />
            <el-table-column prop="pillars" label="對應柱位" min-width="200" />
            <el-table-column prop="detail" label="說明" min-width="220" />
          </el-table>
        </div>
      </template>
      <el-empty v-else description="目前盤面未檢出明顯的合、沖、刑、破組合。" :image-size="72" />
    </div>
  </el-card>
</template>

<script setup lang="ts">
import RelationBrackets, { type BracketLink } from './RelationBrackets.vue'
import LuckStrip from './LuckStrip.vue'
import StrengthBar from './StrengthBar.vue'
import ElementFlowRing from './ElementFlowRing.vue'
import HiddenStemColumns, { type HiddenRootMark } from './HiddenStemColumns.vue'
import { annualFromKey, annualKey, annualPreviewPillar, annualRows, luckPreviewPillar, luckRows, type AnnualRowView, type LuckRowView } from '../utils/luckPreview'
import { baziYearAt } from '../utils/baziNow'
import { useLuckSelection } from '../composables/useLuckSelection'
import StrengthCharTags, { type StrengthCharInfo, type StrengthCharTag } from './StrengthCharTags.vue'
import { cellStatesAt, helpsParty, strengthShift, type ShiftDir } from '../utils/strengthShift'
import { stemRoots, type RootLevel, type StemRoot } from '../utils/stemRoots'
import { lessonPreferences, type Favor } from '../utils/lessonPreference'
import { findNatalVerdict, natalRelationVerdicts, previewRelationVerdict, type RelationVerdict } from '../utils/relationFavor'
import { useViewMode } from "../composables/useViewMode";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { BaziResponse, LuckPreviewPillar, Pillar, QuantModelInteraction } from '../types/bazi'
import { BRANCH_WUXING, STEM_WUXING, type WuXing } from '../constants/wuxing'

const { isResearch } = useViewMode();
const props = withDefaults(
  defineProps<{
    result: BaziResponse
    previewPillars?: LuckPreviewPillar[]
    compact?: boolean
    showPillars?: boolean
    showRelationMap?: boolean
    showInteractionSection?: boolean
  }>(),
  {
    compact: false,
    showPillars: true,
    showRelationMap: true,
    showInteractionSection: true
  }
)

const emit = defineEmits<{
  'preview-change': [pillars: LuckPreviewPillar[] | null]
}>()

type DisplayPillarItem = {
  key: string
  label: string
  value: Pillar
  stemTenGod: string
  hiddenStems: string[]
  branchTenGods: string[]
  shenSha: string[]
  stemElement: WuXing
  branchElement: WuXing
  topScoreLabel: string
  topScoreValue: string
  bottomScoreLabel: string
  bottomScoreValue: string
  isPreview: boolean
  isCurrentYear: boolean
}
type InteractionRow = {
  scope: '天干' | '地支' | '祿'
  type: string
  target: string
  pillars: string
  detail: string
  participantKeys: string[]
}
type DiagramNode = {
  key: string
  label: string
  stem: string
  branch: string
  stemTenGod: string
  hiddenStemPairs: Array<{
    stem: string
    tenGod: string
  }>
  isPreview: boolean
  isCurrentYear: boolean
}
type InteractionSection = {
  title: string
  rows: InteractionRow[]
}

const STEM_COMBINATION_ELEMENTS: Record<string, string> = {
  '甲己': '土',
  '乙庚': '金',
  '丙辛': '水',
  '丁壬': '木',
  '戊癸': '火'
}

const STEM_CLASH_PAIRS = new Set(['甲庚', '乙辛', '丙壬', '丁癸'])

const BRANCH_SIX_COMBINATION_ELEMENTS: Record<string, string> = {
  '子丑': '土',
  '寅亥': '木',
  '卯戌': '火',
  '辰酉': '金',
  '巳申': '水',
  '午未': '土'
}

const BRANCH_CLASH_PAIRS = new Set(['子午', '丑未', '寅申', '卯酉', '辰戌', '巳亥'])

const BRANCH_HALF_COMBINATIONS: Record<string, string> = {
  '寅午': '火',
  '午戌': '火',
  '亥卯': '木',
  '卯未': '木',
  '申子': '水',
  '子辰': '水',
  '巳酉': '金',
  '酉丑': '金'
}

const BRANCH_THREE_COMBINATIONS: Array<{ key: string; element: string }> = [
  { key: '寅午戌', element: '火' },
  { key: '亥卯未', element: '木' },
  { key: '申子辰', element: '水' },
  { key: '巳酉丑', element: '金' }
]

const BRANCH_THREE_MEETINGS: Array<{ key: string; element: string }> = [
  { key: '寅卯辰', element: '木' },
  { key: '巳午未', element: '火' },
  { key: '申酉戌', element: '金' },
  { key: '亥子丑', element: '水' }
]

const BRANCH_PUNISH_RULES: Array<{ pair: string; detail: string }> = [
  { pair: '子卯', detail: '子卯相刑(無禮之刑)' },
  { pair: '寅巳', detail: '寅巳申三刑(恃勢之刑)' },
  { pair: '巳申', detail: '寅巳申三刑(恃勢之刑)' },
  { pair: '寅申', detail: '寅巳申三刑(恃勢之刑)' },
  { pair: '丑戌', detail: '丑戌未三刑(無恩之刑)' },
  { pair: '戌未', detail: '丑戌未三刑(無恩之刑)' },
  { pair: '丑未', detail: '丑戌未三刑(無恩之刑)' }
]

const BRANCH_BREAK_RULES: Array<{ pair: string; detail: string }> = [
  { pair: '子酉', detail: '地支相破' },
  { pair: '卯午', detail: '地支相破' },
  { pair: '辰丑', detail: '地支相破' },
  { pair: '未戌', detail: '地支相破' },
  { pair: '寅亥', detail: '地支相破' },
  { pair: '巳申', detail: '地支相破' }
]

const STEM_LU_BRANCH: Record<string, string> = {
  甲: '寅',
  乙: '卯',
  丙: '巳',
  丁: '午',
  戊: '巳',
  己: '午',
  庚: '申',
  辛: '酉',
  壬: '亥',
  癸: '子'
}

const RELATION_TYPE_STYLES: Record<string, {
  color: string
  strokeWidth: number
  opacity: number
  dasharray?: string
}> = {
  合: { color: '#2f855a', strokeWidth: 0.9, opacity: 0.95 },
  祿: { color: '#b7791f', strokeWidth: 0.88, opacity: 0.94 },
  半合: { color: '#4cae77', strokeWidth: 0.78, opacity: 0.88, dasharray: '3 2.2' },
  三合: { color: '#2b6cb0', strokeWidth: 0.94, opacity: 0.92 },
  三會: { color: '#7c5cc4', strokeWidth: 0.84, opacity: 0.9, dasharray: '1.8 2.2' },
  沖: { color: '#c53030', strokeWidth: 0.92, opacity: 0.92, dasharray: '5.5 3' },
  刑: { color: '#dd6b20', strokeWidth: 0.82, opacity: 0.9, dasharray: '2.2 1.8' },
  破: { color: '#718096', strokeWidth: 0.76, opacity: 0.84, dasharray: '1.2 2.4' },
  暗合: { color: '#2c9a9a', strokeWidth: 0.76, opacity: 0.84, dasharray: '1 2.4' }
}

// 地支暗合：兩支本氣天干相合（寅中甲合丑中己、午中丁合亥中壬、卯中乙合申中庚）。
// 書上（蘇民峰、陸致極）常用的三組；老師不用暗合造格，所以只標出來、不判吉凶。
const BRANCH_HIDDEN_COMBINATIONS: Array<{ pair: string; element: string; detail: string }> = [
  { pair: '寅丑', element: '土', detail: '寅中甲木合丑中己土' },
  { pair: '午亥', element: '木', detail: '午中丁火合亥中壬水' },
  { pair: '卯申', element: '金', detail: '卯中乙木合申中庚金' }
]
// 同柱干支暗合：天干和自己坐下地支的藏干相合（講義第 25 章，十二組；六組較純粹）。
const PILLAR_HIDDEN_COMBINATIONS: Record<string, { detail: string; pure: boolean }> = {
  甲午: { detail: '甲木合午中己土', pure: true },
  乙巳: { detail: '乙木合巳中庚金', pure: false },
  丙戌: { detail: '丙火合戌中辛金', pure: false },
  丁亥: { detail: '丁火合亥中壬水', pure: true },
  戊子: { detail: '戊土合子中癸水', pure: true },
  戊辰: { detail: '戊土合辰中癸水', pure: false },
  己亥: { detail: '己土合亥中甲木', pure: true },
  庚辰: { detail: '庚金合辰中乙木', pure: false },
  辛巳: { detail: '辛金合巳中丙火', pure: true },
  壬午: { detail: '壬水合午中丁火', pure: true },
  壬戌: { detail: '壬水合戌中丁火', pure: false },
  癸巳: { detail: '癸水合巳中戊土', pure: false }
}

function stemElement(stem: string): WuXing {
  return STEM_WUXING[stem] ?? 'earth'
}

function branchElement(branch: string): WuXing {
  return BRANCH_WUXING[branch] ?? 'earth'
}

// 日柱天干的「日元」顯示成元男／元女（資料仍是「日元」，判斷邏輯不受影響）
const dayMasterLabel = computed(() => (props.result.gender === 'FEMALE' ? '元女' : props.result.gender === 'MALE' ? '元男' : '日元'))
const tenGodText = (tenGod?: string) => (tenGod === '日元' ? dayMasterLabel.value : tenGod)

function tenGodClass(tenGod?: string) {
  if (!tenGod || tenGod === '-') {
    return 'ten-god-neutral'
  }
  if (tenGod === '日元') {
    return 'ten-god-daymaster'
  }
  if (['比肩', '劫財'].includes(tenGod)) {
    return 'ten-god-peer'
  }
  if (['食神', '傷官'].includes(tenGod)) {
    return 'ten-god-output'
  }
  if (['偏財', '正財'].includes(tenGod)) {
    return 'ten-god-wealth'
  }
  if (['七殺', '正官'].includes(tenGod)) {
    return 'ten-god-officer'
  }
  if (['偏印', '正印'].includes(tenGod)) {
    return 'ten-god-resource'
  }
  return 'ten-god-neutral'
}

function formatScore(value?: number): string {
  if (value == null || Number.isNaN(value)) {
    return '-'
  }
  return value > 0 ? `+${value.toFixed(1)}` : value.toFixed(1)
}

function normalizeStrengthLabel(label?: string): string {
  if (!label) {
    return '未判定'
  }
  return label
}

function findStemScore(pillar: string): number | undefined {
  return props.result.quantModel?.stemScores.find((item) => item.pillar === pillar)?.finalScore
}

function findBranchScore(pillar: string): number | undefined {
  return props.result.quantModel?.branchScores.find((item) => item.pillar === pillar)?.finalScore
}

function centerMainQi<T>(items: T[]): T[] {
  if (items.length < 3) {
    return items
  }
  return [items[1], items[0], ...items.slice(2)]
}

function hiddenStemTenGodPairs(item: DisplayPillarItem) {
  return centerMainQi(
    item.hiddenStems.map((stem, index) => ({
      stem,
      tenGod: item.branchTenGods[index] || '-'
    }))
  )
}

function createPlaceholderPillar(ganZhi: string, xun?: string, xunKong?: string): Pillar {
  return {
    ganZhi,
    stem: ganZhi.charAt(0),
    branch: ganZhi.charAt(1),
    naYin: '-',
    wuXing: '-',
    diShi: '-',
    xun: xun || '-',
    xunKong: xunKong || '-'
  }
}

function unorderedPairKey(a: string, b: string) {
  return [a, b].sort().join('')
}

function unorderedTripleKey(values: string[]) {
  return [...values].sort().join('')
}

function findPairEntry(record: Record<string, string>, a: string, b: string) {
  const target = unorderedPairKey(a, b)
  return Object.entries(record).find(([key]) => unorderedPairKey(key.charAt(0), key.charAt(1)) === target)
}

function hasPairMatch(values: Set<string>, a: string, b: string) {
  const target = unorderedPairKey(a, b)
  return [...values].some((key) => unorderedPairKey(key.charAt(0), key.charAt(1)) === target)
}

function findPairRule(rules: Array<{ pair: string; detail: string }>, a: string, b: string) {
  const target = unorderedPairKey(a, b)
  return rules.find((rule) => unorderedPairKey(rule.pair.charAt(0), rule.pair.charAt(1)) === target)
}

function findTripleRule<T extends { key: string }>(rules: T[], values: string[]) {
  const target = unorderedTripleKey(values)
  return rules.find((rule) => unorderedTripleKey([...rule.key]) === target)
}

function isPairCoveredByMatchedThreeCombination(branchA: string, branchB: string, matchedKeys: Set<string>) {
  return [...matchedKeys].some((key) => key.includes(branchA) && key.includes(branchB))
}

function toPillarRef(item: DisplayPillarItem, part: 'stem' | 'branch') {
  const value = part === 'stem' ? item.value.stem : item.value.branch
  return `${item.label}(${value})`
}

function buildPairInteractionRow(
  scope: '天干' | '地支',
  type: string,
  target: string,
  left: DisplayPillarItem,
  right: DisplayPillarItem,
  detail: string
): InteractionRow {
  const part = scope === '天干' ? 'stem' : 'branch'
  return {
    scope,
    type,
    target,
    pillars: [toPillarRef(left, part), toPillarRef(right, part)].join('、'),
    detail,
    participantKeys: [left.key, right.key]
  }
}

function buildLuInteractionRow(
  stemItem: DisplayPillarItem,
  branchItem: DisplayPillarItem,
  luBranch: string
): InteractionRow {
  return {
    scope: '祿',
    type: '祿',
    target: `${stemItem.value.stem}${luBranch}`,
    pillars: `${stemItem.label}(${stemItem.value.stem})、${branchItem.label}(${branchItem.value.branch})`,
    detail: `天干${stemItem.value.stem}祿在${luBranch}。遇三會不化、雙三合不化等特例，再補額外增值。`,
    participantKeys: [stemItem.key, branchItem.key]
  }
}

function resolveLuBranches(stemItem: DisplayPillarItem): string[] {
  const stem = stemItem.value.stem
  if (stem === '戊' || stem === '己') {
    return stemItem.key === 'day' ? ['巳', '午'] : []
  }
  const luBranch = STEM_LU_BRANCH[stem]
  return luBranch ? [luBranch] : []
}

function buildTripleInteractionRow(
  type: string,
  target: string,
  items: DisplayPillarItem[],
  detail: string
): InteractionRow {
  return {
    scope: '地支',
    type,
    target,
    pillars: items.map((item) => toPillarRef(item, 'branch')).join('、'),
    detail,
    participantKeys: items.map((item) => item.key)
  }
}

function normalizeInteractionType(type: string): string {
  if (type.includes('祿')) {
    return '祿'
  }
  if (type.includes('三會')) {
    return '三會'
  }
  if (type.includes('三合')) {
    return '三合'
  }
  if (type.includes('半合')) {
    return '半合'
  }
  if (type.includes('合')) {
    return '合'
  }
  if (type.includes('沖')) {
    return '沖'
  }
  if (type.includes('刑')) {
    return '刑'
  }
  if (type.includes('破')) {
    return '破'
  }
  return type
}

function normalizeInteractionScope(scope: string): InteractionRow['scope'] {
  if (scope.includes('祿')) {
    return '祿'
  }
  if (scope.includes('干')) {
    return '天干'
  }
  return '地支'
}

function mapPillarTokenToKey(token: string): string | undefined {
  const normalized = token.trim()
  if (!normalized) {
    return undefined
  }
  if (normalized.includes('年')) {
    return 'year'
  }
  if (normalized.includes('月')) {
    return 'month'
  }
  if (normalized.includes('日')) {
    return 'day'
  }
  if (normalized.includes('時')) {
    return 'hour'
  }
  if (normalized.includes('大運')) {
    return previewPillarItems.value.find((item) => item.label === '大運')?.key
  }
  if (normalized.includes('流年')) {
    return previewPillarItems.value.find((item) => item.label === '流年')?.key
  }
  return undefined
}

function splitInteractionParticipants(pillars: string): string[] {
  return pillars
    .split(/[、,，]/)
    .map((token) => mapPillarTokenToKey(token))
    .filter((value): value is string => !!value)
}

function mergeInteractionDetail(interaction: QuantModelInteraction): string {
  const detail = interaction.detail?.trim() || ''
  const outcome = interaction.outcome?.trim() || ''
  if (!outcome || detail.includes(outcome)) {
    return detail
  }
  return detail ? `${detail} ${outcome}` : outcome
}

function sortInteractionRows(rows: InteractionRow[]) {
  const typePriority: Record<string, number> = {
    合: 1,
    祿: 2,
    半合: 3,
    三合: 4,
    三會: 5,
    沖: 6,
    刑: 7,
    破: 8,
    暗合: 9
  }

  return [...rows].sort((a, b) => {
    const scopeCompare = a.scope.localeCompare(b.scope, 'zh-Hant')
    if (scopeCompare !== 0) {
      return scopeCompare
    }
    const typeCompare = (typePriority[a.type] || 99) - (typePriority[b.type] || 99)
    if (typeCompare !== 0) {
      return typeCompare
    }
    return a.pillars.localeCompare(b.pillars, 'zh-Hant')
  })
}

const natalPillarItems = computed<DisplayPillarItem[]>(() => [
  {
    key: 'hour',
    label: '時柱',
    value: props.result.hourPillar,
    stemTenGod: props.result.hourStemTenGod,
    hiddenStems: props.result.hourHiddenStems,
    branchTenGods: props.result.hourBranchTenGods,
    shenSha: props.result.shenSha?.hour || [],
    stemElement: stemElement(props.result.hourPillar.stem),
    branchElement: branchElement(props.result.hourPillar.branch),
    topScoreLabel: `${props.result.hourStemTenGod}：`,
    topScoreValue: formatScore(findStemScore('時干')),
    bottomScoreLabel: `${props.result.hourBranchTenGods[0] || '地支'}：`,
    bottomScoreValue: formatScore(findBranchScore('時支')),
    isPreview: false,
    isCurrentYear: false
  },
  {
    key: 'day',
    label: '日柱',
    value: props.result.dayPillar,
    stemTenGod: props.result.dayStemTenGod,
    hiddenStems: props.result.dayHiddenStems,
    branchTenGods: props.result.dayBranchTenGods,
    shenSha: props.result.shenSha?.day || [],
    stemElement: stemElement(props.result.dayPillar.stem),
    branchElement: branchElement(props.result.dayPillar.branch),
    topScoreLabel: `${dayMasterLabel.value}：`,
    topScoreValue: normalizeStrengthLabel(props.result.quantModel?.strengthLabel),
    bottomScoreLabel: `${props.result.dayBranchTenGods[0] || '地支'}：`,
    bottomScoreValue: formatScore(findBranchScore('日支')),
    isPreview: false,
    isCurrentYear: false
  },
  {
    key: 'month',
    label: '月柱',
    value: props.result.monthPillar,
    stemTenGod: props.result.monthStemTenGod,
    hiddenStems: props.result.monthHiddenStems,
    branchTenGods: props.result.monthBranchTenGods,
    shenSha: props.result.shenSha?.month || [],
    stemElement: stemElement(props.result.monthPillar.stem),
    branchElement: branchElement(props.result.monthPillar.branch),
    topScoreLabel: `${props.result.monthStemTenGod}：`,
    topScoreValue: formatScore(findStemScore('月干')),
    bottomScoreLabel: `${props.result.monthBranchTenGods[0] || '地支'}：`,
    bottomScoreValue: formatScore(findBranchScore('月支')),
    isPreview: false,
    isCurrentYear: false
  },
  {
    key: 'year',
    label: '年柱',
    value: props.result.yearPillar,
    stemTenGod: props.result.yearStemTenGod,
    hiddenStems: props.result.yearHiddenStems,
    branchTenGods: props.result.yearBranchTenGods,
    shenSha: props.result.shenSha?.year || [],
    stemElement: stemElement(props.result.yearPillar.stem),
    branchElement: branchElement(props.result.yearPillar.branch),
    topScoreLabel: `${props.result.yearStemTenGod}：`,
    topScoreValue: formatScore(findStemScore('年干')),
    bottomScoreLabel: `${props.result.yearBranchTenGods[0] || '地支'}：`,
    bottomScoreValue: formatScore(findBranchScore('年支')),
    isPreview: false,
    isCurrentYear: false
  }
])

const previewPillarItems = computed<DisplayPillarItem[]>(() =>
  (props.previewPillars || []).map((item) => ({
    key: item.key,
    label: item.label,
    value: createPlaceholderPillar(item.ganZhi, item.xun, item.xunKong),
    stemTenGod: item.stemTenGod || '-',
    hiddenStems: item.branchHiddenStems || [],
    branchTenGods: item.branchTenGods || [],
    shenSha: [],
    stemElement: stemElement(item.ganZhi.charAt(0)),
    branchElement: branchElement(item.ganZhi.charAt(1)),
    topScoreLabel: `${item.stemTenGod || '天干十神'}：`,
    topScoreValue: '-',
    bottomScoreLabel: `${item.branchTenGods?.[0] || '地支'}：`,
    bottomScoreValue: '-',
    isPreview: true,
    isCurrentYear: !!item.isCurrentYear
  }))
)

const pillarItems = computed<DisplayPillarItem[]>(() => [...natalPillarItems.value, ...previewPillarItems.value])

const showPillars = computed(() => props.showPillars)
const showRelationMap = computed(() => props.showRelationMap)
const showInteractionSection = computed(() => props.showInteractionSection)
const panelTitle = computed(() => {
  if (showPillars.value) {
    return '四柱'
  }
  return '盤面關係'
})


const previewKeyGroups = computed(() => ({
  annual: new Set(previewPillarItems.value.filter((item) => item.label === '流年').map((item) => item.key)),
  luck: new Set(previewPillarItems.value.filter((item) => item.label === '大運').map((item) => item.key))
}))

const derivedInteractionRows = computed<InteractionRow[]>(() => {
  const rows: InteractionRow[] = []
  const items = pillarItems.value
  const matchedThreeCombinationKeys = new Set<string>()

  for (let i = 0; i < items.length; i += 1) {
    for (let j = i + 1; j < items.length; j += 1) {
      for (let k = j + 1; k < items.length; k += 1) {
        const branches = [
          items[i].value.branch,
          items[j].value.branch,
          items[k].value.branch
        ]
        const threeCombination = findTripleRule(BRANCH_THREE_COMBINATIONS, branches)
        if (threeCombination) {
          matchedThreeCombinationKeys.add(threeCombination.key)
        }
      }
    }
  }

  for (let i = 0; i < items.length; i += 1) {
    for (let j = i + 1; j < items.length; j += 1) {
      const left = items[i]
      const right = items[j]

      const stemEntry = findPairEntry(STEM_COMBINATION_ELEMENTS, left.value.stem, right.value.stem)
      if (stemEntry) {
        rows.push(
          buildPairInteractionRow('天干', '合', stemEntry[0], left, right, `天干五合，對應 ${stemEntry[1]}`)
        )
      }
      if (hasPairMatch(STEM_CLASH_PAIRS, left.value.stem, right.value.stem)) {
        rows.push(
          buildPairInteractionRow(
            '天干',
            '沖',
            `${left.value.stem}${right.value.stem}`,
            left,
            right,
            '天干相沖'
          )
        )
      }

      const sixCombinationEntry = findPairEntry(BRANCH_SIX_COMBINATION_ELEMENTS, left.value.branch, right.value.branch)
      if (sixCombinationEntry) {
        rows.push(
          buildPairInteractionRow('地支', '合', sixCombinationEntry[0], left, right, `地支六合，對應 ${sixCombinationEntry[1]}`)
        )
      }
      const halfCombinationEntry = findPairEntry(BRANCH_HALF_COMBINATIONS, left.value.branch, right.value.branch)
      if (
        halfCombinationEntry
        && !isPairCoveredByMatchedThreeCombination(left.value.branch, right.value.branch, matchedThreeCombinationKeys)
      ) {
        rows.push(
          buildPairInteractionRow('地支', '半合', halfCombinationEntry[0], left, right, `地支半合，對應 ${halfCombinationEntry[1]}`)
        )
      }
      if (hasPairMatch(BRANCH_CLASH_PAIRS, left.value.branch, right.value.branch)) {
        rows.push(
          buildPairInteractionRow(
            '地支',
            '沖',
            `${left.value.branch}${right.value.branch}`,
            left,
            right,
            '地支六沖'
          )
        )
      }

      const punishRule = findPairRule(BRANCH_PUNISH_RULES, left.value.branch, right.value.branch)
      if (punishRule) {
        rows.push(buildPairInteractionRow('地支', '刑', punishRule.pair, left, right, punishRule.detail))
      }

      const breakRule = findPairRule(BRANCH_BREAK_RULES, left.value.branch, right.value.branch)
      if (breakRule) {
        rows.push(buildPairInteractionRow('地支', '破', breakRule.pair, left, right, breakRule.detail))
      }

      const hiddenRule = BRANCH_HIDDEN_COMBINATIONS.find((rule) =>
        rule.pair === `${left.value.branch}${right.value.branch}` || rule.pair === `${right.value.branch}${left.value.branch}`
      )
      if (hiddenRule) {
        rows.push(
          buildPairInteractionRow('地支', '暗合', hiddenRule.pair, left, right, `地支暗合：${hiddenRule.detail}，對應 ${hiddenRule.element}`)
        )
      }
    }
  }

  // 為所有天干與其祿地地支產生祿線
  for (const stemItem of items) {
    const luBranches = resolveLuBranches(stemItem)
    if (luBranches.length) {
      for (const branchItem of items) {
        if (luBranches.includes(branchItem.value.branch)) {
          rows.push(buildLuInteractionRow(stemItem, branchItem, branchItem.value.branch))
        }
      }
    }
  }

  for (let i = 0; i < items.length; i += 1) {
    for (let j = i + 1; j < items.length; j += 1) {
      for (let k = j + 1; k < items.length; k += 1) {
        const tripleItems = [items[i], items[j], items[k]]
        const branches = [
          tripleItems[0].value.branch,
          tripleItems[1].value.branch,
          tripleItems[2].value.branch
        ]

        const threeCombination = findTripleRule(BRANCH_THREE_COMBINATIONS, branches)
        if (threeCombination) {
          rows.push(
            buildTripleInteractionRow('三合', threeCombination.key, tripleItems, `地支三合，對應 ${threeCombination.element}`)
          )
        }

        const threeMeeting = findTripleRule(BRANCH_THREE_MEETINGS, branches)
        if (threeMeeting) {
          rows.push(
            buildTripleInteractionRow('三會', threeMeeting.key, tripleItems, `地支三會，對應 ${threeMeeting.element}`)
          )
        }
      }
    }
  }

  for (const branch of ['辰', '午', '酉', '亥']) {
    const selfPunishItems = items.filter((item) => item.value.branch === branch)
    if (selfPunishItems.length >= 2) {
      rows.push(buildTripleInteractionRow('刑', `${branch}${branch}`, selfPunishItems, '自刑'))
    }
  }

  return sortInteractionRows(rows)
})

const backendNatalInteractionRows = computed<InteractionRow[]>(() => {
  const interactions = props.result.quantModel?.interactions || []
  if (!interactions.length) {
    return []
  }

  const deduped = new Map<string, InteractionRow>()
  for (const interaction of interactions) {
    const participantKeys = splitInteractionParticipants(interaction.pillars)
    if (participantKeys.length < 2) {
      continue
    }
    const row: InteractionRow = {
      scope: normalizeInteractionScope(interaction.scope),
      type: normalizeInteractionType(interaction.type),
      target: interaction.target,
      pillars: interaction.pillars,
      detail: mergeInteractionDetail(interaction),
      participantKeys
    }
    const dedupeKey = [
      row.scope,
      row.type,
      row.target,
      [...row.participantKeys].sort().join('-')
    ].join('|')
    deduped.set(dedupeKey, row)
  }

  return sortInteractionRows([...deduped.values()])
})

const groupedInteractionSections = computed<{
  annual: InteractionSection | null
  luck: InteractionSection | null
  natal: InteractionSection
}>(() => {
  const annualRows = derivedInteractionRows.value.filter((row) =>
    row.participantKeys.some((key) => previewKeyGroups.value.annual.has(key))
  )
  const luckRows = derivedInteractionRows.value.filter((row) =>
    !row.participantKeys.some((key) => previewKeyGroups.value.annual.has(key))
    && row.participantKeys.some((key) => previewKeyGroups.value.luck.has(key))
  )
  const fallbackNatalRows = derivedInteractionRows.value.filter((row) =>
    !row.participantKeys.some((key) => previewKeyGroups.value.annual.has(key) || previewKeyGroups.value.luck.has(key))
  )
  // 後端（量化模型）沒有算暗合，原局的暗合由前端補上。
  const natalRows = backendNatalInteractionRows.value.length
    ? sortInteractionRows([
        ...backendNatalInteractionRows.value,
        ...fallbackNatalRows.filter((row) => row.type === '暗合')
      ])
    : fallbackNatalRows

  return {
    annual: annualRows.length ? { title: '流年加入後生成的', rows: annualRows } : null,
    luck: luckRows.length ? { title: '大運加入後生成的', rows: luckRows } : null,
    natal: { title: '原局四柱本來有的', rows: natalRows }
  }
})

const interactionRows = computed<InteractionRow[]>(() =>
  sortInteractionRows([
    ...(groupedInteractionSections.value.annual?.rows || []),
    ...(groupedInteractionSections.value.luck?.rows || []),
    ...groupedInteractionSections.value.natal.rows
  ])
)

const diagramNodes = computed<DiagramNode[]>(() =>
  pillarItems.value.map((item) => ({
    key: item.key,
    label: item.label,
    stem: item.value.stem,
    branch: item.value.branch,
    stemTenGod: item.stemTenGod,
    hiddenStemPairs: item.hiddenStems.map((stem, pairIndex) => ({
      stem,
      tenGod: item.branchTenGods[pairIndex] || '-'
    })),
    isPreview: item.isPreview,
    isCurrentYear: item.isCurrentYear
  }))
)

function resolveRelationLabel(row: InteractionRow) {
  if (!['合', '半合', '三合', '三會', '暗合'].includes(row.type)) {
    return row.type
  }
  const element = row.detail.match(/對應\s*([木火土金水])/)?.[1]
  return element ? `${row.type}(${element})` : row.type
}

// 括號線：天干的關係在上方、地支的關係在下方；祿不畫線，改成地支旁的小標籤。
const PILLAR_CHARS = '甲乙丙丁戊己庚辛壬癸子丑寅卯辰巳午未申酉戌亥'
const BRACKET_DASH: Record<string, string | undefined> = {
  半合: '6 4',
  三會: '3 4',
  沖: '8 5',
  刑: '4 3',
  破: '2 4',
  暗合: '1.5 3.5'
}
const activeRelation = ref<string | null>(null)
const columnOf = computed(() => new Map(pillarItems.value.map((item, index) => [item.key, index])))
const previewKeys = computed(() => new Set(previewPillarItems.value.map((item) => item.key)))

function relationName(row: InteractionRow, cols: number[]) {
  const part = row.scope === '天干' ? 'stem' : 'branch'
  const chars = Array.from(row.target).every((char) => PILLAR_CHARS.includes(char))
    ? row.target
    : [...cols].sort((a, b) => a - b).map((col) => pillarItems.value[col].value[part]).join('')
  if (row.type === '刑' && row.detail === '自刑') {
    return `${chars}自刑`
  }
  return `${chars}${resolveRelationLabel(row)}`
}

// 箭頭：▲ 對日主有利、▼ 不利。原局用課堂三步法已判好的結果，大運流年用和大運面板相同的規則。
const FAVOR_TEXT: Record<Favor, string> = { good: '▲ 對日主有利', bad: '▼ 對日主不利', mixed: '◆ 有利有弊', neutral: '未判喜忌' }
const lessonAnalysis = computed(() => props.result.quantModel?.lessonContent?.threeStepAnalysis)
const natalVerdicts = computed(() => natalRelationVerdicts(lessonAnalysis.value))
const lessonPrefs = computed(() => lessonPreferences(lessonAnalysis.value))
const natalElementSet = computed(
  () => new Set(natalPillarItems.value.flatMap((item) => [item.stemElement, item.branchElement]))
)

function relationVerdict(row: InteractionRow, cols: number[]): RelationVerdict {
  const items = cols.map((col) => pillarItems.value[col])
  const previews = items.filter((item) => item.isPreview)
  const natals = items.filter((item) => !item.isPreview)
  const scope = row.scope === '天干' ? 'stem' : 'branch'
  if (row.type === '暗合') {
    return { favor: 'neutral', reason: '暗合只標出來、暫不判吉凶（老師不用暗合論命，書上看法不一）' }
  }
  if (!previews.length) {
    return (
      findNatalVerdict(natalVerdicts.value, scope, row.type, natals.map((item) => item.label))
      ?? { favor: 'neutral', reason: lessonAnalysis.value ? '課堂三步法沒有判這一項' : '' }
    )
  }
  const part = scope === 'stem' ? 'stem' : 'branch'
  const luck = previews.length === 1 ? previews[0] : null
  return previewRelationVerdict(
    {
      scope,
      type: row.type,
      elementName: row.detail.match(/對應\s*([木火土金水])/)?.[1],
      luck: luck ? { char: luck.value[part], seat: luck.value.branch } : null,
      natal: natals.map((item) => ({ char: item.value[part], isMonth: item.key === 'month' })),
      natalElements: natalElementSet.value
    },
    lessonPrefs.value
  )
}

const activeLink = computed(() => bracketLinks.value.find((link) => link.key === activeRelation.value) ?? null)

const bracketLinks = computed<BracketLink[]>(() =>
  interactionRows.value.flatMap((row) => {
    if (row.scope === '祿') {
      return []
    }
    const cols = [...new Set(row.participantKeys.map((key) => columnOf.value.get(key)))].filter(
      (col): col is number => col != null
    )
    if (cols.length < 2) {
      return []
    }
    const color = RELATION_TYPE_STYLES[row.type]?.color || '#4a5568'
    const verdict = relationVerdict(row, cols)
    return [{
      key: `${row.scope}|${row.type}|${row.target}|${[...cols].sort((a, b) => a - b).join('-')}`,
      side: row.scope === '天干' ? 'top' : 'bottom',
      cols,
      label: relationName(row, cols),
      color,
      dash: BRACKET_DASH[row.type],
      preview: row.participantKeys.some((key) => previewKeys.value.has(key)),
      favor: verdict.favor,
      reason: verdict.reason,
      title: `${row.pillars}：${row.detail}${verdict.reason ? `\n${FAVOR_TEXT[verdict.favor]}：${verdict.reason}` : ''}`
    } satisfies BracketLink]
  })
)

// 換盤或大運預覽改變時，原本點亮的關係可能已經不存在。
watch(bracketLinks, (links) => {
  if (activeRelation.value && !links.some((link) => link.key === activeRelation.value)) {
    activeRelation.value = null
  }
})

const relatedChars = computed(() => {
  const link = bracketLinks.value.find((item) => item.key === activeRelation.value)
  if (!link) {
    return new Set<string>()
  }
  const part = link.side === 'top' ? 'stem' : 'branch'
  return new Set(link.cols.map((col) => `${col}:${part}`))
})
const isRelated = (col: number, part: 'stem' | 'branch') => relatedChars.value.has(`${col}:${part}`)

// ---- 強弱模式：把「強弱變化圖」直接畫在四柱上 ----
// 字的底色＝同黨（暖）／異黨（冷），字下貼作用標記；下方 StrengthBar 控制做到第幾步。
const VIEW_STORAGE_KEY = 'bazi:pillar-view:v1'
type PillarView = 'relation' | 'strength' | 'flow'
function readView(): PillarView {
  try {
    const saved = localStorage.getItem(VIEW_STORAGE_KEY)
    return saved === 'strength' || saved === 'flow' ? saved : 'relation'
  } catch {
    return 'relation'
  }
}
// 三種看法：關係（合沖刑害括號）、強弱（同黨異黨＋通根）、流通（五行圈＋重點，ElementFlowRing）
const pillarView = ref<PillarView>(readView())
const strengthView = computed(() => pillarView.value === 'strength')
const flowOn = computed(() => pillarView.value === 'flow' && !!shift.value)
function setView(view: PillarView) {
  pillarView.value = view
  activeRelation.value = null
  try {
    localStorage.setItem(VIEW_STORAGE_KEY, view)
  } catch {
    // 無痕視窗等情況存不了，只影響下次開啟的預設
  }
}

/** 大運、流年預覽柱：都算進強弱和流通（流年接在大運後面）。 */
const luckPreview = computed(() => previewPillarItems.value.find((item) => item.label === '大運') ?? null)
const annualPreview = computed(() => previewPillarItems.value.find((item) => item.label === '流年') ?? null)
const shift = computed(() => {
  const luck = luckPreview.value
  let label = ''
  if (luck) {
    const row = props.result.daYun?.find((item) => item.ganZhi === luck.value.stem + luck.value.branch)
    if (row && row.startAge != null) label = `${row.startAge}–${row.endAge == null ? '' : row.endAge + 1} 歲`
  }
  const annual = annualPreview.value
  const annualRow = annual ? annualFromKey(props.result, annual.key) : null
  return strengthShift(
    lessonAnalysis.value,
    luck ? { ganZhi: luck.value.stem + luck.value.branch, label } : null,
    annual ? { ganZhi: annual.value.stem + annual.value.branch, label: annualRow ? `${annualRow.year} 年` : '' } : null
  )
})
const strengthOn = computed(() => strengthView.value && !!shift.value)
const applied = ref(0)
const focus = ref(-1)
watch(
  shift,
  (value) => {
    applied.value = value?.steps.length ?? 0
    focus.value = -1
  },
  { immediate: true }
)

const PILLAR_PREFIX: Record<string, string> = { hour: '時', day: '日', month: '月', year: '年' }
function cellIdOf(col: number, part: 'stem' | 'branch') {
  const item = pillarItems.value[col]
  if (!item) return null
  const prefix = item.isPreview
    ? item.key === luckPreview.value?.key
      ? '運'
      : item.key === annualPreview.value?.key
        ? '歲'
        : null
    : PILLAR_PREFIX[item.key]
  return prefix ? `${prefix}${part === 'stem' ? '干' : '支'}` : null
}
const colOfCell = computed(() => {
  const map = new Map<string, number>()
  pillarItems.value.forEach((_, col) => {
    for (const part of ['stem', 'branch'] as const) {
      const id = cellIdOf(col, part)
      if (id) map.set(id, col)
    }
  })
  return map
})
const cellStates = computed(() => (shift.value ? cellStatesAt(shift.value, applied.value) : null))
const focusStep = computed(() => (shift.value && focus.value >= 0 ? shift.value.steps[focus.value] ?? null : null))
const DIR_SYMBOL: Record<ShiftDir, string> = { up: '↑', down: '↓', change: '⇄', bind: '∞' }

function svInfo(col: number, part: 'stem' | 'branch'): StrengthCharInfo | null {
  if (!strengthOn.value || !shift.value) return null
  const id = cellIdOf(col, part)
  const cell = id ? shift.value.cells.find((item) => item.id === id) : null
  const state = id ? cellStates.value?.get(id) : null
  if (!cell || !state) return null
  const step = focusStep.value
  const inFocus = !!step && step.cells.includes(cell.id)
  const currentEffects = step ? step.effects.filter((effect) => effect.cell === cell.id) : []
  // 有選步驟：只標這一步；看全部：每種方向留一個符號，免得字下擠滿。
  const source = step ? currentEffects : state.marks
  const seen = new Set<string>()
  const tags: StrengthCharTag[] = []
  for (const effect of source) {
    if (!step && effect.dir === 'change' && state.transformed) continue
    const symbol = DIR_SYMBOL[effect.dir]
    const help = helpsParty(effect, cell.party)
    const dedupe = step ? `${tags.length}` : `${symbol}${help}${effect.light ? 'l' : ''}`
    if (seen.has(dedupe)) continue
    seen.add(dedupe)
    tags.push({ symbol, text: effect.text, help, light: effect.light, current: !!step })
  }
  return {
    party: state.party,
    dayMaster: cell.isDayMaster,
    focus: inFocus,
    dim: !!step && !inFocus,
    toElement: state.transformed && !currentEffects.some((effect) => effect.dir === 'change') ? state.element : null,
    tags
  }
}
function svClass(col: number, part: 'stem' | 'branch') {
  const info = svInfo(col, part)
  if (!info) return strengthOn.value ? 'sv-none' : ''
  return ['sv-char', `sv-${info.party}`, { 'sv-dm': info.dayMaster, 'sv-focus': info.focus, 'sv-dim': info.dim, 'sv-trans': !!info.toElement }]
}

// ---- 通根：天干在哪些地支有根，畫成從天干連到地支的線 ----
const rootFocusCol = ref<number | null>(null)
const dayCol = computed(() => pillarItems.value.findIndex((item) => item.key === 'day'))
watch(
  () => props.result,
  () => {
    rootFocusCol.value = dayCol.value >= 0 ? dayCol.value : null
  },
  { immediate: true }
)
function toggleRootFocus(col: number) {
  rootFocusCol.value = rootFocusCol.value === col ? null : col
}
/** 強弱模式看得到的地支（原局四柱＋大運；流年不算）。 */
const rootBranchCols = computed(() =>
  pillarItems.value.flatMap((item, col) => (cellIdOf(col, 'branch') ? [{ col, branch: item.value.branch }] : []))
)
function rootsOf(col: number): StemRoot[] {
  const item = pillarItems.value[col]
  if (!item || !cellIdOf(col, 'stem')) return []
  return stemRoots(item.value.stem, col, rootBranchCols.value)
}
/** 已套用的步驟裡，這個地支有沒有被沖（被沖的根要打折）。 */
function branchShaken(col: number) {
  const id = cellIdOf(col, 'branch')
  if (!id || !shift.value) return false
  return shift.value.steps
    .slice(0, applied.value)
    .some((step) => !step.inactive && step.tag.includes('沖') && step.effects.some((effect) => effect.cell === id && effect.dir === 'down' && !effect.light))
}
function rootChip(col: number) {
  if (!strengthOn.value || !cellIdOf(col, 'stem')) return null
  const roots = rootsOf(col)
  const on = rootFocusCol.value === col
  const stem = pillarItems.value[col].value.stem
  const shakenAll = roots.length > 0 && roots.every((root) => branchShaken(root.col))
  return {
    text: roots.length ? `有根${roots.length > 1 ? `×${roots.length}` : ''}` : '無根',
    cls: { 'is-rooted': roots.length > 0, 'is-rootless': !roots.length, 'is-on': on, 'is-shaken': shakenAll },
    title: roots.length
      ? `${stem}的根：${roots.map((root) => `${pillarItems.value[root.col].label}${root.branch}（${root.level}${root.seated ? '、坐下' : ''}）`).join('、')}。點一下${on ? '收起' : '畫出'}連線。`
      : `${stem}在地支沒有同五行的藏干，是無根、虛浮的字。`
  }
}
/** 根的顏色跟著天干的五行走（甲乙木綠、丙丁火紅……）。 */
function rootColor(stemCol: number) {
  const item = pillarItems.value[stemCol]
  return item ? `var(--wx-${item.stemElement})` : 'var(--wx-wood)'
}
/** 根落在這個地支的哪一個藏干（同五行的那一個）。 */
function rootHiddenStem(stemCol: number, branchCol: number) {
  const stem = pillarItems.value[stemCol]?.value.stem
  const element = stem ? STEM_WUXING[stem] : null
  return pillarItems.value[branchCol]?.hiddenStems.find((char) => STEM_WUXING[char] === element) ?? null
}
/** 藏干欄上要框起來的根（目前選的天干）。 */
function hiddenRootsOf(col: number): HiddenRootMark[] | null {
  const focusCol = rootFocusCol.value
  if (!strengthOn.value || focusCol == null) return null
  const root = rootsOf(focusCol).find((item) => item.col === col)
  const stem = root ? rootHiddenStem(focusCol, col) : null
  if (!root || !stem) return null
  return [{ stem, level: root.level, color: rootColor(focusCol), shaken: branchShaken(col) }]
}
const LEVEL_WORD: Record<RootLevel, string> = { 本氣: '本氣根最有力', 中氣: '中氣根次之', 餘氣: '餘氣根最輕' }
const rootNote = computed(() => {
  const col = rootFocusCol.value
  if (!strengthOn.value || col == null || !cellIdOf(col, 'stem')) return ''
  const item = pillarItems.value[col]
  const who = item.key === 'day' ? `日主${item.value.stem}` : `${item.label}天干${item.value.stem}`
  const roots = rootsOf(col)
  if (!roots.length) {
    return `<b>${who}無根</b>：地支沒有同五行的藏干，字浮在天上，力量較虛${item.key === 'day' ? '（得地 ○）' : ''}。點其他天干下的「有根／無根」換看別的字。`
  }
  const parts = roots.map((root) => {
    const shaken = branchShaken(root.col)
    return `${pillarItems.value[root.col].label}${root.branch}（${root.level}${root.seated ? '、坐下' : ''}${shaken ? '、<span class="sv-warn">被沖</span>' : ''}）`
  })
  const levels = [...new Set(roots.map((root) => LEVEL_WORD[root.level]))].join('，')
  const stemId = cellIdOf(col, 'stem')
  const state = stemId ? cellStates.value?.get(stemId) : null
  const transformedNote = state?.transformed ? `（${item.value.stem}已合化成${state.element}，這裡看的是它原本五行的根）` : ''
  return `<b>${who}的根</b>${transformedNote}：${parts.join('、')}。${levels}；坐下的根最近。線的顏色就是這個字的五行，越粗根越有力；變淡、打 ✕ 的是被沖動的根。`
})

type RootLine = { key: string; d: string; ex: number; ey: number; level: RootLevel; shaken: boolean; color: string }
const rootLines = ref<RootLine[]>([])
let gridEl: HTMLElement | null = null
let resizeObserver: ResizeObserver | null = null
function measureRootLines() {
  const col = rootFocusCol.value
  gridEl = document.querySelector<HTMLElement>(`[data-pillar-grid="${panelUid}"]`)
  if (!strengthOn.value || col == null || !gridEl) {
    rootLines.value = []
    return
  }
  const box = gridEl.getBoundingClientRect()
  // 線從天干下面的「有根」鈕出發，不蓋到鈕上的字。
  const stemEl = gridEl.querySelector<HTMLElement>(`[data-sv-root="${col}"]`) ?? gridEl.querySelector<HTMLElement>(`[data-sv="${col}-stem"]`)
  if (!stemEl) {
    rootLines.value = []
    return
  }
  const s = stemEl.getBoundingClientRect()
  const sx = s.left + s.width / 2 - box.left
  const sy = s.bottom - box.top
  let offset = 0
  rootLines.value = rootsOf(col).flatMap((root) => {
    // 線接到藏干欄（同五行的那一個藏干），找不到才接地支字。
    const hidden = rootHiddenStem(col, root.col)
    const el =
      (hidden ? gridEl!.querySelector<HTMLElement>(`[data-sv-hidden="${root.col}-${hidden}"]`) : null) ??
      gridEl!.querySelector<HTMLElement>(`[data-sv="${root.col}-branch"]`)
    if (!el) return []
    const r = el.getBoundingClientRect()
    const ex = r.left + r.width / 2 - box.left
    const ey = r.top - box.top
    // 同一個方向有好幾條線時，彎的高度錯開，不會疊成一條。
    const bend = Math.max(24, (ey - sy) * 0.6) - (root.col === col ? 0 : offset++ * 7)
    const d = root.col === col ? `M${sx},${sy} L${ex},${ey}` : `M${sx},${sy} C${sx},${sy + bend} ${ex},${ey - bend} ${ex},${ey}`
    return [{ key: `${col}-${root.col}`, d, ex, ey, level: root.level, shaken: branchShaken(root.col), color: rootColor(col) }]
  })
}
const panelUid = `pp${Math.random().toString(36).slice(2, 8)}`
/** 四柱的地支列、藏干列各自對齊同一條水平線（上面的標記多寡不同，高度會參差）。 */
function alignRows() {
  const grid = document.querySelector<HTMLElement>(`[data-pillar-grid="${panelUid}"]`)
  if (!grid) return
  // 由上往下對齊：十神、地支、藏干（上一排對齊後，天干自然跟著對齊）
  for (const name of ['chip', 'branch', 'hidden']) {
    const rows = [...grid.querySelectorAll<HTMLElement>(`[data-align="${name}"]`)]
    rows.forEach((row) => (row.style.marginTop = ''))
    const tops = rows.map((row) => row.getBoundingClientRect().top)
    const max = Math.max(...tops)
    rows.forEach((row, i) => {
      const gap = max - tops[i]
      if (gap > 0.5) row.style.marginTop = `${parseFloat(getComputedStyle(row).marginTop) + gap}px`
    })
  }
}
function scheduleRootLines() {
  nextTick(() =>
    requestAnimationFrame(() => {
      alignRows()
      measureRootLines()
    })
  )
}
watch([strengthOn, rootFocusCol, applied, focus, pillarItems, () => props.compact, isResearch], scheduleRootLines)
onMounted(() => {
  scheduleRootLines()
  if (typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver(scheduleRootLines)
    const grid = document.querySelector<HTMLElement>(`[data-pillar-grid="${panelUid}"]`)
    if (grid) resizeObserver.observe(grid)
  }
})
onBeforeUnmount(() => resizeObserver?.disconnect())

// ---- 四柱上的大運柱：點兩下跳出清單，直接換一步大運 ----
const { luckIndex, annualYear, monthIndex } = useLuckSelection()
const luckPickerFor = ref<string | null>(null)
// 大運柱、流年柱都可以點兩下換。和下方大運列一樣由右到左（最早的在右邊）。
const isLuckItem = (item: DisplayPillarItem) => item.isPreview && (item.key.startsWith('luck-') || item.key.startsWith('annual-'))
const pickerKind = computed(() => (luckPickerFor.value?.startsWith('annual-') ? '流年' : '大運'))
interface PickerCell {
  key: string
  ganZhi: string
  stemGod: string
  branchGod: string
  sub: string
  title: string
  isCurrent: boolean
  pick: () => void
}
const pickerCells = computed<PickerCell[]>(() => {
  if (!luckPickerFor.value) return []
  if (pickerKind.value === '流年') {
    const current = annualFromKey(props.result, luckPickerFor.value)
    const luck = props.result.daYun?.find((row) => row.index === current?.luckIndex)
    return annualRows(luck)
      .reverse()
      .map((row) => ({
        key: annualKey(row.luckIndex, row.index),
        ganZhi: row.ganZhi,
        stemGod: row.stemTenGod ?? '',
        branchGod: row.branchTenGods?.[0] ?? '',
        sub: `${row.year}`,
        title: `${row.year} ${row.ganZhi}流年（${row.age} 歲）`,
        isCurrent: row.isCurrent,
        pick: () => pickAnnual(row)
      }))
  }
  return luckRows(props.result)
    .reverse()
    .map((row) => ({
      key: `luck-${row.index}`,
      ganZhi: row.ganZhi,
      stemGod: row.stemTenGod ?? '',
      branchGod: row.branchTenGods?.[0] ?? '',
      sub: row.ages,
      title: `${row.ganZhi}大運 ${row.ages}歲（${row.years}）`,
      isCurrent: row.isCurrent,
      pick: () => pickLuck(row)
    }))
})
let lastTap = { key: '', time: 0 }
/** 用兩次點擊的間隔判斷「點兩下」，滑鼠和手機觸控都適用。 */
function onPillarTap(item: DisplayPillarItem, event: MouseEvent) {
  if (!isLuckItem(item)) return
  if ((event.target as HTMLElement | null)?.closest('button, a, .rb-label')) return
  const now = Date.now()
  if (lastTap.key === item.key && now - lastTap.time < 450) {
    lastTap = { key: '', time: 0 }
    window.getSelection?.()?.removeAllRanges()
    luckPickerFor.value = luckPickerFor.value === item.key ? null : item.key
    if (luckPickerFor.value) {
      nextTick(() => {
        const track = document.querySelector<HTMLElement>('.luck-picker-track')
        const cell = track?.querySelector<HTMLElement>('.luck-picker-cell.is-on')
        if (track && cell) track.scrollLeft = cell.offsetLeft - (track.clientWidth - cell.offsetWidth) / 2
      })
    }
    return
  }
  lastTap = { key: item.key, time: now }
}
function pickLuck(row: LuckRowView) {
  luckPickerFor.value = null
  const year = baziYearAt()
  luckIndex.value = row.index
  annualYear.value = row.liuNian?.some((item) => item.year === year) ? year : null
  monthIndex.value = null
  // 原本有流年柱：換到新大運裡同一個位置的流年（新大運有今年就用今年）
  const oldAnnual = annualFromKey(props.result, annualPreview.value?.key)
  if (!oldAnnual) {
    emit('preview-change', [luckPreviewPillar(row)])
    return
  }
  const years = annualRows(row, year)
  const next = years.find((item) => item.isCurrent) ?? years.find((item) => item.index === oldAnnual.index) ?? years[0]
  if (next) annualYear.value = next.year
  emit('preview-change', next ? [luckPreviewPillar(row), annualPreviewPillar(next)] : [luckPreviewPillar(row)])
}
function pickAnnual(row: AnnualRowView) {
  luckPickerFor.value = null
  const luck = luckRows(props.result).find((item) => item.index === row.luckIndex)
  if (!luck) return
  luckIndex.value = luck.index
  annualYear.value = row.year
  monthIndex.value = null
  emit('preview-change', [luckPreviewPillar(luck), annualPreviewPillar(row)])
}
function closeLuckPicker(event: Event) {
  if (!luckPickerFor.value) return
  if (event instanceof KeyboardEvent && event.key !== 'Escape') return
  luckPickerFor.value = null
}
watch(previewPillarItems, (items) => {
  if (luckPickerFor.value && !items.some((item) => item.key === luckPickerFor.value)) luckPickerFor.value = null
})
onMounted(() => {
  document.addEventListener('pointerdown', closeLuckPicker)
  document.addEventListener('keydown', closeLuckPicker)
})
onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', closeLuckPicker)
  document.removeEventListener('keydown', closeLuckPicker)
})

/** 這一步對應到哪一條關係線（同一排、同一組柱）。 */
function linkForStep(stepIndex: number) {
  const step = shift.value?.steps[stepIndex]
  if (!step) return null
  const parts = new Set(step.cells.map((id) => (id.endsWith('干') ? 'top' : 'bottom')))
  if (parts.size !== 1) return null
  const cols = [...new Set(step.cells.map((id) => colOfCell.value.get(id)))]
  if (cols.some((col) => col == null)) return null
  const want = (cols as number[]).sort((a, b) => a - b).join('-')
  const side = [...parts][0]
  return bracketLinks.value.find((link) => link.side === side && [...link.cols].sort((a, b) => a - b).join('-') === want) ?? null
}
const bracketActive = computed(() => {
  if (!strengthOn.value) return activeRelation.value
  return (focus.value >= 0 ? linkForStep(focus.value)?.key : null) ?? activeRelation.value
})
function onBracketActive(key: string | null) {
  if (!strengthOn.value || !key || !shift.value) {
    activeRelation.value = key
    if (strengthOn.value) focus.value = -1
    return
  }
  // 點了關係線：跳到這條線在強弱步驟裡的那一步。
  const steps = shift.value.steps
  let index = -1
  for (let i = steps.length - 1; i >= 0; i -= 1) {
    if (linkForStep(i)?.key === key) {
      index = i
      break
    }
  }
  if (index < 0) {
    activeRelation.value = key
    focus.value = -1
    return
  }
  activeRelation.value = null
  applied.value = index + 1
  focus.value = index
}
watch(focus, (value) => {
  if (value >= 0) activeRelation.value = null
})

/** 這一柱的地支是哪些天干的祿（例：甲祿在寅）。戊己只看日干，同上面祿線的規則。 */
function luTagsOf(col: number) {
  const target = pillarItems.value[col]
  if (!target) {
    return []
  }
  // 同一個天干字出現好幾柱（例：四個庚）只標一次；是哪幾柱寫在滑鼠提示裡。
  const groups = new Map<string, string[]>()
  for (const stemItem of pillarItems.value) {
    if (!resolveLuBranches(stemItem).includes(target.value.branch)) continue
    groups.set(stemItem.value.stem, [...(groups.get(stemItem.value.stem) ?? []), stemItem.label])
  }
  const tags: Array<{ text: string; title: string; kind?: 'hidden' }> = [...groups].map(([stem, labels]) => ({
    text: `${stem}祿`,
    title: `${labels.join('、')}天干${stem}的祿在${target.value.branch}（${target.label}）`
  }))
  const hidden = PILLAR_HIDDEN_COMBINATIONS[`${target.value.stem}${target.value.branch}`]
  if (hidden) {
    tags.push({
      text: '暗合',
      kind: 'hidden',
      title: `${target.label}${target.value.stem}${target.value.branch}干支暗合：${hidden.detail}${hidden.pure ? '（六組較純粹之一）' : ''}。天干和坐下藏干互相牽絆，暫不判吉凶。`
    })
  }
  return tags
}
</script>

<style scoped>
.pillar-card-grid {
  display: grid;
  gap: 16px;
}
.pillar-panel-compact .pillar-card-grid {
  gap: 8px;
}
.pillar-panel-compact .pillar-card-grid.is-dense {
  gap: 4px;
}
.pillar-cell {
  display: flex;
  min-width: 0;
}
.pillar-cell > .pillar-box {
  flex: 1;
  min-width: 0;
}
.pillar-panel-compact .is-dense .pillar-box.is-compact {
  padding-left: 4px;
  padding-right: 4px;
}
.pillar-panel-compact .is-dense .pillar-main-label {
  display: none;
}
.relation-explain {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 10px;
  margin: 4px 0 0;
  padding: 6px 10px;
  border-left: 4px solid var(--bazi-border-strong, #c9b99b);
  border-radius: 8px;
  background: var(--bazi-surface-strong, #fffdf8);
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
  white-space: pre-line;
}
.relation-explain.is-good { border-left-color: #2f9e44; }
.relation-explain.is-bad { border-left-color: #c53030; }
.relation-explain.is-good .relation-explain-favor { color: #23703c; font-weight: 700; }
.relation-explain.is-bad .relation-explain-favor { color: #c53030; font-weight: 700; }
.pillar-flow {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px dashed #eadfcc;
}
.relation-hint {
  margin: 4px 0 0;
  color: var(--el-text-color-secondary);
  font-size: calc(12px * var(--app-font-scale));
}
.wuxing-char.is-related {
  border-radius: 8px;
  background: rgba(250, 204, 21, 0.35);
  box-shadow: 0 0 0 2px rgba(202, 138, 4, 0.75);
}
.lu-tags {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 2px;
  margin-top: 2px;
}
.lu-tag {
  display: inline-block;
  margin-left: 4px;
  padding: 0 5px;
  border: 1px solid #b7791f;
  border-radius: 999px;
  background: #fff7ea;
  color: #b7791f;
  font-size: calc(11px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.5;
  white-space: nowrap;
  cursor: help;
}
.lu-tags .lu-tag {
  margin-left: 0;
}
.lu-tag.is-hidden-combo {
  border-color: #2c9a9a;
  background: #e8f6f6;
  color: #237a7a;
}
.branch-anchor {
  position: relative;
  display: inline-block;
}
.lu-float {
  position: absolute;
  left: 100%;
  bottom: 6px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  margin-left: 2px;
  pointer-events: auto;
}
.lu-float .lu-tag {
  margin-left: 0;
}
.pillar-card-grid.is-dense .lu-float .lu-tag {
  padding: 0 3px;
  font-size: calc(10px * var(--app-font-scale));
}
.relation-node-grid {
  display: grid;
  gap: 8px;
}
.relation-map {
  padding-left: 10px;
  padding-right: 10px;
}

.score-band {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: calc(14px * var(--app-font-scale));
  line-height: 1.2;
  color: var(--el-text-color-primary);
  padding: 4px 0;
  border-bottom: 1px solid rgba(188, 195, 211, 0.45);
}

.score-band-top {
  margin-bottom: 8px;
}

.score-band-bottom {
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px solid rgba(188, 195, 211, 0.45);
  border-bottom: none;
}

.score-band-label {
  color: var(--el-text-color-primary);
}

.score-band-value {
  font-weight: 700;
}

.pillar-box {
  border: 1px solid rgba(188, 195, 211, 0.45);
  border-radius: 14px;
  padding: 14px 12px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.95));
  overflow: hidden;
}

.pillar-box.is-preview {
  border-style: dashed;
  background: linear-gradient(180deg, rgba(255, 251, 235, 0.98), rgba(255, 247, 237, 0.96));
}

.pillar-card-grid {
  margin-bottom: 20px;
}

.pillar-box.is-current-year-preview {
  border-color: #d97706;
  box-shadow: 0 0 0 1px rgba(217, 119, 6, 0.12), 0 12px 24px rgba(217, 119, 6, 0.12);
}

.relation-map-section {
  display: flow-root;
  clear: both;
  position: relative;
  z-index: 0;
  margin-top: 8px;
}

.relation-map-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 8px 12px;
  align-items: center;
  margin-bottom: 12px;
}

.relation-map-header h3 {
  margin: 0;
  font-size: calc(16px * var(--app-font-scale));
}




.relation-map {
  position: relative;
  border: 1px solid rgba(188, 195, 211, 0.45);
  border-radius: 18px;
  background:
    radial-gradient(circle at top, rgba(255, 251, 235, 0.8), rgba(255, 255, 255, 0) 55%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.96));
  overflow: hidden;
}





.relation-node {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 10px 12px;
  border-radius: 16px;
  border: 1px solid rgba(188, 195, 211, 0.7);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.96));
  text-align: center;
  box-shadow:
    0 12px 28px rgba(15, 23, 42, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.88);
  transition: transform 180ms ease, box-shadow 180ms ease, border-color 180ms ease, background 180ms ease;
  overflow: hidden;
}

.relation-node.is-preview {
  border-style: dashed;
  border-color: rgba(180, 122, 28, 0.45);
  background:
    linear-gradient(180deg, rgba(255, 250, 240, 0.98), rgba(255, 245, 235, 0.98));
}

.relation-node.is-current-year {
  border-color: #d97706;
  background:
    linear-gradient(180deg, rgba(255, 251, 235, 0.99), rgba(255, 243, 214, 0.96));
  box-shadow:
    0 0 0 1px rgba(217, 119, 6, 0.12),
    0 14px 30px rgba(217, 119, 6, 0.16),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.relation-node-label {
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
  line-height: 1.1;
  letter-spacing: 0.08em;
}

.relation-node-ten-god {
  margin-top: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 20px;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: calc(11px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.1;
  border: 1px solid transparent;
}

.relation-node-pillar {
  margin-top: 6px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 16px;
}

.relation-node-stem,
.relation-node-branch {
  font-size: calc(28px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1;
  text-shadow: 0 1px 0 rgba(255, 255, 255, 0.75);
}

.relation-node-hidden-stems {
  margin-top: 10px;
  width: 100%;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  gap: 6px;
  min-height: max(66px, var(--relation-hidden-fit, 0px));
  overflow: hidden;
}

.relation-node-hidden-stems.is-empty {
  visibility: hidden;
}

.relation-node-hidden-item {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 18px;
}

.relation-node-hidden-stem {
  font-size: calc(16px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1;
}

.relation-node-hidden-ten-god {
  display: inline-flex;
  white-space: nowrap;
  align-items: center;
  justify-content: center;
  min-height: 18px;
  padding: 1px 6px;
  border-radius: 999px;
  font-size: calc(10px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.1;
  border: 1px solid transparent;
}

.ten-god-peer {
  color: #1e40af;
  background: rgba(219, 234, 254, 0.88);
  border-color: rgba(96, 165, 250, 0.42);
}

.ten-god-output {
  color: #065f46;
  background: rgba(209, 250, 229, 0.88);
  border-color: rgba(52, 211, 153, 0.42);
}

.ten-god-wealth {
  color: #92400e;
  background: rgba(254, 243, 199, 0.9);
  border-color: rgba(251, 191, 36, 0.44);
}

.ten-god-officer {
  color: #9a3412;
  background: rgba(254, 226, 226, 0.9);
  border-color: rgba(248, 113, 113, 0.42);
}

.ten-god-resource {
  color: #6b21a8;
  background: rgba(243, 232, 255, 0.92);
  border-color: rgba(192, 132, 252, 0.42);
}

.ten-god-daymaster {
  color: #0f172a;
  background: rgba(226, 232, 240, 0.9);
  border-color: rgba(148, 163, 184, 0.45);
}

.ten-god-neutral {
  color: #475569;
  background: rgba(241, 245, 249, 0.92);
  border-color: rgba(148, 163, 184, 0.35);
}

.interaction-section {
  margin-top: 20px;
  padding-top: 18px;
  border-top: 1px solid rgba(188, 195, 211, 0.45);
}

.interaction-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 6px;
}

.interaction-header h3 {
  margin: 0;
  font-size: calc(16px * var(--app-font-scale));
}

.interaction-count {
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
}

.interaction-hint {
  margin: 0 0 12px;
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
}

.interaction-group.with-divider {
  margin-top: 18px;
  padding-top: 18px;
  border-top: 1px solid rgba(188, 195, 211, 0.45);
}

.interaction-group-title {
  margin-bottom: 10px;
  font-size: calc(13px * var(--app-font-scale));
  font-weight: 700;
  color: #6a4f1f;
}

.pillar-panel-compact :deep(.el-card__body) {
  padding: 12px;
}

.pillar-box.is-compact {
  padding: 8px 6px;
  border-radius: 14px;
}

.compact-pillar-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 4px;
  margin-bottom: 4px;
}

.compact-pillar-header .pillar-label {
  white-space: nowrap;
}
.compact-score-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 16px;
  padding: 1px 5px;
  border-radius: 999px;
  background: rgba(146, 64, 14, 0.12);
  color: #92400e;
  font-size: calc(9px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.1;
  white-space: nowrap;
}

.compact-ten-god-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 16px;
  padding: 1px 6px;
  border-radius: 999px;
  margin-bottom: 6px;
  font-size: calc(9px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.1;
  border: 1px solid transparent;
}

.compact-pillar-ganzhi {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 6px;
}

.compact-pillar-char-block {
  padding: 4px 3px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.7);
  text-align: center;
}

.compact-pillar-line {
  display: grid;
  gap: 2px;
  margin-bottom: 5px;
}

.compact-score-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 0 6px;
  margin-top: 2px;
  padding-top: 5px;
  border-top: 1px solid rgba(188, 195, 211, 0.35);
}

.compact-score-label {
  color: var(--el-text-color-secondary);
  font-size: calc(9px * var(--app-font-scale));
  line-height: 1.2;
  white-space: nowrap;
}

.compact-score-value {
  color: #0f172a;
  font-size: calc(10px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.1;
  white-space: nowrap;
}

.compact-line-label {
  color: var(--el-text-color-secondary);
  font-size: calc(9px * var(--app-font-scale));
  line-height: 1.2;
}

.compact-line-value {
  font-size: calc(10px * var(--app-font-scale));
  font-weight: 600;
  line-height: 1.25;
  word-break: break-word;
}

.compact-hidden-stems {
  display: flex;
  flex-wrap: wrap;
  gap: 0 2px;
}

.compact-pillar-foot {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 6px;
  margin-top: 4px;
  padding-top: 5px;
  border-top: 1px solid rgba(188, 195, 211, 0.35);
  color: #6b7280;
  font-size: calc(9px * var(--app-font-scale));
  line-height: 1.2;
}

.pillar-panel-compact .relation-map-section {
  margin-top: 0;
}

.pillar-panel-compact .relation-map-header h3,
.pillar-panel-compact .interaction-header h3 {
  font-size: calc(14px * var(--app-font-scale));
}


.pillar-panel-compact .relation-node-label {
  font-size: calc(10px * var(--app-font-scale));
}

.pillar-panel-compact .relation-node-ten-god,
.pillar-panel-compact .relation-node-hidden-ten-god {
  font-size: calc(9px * var(--app-font-scale));
}

.pillar-panel-compact .relation-node-hidden-stem {
  font-size: calc(12px * var(--app-font-scale));
}

.pillar-panel-compact .relation-map {
}

.pillar-panel-compact .relation-node {
  padding: 6px 6px 8px;
}

.pillar-panel-compact .relation-node-stem,
.pillar-panel-compact .relation-node-branch {
  font-size: calc(18px * var(--app-font-scale));
}

.pillar-panel-compact .relation-node-hidden-stems {
  min-height: max(52px, var(--relation-hidden-fit, 0px));
  gap: 4px;
}

.pillar-panel-compact :deep(.el-table .cell) {
  font-size: calc(11px * var(--app-font-scale));
  line-height: 1.35;
}

.pillar-box.is-compact .pillar-label,
.pillar-box.is-compact .meta-value,
.pillar-box.is-compact .pillar-main-value {
  word-break: break-word;
}

.pillar-box.is-compact .pillar-label {
  margin-bottom: 8px;
  font-size: calc(13px * var(--app-font-scale));
}

.pillar-box.is-compact .meta-row {
  margin-bottom: 6px;
}

.pillar-box.is-compact .meta-label,
.pillar-box.is-compact .pillar-main-label {
  font-size: calc(10px * var(--app-font-scale));
}

.pillar-box.is-compact .meta-value {
  font-size: calc(13px * var(--app-font-scale));
}

.pillar-box.is-compact .pillar-main {
  gap: 3px;
  margin-bottom: 6px;
}

.pillar-box.is-compact .pillar-main-value {
  font-size: calc(26px * var(--app-font-scale));
}

@media (max-width: 767px) {
  .relation-map-header {
    flex-direction: column;
  }


  .relation-map {
  }

  .relation-node {
    padding: 6px 6px 8px;
  }

  .relation-node-stem,
  .relation-node-branch {
    font-size: calc(20px * var(--app-font-scale));
  }

  .relation-node-hidden-stems {
    min-height: max(54px, var(--relation-hidden-fit, 0px));
    gap: 4px;
  }

  .relation-node-hidden-stem {
    font-size: calc(13px * var(--app-font-scale));
  }

  .relation-node-hidden-ten-god {
    padding: 1px 5px;
    font-size: calc(9px * var(--app-font-scale));
  }

  .meta-value {
    font-size: calc(15px * var(--app-font-scale));
  }

  .pillar-main-value {
    font-size: calc(28px * var(--app-font-scale));
  }

  .pillar-panel-compact :deep(.el-row) {
    row-gap: 8px;
  }

  .pillar-box.is-compact .compact-line-value {
    font-size: calc(10px * var(--app-font-scale));
  }

  .pillar-panel-compact .relation-map {
  }

  .pillar-panel-compact .relation-node {
  }

  .pillar-panel-compact .relation-node-stem,
  .pillar-panel-compact .relation-node-branch {
    font-size: calc(16px * var(--app-font-scale));
  }
}

@media (max-width: 480px) {
  .relation-map {
  }

  .relation-node {
  }

  .relation-node-stem,
  .relation-node-branch {
    font-size: calc(18px * var(--app-font-scale));
  }

  .meta-value {
    font-size: calc(14px * var(--app-font-scale));
  }

  .pillar-main-value {
    font-size: calc(26px * var(--app-font-scale));
  }

  .pillar-panel-compact :deep(.el-card__body) {
    padding: 10px;
  }

  .pillar-box.is-compact {
    padding: 7px 5px;
  }

  .pillar-box.is-compact .pillar-label {
    font-size: calc(11px * var(--app-font-scale));
  }

  .pillar-box.is-compact .meta-value {
    font-size: calc(12px * var(--app-font-scale));
  }

  .pillar-box.is-compact .pillar-main-value {
    font-size: calc(21px * var(--app-font-scale));
  }

  .compact-score-chip,
  .compact-ten-god-chip,
  .compact-line-label,
  .compact-line-value,
  .compact-pillar-foot {
    font-size: calc(9px * var(--app-font-scale));
  }

  .compact-pillar-char-block {
    padding: 3px 2px;
  }

  .compact-pillar-ganzhi {
    gap: 3px;
  }

  .pillar-panel-compact .relation-map-header h3,
  .pillar-panel-compact .interaction-header h3 {
    font-size: calc(13px * var(--app-font-scale));
  }


  .pillar-panel-compact .relation-map {
  }

  .pillar-panel-compact .relation-node {
  }

  .pillar-panel-compact .relation-node-stem,
  .pillar-panel-compact .relation-node-branch {
    font-size: calc(14px * var(--app-font-scale));
  }
}

.pillar-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 8px;
}

.meta-row {
  text-align: center;
  margin-bottom: 8px;
}

.meta-label {
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
}

.meta-value {
  font-size: calc(18px * var(--app-font-scale));
  font-weight: 600;
}

.meta-value-shen-sha {
  font-size: calc(10px * var(--app-font-scale));
  font-weight: 500;
  line-height: 1.5;
}

.pillar-main-row {
  text-align: center;
}

.pillar-main-label {
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
}

.pillar-main-value {
  font-size: calc(36px * var(--app-font-scale));
  font-weight: 700;
}

.wuxing-char {
  display: inline-block;
  min-width: 1.2em;
  text-align: center;
}

.hidden-sep {
  color: var(--el-text-color-secondary);
}

.wuxing-wood {
  color: var(--wx-wood);
}

.wuxing-fire {
  color: var(--wx-fire);
}

.wuxing-earth {
  color: var(--wx-earth);
}

.wuxing-metal {
  color: var(--wx-metal);
}

.wuxing-water {
  color: var(--wx-water);
}


.score-band,
.score-band-bottom {
  border-color: rgba(174, 128, 72, 0.22);
}

.score-band-label,

.score-band-value,
.relation-map-header h3,
.interaction-header h3,
.interaction-group-title {
  color: var(--bazi-primary);
  font-weight: 800;
}

.pillar-box {
  border-color: var(--bazi-border);
  border-radius: var(--bazi-radius-md);
  background: linear-gradient(180deg, rgba(255, 253, 248, 0.98), rgba(255, 248, 236, 0.94));
  box-shadow: var(--bazi-shadow-sm);
}

.pillar-box.is-preview {
  border-color: rgba(217, 164, 65, 0.65);
  background: linear-gradient(180deg, rgba(255, 247, 229, 0.98), rgba(249, 233, 191, 0.86));
}

.pillar-box.is-current-year-preview {
  border-color: var(--bazi-accent);
  box-shadow: 0 0 0 1px rgba(180, 83, 9, 0.14), 0 12px 24px rgba(180, 83, 9, 0.14);
}

.relation-map-section,
.interaction-section {
  border-radius: 18px;
}

/* ---- 關係／強弱 切換 ---- */
.view-switch {
  display: inline-flex;
  padding: 3px;
  border-radius: 999px;
  background: #f1f3f5;
  flex-shrink: 0;
}
.view-switch button {
  border: 0;
  background: transparent;
  padding: 4px 14px;
  border-radius: 999px;
  font-size: calc(13px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
  cursor: pointer;
  transition: background 0.2s, color 0.2s, box-shadow 0.2s;
}
.view-switch button.is-on {
  background: #fff;
  color: var(--el-text-color-primary);
  font-weight: 700;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
}

/* ---- 強弱模式：字的底色＝同黨／異黨 ---- */
.pillar-main-value.sv-char,
.pillar-main-value.sv-none {
  position: relative;
  min-width: 1.5em;
  padding: 1px 4px;
  border-radius: 10px;
  transition: background 0.35s, box-shadow 0.35s, opacity 0.3s, transform 0.3s;
}
.pillar-main-value.sv-same {
  background: #fff1dc;
  box-shadow: inset 0 0 0 1.5px #f0b46a;
}
.pillar-main-value.sv-opp {
  background: #eaf1f8;
  box-shadow: inset 0 0 0 1.5px #9db8d4;
}
.pillar-main-value.sv-dm {
  box-shadow: inset 0 0 0 2.5px #c2620a;
}
.pillar-main-value.sv-dm::after {
  content: '日主';
  position: absolute;
  top: -7px;
  right: -10px;
  padding: 0 4px;
  border-radius: 999px;
  background: #c2620a;
  color: #fff;
  font-size: calc(9px * var(--app-font-scale));
  font-weight: 600;
  line-height: 1.5;
}
.pillar-main-value.sv-trans {
  background-image: repeating-linear-gradient(135deg, transparent 0 5px, rgba(155, 132, 201, 0.16) 5px 8px);
}
.pillar-main-value.sv-focus {
  transform: scale(1.04);
  box-shadow: inset 0 0 0 2px #c2620a, 0 0 0 4px rgba(250, 204, 21, 0.45);
}
.pillar-main-value.sv-opp.sv-focus {
  box-shadow: inset 0 0 0 2px #3d6a96, 0 0 0 4px rgba(250, 204, 21, 0.45);
}
.pillar-main-value.sv-dim,
.pillar-main-value.sv-none {
  opacity: 0.38;
}
.pillar-card-grid.is-strength .pillar-main-value.is-related {
  background-color: rgba(250, 204, 21, 0.35);
}
.section-header.pillar-section-header {
  flex-direction: row;
  align-items: center;
}

/* ---- 通根連線 ---- */
.pillar-card-grid.is-strength {
  position: relative;
}
.sv-root-lines {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
  pointer-events: none;
  z-index: 2;
}
.sv-root-line path {
  fill: none;
  stroke: var(--root-c, #2f9e44);
  stroke-linecap: round;
  opacity: 0.85;
}
.sv-root-line circle {
  fill: var(--root-c, #2f9e44);
}
.sv-root-line.lv-本氣 path { stroke-width: 3.5; }
.sv-root-line.lv-中氣 path { stroke-width: 2.2; stroke-dasharray: 7 4; }
.sv-root-line.lv-餘氣 path { stroke-width: 1.5; stroke-dasharray: 2 4; }
/* 被沖的根：同色但變淡，終點打 ✕ */
.sv-root-line.shaken path { opacity: 0.4; }
.sv-root-x {
  fill: var(--root-c, #2f9e44);
  font-size: 13px;
  font-weight: 800;
}
.sv-root-chip {
  display: table;
  margin: 3px auto 0;
  padding: 0 7px;
  border-radius: 999px;
  font-size: calc(11px * var(--app-font-scale));
  line-height: 1.6;
  font-weight: 700;
  cursor: pointer;
  border: 1px solid var(--root-c, #2f9e44);
  background: #fff;
  color: var(--root-c, #2f9e44);
}
.sv-root-chip.is-rooted.is-on {
  background: var(--root-c, #2f9e44);
  color: #fff;
}
.sv-root-chip.is-shaken {
  border-style: dashed;
}
.sv-root-chip.is-rootless {
  border: 1px dashed #b8bec6;
  color: #7a838d;
}
.sv-root-chip.is-rootless.is-on {
  background: #f1f3f5;
  border-style: solid;
  color: #4a5560;
}
.sv-root-note {
  margin: 0 0 10px;
  padding: 8px 12px;
  border-radius: 10px;
  border-left: 4px solid var(--root-c, #2f9e44);
  background: #f7f5f0;
  color: #3a3f45;
  font-size: calc(13px * var(--app-font-scale));
  line-height: 1.6;
}
.sv-root-note :deep(.sv-warn) {
  color: var(--root-c, #c53030);
  font-weight: 700;
}
/* 根的連線從字的後面穿過，不蓋住干支 */
.pillar-card-grid.is-strength .pillar-main-value.sv-char,
.pillar-card-grid.is-strength .sv-root-chip {
  position: relative;
  z-index: 3;
}

/* ---- 大運柱：點兩下換大運 ---- */
.pillar-cell {
  position: relative;
}
.pillar-box.is-luck-switchable {
  cursor: pointer;
  user-select: none;
  -webkit-user-select: none;
}
.luck-switch-mark {
  margin-left: 4px;
  font-size: 0.8em;
  color: #b7791f;
}
.pillar-card-grid {
  position: relative;
}
.luck-picker {
  position: absolute;
  top: 30px;
  left: 0;
  right: 0;
  z-index: 30;
  border: 1px solid #e3c79a;
  border-radius: 12px;
  background: #fffdf8;
  box-shadow: 0 12px 28px rgba(80, 50, 10, 0.22);
}
.luck-picker-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 8px 10px 4px 12px;
  font-weight: 700;
  font-size: calc(13px * var(--app-font-scale));
  color: #7a4a12;
}
.luck-picker-tip {
  flex: 1;
  font-weight: 400;
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
}
.luck-picker-close {
  border: 0;
  background: transparent;
  color: #9a8a74;
  font-size: 14px;
  cursor: pointer;
  padding: 2px 4px;
}
.luck-picker-track {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  padding: 10px 10px 10px;
  scrollbar-width: thin;
}
.luck-picker-cell {
  position: relative;
  flex: 1 0 calc(50px * var(--app-font-scale));
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0;
  padding: 8px 4px 6px;
  border: 1px solid rgba(188, 195, 211, 0.8);
  border-radius: 10px;
  background: #fff;
  cursor: pointer;
}
.luck-picker-cell:hover {
  background: #fbf3e4;
}
.luck-picker-cell.is-on {
  border: 2px solid #d97706;
  background: #fff4e0;
}
.luck-picker-char {
  font-size: calc(22px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.2;
}
.luck-picker-god {
  min-height: 1.3em;
  font-size: calc(10px * var(--app-font-scale));
  line-height: 1.3;
  color: #7a6a55;
  white-space: nowrap;
}
.luck-picker-age {
  margin-top: 2px;
  font-size: calc(11px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
  white-space: nowrap;
}
.luck-picker-now {
  position: absolute;
  top: -8px;
  left: 50%;
  transform: translateX(-50%);
  padding: 0 6px;
  border-radius: 999px;
  background: #d97706;
  color: #fff;
  font-size: calc(10px * var(--app-font-scale));
  line-height: 1.6;
  white-space: nowrap;
}
</style>
