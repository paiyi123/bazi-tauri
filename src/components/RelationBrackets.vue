<template>
  <div
    ref="rootRef"
    class="relation-brackets"
    :style="{ paddingTop: `${topSpace}px`, paddingBottom: `${bottomSpace}px` }"
    @click="setActive(null)"
  >
    <slot />
    <svg
      v-if="layout.length && metrics"
      class="rb-svg"
      :width="metrics.width"
      :height="metrics.height"
      :viewBox="`0 0 ${metrics.width} ${metrics.height}`"
      aria-hidden="true"
    >
      <g
        v-for="item in layout"
        :key="item.key"
        :class="['rb-link', { 'is-dim': isDim(item.key), 'is-active': active === item.key, 'is-preview': item.preview }]"
      >
        <path
          :d="item.path"
          :stroke="item.color"
          :stroke-dasharray="item.dash"
          stroke-width="2"
          fill="none"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
        <circle v-for="(dot, index) in item.dots" :key="index" :cx="dot.x" :cy="dot.y" r="3.5" :fill="item.color" />
      </g>
    </svg>
    <button
      v-for="item in layout"
      :key="`label-${item.key}`"
      type="button"
      :class="['rb-label', { 'is-dim': isDim(item.key), 'is-active': active === item.key, 'is-preview': item.preview }]"
      :style="{ left: `${item.labelX}px`, top: `${item.labelY}px`, '--rb-color': item.color }"
      :title="item.title"
      @click.stop="setActive(active === item.key ? null : item.key)"
    >
      {{ item.label }}<span v-if="ARROW[item.favor ?? 'neutral']" :class="['rb-arrow', `is-${item.favor}`]">{{ ARROW[item.favor ?? 'neutral'] }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";

// 在一排柱子上下畫「括號線」：天干的關係畫在上方，地支的關係畫在下方，線不進卡片。
// 每個關係一條橫線，兩端（三合是三端）垂直接到那一柱，名稱寫在橫線中間；同一層放不下就往外加一層，所以線不交叉。
// 柱子由 slot 提供，每一柱的卡片要有 data-relation-col="欄位序號"。

export type BracketLink = {
  key: string;
  side: "top" | "bottom";
  cols: number[];
  label: string;
  color: string;
  dash?: string;
  preview?: boolean;
  title?: string;
  /** 對日主有利（箭頭往上）、不利（往下）、有利有弊；沒判就不畫箭頭。 */
  favor?: "good" | "bad" | "mixed" | "neutral";
  reason?: string;
};

const ARROW: Record<string, string> = { good: "▲", bad: "▼", mixed: "◆", neutral: "" };

type Metrics = {
  width: number;
  height: number;
  fontScale: number;
  centers: number[];
  top: number;
  bottom: number;
};

type Placed = BracketLink & {
  lane: number;
  path: string;
  dots: { x: number; y: number }[];
  labelX: number;
  labelY: number;
};

const props = defineProps<{ links: BracketLink[]; active?: string | null }>();
const emit = defineEmits<{ "update:active": [key: string | null] }>();

const rootRef = ref<HTMLElement | null>(null);
const metrics = ref<Metrics | null>(null);

const laneHeight = computed(() => Math.round(20 * (metrics.value?.fontScale ?? 1)) + 8);
const GAP = 14; // 卡片邊緣到第一層橫線
const TICK = 7; // 同一柱接了好幾條線時，每條錯開的距離

function labelWidth(link: BracketLink, fontScale: number) {
  const ems =
    Array.from(link.label).reduce((sum, ch) => sum + (/[\x00-\x7f]/.test(ch) ? 0.55 : 1), 0) +
    (ARROW[link.favor ?? "neutral"] ? 1.2 : 0);
  return ems * 12 * fontScale + 18;
}

/** 依各柱中心位置分層：跨距短的放內層，和同一層的線（含名稱）重疊就往外一層。 */
function assignLanes(links: BracketLink[], m: Metrics) {
  const lanes: Array<Array<{ left: number; right: number }>> = [];
  const result = new Map<string, number>();
  const items = links
    .map((link) => {
      const xs = link.cols.map((col) => m.centers[col]);
      const a = Math.min(...xs);
      const b = Math.max(...xs);
      const half = labelWidth(link, m.fontScale) / 2;
      const mid = (a + b) / 2;
      return { link, a, b, left: Math.min(a, mid - half) - 6, right: Math.max(b, mid + half) + 6 };
    })
    .sort((x, y) => x.b - x.a - (y.b - y.a) || x.a - y.a);
  for (const item of items) {
    let lane = 0;
    while (lanes[lane]?.some((box) => item.left < box.right && box.left < item.right)) lane += 1;
    (lanes[lane] ??= []).push({ left: item.left, right: item.right });
    result.set(item.link.key, lane);
  }
  return { lanes: result, count: lanes.length };
}

const sides = computed(() => {
  const m = metrics.value;
  const empty = { lanes: new Map<string, number>(), count: 0 };
  if (!m) return { top: empty, bottom: empty };
  const valid = props.links.filter((link) => link.cols.length >= 2 && link.cols.every((col) => m.centers[col] != null));
  return {
    top: assignLanes(valid.filter((link) => link.side === "top"), m),
    bottom: assignLanes(valid.filter((link) => link.side === "bottom"), m),
  };
});

// 預估層數（量到位置前先用連結數估），避免第一次畫面跳動。
const topSpace = computed(() => {
  const count = metrics.value ? sides.value.top.count : Math.min(2, props.links.filter((l) => l.side === "top").length);
  return count ? GAP + count * laneHeight.value : 6;
});
const bottomSpace = computed(() => {
  const count = metrics.value ? sides.value.bottom.count : Math.min(2, props.links.filter((l) => l.side === "bottom").length);
  return count ? GAP + count * laneHeight.value : 6;
});

const layout = computed<Placed[]>(() => {
  const m = metrics.value;
  if (!m) return [];
  const placed: Placed[] = [];
  const half = (link: BracketLink) => labelWidth(link, m.fontScale) / 2;
  for (const side of ["top", "bottom"] as const) {
    const { lanes } = sides.value[side];
    const links = props.links.filter((link) => link.side === side && lanes.has(link.key));

    // 同一柱接了好幾條線：往右延伸的線排在柱中心右邊、往左的排左邊，外層的靠近中心，這樣垂直線不會穿過別的橫線。
    const offsets = new Map<string, number>();
    const byCol = new Map<number, Array<{ link: BracketLink; dir: number }>>();
    for (const link of links) {
      const min = Math.min(...link.cols);
      const max = Math.max(...link.cols);
      for (const col of new Set(link.cols)) {
        const dir = col === min ? 1 : col === max ? -1 : 0;
        (byCol.get(col) ?? byCol.set(col, []).get(col)!).push({ link, dir });
      }
    }
    for (const [col, list] of byCol) {
      if (list.length === 1) {
        offsets.set(`${list[0].link.key}@${col}`, 0);
        continue;
      }
      for (const dir of [1, -1, 0]) {
        list
          .filter((item) => item.dir === dir)
          .sort((x, y) => lanes.get(y.link.key)! - lanes.get(x.link.key)!)
          .forEach((item, index) => offsets.set(`${item.link.key}@${col}`, dir * (index + 0.5) * TICK));
      }
    }

    for (const link of links) {
      const lane = lanes.get(link.key)!;
      const barY = side === "top" ? m.top - GAP - lane * laneHeight.value : m.bottom + GAP + lane * laneHeight.value;
      const edgeY = side === "top" ? m.top : m.bottom;
      const xs = link.cols.map((col) => m.centers[col] + (offsets.get(`${link.key}@${col}`) ?? 0));
      const left = Math.min(...xs);
      const right = Math.max(...xs);
      const path = [`M ${left} ${barY} H ${right}`, ...xs.map((x) => `M ${x} ${barY} V ${edgeY}`)].join(" ");
      const centerX = (Math.min(...link.cols.map((c) => m.centers[c])) + Math.max(...link.cols.map((c) => m.centers[c]))) / 2;
      placed.push({
        ...link,
        lane,
        path,
        dots: xs.map((x) => ({ x, y: edgeY })),
        labelX: Math.min(Math.max(centerX, half(link) + 2), m.width - half(link) - 2),
        labelY: barY,
      });
    }
  }
  return placed;
});

const isDim = (key: string) => !!props.active && props.active !== key;
function setActive(key: string | null) {
  if (key !== (props.active ?? null)) emit("update:active", key);
}

function measure() {
  const root = rootRef.value;
  if (!root) return;
  const rootRect = root.getBoundingClientRect();
  const cells = Array.from(root.querySelectorAll<HTMLElement>("[data-relation-col]"));
  if (!cells.length) {
    metrics.value = null;
    return;
  }
  const centers: number[] = [];
  let top = Infinity;
  let bottom = -Infinity;
  for (const cell of cells) {
    const col = Number(cell.dataset.relationCol);
    const rect = cell.getBoundingClientRect();
    centers[col] = Math.round(rect.left - rootRect.left + rect.width / 2);
    top = Math.min(top, rect.top - rootRect.top);
    bottom = Math.max(bottom, rect.bottom - rootRect.top);
  }
  const fontScale = parseFloat(getComputedStyle(root).getPropertyValue("--app-font-scale")) || 1;
  const next: Metrics = {
    width: Math.round(rootRect.width),
    height: Math.round(rootRect.height),
    fontScale,
    centers,
    top: Math.round(top),
    bottom: Math.round(bottom),
  };
  const prev = metrics.value;
  if (
    !prev ||
    prev.width !== next.width ||
    prev.height !== next.height ||
    prev.top !== next.top ||
    prev.bottom !== next.bottom ||
    prev.fontScale !== next.fontScale ||
    prev.centers.join() !== next.centers.join()
  ) {
    metrics.value = next;
  }
}

let frame = 0;
function schedule() {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = 0;
    measure();
  });
}

let resizeObserver: ResizeObserver | null = null;
let styleObserver: MutationObserver | null = null;
function observe() {
  resizeObserver?.disconnect();
  const root = rootRef.value;
  if (!root || typeof ResizeObserver === "undefined") return;
  resizeObserver = new ResizeObserver(schedule);
  resizeObserver.observe(root);
  root.querySelectorAll("[data-relation-col]").forEach((cell) => resizeObserver?.observe(cell));
  schedule();
}

onMounted(() => {
  observe();
  // 字體大小滑桿改的是祖先元素的 --app-font-scale。
  if (typeof MutationObserver !== "undefined") {
    styleObserver = new MutationObserver(schedule);
    styleObserver.observe(document.body, { attributes: true, attributeFilter: ["style"], subtree: true });
  }
});
onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  styleObserver?.disconnect();
  if (frame) cancelAnimationFrame(frame);
});
watch(
  () => props.links.map((link) => link.key).join(","),
  () => nextTick(observe),
);
defineExpose({ remeasure: () => nextTick(observe) });
</script>

<style scoped>
.relation-brackets {
  position: relative;
}
.rb-svg {
  position: absolute;
  inset: 0;
  z-index: 2;
  overflow: visible;
  pointer-events: none;
}
.rb-link {
  transition: opacity 160ms ease;
}
.rb-link.is-preview {
  opacity: 0.7;
}
.rb-link.is-dim,
.rb-label.is-dim {
  opacity: 0.15;
}
.rb-link.is-active path {
  stroke-width: 3;
}
.rb-label {
  position: absolute;
  z-index: 3;
  transform: translate(-50%, -50%);
  margin: 0;
  padding: 0 8px;
  border: 1px solid var(--rb-color);
  border-radius: 999px;
  background: #fffdf8;
  color: var(--rb-color);
  font: inherit;
  font-size: calc(12px * var(--app-font-scale));
  font-weight: 700;
  line-height: 1.55;
  white-space: nowrap;
  cursor: pointer;
  transition: opacity 160ms ease, background 160ms ease;
}
.rb-arrow {
  margin-left: 3px;
  font-size: 0.9em;
}
.rb-arrow.is-good {
  color: #1f8a3b;
}
.rb-arrow.is-bad {
  color: #c53030;
}
.rb-arrow.is-mixed {
  color: #a16207;
}
.rb-label.is-active .rb-arrow {
  color: #fff;
}
.rb-label.is-preview {
  border-style: dashed;
}
.rb-label.is-active {
  background: var(--rb-color);
  color: #fff;
}
</style>
