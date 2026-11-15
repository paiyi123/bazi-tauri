<template>
  <div class="learn-panel">
    <el-card shadow="never" class="learn-card">
      <template #header><h2>學習設定</h2></template>
      <div class="setting-row">
        <span class="setting-label">課程進度</span>
        <el-select v-model="courseStage" class="stage-select">
          <el-option
            v-for="stage in COURSE_STAGES"
            :key="stage.value"
            :label="stage.dates ? `${stage.label}（${stage.dates}）` : stage.label"
            :value="stage.value"
          />
        </el-select>
      </div>
      <p class="note">選「學到哪裡」，解盤 10 步裡還沒上到的步驟會收起來，點開仍可預覽。</p>
      <div class="setting-row">
        <span class="setting-label">練習模式</span>
        <el-switch v-model="practiceMode" />
      </div>
      <p class="note">打開後，解盤 10 步的「強弱」和「用神」會先藏答案，讓你自己判斷。</p>
    </el-card>

    <el-card shadow="never" class="learn-card">
      <template #header>
        <div class="head">
          <h2>反驗筆記</h2>
          <span v-if="checked.rate !== null" class="rate" :class="{ warn: checked.warn }">符合 {{ checked.hits }}／{{ checked.hits + checked.misses }}（{{ checked.rate }}%）</span>
        </div>
      </template>
      <p class="note">
        記下過去發生的事，和那一年的喜忌對照：好事落在喜年（▲）、壞事落在忌年（▼）算符合。婚戀、財務、健康只記錄不判斷。
        筆記只存在這台裝置。
      </p>
      <p v-if="checked.warn" class="warn-line">
        ⚠ 一半以上不符合。各書建議回頭檢查第 4 步（強弱）和第 6 步（用神），這是最容易判錯的兩步。
      </p>
      <form class="note-form" @submit.prevent="addNote">
        <el-input-number v-model="draft.year" :min="minYear" :max="maxYear" controls-position="right" class="year-input" />
        <el-select v-model="draft.kind" class="kind-select">
          <el-option v-for="kind in NOTE_KINDS" :key="kind.value" :label="kind.label" :value="kind.value" />
        </el-select>
        <el-input v-model="draft.text" placeholder="發生了什麼，例如：換工作、結婚、住院" class="text-input" />
        <el-button type="primary" native-type="submit" :disabled="!draft.text.trim()">記下</el-button>
      </form>
      <ul v-if="checked.notes.length" class="notes">
        <li v-for="note in checked.notes" :key="note.id" :class="`m-${note.match}`">
          <span class="year">{{ note.year }}<small v-if="note.view?.age != null">（{{ note.view.age }}歲）</small></span>
          <span class="kind">{{ kindLabel(note.kind) }}</span>
          <span class="text">{{ note.text }}</span>
          <span v-if="note.view" class="gz">
            運 {{ note.view.luck }} {{ marks(note.view.luckMarks) }}／年 {{ note.view.annual }} {{ marks(note.view.annualMarks) }}
          </span>
          <span v-else class="gz">這一年不在大運表內</span>
          <span class="match">{{ MATCH_LABEL[note.match] }}</span>
          <el-button link type="danger" size="small" @click="removeNote(note.id)">刪除</el-button>
        </li>
      </ul>
    </el-card>

    <el-card shadow="never" class="learn-card">
      <template #header><h2>書上相似命例</h2></template>
      <p class="note">從筆記裡約 2,900 個命盤找日干、月令、日柱相同的例子，可以對照書上怎麼論。檔案在 docs/books（也有一份在 bazi-research\docs）。四柱依「年 月 日 時」排列；按右邊的 → 直接用這組四柱排盤（性別沿用目前這張盤）。</p>
      <p v-if="loadingCases" class="note">載入中…</p>
      <ul v-else-if="similar.length" class="cases">
        <li v-for="item in similar" :key="`${item.file}:${item.line}`">
          <div class="case-head">
            <strong>{{ item.natal.join(" ") }}</strong>
            <span class="why">{{ item.why }}</span>
            <button type="button" class="case-import" :title="`用 ${item.natal.join(' ')} 排盤`" :aria-label="`用 ${item.natal.join(' ')} 排盤`" @click="openCase(item)">→</button>
          </div>
          <div class="case-src">《{{ item.book }}》{{ item.heading }}</div>
          <div class="case-file">{{ item.file }} 第 {{ item.line }} 行</div>
        </li>
      </ul>
      <p v-else class="note">沒有找到相似命例。</p>
    </el-card>

    <el-card shadow="never" class="learn-card">
      <template #header><h2>名詞表</h2></template>
      <p class="note">畫面上有虛線底線的詞都可以點一下看解釋。</p>
      <el-input v-model="query" placeholder="搜尋名詞" clearable class="search" />
      <dl class="glossary">
        <template v-for="entry in filteredGlossary" :key="entry.term">
          <dt>{{ entry.term }}<small v-if="entry.aliases?.length">（{{ entry.aliases.join("、") }}）</small></dt>
          <dd>{{ entry.text }}<small>出處：{{ entry.source }}</small></dd>
        </template>
      </dl>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import type { BaziResponse } from "../types/bazi";
import { baziYearAt } from "../utils/baziNow";
import { COURSE_STAGES, useLearning } from "../composables/useLearning";
import { GLOSSARY } from "../constants/glossary";
import { NOTE_KINDS, chartKey, checkNotes, type NoteKind, type VerifyNote } from "../utils/verifyNotes";
import { similarCases, type BookCase, type SimilarCase } from "../utils/similarCases";
import { usePillarImport } from "../composables/usePillarImport";
import type { FavorMark } from "../utils/conclusion";

const props = defineProps<{ result: BaziResponse }>();
const { courseStage, practiceMode } = useLearning();

// ---- 反驗筆記（localStorage，依命盤分開存） ----
const NOTES_KEY = "bazi:verify-notes:v1";
const MATCH_LABEL = { match: "✓ 符合", miss: "✗ 不符", none: "記錄" } as const;

function loadAll(): Record<string, VerifyNote[]> {
  try {
    return JSON.parse(window.localStorage.getItem(NOTES_KEY) ?? "{}") ?? {};
  } catch {
    return {};
  }
}

const allNotes = ref<Record<string, VerifyNote[]>>(loadAll());
const key = computed(() => chartKey(props.result));
const notes = computed(() => allNotes.value[key.value] ?? []);
const checked = computed(() => checkNotes(props.result, notes.value));

const years = computed(() => (props.result.daYun ?? []).flatMap((luck) => luck.liuNian?.map((item) => item.year) ?? []));
const thisYear = baziYearAt();
const minYear = computed(() => Math.min(...years.value, thisYear));
const maxYear = computed(() => Math.max(...years.value, thisYear));
const draft = reactive<{ year: number; kind: NoteKind; text: string }>({ year: thisYear - 1, kind: "good", text: "" });

function save(next: Record<string, VerifyNote[]>) {
  allNotes.value = next;
  try {
    window.localStorage.setItem(NOTES_KEY, JSON.stringify(next));
  } catch {
    // 無法儲存時這次仍可看到，但關掉就沒了
  }
}

function addNote() {
  const text = draft.text.trim();
  if (!text) return;
  const note: VerifyNote = { id: `${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, year: draft.year, kind: draft.kind, text };
  save({ ...allNotes.value, [key.value]: [...notes.value, note] });
  draft.text = "";
}

function removeNote(id: string) {
  save({ ...allNotes.value, [key.value]: notes.value.filter((note) => note.id !== id) });
}

const kindLabel = (kind: NoteKind) => NOTE_KINDS.find((item) => item.value === kind)?.label ?? kind;
const MARK = { good: "▲", bad: "▼", mixed: "◆", neutral: "・" } as const;
const marks = (list: FavorMark[]) => list.map((mark) => `${mark.char}${MARK[mark.favor]}`).join("");

// ---- 書上相似命例（索引約 600 KB，用到才載入） ----
const bookCases = ref<BookCase[]>([]);
const loadingCases = ref(true);
onMounted(async () => {
  try {
    bookCases.value = (await import("../data/bookCases.json")).default as BookCase[];
  } finally {
    loadingCases.value = false;
  }
});
const natal = computed(() => [props.result.yearPillar, props.result.monthPillar, props.result.dayPillar, props.result.hourPillar].map((p) => `${p?.stem ?? ""}${p?.branch ?? ""}`));
const similar = computed(() => similarCases(natal.value, bookCases.value));
const { importPillars } = usePillarImport();
// 索引裡的四柱是「年 月 日 時」
function openCase(item: SimilarCase) {
  const [year, month, day, hour] = item.natal;
  importPillars({ year, month, day, hour }, `《${item.book}》${item.heading}`);
}

// ---- 名詞表 ----
const query = ref("");
const filteredGlossary = computed(() => {
  const q = query.value.trim();
  if (!q) return GLOSSARY;
  return GLOSSARY.filter((entry) => entry.term.includes(q) || entry.text.includes(q) || entry.aliases?.some((alias) => alias.includes(q)));
});

watch(key, () => {
  draft.text = "";
});
</script>

<style scoped>
.learn-panel {
  display: grid;
  gap: 14px;
  min-width: 0;
  font-size: calc(15px * var(--app-font-scale));
  line-height: 1.6;
}
.learn-card {
  min-width: 0;
}
h2 {
  margin: 0;
  font-size: calc(18px * var(--app-font-scale));
}
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 4px 10px;
}
.rate {
  padding: 0 10px;
  border-radius: 999px;
  background: #e8f5ec;
  color: #23703c;
  font-weight: 600;
}
.rate.warn {
  background: #fdecea;
  color: var(--bazi-danger);
}
.note {
  margin: 0 0 8px;
  color: var(--bazi-text-muted);
  font-size: 0.9em;
}
.warn-line {
  margin: 0 0 8px;
  color: var(--bazi-danger);
  font-weight: 600;
}
.setting-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 12px;
  margin-bottom: 4px;
}
.setting-label {
  min-width: 4.5em;
  font-weight: 700;
}
.stage-select {
  width: min(320px, 100%);
}
.note-form {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
}
.year-input {
  width: 120px;
}
.kind-select {
  width: 100px;
}
.text-input {
  flex: 1 1 220px;
}
.notes,
.cases {
  display: grid;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}
.notes li {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 2px 10px;
  padding: 6px 10px;
  border-left: 4px solid var(--bazi-border-strong);
  border-radius: 8px;
  background: var(--bazi-surface-strong);
}
.notes li.m-match { border-left-color: #2f9e44; }
.notes li.m-miss { border-left-color: var(--bazi-danger); }
.year { font-weight: 700; }
.year small, .gz { color: var(--bazi-text-muted); font-size: 0.88em; }
.kind {
  padding: 0 6px;
  border-radius: 6px;
  background: var(--bazi-surface-muted);
  font-size: 0.85em;
}
.text { flex: 1 1 160px; }
.match { font-weight: 600; }
.m-match .match { color: #23703c; }
.m-miss .match { color: var(--bazi-danger); }
.cases li {
  padding: 6px 10px;
  border-radius: 8px;
  background: var(--bazi-surface-strong);
}
.case-head {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 10px;
  align-items: baseline;
}
.case-import {
  margin-left: auto;
  align-self: center;
  width: 32px;
  height: 32px;
  border: 1px solid #d97706;
  border-radius: 50%;
  background: #fff7ea;
  color: #b45309;
  font-size: 18px;
  font-weight: 700;
  line-height: 1;
  cursor: pointer;
}
.case-import:hover {
  background: #d97706;
  color: #fff;
}
.why, .case-file {
  color: var(--bazi-text-muted);
  font-size: 0.85em;
}
.search {
  max-width: 320px;
  margin-bottom: 10px;
}
.glossary {
  display: grid;
  grid-template-columns: minmax(5em, max-content) minmax(0, 1fr);
  gap: 6px 12px;
  margin: 0;
}
.glossary dt {
  font-weight: 700;
}
.glossary dt small {
  color: var(--bazi-text-muted);
  font-weight: 400;
}
.glossary dd {
  margin: 0;
}
.glossary dd small {
  display: block;
  color: var(--bazi-text-muted);
}
@media (max-width: 560px) {
  .glossary {
    grid-template-columns: 1fr;
  }
  .glossary dd {
    margin-bottom: 6px;
  }
}
</style>
