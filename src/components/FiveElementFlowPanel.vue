<template>
  <el-card shadow="hover" class="flow-panel" :class="{ 'flow-panel-compact': compact }">
    <template #header>
      <div class="section-header">
        <h2>五行流通圖</h2>
        <span class="flow-subtitle">本局四柱天干地支</span>
      </div>
    </template>

    <div ref="flowScrollRef" class="flow-scroll" :style="flowScrollStyle">
      <div class="flow-board" :style="flowBoardStyle">
        <div
          v-for="node in stemNodes"
          :key="node.key"
          class="flow-node flow-node-stem"
          :class="`flow-node-${node.element}`"
          :style="{ left: `${node.x}px`, top: `${node.y}px` }"
        >
          <div class="flow-node-ten-god">{{ node.tenGod }}</div>
          <div class="flow-node-char">{{ node.char }}</div>
        </div>

        <div
          v-for="node in branchNodes"
          :key="node.key"
          class="flow-node flow-node-branch"
          :class="`flow-node-${node.element}`"
          :style="{ left: `${node.x}px`, top: `${node.y}px` }"
        >
          <div class="flow-node-char">{{ node.char }}</div>
          <div class="flow-node-ten-god flow-node-ten-god-bottom">{{ node.tenGod }}</div>
        </div>

        <template v-for="link in pngLinks" :key="link.key">
          <img
            v-if="link.image"
            class="flow-link-img"
            :class="[`flow-link-img-${link.orientation}`, { 'flow-link-img-bad': link.tone === 'bad' }]"
            :src="link.image"
            alt=""
            draggable="false"
            :style="{ left: `${link.x}px`, top: `${link.y}px` }"
          />
        </template>

        <div
          v-for="badge in floatingBadges"
          :key="badge.key"
          class="flow-floating-badge flow-floating-badge-bad"
          :style="{ left: `${badge.x}px`, top: `${badge.y}px` }"
        >
          {{ badge.label }}
        </div>

        <div
          v-for="(item, index) in bottomFlags"
          :key="`${index}-${item || 'empty'}`"
          class="flow-bottom-flag"
          :class="{ 'flow-bottom-flag-empty': !item }"
          :style="{ left: `${pillarXs[index]}px` }"
        >
          {{ item || "占位" }}
        </div>
      </div>
    </div>

    <div class="flow-legend">
      <div class="legend-row">
        <span class="legend-icon legend-icon-good">✓</span>
        <span>流通：三合、六合、相生、相助</span>
      </div>
      <div class="legend-row">
        <span class="legend-icon legend-icon-bad">×</span>
        <span>阻塞：相沖、相刑、相克</span>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { BaziResponse, Pillar, QuantModelInteraction } from "../types/bazi";
import ltChImg from "../assets/images/ganzhi/lt_ch.png";
import ltCsImg from "../assets/images/ganzhi/lt_cs.png";
import ltHImg from "../assets/images/ganzhi/lt_h.png";
import ltKhImg from "../assets/images/ganzhi/lt_kh.png";
import ltKsImg from "../assets/images/ganzhi/lt_ks.png";
import ltSsImg from "../assets/images/ganzhi/lt_ss.png";
import ltSxImg from "../assets/images/ganzhi/lt_sx.png";
import ltSyImg from "../assets/images/ganzhi/lt_sy.png";
import ltSzImg from "../assets/images/ganzhi/lt_sz.png";
import ltZhImg from "../assets/images/ganzhi/lt_zh.png";
import ltZsImg from "../assets/images/ganzhi/lt_zs.png";

type ElementKey = "wood" | "fire" | "earth" | "metal" | "water";
type NodeKind = "stem" | "branch";
type FlowNode = {
  key: string;
  char: string;
  tenGod: string;
  element: ElementKey;
  kind: NodeKind;
  x: number;
  y: number;
};
type PngLink = {
  key: string;
  image: string;
  tone: "good" | "bad";
  orientation: "horizontal" | "vertical";
  x: number;
  y: number;
};
type FloatingBadge = {
  key: string;
  label: string;
  x: number;
  y: number;
};
type FlowRelation = {
  label: string;
  tone: "good" | "bad";
  direction?: "forward" | "reverse";
};

const props = withDefaults(
  defineProps<{
    result: BaziResponse;
    compact?: boolean;
  }>(),
  {
    compact: false,
  },
);

const pillarXs = [112, 330, 548, 766];
const stemY = 104;
const branchY = 306;
const boardWidth = 878;
const boardHeight = 440;
const horizontalLinkWidth = 146;
const horizontalLinkHeight = 35;
const verticalLinkHeight = branchY - stemY;
const verticalLinkWidth = 42;

const flowScrollRef = ref<HTMLElement | null>(null);
const flowScale = ref(1);
let flowResizeObserver: ResizeObserver | null = null;

const flowScrollStyle = computed(() => {
  if (!props.compact) {
    return undefined;
  }
  return {
    height: `${boardHeight * flowScale.value}px`,
  };
});

const flowBoardStyle = computed(() => {
  if (!props.compact) {
    return undefined;
  }
  return {
    transform: `scale(${flowScale.value})`,
  };
});

function updateFlowScale() {
  if (!props.compact) {
    flowScale.value = 1;
    return;
  }
  const width = flowScrollRef.value?.clientWidth || boardWidth;
  flowScale.value = Math.min(1, width / boardWidth);
}

onMounted(() => {
  flowResizeObserver = new ResizeObserver(updateFlowScale);
  if (flowScrollRef.value) {
    flowResizeObserver.observe(flowScrollRef.value);
  }
  updateFlowScale();
});

onBeforeUnmount(() => {
  flowResizeObserver?.disconnect();
  flowResizeObserver = null;
});

watch(
  () => props.compact,
  async () => {
    await nextTick();
    updateFlowScale();
  },
);

const STEM_ELEMENTS: Record<string, ElementKey> = {
  甲: "wood",
  乙: "wood",
  丙: "fire",
  丁: "fire",
  戊: "earth",
  己: "earth",
  庚: "metal",
  辛: "metal",
  壬: "water",
  癸: "water",
};

const BRANCH_ELEMENTS: Record<string, ElementKey> = {
  子: "water",
  丑: "earth",
  寅: "wood",
  卯: "wood",
  辰: "earth",
  巳: "fire",
  午: "fire",
  未: "earth",
  申: "metal",
  酉: "metal",
  戌: "earth",
  亥: "water",
};

const GENERATES: Record<ElementKey, ElementKey> = {
  wood: "fire",
  fire: "earth",
  earth: "metal",
  metal: "water",
  water: "wood",
};

const CONTROLS: Record<ElementKey, ElementKey> = {
  wood: "earth",
  earth: "water",
  water: "fire",
  fire: "metal",
  metal: "wood",
};

const STEM_COMBINATIONS = new Set(["甲己", "乙庚", "丙辛", "丁壬", "戊癸"]);
const BRANCH_SIX_COMBINATIONS = new Set(["子丑", "寅亥", "卯戌", "辰酉", "巳申", "午未"]);
const BRANCH_HALF_COMBINATIONS = new Set(["寅午", "午戌", "亥卯", "卯未", "申子", "子辰", "巳酉", "酉丑"]);

const pillars = computed(() => [
  props.result.hourPillar,
  props.result.dayPillar,
  props.result.monthPillar,
  props.result.yearPillar,
]);

const stemTenGods = computed(() => [
  props.result.hourStemTenGod,
  props.result.dayStemTenGod || "日主",
  props.result.monthStemTenGod,
  props.result.yearStemTenGod,
]);

const branchTenGods = computed(() => [
  props.result.hourBranchTenGods?.[0] || "",
  props.result.dayBranchTenGods?.[0] || "",
  props.result.monthBranchTenGods?.[0] || "",
  props.result.yearBranchTenGods?.[0] || "",
]);

const stemNodes = computed(() =>
  pillars.value.map((pillar, index) =>
    makeNode(`stem-${index}`, pillar, stemTenGods.value[index], "stem", pillarXs[index], stemY),
  ),
);

const branchNodes = computed(() =>
  pillars.value.map((pillar, index) =>
    makeNode(`branch-${index}`, pillar, branchTenGods.value[index], "branch", pillarXs[index], branchY),
  ),
);

const pngLinks = computed(() => {
  const links: PngLink[] = [];

  for (let index = 0; index < stemNodes.value.length - 1; index += 1) {
    const from = stemNodes.value[index];
    const to = stemNodes.value[index + 1];
    const relation = stemRelation(from, to);
    if (relation) {
      links.push(horizontalLink(`stem-${index}`, index, relation, from.x < to.x));
    }
  }

  for (let index = 0; index < branchNodes.value.length - 1; index += 1) {
    const from = branchNodes.value[index];
    const to = branchNodes.value[index + 1];
    const relation = branchRelation(from, to);
    if (relation && relation.tone === "good") {
      links.push(horizontalLink(`branch-${index}`, index, relation, from.x < to.x, true));
    }
  }

  stemNodes.value.forEach((stem, index) => {
    const branch = branchNodes.value[index];
    const relation = elementFlowRelation(branch.element, stem.element);
    if (relation && relation.tone === "good") {
      links.push(verticalLink(`vertical-${index}`, index, relation));
    }
  });

  return links.filter((link) => link.image);
});

const floatingBadges = computed(() => {
  const badges: FloatingBadge[] = [];

  for (let index = 0; index < branchNodes.value.length - 1; index += 1) {
    const from = branchNodes.value[index];
    const to = branchNodes.value[index + 1];
    const relation = branchRelation(from, to);
    if (relation?.tone === "bad") {
      badges.push({
        key: `branch-${index}-bad-badge`,
        label: relation.label,
        x: (from.x + to.x) / 2,
        y: branchY,
      });
    }
  }

  stemNodes.value.forEach((stem, index) => {
    const branch = branchNodes.value[index];
    const relation = elementFlowRelation(branch.element, stem.element);
    if (relation?.tone === "bad") {
      badges.push({
        key: `vertical-${index}-bad-badge`,
        label: relation.label,
        x: stem.x,
        y: (stem.y + branch.y) / 2,
      });
    }
  });

  props.result.quantModel?.interactions?.forEach((interaction, index) => {
    if (!isBlockingInteraction(interaction.type)) {
      return;
    }
    const matched = nodesForInteraction(interaction);
    if (matched.length < 2 || !isAdjacentInteractionNodes(matched)) {
      return;
    }
    badges.push({
      key: `interaction-badge-${index}`,
      label: shortInteractionLabel(interaction.type),
      x: (matched[0].x + matched[1].x) / 2,
      y: (matched[0].y + matched[1].y) / 2,
    });
  });

  return dedupeBadges(badges);
});

const bottomFlags = computed(() =>
  pillars.value.map((pillar) => {
    const gz = `${pillar.stem}${pillar.branch}`;
    if (/(甲辰)|(甲戌)|(乙丑)|(乙未)|(丙申)|(丁酉)|(戊子)|(己亥)|(庚寅)|(辛卯)|(壬午)|(癸巳)/.test(gz)) {
      return "盖头";
    }
    if (/(甲申)|(乙酉)|(丙子)|(丁亥)|(戊寅)|(己卯)|(庚午)|(辛巳)|(壬辰)|(癸丑)|(癸未)|(壬戌)/.test(gz)) {
      return "截脚";
    }
    return "";
  }),
);

function makeNode(key: string, pillar: Pillar, tenGod: string, kind: NodeKind, x: number, y: number): FlowNode {
  const char = kind === "stem" ? pillar.stem : pillar.branch;
  const element = kind === "stem" ? STEM_ELEMENTS[char] : BRANCH_ELEMENTS[char];
  return {
    key,
    char,
    tenGod,
    element: element || "earth",
    kind,
    x,
    y,
  };
}

function stemRelation(from: FlowNode, to: FlowNode) {
  if (isStemCombination(from.char, to.char)) {
    return { label: "合", tone: "good" as const };
  }
  return elementFlowRelation(from.element, to.element);
}

function branchRelation(from: FlowNode, to: FlowNode): FlowRelation | null {
  if (isBranchCombination(from.char, to.char)) {
    return { label: "合", tone: "good" as const };
  }
  const relation = elementFlowRelation(from.element, to.element);
  if (relation?.tone === "good" && hasBlockingInteractionBetween("地支", from.char, to.char)) {
    return null;
  }
  return relation;
}

function elementFlowRelation(from: ElementKey, to: ElementKey): FlowRelation | null {
  if (from === to) {
    return { label: "助", tone: "good" as const };
  }
  if (GENERATES[from] === to) {
    return { label: "生", tone: "good" as const, direction: "forward" };
  }
  if (GENERATES[to] === from) {
    return { label: "生", tone: "good" as const, direction: "reverse" };
  }
  if (CONTROLS[from] === to || CONTROLS[to] === from) {
    return { label: "克", tone: "bad" as const };
  }
  return null;
}

function horizontalLink(
  key: string,
  index: number,
  relation: FlowRelation,
  leftToRight: boolean,
  branch = false,
): PngLink {
  const midpointX = (pillarXs[index] + pillarXs[index + 1]) / 2;
  return {
    key,
    image: imageForHorizontal(relation.label, relation.direction ?? (leftToRight ? "forward" : "reverse"), relation.tone),
    tone: relation.tone,
    orientation: "horizontal",
    x: midpointX - horizontalLinkWidth / 2,
    y: (branch ? branchY : stemY) - horizontalLinkHeight / 2,
  };
}

function verticalLink(key: string, index: number, relation: FlowRelation): PngLink {
  const charCenterX = pillarXs[index];
  const charCenterY = (stemY + branchY) / 2;
  return {
    key,
    image: imageForVertical(relation.label, relation.direction ?? "forward", relation.tone),
    tone: relation.tone,
    orientation: "vertical",
    x: charCenterX - verticalLinkWidth / 2,
    y: charCenterY - verticalLinkHeight / 2,
  };
}

function imageForHorizontal(label: string, direction: "forward" | "reverse", tone: "good" | "bad") {
  if (label === "冲") return ltChImg;
  if (label === "合") return ltHImg;
  if (tone === "bad") return ltKhImg;
  if (label === "助") {
    return ltZhImg;
  }
  return direction === "forward" ? ltSyImg : ltSzImg;
}

function imageForVertical(label: string, direction: "forward" | "reverse", tone: "good" | "bad") {
  if (label === "冲") return ltCsImg;
  if (tone === "bad") return ltKsImg;
  if (label === "助") return ltZsImg;
  return direction === "forward" ? ltSsImg : ltSxImg;
}

function nodesForInteraction(interaction: QuantModelInteraction) {
  const scope = interaction.scope.includes("天干") ? "stem" : interaction.scope.includes("地支") ? "branch" : null;
  const chars = [...interaction.target].filter((char) => STEM_ELEMENTS[char] || BRANCH_ELEMENTS[char]);
  return [...stemNodes.value, ...branchNodes.value].filter(
    (node) => (!scope || node.kind === scope) && chars.includes(node.char),
  );
}

function isAdjacentInteractionNodes(nodes: FlowNode[]) {
  const [first, second] = nodes;
  if (!first || !second || first.kind !== second.kind) {
    return false;
  }

  const sourceNodes = first.kind === "stem" ? stemNodes.value : branchNodes.value;
  const firstIndex = sourceNodes.findIndex((node) => node.key === first.key);
  const secondIndex = sourceNodes.findIndex((node) => node.key === second.key);
  return firstIndex >= 0 && secondIndex >= 0 && Math.abs(firstIndex - secondIndex) === 1;
}

function isStemCombination(left: string, right: string) {
  return STEM_COMBINATIONS.has(`${left}${right}`) || STEM_COMBINATIONS.has(`${right}${left}`);
}

function isBranchCombination(left: string, right: string) {
  const pair = `${left}${right}`;
  const reversedPair = `${right}${left}`;
  return (
    BRANCH_SIX_COMBINATIONS.has(pair) ||
    BRANCH_SIX_COMBINATIONS.has(reversedPair) ||
    BRANCH_HALF_COMBINATIONS.has(pair) ||
    BRANCH_HALF_COMBINATIONS.has(reversedPair)
  );
}

function hasBlockingInteractionBetween(scope: "天干" | "地支", left: string, right: string) {
  return (
    props.result.quantModel?.interactions?.some((interaction) => {
      if (!interaction.scope.includes(scope) || !isBlockingInteraction(interaction.type)) {
        return false;
      }
      const chars = [...interaction.target];
      return chars.includes(left) && chars.includes(right);
    }) || false
  );
}

function isBlockingInteraction(type: string) {
  return type.includes("沖") || type.includes("刑") || type.includes("破") || type.includes("害") || type.includes("克");
}

function shortInteractionLabel(type: string) {
  if (type.includes("沖")) return "冲";
  if (type.includes("刑")) return "刑";
  if (type.includes("破")) return "破";
  if (type.includes("害")) return "害";
  if (type.includes("克")) return "克";
  return type.slice(0, 1);
}

function dedupeBadges(badges: FloatingBadge[]) {
  const seen = new Set<string>();
  return badges.filter((badge) => {
    const key = `${Math.round(badge.x / 8)}-${Math.round(badge.y / 8)}-${badge.label}`;
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}
</script>

<style scoped>
.flow-panel {
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
}

.section-header h2 {
  margin: 0;
}

.flow-subtitle {
  color: #8c8c8c;
  font-size: 0.9em;
}

.flow-scroll {
  overflow-x: auto;
}

.flow-board {
  position: relative;
  width: 878px;
  height: 440px;
  margin: 0 auto;
  background: #fff;
  overflow: hidden;
}

.flow-node {
  position: absolute;
  width: 96px;
  height: 64px;
  transform: translate(-50%, -50%);
  display: grid;
  place-items: center;
  z-index: 2;
}

.flow-node-ten-god {
  position: absolute;
  top: -30px;
  left: 0;
  width: 100%;
  color: #a1a1a1;
  font-family: "BiauKai", "DFKai-SB", "KaiTi", serif;
  font-size: 22px;
  line-height: 1;
  text-align: center;
}

.flow-node-ten-god-bottom {
  top: auto;
  bottom: -30px;
  margin-top: 0;
}

.flow-node-char {
  font-family: "BiauKai", "DFKai-SB", "KaiTi", serif;
  font-size: 56px;
  line-height: 1;
}

.flow-node-wood .flow-node-char {
  color: #00c535;
}

.flow-node-fire .flow-node-char {
  color: #e60012;
}

.flow-node-earth .flow-node-char {
  color: #9b6a00;
}

.flow-node-metal .flow-node-char {
  color: #9b9b9b;
}

.flow-node-water .flow-node-char {
  color: #1769ff;
}

.flow-link-img {
  position: absolute;
  z-index: 1;
  object-fit: contain;
  user-select: none;
  pointer-events: none;
}

.flow-link-img-horizontal {
  width: 146px;
  height: 35px;
}

.flow-link-img-vertical {
  width: 42px;
  height: 202px;
}

.flow-link-img-bad {
  opacity: 0.9;
}

.flow-floating-badge {
  position: absolute;
  z-index: 3;
  width: 34px;
  height: 34px;
  transform: translate(-50%, -50%);
  display: grid;
  place-items: center;
  border-radius: 999px;
  color: #fff;
  font-family: "BiauKai", "DFKai-SB", "KaiTi", serif;
  font-size: 18px;
  font-weight: 700;
}

.flow-floating-badge-bad {
  background: #f23932;
}

.flow-bottom-flag {
  position: absolute;
  bottom: 26px;
  transform: translateX(-50%);
  color: #bc8f32;
  font-family: "BiauKai", "DFKai-SB", "KaiTi", serif;
  font-size: 18px;
  line-height: 1;
  z-index: 2;
}

.flow-bottom-flag-empty {
  color: transparent;
}

.flow-legend {
  display: grid;
  gap: 10px;
  max-width: 878px;
  margin: 18px auto 2px;
  padding-left: 4px;
  font-size: 18px;
}

.legend-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.legend-icon {
  display: inline-grid;
  place-items: center;
  width: 34px;
  height: 34px;
  border-radius: 999px;
  color: white;
  font-weight: 800;
}

.legend-icon-good {
  background: #009f5d;
}

.legend-icon-bad {
  background: #f20d0d;
}

.flow-panel-compact .flow-board {
  margin: 0;
  transform-origin: top left;
}

.flow-panel-compact .flow-scroll {
  overflow: hidden;
}

@media (max-width: 720px) {
  .section-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .flow-legend {
    max-width: 100%;
    font-size: 15px;
  }

  .legend-row {
    gap: 8px;
  }

  .legend-icon {
    width: 28px;
    height: 28px;
  }
}
</style>
