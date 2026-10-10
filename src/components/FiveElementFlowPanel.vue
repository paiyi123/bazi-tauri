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

        <!-- 隔柱或三支的關係：天干畫在上方、地支畫在下方的括線 -->
        <svg
          v-if="arcGroups.length"
          class="flow-arc-layer"
          :width="boardWidth"
          :height="boardHeight"
          :viewBox="`0 0 ${boardWidth} ${boardHeight}`"
        >
          <path
            v-for="arc in arcGroups"
            :key="arc.key"
            :d="arc.path"
            :class="`flow-arc-${arc.tone}`"
          />
        </svg>

        <template v-for="badge in floatingBadges" :key="badge.key">
          <!-- 相剋有方向：箭頭指向被剋的一方 -->
          <div
            v-if="badge.arrow"
            :class="['flow-control-arrow', `is-${badge.arrow}`]"
            :style="{ left: `${badge.x}px`, top: `${badge.y}px` }"
          />
          <div
            class="flow-floating-badge"
            :class="`flow-floating-badge-${badge.tone}`"
            :style="{ left: `${badge.x}px`, top: `${badge.y}px` }"
            :title="badge.title"
          >
            {{ badge.label }}
          </div>
        </template>

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
        <span>流通：三合、三會、六合、半合、相生、相助</span>
      </div>
      <div class="legend-row">
        <span class="legend-icon legend-icon-bad">×</span>
        <span>阻塞：相沖、相刑、相破、相害、相克（克的箭頭指向被剋的一方）</span>
      </div>
      <div class="legend-note">
        相鄰兩柱有多種關係時往內上下排列；隔柱或三支的關係用括線連起，天干畫在上方、地支畫在下方。滑鼠停在圓標上可看說明。
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
import { BRANCH_WUXING, STEM_WUXING, type WuXing } from "../constants/wuxing";

type ElementKey = WuXing;
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
type Tone = "good" | "bad";
type FloatingBadge = {
  key: string;
  label: string;
  tone: Tone;
  x: number;
  y: number;
  arrow?: "left" | "right" | "up" | "down";
  title?: string;
};
type FlowRelation = {
  label: string;
  tone: Tone;
  direction?: "forward" | "reverse";
  // For 克: which side of (from, to) does the controlling.
  controller?: "from" | "to";
  title?: string;
};
// One relation from the quant model, located by pillar index (0 時, 1 日, 2 月, 3 年).
type NatalRelation = {
  kind: NodeKind;
  indexes: number[];
  label: string;
  tone: Tone;
  title: string;
};
// All relations between two adjacent pillars; entry 0 sits on the line, the rest stack toward the middle.
type PairStack = {
  key: string;
  kind: NodeKind;
  from: FlowNode;
  to: FlowNode;
  entries: FlowRelation[];
};
// Relations between non-adjacent pillars (or three branches), drawn as a bracket outside the rows.
type ArcGroup = {
  key: string;
  kind: NodeKind;
  indexes: number[];
  relations: NatalRelation[];
  layer: number;
  tone: Tone | "mixed";
  path: string;
  badgeX: number;
  levelY: number;
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
const baseStemY = 104;
const rowGap = 202;
const baseBoardHeight = 440;
const boardWidth = 878;
const horizontalLinkWidth = 146;
const horizontalLinkHeight = 35;
const verticalLinkHeight = rowGap;
const verticalLinkWidth = 42;
// Extra relations of one pillar pair stack toward the middle of the board, this far apart.
const stackGap = 40;
// Brackets: the stub starts past the ten-god label, the first level sits arcOffset from the row.
const arcStubOffset = 70;
const arcOffset = 96;
const arcLayerGap = 38;
const arcCornerRadius = 10;
const badgeStep = 46;

const flowScrollRef = ref<HTMLElement | null>(null);
const flowScale = ref(1);
let flowResizeObserver: ResizeObserver | null = null;

const flowScrollStyle = computed(() => {
  if (!props.compact) {
    return undefined;
  }
  return {
    height: `${boardHeight.value * flowScale.value}px`,
  };
});

const flowBoardStyle = computed(() => ({
  height: `${boardHeight.value}px`,
  ...(props.compact ? { transform: `scale(${flowScale.value})` } : {}),
}));

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

const STEM_ELEMENTS = STEM_WUXING;
const BRANCH_ELEMENTS = BRANCH_WUXING;

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

const natalRelations = computed(() => {
  const relations: NatalRelation[] = [];
  const seen = new Set<string>();
  props.result.quantModel?.interactions?.forEach((interaction) => {
    const members = interactionMembers(interaction);
    if (!members) {
      return;
    }
    const label = shortInteractionLabel(interaction.type);
    const key = `${members.kind}-${members.indexes.join("")}-${label}`;
    if (seen.has(key)) {
      return;
    }
    seen.add(key);
    relations.push({
      ...members,
      label,
      tone: isBlockingInteraction(interaction.type) ? "bad" : "good",
      title: interactionTitle(interaction),
    });
  });
  return relations;
});

// Bracket layers: shorter spans sit closer to the row; overlapping spans move one level further out.
const arcLayout = computed(() => {
  const groups = new Map<string, { kind: NodeKind; indexes: number[]; relations: NatalRelation[] }>();
  natalRelations.value
    .filter((relation) => !isAdjacentPair(relation.indexes))
    .forEach((relation) => {
      const key = `${relation.kind}-${relation.indexes.join("")}`;
      const group = groups.get(key) ?? { kind: relation.kind, indexes: relation.indexes, relations: [] };
      group.relations.push(relation);
      groups.set(key, group);
    });

  const sorted = [...groups.entries()].sort(([, left], [, right]) => {
    const spanDiff = span(left.indexes) - span(right.indexes);
    return spanDiff || left.indexes[0] - right.indexes[0];
  });
  const placed: { kind: NodeKind; indexes: number[]; layer: number }[] = [];
  return sorted.map(([key, group]) => {
    let layer = 0;
    while (
      placed.some(
        (other) =>
          other.kind === group.kind && other.layer === layer && rangesTouch(other.indexes, group.indexes),
      )
    ) {
      layer += 1;
    }
    placed.push({ kind: group.kind, indexes: group.indexes, layer });
    return { key, ...group, layer };
  });
});

const stemArcLayers = computed(() => layerCount("stem"));
const branchArcLayers = computed(() => layerCount("branch"));

// Room above the stems for their brackets; badges on the outermost level need ~22px.
const topPad = computed(() =>
  stemArcLayers.value
    ? Math.max(0, arcOffset + (stemArcLayers.value - 1) * arcLayerGap + 22 - baseStemY)
    : 0,
);
const stemY = computed(() => baseStemY + topPad.value);
const branchY = computed(() => stemY.value + rowGap);
// Brackets under the branches need room above the 蓋頭/截腳 flags (bottom 26px, 18px tall).
const boardHeight = computed(() => {
  const base = baseBoardHeight + topPad.value;
  if (!branchArcLayers.value) {
    return base;
  }
  return Math.max(base, branchY.value + arcOffset + (branchArcLayers.value - 1) * arcLayerGap + 71);
});

const stemNodes = computed(() =>
  pillars.value.map((pillar, index) =>
    makeNode(`stem-${index}`, pillar, stemTenGods.value[index], "stem", pillarXs[index], stemY.value),
  ),
);

const branchNodes = computed(() =>
  pillars.value.map((pillar, index) =>
    makeNode(`branch-${index}`, pillar, branchTenGods.value[index], "branch", pillarXs[index], branchY.value),
  ),
);

const pairStacks = computed(() => {
  const stacks: PairStack[] = [];
  (["stem", "branch"] as const).forEach((kind) => {
    const nodes = kind === "stem" ? stemNodes.value : branchNodes.value;
    for (let index = 0; index < nodes.length - 1; index += 1) {
      const from = nodes[index];
      const to = nodes[index + 1];
      const entries: FlowRelation[] = [];
      const combined =
        kind === "stem" ? isStemCombination(from.char, to.char) : isBranchCombination(from.char, to.char);
      if (combined) {
        entries.push({ label: "合", tone: "good" });
      }
      const element = elementFlowRelation(from.element, to.element);
      if (element) {
        entries.push(element);
      }
      natalRelations.value
        .filter((relation) => relation.kind === kind && relation.indexes.join() === `${index},${index + 1}`)
        .forEach((relation) => {
          // The 合 link on the line already shows this combination.
          if (relation.tone === "good" && combined) {
            return;
          }
          if (entries.some((entry) => entry.label === relation.label)) {
            return;
          }
          entries.push({ label: relation.label, tone: relation.tone, title: relation.title });
        });
      stacks.push({ key: `${kind}-${index}`, kind, from, to, entries });
    }
  });
  return stacks;
});

const arcGroups = computed<ArcGroup[]>(() =>
  arcLayout.value.map((group) => {
    const direction = group.kind === "stem" ? -1 : 1;
    const rowY = group.kind === "stem" ? stemY.value : branchY.value;
    const startY = rowY + direction * arcStubOffset;
    const levelY = rowY + direction * (arcOffset + group.layer * arcLayerGap);
    const xs = group.indexes.map((index) => pillarXs[index]);
    const left = xs[0];
    const right = xs[xs.length - 1];
    const r = arcCornerRadius;
    const outline =
      `M ${left} ${startY} V ${levelY - direction * r} Q ${left} ${levelY} ${left + r} ${levelY} ` +
      `H ${right - r} Q ${right} ${levelY} ${right} ${levelY - direction * r} V ${startY}`;
    const middles = xs.slice(1, -1).map((x) => ` M ${x} ${startY} V ${levelY}`).join("");
    const tones = new Set(group.relations.map((relation) => relation.tone));
    return {
      key: `arc-${group.key}`,
      kind: group.kind,
      indexes: group.indexes,
      relations: group.relations,
      layer: group.layer,
      tone: tones.size > 1 ? "mixed" : group.relations[0].tone,
      path: outline + middles,
      badgeX: arcBadgeX(group.indexes, group.layer),
      levelY,
    };
  }),
);

const pngLinks = computed(() => {
  const links: PngLink[] = [];

  pairStacks.value.forEach((stack) => {
    stack.entries.forEach((relation, slot) => {
      if (hasLinkImage(relation)) {
        links.push(
          horizontalLink(`${stack.key}-${slot}`, stack, relation, stackY(stack, slot)),
        );
      }
    });
  });

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

  pairStacks.value.forEach((stack) => {
    stack.entries.forEach((relation, slot) => {
      if (hasLinkImage(relation)) {
        return;
      }
      badges.push({
        key: `${stack.key}-${slot}-badge`,
        label: relation.label,
        tone: relation.tone,
        x: (stack.from.x + stack.to.x) / 2,
        y: stackY(stack, slot),
        title: relation.title,
        ...controlArrow(relation, stack.from, stack.to, "right", "left"),
      });
    });
  });

  stemNodes.value.forEach((stem, index) => {
    const branch = branchNodes.value[index];
    const relation = elementFlowRelation(branch.element, stem.element);
    if (relation?.tone === "bad") {
      badges.push({
        key: `vertical-${index}-bad-badge`,
        label: relation.label,
        tone: relation.tone,
        x: stem.x,
        y: (stem.y + branch.y) / 2,
        ...controlArrow(relation, branch, stem, "up", "down"),
      });
    }
  });

  arcGroups.value.forEach((arc) => {
    arc.relations.forEach((relation, position) => {
      badges.push({
        key: `${arc.key}-${relation.label}`,
        label: relation.label,
        tone: relation.tone,
        x: arc.badgeX + (position - (arc.relations.length - 1) / 2) * badgeStep,
        y: arc.levelY,
        title: relation.title,
      });
    });
  });

  return badges;
});

const bottomFlags = computed(() =>
  pillars.value.map((pillar) => {
    const gz = `${pillar.stem}${pillar.branch}`;
    if (/(甲辰)|(甲戌)|(乙丑)|(乙未)|(丙申)|(丁酉)|(戊子)|(己亥)|(庚寅)|(辛卯)|(壬午)|(癸巳)/.test(gz)) {
      return "蓋頭";
    }
    if (/(甲申)|(乙酉)|(丙子)|(丁亥)|(戊寅)|(己卯)|(庚午)|(辛巳)|(壬辰)|(癸丑)|(癸未)|(壬戌)/.test(gz)) {
      return "截腳";
    }
    return "";
  }),
);

// Arrow points from the controller toward the controlled node.
function controlArrow(
  relation: FlowRelation,
  from: FlowNode,
  to: FlowNode,
  towardTo: NonNullable<FloatingBadge["arrow"]>,
  towardFrom: NonNullable<FloatingBadge["arrow"]>,
): Pick<FloatingBadge, "arrow" | "title"> {
  if (!relation.controller) {
    return {};
  }
  const [controller, controlled] = relation.controller === "from" ? [from, to] : [to, from];
  return {
    arrow: relation.controller === "from" ? towardTo : towardFrom,
    title: `${controller.char}剋${controlled.char}`,
  };
}

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

// Slot 0 is on the line between the two characters; later slots step toward the middle of the board.
function stackY(stack: PairStack, slot: number) {
  const rowY = stack.kind === "stem" ? stemY.value : branchY.value;
  return rowY + (stack.kind === "stem" ? 1 : -1) * slot * stackGap;
}

function hasLinkImage(relation: FlowRelation) {
  return relation.tone === "good" && ["合", "生", "助"].includes(relation.label);
}

function layerCount(kind: NodeKind) {
  const layers = arcLayout.value.filter((group) => group.kind === kind).map((group) => group.layer);
  return layers.length ? Math.max(...layers) + 1 : 0;
}

function span(indexes: number[]) {
  return indexes[indexes.length - 1] - indexes[0];
}

function isAdjacentPair(indexes: number[]) {
  return indexes.length === 2 && span(indexes) === 1;
}

function rangesTouch(left: number[], right: number[]) {
  return left[0] <= right[right.length - 1] && right[0] <= left[left.length - 1];
}

// Badges go between two pillars, never on a pillar: other brackets' stubs run down the pillar lines.
function arcBadgeX(indexes: number[], layer: number) {
  const center = (pillarXs[indexes[0]] + pillarXs[indexes[indexes.length - 1]]) / 2;
  const gaps: number[] = [];
  for (let index = indexes[0]; index < indexes[indexes.length - 1]; index += 1) {
    gaps.push((pillarXs[index] + pillarXs[index + 1]) / 2);
  }
  const nearest = Math.min(...gaps.map((x) => Math.abs(x - center)));
  const candidates = gaps.filter((x) => Math.abs(x - center) === nearest);
  return candidates[layer % 2 === 0 ? 0 : candidates.length - 1];
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
  if (CONTROLS[from] === to) {
    return { label: "克", tone: "bad" as const, controller: "from" };
  }
  if (CONTROLS[to] === from) {
    return { label: "克", tone: "bad" as const, controller: "to" };
  }
  return null;
}

function horizontalLink(key: string, stack: PairStack, relation: FlowRelation, centerY: number): PngLink {
  const midpointX = (stack.from.x + stack.to.x) / 2;
  const leftToRight = stack.from.x < stack.to.x;
  return {
    key,
    image: imageForHorizontal(relation.label, relation.direction ?? (leftToRight ? "forward" : "reverse"), relation.tone),
    tone: relation.tone,
    orientation: "horizontal",
    x: midpointX - horizontalLinkWidth / 2,
    y: centerY - horizontalLinkHeight / 2,
  };
}

function verticalLink(key: string, index: number, relation: FlowRelation): PngLink {
  const charCenterX = pillarXs[index];
  const charCenterY = (stemY.value + branchY.value) / 2;
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

// Display order of the board: 時、日、月、年
const PILLAR_LABEL_INDEX: Record<string, number> = { 時: 0, 日: 1, 月: 2, 年: 3 };

// Locates a natal interaction on the board. Only relations among the four natal pillars are drawn
// (rows such as 「大運 - 月柱」 are skipped).
function interactionMembers(interaction: QuantModelInteraction): Pick<NatalRelation, "kind" | "indexes"> | null {
  const kind: NodeKind | null = interaction.scope.includes("天干")
    ? "stem"
    : interaction.scope.includes("地支")
      ? "branch"
      : null;
  if (!kind) {
    return null;
  }
  // Prefer the pillar positions (e.g. 「日柱(子)、月柱(午)」): matching by character alone puts a
  // 子午沖 between two 子 when the chart has 子 twice.
  const segments = (interaction.pillars ?? "")
    .split("、")
    .map((segment) => segment.trim())
    .filter(Boolean);
  let indexes: number[];
  if (segments.some((segment) => segment.includes("柱"))) {
    indexes = segments.map((segment) => {
      const match = segment.match(/^([年月日時])柱/);
      return match ? PILLAR_LABEL_INDEX[match[1]] : -1;
    });
    if (indexes.includes(-1)) {
      return null;
    }
  } else {
    const chars = pillars.value.map((pillar) => (kind === "stem" ? pillar.stem : pillar.branch));
    const targets = [...interaction.target].filter((char) => STEM_ELEMENTS[char] || BRANCH_ELEMENTS[char]);
    indexes = chars.flatMap((char, index) => (targets.includes(char) ? [index] : []));
    // Ambiguous when a character appears in more pillars than the relation has members.
    if (indexes.length !== targets.length) {
      return null;
    }
  }
  const unique = [...new Set(indexes)].sort((left, right) => left - right);
  return unique.length >= 2 ? { kind, indexes: unique } : null;
}

function interactionTitle(interaction: QuantModelInteraction) {
  const outcome = interaction.outcome && interaction.outcome !== "-" ? `（${interaction.outcome}）` : "";
  const detail = interaction.detail && interaction.detail !== "-" ? `：${interaction.detail}` : "";
  return `${interaction.target}${interaction.type}${outcome}${detail}`;
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

function isBlockingInteraction(type: string) {
  return type.includes("沖") || type.includes("刑") || type.includes("破") || type.includes("害") || type.includes("克");
}

// 「三合化」→三合、「半合化」→半合、「自刑」→刑
function shortInteractionLabel(type: string) {
  const label = ["三合", "三會", "半合", "沖", "刑", "破", "害", "克", "合"].find((item) => type.includes(item));
  return label ?? type.slice(0, 2);
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
  color: var(--wx-wood);
}

.flow-node-fire .flow-node-char {
  color: var(--wx-fire);
}

.flow-node-earth .flow-node-char {
  color: var(--wx-earth);
}

.flow-node-metal .flow-node-char {
  color: var(--wx-metal);
}

.flow-node-water .flow-node-char {
  color: var(--wx-water);
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
  box-sizing: border-box;
  min-width: 34px;
  height: 34px;
  padding: 0 7px;
  white-space: nowrap;
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

.flow-floating-badge-good {
  background: #009f5d;
}

.flow-arc-layer {
  position: absolute;
  top: 0;
  left: 0;
  z-index: 1;
  overflow: visible;
  pointer-events: none;
}

.flow-arc-layer path {
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
}

.flow-arc-layer .flow-arc-good {
  stroke: #009f5d;
}

.flow-arc-layer .flow-arc-bad {
  stroke: #f23932;
  stroke-dasharray: 6 5;
}

.flow-arc-layer .flow-arc-mixed {
  stroke: #bc8f32;
  stroke-dasharray: 6 5;
}

.flow-control-arrow {
  position: absolute;
  z-index: 2;
  transform: translate(-50%, -50%);
  pointer-events: none;
}

.flow-control-arrow.is-left,
.flow-control-arrow.is-right {
  width: 128px;
  height: 0;
  border-top: 2px dashed #f23932;
}

.flow-control-arrow.is-up,
.flow-control-arrow.is-down {
  width: 0;
  height: 100px;
  border-left: 2px dashed #f23932;
}

.flow-control-arrow::after {
  content: "";
  position: absolute;
  border: 7px solid transparent;
}

.flow-control-arrow.is-right::after {
  top: -8px;
  right: -8px;
  border-left: 12px solid #f23932;
  border-right-width: 0;
}

.flow-control-arrow.is-left::after {
  top: -8px;
  left: -8px;
  border-right: 12px solid #f23932;
  border-left-width: 0;
}

.flow-control-arrow.is-down::after {
  bottom: -8px;
  left: -8px;
  border-top: 12px solid #f23932;
  border-bottom-width: 0;
}

.flow-control-arrow.is-up::after {
  top: -8px;
  left: -8px;
  border-bottom: 12px solid #f23932;
  border-top-width: 0;
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

.legend-note {
  font-size: 15px;
  line-height: 1.5;
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


.flow-subtitle {
  color: var(--bazi-text-muted);
}

.flow-scroll {
  border-radius: 18px;
  background: linear-gradient(180deg, rgba(255, 251, 245, 0.92), rgba(247, 234, 215, 0.74));
  border: 1px solid var(--bazi-border);
}

.flow-board {
  background:
    linear-gradient(rgba(174, 128, 72, 0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(174, 128, 72, 0.05) 1px, transparent 1px),
    rgba(255, 253, 248, 0.96);
  background-size: 28px 28px;
}

.flow-node-char {
  text-shadow: 0 2px 8px rgba(84, 45, 18, 0.1);
}

.flow-floating-badge {
  box-shadow: var(--bazi-shadow-sm);
}

.flow-legend {
  color: var(--bazi-text-muted);
}
</style>
