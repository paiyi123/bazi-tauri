<template>
  <el-card v-if="items.length" shadow="never" class="disagreements">
    <template #header>
      <div class="head">
        <h2>這張盤各書看法不同的地方</h2>
        <small>對應《八字諸書融會貫通指南》分歧表；以課堂為準</small>
      </div>
    </template>
    <ul>
      <li v-for="item in items" :key="item.no" :class="{ ask: item.askTeacher }">
        <div class="title">
          <span class="no">#{{ item.no }}</span>
          <strong>{{ item.title }}</strong>
          <span v-if="item.askTeacher" class="ask-tag">待問老師</span>
        </div>
        <p><GlossaryText :text="item.why" /></p>
        <dl>
          <dt>課堂</dt>
          <dd><GlossaryText :text="item.classroom" /></dd>
          <dt>其他書</dt>
          <dd>{{ item.others }}</dd>
          <dt>建議</dt>
          <dd><GlossaryText :text="item.advice" /></dd>
        </dl>
      </li>
    </ul>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BaziResponse } from "../types/bazi";
import { disagreementsOf } from "../utils/disagreements";
import GlossaryText from "./GlossaryText.vue";

const props = defineProps<{ result: BaziResponse }>();
const items = computed(() => disagreementsOf(props.result));
</script>

<style scoped>
.disagreements {
  min-width: 0;
  font-size: calc(15px * var(--app-font-scale));
  line-height: 1.6;
}
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 4px 10px;
}
h2 {
  margin: 0;
  font-size: calc(18px * var(--app-font-scale));
}
small {
  color: var(--bazi-text-muted);
}
ul {
  display: grid;
  gap: 10px;
  margin: 0;
  padding: 0;
  list-style: none;
}
li {
  padding: 8px 12px;
  border-left: 4px solid var(--bazi-gold);
  border-radius: 8px;
  background: #fffaf0;
}
li.ask {
  border-left-color: #7c3aed;
}
.title {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px;
}
.no {
  color: var(--bazi-text-muted);
  font-size: 0.85em;
}
.ask-tag {
  padding: 0 8px;
  border-radius: 999px;
  background: #ede9fe;
  color: #5b21b6;
  font-size: 0.82em;
  font-weight: 600;
}
p {
  margin: 4px 0;
}
dl {
  display: grid;
  grid-template-columns: 3.6em minmax(0, 1fr);
  gap: 2px 8px;
  margin: 0;
  font-size: 0.92em;
}
dt {
  color: var(--bazi-text-muted);
  font-weight: 700;
}
dd {
  margin: 0;
}
</style>
