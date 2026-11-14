<template>
  <el-card shadow="never" class="conclusion-card" :class="{ compact }">
    <template #header>
      <div class="head">
        <h2>本命結論<small>課堂三步法</small></h2>
        <span v-if="data" class="agree" :class="`is-${data.agreement.status}`" :title="data.agreement.detail">
          {{ agreementIcon }} {{ data.agreement.text }}
        </span>
      </div>
    </template>

    <template v-if="data">
      <dl class="rows">
        <div class="row">
          <dt>日主</dt>
          <dd>
            <strong class="big" :class="`wx-${dayElement}`">{{ data.dayMaster }}</strong>
            <span v-if="data.pattern" class="sub">{{ data.pattern }}</span>
          </dd>
        </div>
        <div class="row">
          <dt>強弱</dt>
          <dd>
            <strong class="big">{{ data.strength.label }}</strong>
            <span class="sub">{{ data.strength.checks }}</span>
            <span class="sub">{{ data.strength.party }}</span>
          </dd>
        </div>
        <div class="row">
          <dt>用神</dt>
          <dd>
            <span v-for="item in data.useElements" :key="item.element" class="wx-chip solid" :class="`wx-${item.element}`">{{ item.name }}</span>
            <span v-if="!data.useElements.length" class="sub">{{ data.noPlanReason ?? "要再比較條件方案" }}</span>
          </dd>
        </div>
        <div class="row">
          <dt>喜</dt>
          <dd>
            <span v-for="item in data.favorable" :key="item.element" class="wx-chip" :class="`wx-${item.element}`">{{ item.name }}</span>
            <span v-if="!data.favorable.length" class="sub">—</span>
            <template v-if="data.undecided.length">
              <span class="sub gap">未定</span>
              <span v-for="item in data.undecided" :key="item.element" class="wx-chip dashed" :class="`wx-${item.element}`">{{ item.name }}</span>
            </template>
          </dd>
        </div>
        <div class="row">
          <dt>忌</dt>
          <dd>
            <span v-for="item in data.unfavorable" :key="item.element" class="wx-chip" :class="`wx-${item.element}`">{{ item.name }}</span>
            <span v-if="!data.unfavorable.length" class="sub">—</span>
          </dd>
        </div>
      </dl>

      <div v-if="data.luck || data.annual" class="now">
        <div v-for="item in [data.luck, data.annual].filter(Boolean)" :key="item!.label" class="now-item">
          <span class="now-label">{{ item!.label }}</span>
          <span class="now-gz">
            <span v-for="mark in item!.marks" :key="mark.char" :class="`wx-${mark.element}`">{{ mark.char }}</span>
          </span>
          <span v-if="item!.range" class="sub">{{ item!.range }}</span>
          <span class="favor" :class="`v-${item!.overall}`">{{ favorText(item!) }}</span>
        </div>
      </div>

      <ul v-if="data.highlights.length" class="highlights">
        <li v-for="item in data.highlights" :key="item.label" :title="item.detail">
          <strong>{{ item.label }}</strong>
          <span class="sub">{{ item.detail }}</span>
        </li>
      </ul>

      <el-collapse class="why">
        <el-collapse-item title="為什麼？" name="why">
          <p v-if="data.strength.reason">{{ data.strength.reason }}</p>
          <p>{{ data.agreement.detail }}</p>
          <p class="sub">
            主答案以許羽賢老師課堂三步法為準（定格 → 判強弱 → 取用神、明喜忌）。量化五行力與陸致極只用來看是否一致；
            ▲▼ 只依喜忌判斷，大運以地支為主。詳細推論在「解盤」各步驟。
          </p>
        </el-collapse-item>
      </el-collapse>
    </template>

    <div v-else class="empty">
      <p><strong class="big">{{ result.dayMaster }}</strong> 日主</p>
      <p class="sub">這張命盤沒有課堂三步分析的結果，所以無法給強弱與喜忌結論。</p>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { STEM_WUXING } from "../constants/wuxing";
import { conclusionOf, type ConclusionLuck } from "../utils/conclusion";

const props = defineProps<{ result: BaziResponse; compact?: boolean }>();

const data = computed(() => conclusionOf(props.result));
const dayElement = computed(() => STEM_WUXING[props.result.dayPillar?.stem ?? ""] ?? "earth");
const agreementIcon = computed(() => ({ agree: "✓", split: "⚠", unknown: "・" })[data.value?.agreement.status ?? "unknown"]);

function favorText(item: ConclusionLuck): string {
  const marks = item.marks.map((mark) => `${mark.char}${({ good: "▲", bad: "▼", mixed: "◆", neutral: "・" })[mark.favor]}`).join(" ");
  const word = { good: "喜", bad: "忌", mixed: "干支不同", neutral: "未列喜忌" }[item.overall];
  return `${word}（${marks}）`;
}
</script>

<style scoped>
.conclusion-card {
  min-width: 0;
  border: 1px solid var(--bazi-border-strong);
  background: linear-gradient(180deg, #fffaf0 0%, var(--bazi-surface-strong) 100%);
  font-size: calc(15px * var(--app-font-scale));
  line-height: 1.6;
  overflow-wrap: anywhere;
}
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 6px 10px;
}
h2 {
  margin: 0;
  color: var(--bazi-primary);
  font-size: calc(20px * var(--app-font-scale));
}
h2 small {
  margin-left: 8px;
  color: var(--bazi-text-subtle);
  font-size: 0.62em;
  font-weight: 500;
}
.agree {
  padding: 1px 10px;
  border-radius: 999px;
  font-size: 0.86em;
  font-weight: 600;
  white-space: nowrap;
}
.agree.is-agree { background: #e8f5ec; color: #23703c; }
.agree.is-split { background: #fff3d6; color: #8a5a00; }
.agree.is-unknown { background: var(--bazi-surface-muted); color: var(--bazi-text-muted); }
.rows {
  display: grid;
  gap: 6px;
  margin: 0;
}
.row {
  display: grid;
  grid-template-columns: 2.6em minmax(0, 1fr);
  gap: 8px;
  align-items: baseline;
}
dt {
  color: var(--bazi-text-muted);
  font-weight: 700;
}
dd {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 8px;
  margin: 0;
  min-width: 0;
}
.big {
  font-size: 1.18em;
}
.sub {
  color: var(--bazi-text-muted);
  font-size: 0.88em;
}
.gap {
  margin-left: 6px;
}
.wx-chip {
  display: inline-block;
  min-width: 1.9em;
  padding: 0 0.45em;
  border: 1px solid currentColor;
  border-radius: 999px;
  font-weight: 700;
  text-align: center;
}
.wx-chip.solid {
  color: #fff !important;
}
.wx-chip.solid.wx-wood { background: var(--wx-wood); border-color: var(--wx-wood); }
.wx-chip.solid.wx-fire { background: var(--wx-fire); border-color: var(--wx-fire); }
.wx-chip.solid.wx-earth { background: var(--wx-earth); border-color: var(--wx-earth); }
.wx-chip.solid.wx-metal { background: var(--wx-metal); border-color: var(--wx-metal); }
.wx-chip.solid.wx-water { background: var(--wx-water); border-color: var(--wx-water); }
.wx-chip.dashed {
  border-style: dashed;
}
.wx-wood { color: var(--wx-wood); }
.wx-fire { color: var(--wx-fire); }
.wx-earth { color: var(--wx-earth); }
.wx-metal { color: var(--wx-metal); }
.wx-water { color: var(--wx-water); }
.now {
  display: grid;
  gap: 4px;
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px dashed var(--bazi-border);
}
.now-item {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 2px 8px;
}
.now-label {
  min-width: 4.2em;
  color: var(--bazi-text-muted);
  font-weight: 700;
}
.now-gz {
  font-size: 1.12em;
  font-weight: 700;
  letter-spacing: 0.05em;
}
.favor {
  font-weight: 600;
}
.v-good { color: #23703c; }
.v-bad { color: var(--bazi-danger); }
.v-mixed, .v-neutral { color: var(--bazi-text-muted); }
.highlights {
  margin: 8px 0 0;
  padding: 0;
  list-style: none;
}
.highlights li {
  padding: 4px 8px;
  border-left: 3px solid var(--bazi-accent);
  background: #fff6e6;
}
.highlights li + li {
  margin-top: 4px;
}
.highlights .sub {
  display: block;
}
.why {
  margin-top: 8px;
  --el-collapse-header-height: 36px;
  border: none;
}
.why p {
  margin: 0 0 6px;
}
.empty p {
  margin: 0 0 4px;
}
.compact .highlights .sub {
  display: none;
}
</style>
