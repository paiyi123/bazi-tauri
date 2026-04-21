<template>
  <el-card shadow="hover" :class="{ 'pillar-panel-compact': compact }">
    <template #header>
      <div class="section-header">
        <h2>{{ panelTitle }}</h2>
      </div>
    </template>

    <el-row v-if="showPillars" :gutter="compact ? 10 : 16" class="pillar-card-grid">
      <el-col
        v-for="item in pillarItems"
        :key="item.key"
        :xs="compact ? 6 : 24"
        :sm="12"
        :md="compact ? 6 : 6"
        :xl="columnSpanXl"
      >
        <div
          :class="[
            'pillar-box',
            { 'is-preview': item.isPreview, 'is-current-year-preview': item.isCurrentYear, 'is-compact': compact }
          ]"
        >
          <template v-if="compact">
            <div class="compact-pillar-header">
              <p class="pillar-label">{{ item.label }}</p>
              <span class="compact-score-chip">{{ item.topScoreValue }}</span>
            </div>
            <div :class="['compact-ten-god-chip', tenGodClass(item.stemTenGod)]">{{ item.stemTenGod }}</div>
            <div class="compact-pillar-ganzhi">
              <div class="compact-pillar-char-block">
                <div class="pillar-main-label">天干</div>
                <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.stemElement}`]">{{ item.value.stem }}</div>
              </div>
              <div class="compact-pillar-char-block">
                <div class="pillar-main-label">地支</div>
                <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.branchElement}`]">{{ item.value.branch }}</div>
              </div>
            </div>
            <div class="compact-pillar-line">
              <span class="compact-line-label">藏干</span>
              <span class="compact-line-value compact-hidden-stems">
                <template v-for="(hiddenStem, idx) in item.hiddenStems" :key="`${hiddenStem}-${idx}`">
                  <span :class="['wuxing-char', `wuxing-${stemElement(hiddenStem)}`]">{{ hiddenStem }}</span>
                  <span v-if="idx < item.hiddenStems.length - 1" class="hidden-sep">、</span>
                </template>
              </span>
            </div>
            <div class="compact-pillar-line">
              <span class="compact-line-label">支神</span>
              <span class="compact-line-value">{{ item.branchTenGods.join('、') }}</span>
            </div>
            <div class="compact-score-row compact-score-row-bottom">
              <span class="compact-score-label">{{ item.bottomScoreLabel }}</span>
              <span class="compact-score-value">{{ item.bottomScoreValue }}</span>
            </div>
            <div class="compact-pillar-foot">
              <span>{{ item.value.naYin }}</span>
              <span>{{ item.value.xunKong }}</span>
            </div>
          </template>
          <template v-else>
            <div class="score-band score-band-top">
              <span class="score-band-label">{{ item.topScoreLabel }}</span>
              <span class="score-band-value">{{ item.topScoreValue }}</span>
            </div>
            <p class="pillar-label">{{ item.label }}</p>
            <div class="meta-row">
              <div class="meta-label">天干十神</div>
              <div class="meta-value">{{ item.stemTenGod }}</div>
            </div>
            <div class="pillar-main">
              <div class="pillar-main-row">
                <div class="pillar-main-label">天干</div>
                <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.stemElement}`]">{{ item.value.stem }}</div>
              </div>
              <div class="pillar-main-row">
                <div class="pillar-main-label">地支</div>
                <div :class="['pillar-main-value', 'wuxing-char', `wuxing-${item.branchElement}`]">{{ item.value.branch }}</div>
              </div>
            </div>
            <div class="meta-row">
              <div class="meta-label">地支藏干</div>
              <div class="meta-value">
                <template v-for="(hiddenStem, idx) in item.hiddenStems" :key="`${hiddenStem}-${idx}`">
                  <span :class="['wuxing-char', `wuxing-${stemElement(hiddenStem)}`]">{{ hiddenStem }}</span>
                  <span v-if="idx < item.hiddenStems.length - 1" class="hidden-sep">、</span>
                </template>
              </div>
            </div>
            <div class="meta-row">
              <div class="meta-label">地支十神</div>
              <div class="meta-value">{{ item.branchTenGods.join('、') }}</div>
            </div>
            <el-descriptions :column="1" size="small">
              <el-descriptions-item label="納音">{{ item.value.naYin }}</el-descriptions-item>
              <el-descriptions-item label="五行">{{ item.value.wuXing }}</el-descriptions-item>
              <el-descriptions-item label="地勢">{{ item.value.diShi }}</el-descriptions-item>
              <el-descriptions-item label="旬 / 空亡">{{ item.value.xun }} / {{ item.value.xunKong }}</el-descriptions-item>
            </el-descriptions>
            <div class="score-band score-band-bottom">
              <span class="score-band-label">{{ item.bottomScoreLabel }}</span>
              <span class="score-band-value">{{ item.bottomScoreValue }}</span>
            </div>
          </template>
        </div>
      </el-col>
    </el-row>

    <div v-if="showRelationMap" class="relation-map-section">
      <div class="relation-map-header">
        <h3>柱位關係圖</h3>
        <div class="relation-legend">
          <span
            v-for="item in relationLegend"
            :key="item.type"
            class="relation-legend-item"
            :style="{ '--relation-color': item.color }"
          >
            {{ item.type }}
          </span>
        </div>
      </div>
      <div class="relation-map">
        <svg viewBox="0 0 100 100" preserveAspectRatio="none" class="relation-map-svg">
          <template v-for="link in relationLinks" :key="link.key">
            <path
              v-if="link.kind === 'path'"
              :class="['relation-map-line', `relation-map-line-${link.type}`]"
              :d="link.path"
              :stroke="link.color"
              fill="none"
              :stroke-width="link.strokeWidth"
              stroke-linecap="round"
              stroke-linejoin="round"
              :stroke-dasharray="link.dasharray"
              :opacity="link.opacity"
            />
            <path
              v-else
              :class="['relation-map-line', `relation-map-line-${link.type}`]"
              :d="link.path"
              :stroke="link.color"
              fill="none"
              :stroke-width="link.strokeWidth"
              stroke-linecap="round"
              stroke-linejoin="round"
              :stroke-dasharray="link.dasharray"
              :opacity="link.opacity"
            />
            <text
              :x="link.labelX"
              :y="link.labelY"
              :fill="link.color"
              :class="['relation-map-label', `relation-map-label-${link.type}`]"
              text-anchor="middle"
            >
              {{ link.label }}
            </text>
          </template>
        </svg>

        <div
          v-for="node in diagramNodes"
          :key="node.key"
          :class="[
            'relation-node',
            { 'is-preview': node.isPreview, 'is-current-year': node.isCurrentYear }
          ]"
          :style="{ left: `${node.x}%`, top: `${node.y}%` }"
        >
          <div class="relation-node-label">{{ node.label }}</div>
          <div :class="['relation-node-ten-god', tenGodClass(node.stemTenGod)]">{{ node.stemTenGod }}</div>
          <div class="relation-node-pillar">
            <div :class="['relation-node-stem', 'wuxing-char', `wuxing-${stemElement(node.stem)}`]">{{ node.stem }}</div>
            <div :class="['relation-node-branch', 'wuxing-char', `wuxing-${branchElement(node.branch)}`]">{{ node.branch }}</div>
          </div>
          <div
            :class="['relation-node-hidden-stems', { 'is-empty': !node.hiddenStemPairs.length }]"
          >
            <span
              v-for="(pair, index) in node.hiddenStemPairs"
              :key="`${node.key}-${pair.stem}-${pair.tenGod}-${index}`"
              class="relation-node-hidden-item"
            >
              <span :class="['relation-node-hidden-stem', 'wuxing-char', `wuxing-${stemElement(pair.stem)}`]">
                {{ pair.stem }}
              </span>
              <span :class="['relation-node-hidden-ten-god', tenGodClass(pair.tenGod)]">
                {{ pair.tenGod }}
              </span>
            </span>
          </div>
        </div>
      </div>
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
import { computed } from 'vue'
import type { BaziResponse, LuckPreviewPillar, Pillar, QuantModelInteraction } from '../types/bazi'

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

type WuXing = 'wood' | 'fire' | 'earth' | 'metal' | 'water'
type DisplayPillarItem = {
  key: string
  label: string
  value: Pillar
  stemTenGod: string
  hiddenStems: string[]
  branchTenGods: string[]
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
  x: number
  y: number
  isPreview: boolean
  isCurrentYear: boolean
}
type RelationLink = {
  key: string
  kind: 'path' | 'multi'
  scope: InteractionRow['scope']
  type: string
  label: string
  color: string
  path: string
  labelX: number
  labelY: number
  strokeWidth: number
  opacity: number
  dasharray?: string
}
type InteractionSection = {
  title: string
  rows: InteractionRow[]
}

const STEM_WUXING: Record<string, WuXing> = {
  甲: 'wood',
  乙: 'wood',
  丙: 'fire',
  丁: 'fire',
  戊: 'earth',
  己: 'earth',
  庚: 'metal',
  辛: 'metal',
  壬: 'water',
  癸: 'water'
}

const BRANCH_WUXING: Record<string, WuXing> = {
  子: 'water',
  丑: 'earth',
  寅: 'wood',
  卯: 'wood',
  辰: 'earth',
  巳: 'fire',
  午: 'fire',
  未: 'earth',
  申: 'metal',
  酉: 'metal',
  戌: 'earth',
  亥: 'water'
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
  { pair: '寅巳', detail: '寅巳申三刑(無恩之刑)' },
  { pair: '巳申', detail: '寅巳申三刑(無恩之刑)' },
  { pair: '寅申', detail: '寅巳申三刑(無恩之刑)' },
  { pair: '丑戌', detail: '丑戌未三刑(恃勢之刑)' },
  { pair: '戌未', detail: '丑戌未三刑(恃勢之刑)' },
  { pair: '丑未', detail: '丑戌未三刑(恃勢之刑)' }
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
  破: { color: '#718096', strokeWidth: 0.76, opacity: 0.84, dasharray: '1.2 2.4' }
}

function stemElement(stem: string): WuXing {
  return STEM_WUXING[stem] ?? 'earth'
}

function branchElement(branch: string): WuXing {
  return BRANCH_WUXING[branch] ?? 'earth'
}

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
    破: 8
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
    stemElement: stemElement(props.result.dayPillar.stem),
    branchElement: branchElement(props.result.dayPillar.branch),
    topScoreLabel: '日元：',
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

const columnSpanXl = computed(() => (previewPillarItems.value.length ? 4 : 6))

const relationLegend = computed(() =>
  ['合', '祿', '半合', '三合', '三會', '沖', '刑', '破'].map((type) => ({
    type,
    color: RELATION_TYPE_STYLES[type]?.color || '#4a5568'
  }))
)

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
  const natalRows = backendNatalInteractionRows.value.length
    ? backendNatalInteractionRows.value
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

function resolveDiagramNodeX(index: number, total: number) {
  if (total <= 1) {
    return 50
  }
  const padding = total <= 4 ? 12 : 8
  const span = 100 - padding * 2
  return padding + (index * span) / (total - 1)
}

const diagramNodes = computed<DiagramNode[]>(() =>
  pillarItems.value.map((item, index, source) => ({
    key: item.key,
    label: item.label,
    stem: item.value.stem,
    branch: item.value.branch,
    stemTenGod: item.stemTenGod,
    hiddenStemPairs: item.hiddenStems.map((stem, pairIndex) => ({
      stem,
      tenGod: item.branchTenGods[pairIndex] || '-'
    })),
    x: resolveDiagramNodeX(index, source.length),
    y: 48,
    isPreview: item.isPreview,
    isCurrentYear: item.isCurrentYear
  }))
)

function buildPairRelationPath(
  left: DiagramNode,
  right: DiagramNode,
  layer: number,
  scope: InteractionRow['scope'],
  type: string
) {
  const start = left.x < right.x ? left : right
  const end = left.x < right.x ? right : left
  const distance = Math.abs(end.x - start.x)
  const sameNode = Math.abs(end.x - start.x) < 0.01
  const spanSteps = Math.max(1, Math.round(distance / 18))

  if (scope === '祿') {
    const controlX = sameNode ? start.x + 6 + layer * 3 : (start.x + end.x) / 2
    const controlY = Math.max(28, 34 - layer * 3)
    return {
      path: `M ${left.x} ${left.y} Q ${controlX} ${controlY} ${right.x} ${right.y}`,
      labelX: sameNode ? controlX + 1.5 : controlX,
      labelY: controlY - 2
    }
  }

  const controlX = (start.x + end.x) / 2
  const isStemScope = scope === '天干'
  const horizontalOffset = layer === 0
    ? 0
    : (layer % 2 === 0 ? -1 : 1) * Math.ceil(layer / 2) * (sameNode ? 1.4 : 0.9)

  if (type === '刑') {
    const bendY = isStemScope
      ? Math.max(3.8, 12.5 - distance * 0.2 - layer * 4.2)
      : Math.min(98, 92.5 + distance * 0.16 + layer * 4)
    const kink = Math.max(3.2, distance * 0.14)
    return {
      path: `M ${start.x} ${start.y} Q ${start.x + kink} ${bendY} ${controlX} ${bendY} T ${end.x} ${end.y}`,
      labelX: controlX + horizontalOffset,
      labelY: isStemScope ? bendY - 1.4 : bendY + 2.8
    }
  }

  const controlY = isStemScope
    ? Math.max(2.4, 12.8 - distance * 0.32 - layer * 4.8)
    : Math.min(99, 92.8 + distance * 0.22 + layer * 4.5)
  const topLabelOffset = 0.9 + Math.max(0, spanSteps - 1) * 0.55
  const bottomLabelOffset = 2.3 + Math.max(0, spanSteps - 1) * 0.7
  return {
    path: `M ${start.x} ${start.y} Q ${controlX} ${controlY} ${end.x} ${end.y}`,
    labelX: controlX + horizontalOffset,
    labelY: isStemScope
      ? controlY - (topLabelOffset * 0.32) - Math.ceil(layer / 2) * 0.08
      : controlY + (bottomLabelOffset * 0.28) + Math.ceil(layer / 2) * 0.12
  }
}

function buildMultiRelationPath(nodes: DiagramNode[], layer: number, scope: InteractionRow['scope']) {
  const sorted = [...nodes].sort((a, b) => a.x - b.x)
  const jointX = sorted.reduce((sum, item) => sum + item.x, 0) / sorted.length
  const isStemScope = scope === '天干'
  const jointY = isStemScope ? 4.5 + layer * 4.8 : 95.5 + layer * 3.8
  const labelX = jointX + (layer === 0 ? 0 : (layer % 2 === 0 ? -0.9 : 0.9))
  const segments = sorted
    .map((node) => `M ${jointX} ${jointY} L ${node.x} ${node.y}`)
    .join(' ')
  return {
    path: segments,
    labelX,
    labelY: isStemScope ? jointY - 1.2 : jointY + 2.2
  }
}

function resolveRelationLabel(row: InteractionRow) {
  if (!['合', '半合', '三合', '三會'].includes(row.type)) {
    return row.type
  }
  const element = row.detail.match(/對應\s*([木火土金水])/)?.[1]
  return element ? `${row.type}(${element})` : row.type
}

function clampLabelPosition(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value))
}

function spreadRelationLabels(links: RelationLink[]) {
  const topLinks = links
    .filter((link) => link.labelY < 50)
    .sort((a, b) => a.labelY - b.labelY || a.labelX - b.labelX)
  const bottomLinks = links
    .filter((link) => link.labelY >= 50)
    .sort((a, b) => a.labelY - b.labelY || a.labelX - b.labelX)

  const adjustGroup = (group: RelationLink[], area: 'top' | 'bottom') => {
    const placed: Array<{ labelX: number; labelY: number }> = []
    // Keep labels inside the map by pushing them toward the center band.
    const yDirection = area === 'top' ? 1 : -1
    const yMin = area === 'top' ? 4 : 54
    const yMax = area === 'top' ? 44 : 98
    const xMin = 12
    const xMax = 88
    const xCollision = 12.8
    const yCollision = 5.8
    const collides = (x: number, y: number) =>
      placed.some((item) => Math.abs(item.labelX - x) < xCollision && Math.abs(item.labelY - y) < yCollision)

    for (const link of group) {
      const baseX = link.labelX
      const baseY = link.labelY
      let nextX = baseX
      let nextY = baseY
      const yStep = link.kind === 'path' ? 1.05 : 1.45
      const xStep = link.kind === 'path' ? 3.1 : 3.8

      if (collides(nextX, nextY)) {
        let found = false

        for (let wave = 1; wave <= 8 && !found; wave += 1) {
          const candidateY = clampLabelPosition(baseY + yDirection * wave * yStep, yMin, yMax)
          const candidateXs = [
            baseX,
            baseX + xStep,
            baseX - xStep,
            baseX + wave * xStep,
            baseX - wave * xStep
          ]

          for (const candidateX of candidateXs) {
            const clampedX = clampLabelPosition(candidateX, xMin, xMax)
            if (!collides(clampedX, candidateY)) {
              nextX = clampedX
              nextY = candidateY
              found = true
              break
            }
          }
        }
      }

      link.labelX = clampLabelPosition(nextX, xMin, xMax)
      link.labelY = clampLabelPosition(nextY, yMin, yMax)
      placed.push({ labelX: link.labelX, labelY: link.labelY })
    }
  }

  adjustGroup(topLinks, 'top')
  adjustGroup(bottomLinks, 'bottom')

  return links
}

const relationLinks = computed<RelationLink[]>(() => {
  const baseNodeMap = new Map(diagramNodes.value.map((node) => [node.key, node]))
  const duplicateCount = new Map<string, number>()

  const rawLinks = interactionRows.value.flatMap((row, rowIndex) => {
    const nodes = row.participantKeys
      .map((key) => baseNodeMap.get(key))
      .filter((value): value is DiagramNode => !!value)
      .map((node) => ({
        ...node,
        y: row.scope === '天干' ? 26 : row.scope === '地支' ? 56 : 0
      }))
      .map((node, index) => ({
        ...node,
        y: row.scope === '祿'
          ? index === 0
            ? 40
            : 56
          : node.y
      }))

    if (nodes.length < 2) {
      return []
    }

    const groupKey = `${row.scope}:${[...row.participantKeys].sort().join('-')}`
    const layer = duplicateCount.get(groupKey) ?? 0
    duplicateCount.set(groupKey, layer + 1)
    const visualStyle = RELATION_TYPE_STYLES[row.type] || {
      color: '#4a5568',
      strokeWidth: 0.78,
      opacity: 0.88,
      dasharray: undefined
    }

    const base = {
      key: `${row.type}-${row.target}-${rowIndex}`,
      kind: (nodes.length === 2 ? 'path' : 'multi') as 'path' | 'multi',
      scope: row.scope,
      type: row.type,
      label: resolveRelationLabel(row),
      color: visualStyle.color,
      dasharray: visualStyle.dasharray,
      strokeWidth: visualStyle.strokeWidth,
      opacity: visualStyle.opacity
    }

    if (nodes.length === 2) {
      const line = buildPairRelationPath(nodes[0], nodes[1], layer, row.scope, row.type)
      return [{ ...base, ...line }]
    }

    const multi = buildMultiRelationPath(nodes, layer, row.scope)
    return [{ ...base, ...multi }]
  })

  return spreadRelationLabels(rawLinks)
})
</script>

<style scoped>
.score-band {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: 14px;
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
  margin-top: 0;
}

.relation-map-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 10px;
}

.relation-map-header h3 {
  margin: 0;
  font-size: 16px;
}

.relation-legend {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
}

.relation-legend-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.relation-legend-item::before {
  content: '';
  width: 18px;
  height: 2px;
  border-radius: 999px;
  background: var(--relation-color);
}

.relation-map {
  position: relative;
  min-height: 360px;
  border: 1px solid rgba(188, 195, 211, 0.45);
  border-radius: 18px;
  background:
    radial-gradient(circle at top, rgba(255, 251, 235, 0.8), rgba(255, 255, 255, 0) 55%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.96));
  overflow: hidden;
}

.relation-map-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

.relation-map-line {
  transition: stroke 180ms ease, opacity 180ms ease, stroke-width 180ms ease;
}

.relation-map-label {
  font-size: 2.8px;
  font-weight: 700;
  paint-order: stroke;
  stroke: rgba(255, 255, 255, 0.96);
  stroke-width: 1.5px;
  stroke-linejoin: round;
  letter-spacing: 0.04em;
}

.relation-map-label-祿 {
  font-size: 2.6px;
}

.relation-node {
  position: absolute;
  transform: translate(-50%, -50%);
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 112px;
  max-width: 148px;
  height: 214px;
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
  font-size: 12px;
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
  font-size: 11px;
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
  font-size: 28px;
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
  min-height: 66px;
  max-height: 66px;
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
  font-size: 16px;
  font-weight: 700;
  line-height: 1;
}

.relation-node-hidden-ten-god {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 18px;
  padding: 1px 6px;
  border-radius: 999px;
  font-size: 10px;
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
  font-size: 16px;
}

.interaction-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.interaction-hint {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.interaction-group.with-divider {
  margin-top: 18px;
  padding-top: 18px;
  border-top: 1px solid rgba(188, 195, 211, 0.45);
}

.interaction-group-title {
  margin-bottom: 10px;
  font-size: 13px;
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
  align-items: center;
  justify-content: space-between;
  gap: 4px;
  margin-bottom: 4px;
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
  font-size: 9px;
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
  font-size: 9px;
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
  gap: 6px;
  margin-top: 2px;
  padding-top: 5px;
  border-top: 1px solid rgba(188, 195, 211, 0.35);
}

.compact-score-label {
  color: var(--el-text-color-secondary);
  font-size: 9px;
  line-height: 1.2;
}

.compact-score-value {
  color: #0f172a;
  font-size: 10px;
  font-weight: 700;
  line-height: 1.1;
  white-space: nowrap;
}

.compact-line-label {
  color: var(--el-text-color-secondary);
  font-size: 9px;
  line-height: 1.2;
}

.compact-line-value {
  font-size: 10px;
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
  font-size: 9px;
  line-height: 1.2;
}

.pillar-panel-compact .relation-map-section {
  margin-top: 0;
}

.pillar-panel-compact .relation-map-header h3,
.pillar-panel-compact .interaction-header h3 {
  font-size: 14px;
}

.pillar-panel-compact .relation-legend-item,
.pillar-panel-compact .interaction-count,
.pillar-panel-compact .interaction-hint,
.pillar-panel-compact .interaction-group-title {
  font-size: 11px;
}

.pillar-panel-compact .relation-node-label {
  font-size: 10px;
}

.pillar-panel-compact .relation-node-ten-god,
.pillar-panel-compact .relation-node-hidden-ten-god {
  font-size: 9px;
}

.pillar-panel-compact .relation-node-hidden-stem {
  font-size: 12px;
}

.pillar-panel-compact .relation-map {
  min-height: 320px;
}

.pillar-panel-compact .relation-node {
  min-width: 84px;
  max-width: 108px;
  height: 156px;
  padding: 6px 6px 8px;
}

.pillar-panel-compact .relation-node-stem,
.pillar-panel-compact .relation-node-branch {
  font-size: 18px;
}

.pillar-panel-compact .relation-node-hidden-stems {
  min-height: 52px;
  max-height: 52px;
  gap: 4px;
}

.pillar-panel-compact :deep(.el-table .cell) {
  font-size: 11px;
  line-height: 1.35;
}

.pillar-box.is-compact .pillar-label,
.pillar-box.is-compact .meta-value,
.pillar-box.is-compact .pillar-main-value {
  word-break: break-word;
}

.pillar-box.is-compact .pillar-label {
  margin-bottom: 8px;
  font-size: 13px;
}

.pillar-box.is-compact .meta-row {
  margin-bottom: 6px;
}

.pillar-box.is-compact .meta-label,
.pillar-box.is-compact .pillar-main-label {
  font-size: 10px;
}

.pillar-box.is-compact .meta-value {
  font-size: 13px;
}

.pillar-box.is-compact .pillar-main {
  gap: 3px;
  margin-bottom: 6px;
}

.pillar-box.is-compact .pillar-main-value {
  font-size: 20px;
}

@media (max-width: 767px) {
  .relation-map-header {
    flex-direction: column;
  }

  .relation-legend {
    justify-content: flex-start;
  }

  .relation-map {
    min-height: 420px;
  }

  .relation-node {
    min-width: 92px;
    max-width: 116px;
    height: 172px;
    padding: 6px 6px 8px;
  }

  .relation-node-stem,
  .relation-node-branch {
    font-size: 20px;
  }

  .relation-node-hidden-stems {
    min-height: 54px;
    max-height: 54px;
    gap: 4px;
  }

  .relation-node-hidden-stem {
    font-size: 13px;
  }

  .relation-node-hidden-ten-god {
    padding: 1px 5px;
    font-size: 9px;
  }

  .meta-value {
    font-size: 15px;
  }

  .pillar-main-value {
    font-size: 22px;
  }

  .pillar-panel-compact :deep(.el-row) {
    row-gap: 8px;
  }

  .pillar-box.is-compact .compact-line-value {
    font-size: 10px;
  }

  .pillar-panel-compact .relation-map {
    min-height: 286px;
  }

  .pillar-panel-compact .relation-node {
    min-width: 76px;
    max-width: 96px;
    height: 142px;
  }

  .pillar-panel-compact .relation-node-stem,
  .pillar-panel-compact .relation-node-branch {
    font-size: 16px;
  }
}

@media (max-width: 480px) {
  .relation-map {
    min-height: 360px;
  }

  .relation-node {
    min-width: 84px;
    max-width: 104px;
    height: 156px;
  }

  .relation-node-stem,
  .relation-node-branch {
    font-size: 18px;
  }

  .meta-value {
    font-size: 14px;
  }

  .pillar-main-value {
    font-size: 20px;
  }

  .pillar-panel-compact :deep(.el-card__body) {
    padding: 10px;
  }

  .pillar-box.is-compact {
    padding: 7px 5px;
  }

  .pillar-box.is-compact .pillar-label {
    font-size: 11px;
  }

  .pillar-box.is-compact .meta-value {
    font-size: 12px;
  }

  .pillar-box.is-compact .pillar-main-value {
    font-size: 16px;
  }

  .compact-score-chip,
  .compact-ten-god-chip,
  .compact-line-label,
  .compact-line-value,
  .compact-pillar-foot {
    font-size: 9px;
  }

  .compact-pillar-char-block {
    padding: 3px 2px;
  }

  .compact-pillar-ganzhi {
    gap: 3px;
  }

  .pillar-panel-compact .relation-map-header h3,
  .pillar-panel-compact .interaction-header h3 {
    font-size: 13px;
  }

  .pillar-panel-compact .relation-legend-item,
  .pillar-panel-compact .interaction-count,
  .pillar-panel-compact .interaction-hint,
  .pillar-panel-compact .interaction-group-title,
  .pillar-panel-compact :deep(.el-table .cell) {
    font-size: 10px;
  }

  .pillar-panel-compact .relation-map {
    min-height: 260px;
  }

  .pillar-panel-compact .relation-node {
    min-width: 70px;
    max-width: 88px;
    height: 132px;
  }

  .pillar-panel-compact .relation-node-stem,
  .pillar-panel-compact .relation-node-branch {
    font-size: 14px;
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
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.meta-value {
  font-size: 18px;
  font-weight: 600;
}

.pillar-main-row {
  text-align: center;
}

.pillar-main-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.pillar-main-value {
  font-size: 28px;
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
  color: #2e7d32;
}

.wuxing-fire {
  color: #c62828;
}

.wuxing-earth {
  color: #8d6e63;
}

.wuxing-metal {
  color: #546e7a;
}

.wuxing-water {
  color: #1565c0;
}
</style>
