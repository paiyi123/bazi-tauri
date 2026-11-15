<template>
  <el-card shadow="hover" class="form-card">
    <template #header>
      <div class="card-heading">
        <div>
          <h2>直接輸入四柱</h2>
          <p>適合已知四柱的命例、書本資料與量化測試。</p>
        </div>
      </div>
    </template>

    <el-form label-position="top" :size="formSize" @submit.prevent="onSubmit">
      <el-row :gutter="12">
        <el-col :span="12">
          <el-form-item label="性別">
            <el-select v-model="form.gender" class="full-width">
              <el-option label="男" value="MALE" />
              <el-option label="女" value="FEMALE" />
            </el-select>
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-alert
            title="每柱由天干與地支組成，直接輸入後會用 Rust 本機邏輯分析。"
            type="info"
            :closable="false"
            show-icon
          />
        </el-col>

        <el-col :span="24">
          <!-- 直接貼上書上的四柱，自動拆成年月日時 -->
          <div class="paste-box">
            <div class="paste-title">貼上四柱</div>
            <el-input
              v-model="pasteText"
              :size="formSize"
              placeholder="例如：壬午 己酉 甲申 甲子"
              clearable
              @keyup.enter="applyPaste(true)"
            />
            <div class="paste-row">
              <span class="paste-label">書上的順序</span>
              <el-radio-group v-model="pasteOrder" :size="formSize">
                <el-radio-button v-for="(label, key) in ORDER_LABEL" :key="key" :value="key">{{ label }}</el-radio-button>
              </el-radio-group>
            </div>
            <div class="paste-row">
              <el-button :size="formSize" :disabled="!pasteText.trim()" @click="applyPaste(false)">填入下面</el-button>
              <el-button :size="formSize" type="primary" :disabled="!pasteText.trim()" :loading="loading" @click="applyPaste(true)">填入並排盤</el-button>
            </div>
            <div v-if="pasteMessage" :class="['paste-note', { 'is-error': pasteError }]">{{ pasteMessage }}</div>
          </div>
        </el-col>

        <el-col :span="24">
          <el-alert :title="`目前四柱：${currentBaZi}`" type="success" :closable="false" show-icon />
        </el-col>

        <el-col :span="24">
          <div class="year-hint-row">
            <el-button :size="formSize" :loading="inferLoading" @click="inferGregorianYears">推算西元年</el-button>
            <el-select v-model="selectedCandidateKey" class="year-select" placeholder="不加年" clearable>
              <el-option label="不加年" :value="null" />
              <el-option
                v-for="candidate in inferredCandidates"
                :key="candidate.solarDateTime"
                :label="candidate.label"
                :value="candidate.solarDateTime"
              />
            </el-select>
            <el-button :size="formSize" :disabled="!selectedCandidate" @click="applyCandidateToBirthForm">
              帶回生日輸入
            </el-button>
          </div>
          <div v-if="candidateDateText" class="year-hint-text candidate-date-text">{{ candidateDateText }}</div>
          <div v-if="yearHintText" class="year-hint-text">{{ yearHintText }}</div>
        </el-col>

        <template v-for="pillar in pillarFields" :key="pillar.key">
          <el-col :span="24">
            <div class="pillar-row-title">{{ pillar.label }}：{{ getPillarValue(pillar.key) }}</div>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="`${pillar.label}天干`">
              <el-select v-model="form[pillar.stemKey]" class="full-width">
                <el-option v-for="stem in stems" :key="stem" :label="stem" :value="stem" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item :label="`${pillar.label}地支`">
              <el-select v-model="form[pillar.branchKey]" class="full-width">
                <el-option v-for="branch in branches" :key="branch" :label="branch" :value="branch" />
              </el-select>
            </el-form-item>
          </el-col>
        </template>

        <el-col :span="24" class="action-row">
          <el-button :size="formSize" type="primary" native-type="submit" :loading="loading" class="submit-button">
            開始分析四柱
          </el-button>
          <el-button :size="formSize" @click="resetSavedInput">清除暫存</el-button>
        </el-col>

        <el-col v-if="error" :span="24">
          <el-alert :title="error" type="error" show-icon :closable="false" />
        </el-col>
      </el-row>
    </el-form>
  </el-card>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { analyzePillars } from "../services/bazi";
import type { BaziRequest, DirectPillarBirthCandidate, PillarAnalyzeRequest, PrintContext } from "../types/bazi";
import { ORDER_LABEL, parsePillarText, type PillarOrder } from "../utils/pillarText";

const STORAGE_KEY = "bazi:pillar-input:v1";

const props = defineProps<{
  loading: boolean;
  error: string;
  compact?: boolean;
}>();

const emit = defineEmits<{
  submit: [payload: PillarAnalyzeRequest];
  applyBirthDraft: [payload: BaziRequest];
  printContext: [payload: PrintContext];
}>();

const stems = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
const branches = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

const pillarFields = [
  { key: "year", label: "年柱", stemKey: "yearStem", branchKey: "yearBranch" },
  { key: "month", label: "月柱", stemKey: "monthStem", branchKey: "monthBranch" },
  { key: "day", label: "日柱", stemKey: "dayStem", branchKey: "dayBranch" },
  { key: "hour", label: "時柱", stemKey: "hourStem", branchKey: "hourBranch" },
] as const;

const defaultForm = () => ({
  yearStem: "甲",
  yearBranch: "子",
  monthStem: "甲",
  monthBranch: "戌",
  dayStem: "戊",
  dayBranch: "寅",
  hourStem: "丙",
  hourBranch: "辰",
  gender: "FEMALE" as PillarAnalyzeRequest["gender"],
});

function loadSavedForm() {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    return raw ? { ...defaultForm(), ...JSON.parse(raw) } : defaultForm();
  } catch {
    return defaultForm();
  }
}

const form = reactive(loadSavedForm());
const formSize = computed(() => (props.compact ? "small" : "default"));
const inferLoading = ref(false);
const inferredCandidates = ref<DirectPillarBirthCandidate[]>([]);
const inferredYearNote = ref("");
const selectedCandidateKey = ref<string | null>(null);

watch(
  form,
  (value) => {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  },
  { deep: true },
);

function getPillarValue(key: "year" | "month" | "day" | "hour") {
  return `${form[`${key}Stem` as const]}${form[`${key}Branch` as const]}`;
}

const currentBaZi = computed(
  () =>
    `${getPillarValue("year")} ${getPillarValue("month")} ${getPillarValue("day")} ${getPillarValue("hour")}`,
);
const selectedCandidate = computed(
  () => inferredCandidates.value.find((candidate) => candidate.solarDateTime === selectedCandidateKey.value) || null,
);
const yearHintText = computed(() => {
  if (inferredCandidates.value.length) {
    return `候選西元年：${inferredCandidates.value.map((candidate) => candidate.year).join("、")}${inferredYearNote.value ? `；${inferredYearNote.value}` : ""}`;
  }
  return inferredYearNote.value;
});
const candidateDateText = computed(() => {
  if (!inferredCandidates.value.length) {
    return "";
  }
  return `候選生日：${inferredCandidates.value.map((candidate) => candidate.label).join("、")}`;
});

async function inferGregorianYears() {
  inferLoading.value = true;
  inferredYearNote.value = "";
  try {
    const response = await analyzePillars({
      yearPillar: getPillarValue("year"),
      monthPillar: getPillarValue("month"),
      dayPillar: getPillarValue("day"),
      hourPillar: getPillarValue("hour"),
      gender: form.gender,
      inferGregorianYears: true,
    });
    inferredCandidates.value = response.directPillarYearHint?.candidates || [];
    inferredYearNote.value =
      response.directPillarYearHint?.note || (inferredCandidates.value.length ? "" : "找不到可用年份");
    if (
      selectedCandidateKey.value &&
      !inferredCandidates.value.some((candidate) => candidate.solarDateTime === selectedCandidateKey.value)
    ) {
      selectedCandidateKey.value = null;
    }
    if (!selectedCandidateKey.value && inferredCandidates.value.length) {
      selectedCandidateKey.value = inferredCandidates.value[0].solarDateTime;
    }
  } catch (error) {
    inferredCandidates.value = [];
    inferredYearNote.value = error instanceof Error ? error.message : "推算西元年失敗";
  } finally {
    inferLoading.value = false;
  }
}

function applyCandidateToBirthForm() {
  if (!selectedCandidate.value) {
    return;
  }
  emit("applyBirthDraft", {
    calendarType: "SOLAR",
    gender: form.gender || "FEMALE",
    yearEra: "AD",
    year: selectedCandidate.value.year,
    month: selectedCandidate.value.month,
    day: selectedCandidate.value.day,
    hour: selectedCandidate.value.hour,
    minute: selectedCandidate.value.minute,
    second: selectedCandidate.value.second,
    baziSect: 2,
    yunSect: 1,
    leapMonth: false,
  });
}

function onSubmit() {
  emit("printContext", {
    source: "pillars",
    gender: form.gender,
    inputText: `四柱：${currentBaZi.value}`,
  });
  emit("submit", {
    yearPillar: getPillarValue("year"),
    monthPillar: getPillarValue("month"),
    dayPillar: getPillarValue("day"),
    hourPillar: getPillarValue("hour"),
    gender: form.gender,
    selectedGregorianYear: null,
  });
}

// ---- 貼上四柱 ----
const pasteText = ref("");
const pasteOrder = ref<PillarOrder>("auto");
const pasteMessage = ref("");
const pasteError = ref(false);
function applyPaste(submit: boolean) {
  const parsed = parsePillarText(pasteText.value, pasteOrder.value);
  if (!parsed.ok) {
    pasteMessage.value = parsed.error;
    pasteError.value = true;
    return;
  }
  const { year, month, day, hour } = parsed.pillars;
  Object.assign(form, {
    yearStem: year[0],
    yearBranch: year[1],
    monthStem: month[0],
    monthBranch: month[1],
    dayStem: day[0],
    dayBranch: day[1],
    hourStem: hour[0],
    hourBranch: hour[1],
  });
  inferredCandidates.value = [];
  inferredYearNote.value = "";
  selectedCandidateKey.value = null;
  pasteError.value = parsed.note.startsWith("⚠");
  pasteMessage.value = `已填入：年 ${year}、月 ${month}、日 ${day}、時 ${hour}。${parsed.note}`;
  if (submit) onSubmit();
}

function resetSavedInput() {
  Object.assign(form, defaultForm());
  inferredCandidates.value = [];
  inferredYearNote.value = "";
  selectedCandidateKey.value = null;
  window.localStorage.removeItem(STORAGE_KEY);
}
</script>

<style scoped>
.paste-box {
  display: grid;
  gap: 8px;
  padding: 10px 12px;
  border: 1px dashed #d9c7a6;
  border-radius: 10px;
  background: #fffbf3;
}
.paste-title {
  font-weight: 700;
}
.paste-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.paste-label {
  color: #526071;
  font-size: calc(12px * var(--app-font-scale));
}
.paste-note {
  color: #23703c;
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.6;
}
.paste-note.is-error {
  color: #b45309;
}

.pillar-row-title {
  font-weight: 600;
  margin: 4px 0 2px;
}

.action-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.year-hint-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
}

.year-select {
  min-width: 220px;
}

.year-hint-text {
  margin-top: 8px;
  font-size: calc(12px * var(--app-font-scale));
  line-height: 1.6;
  color: #526071;
}

.candidate-date-text {
  color: #253046;
  font-weight: 500;
}

@media (max-width: 768px) {
  :deep(.el-form-item) {
    margin-bottom: 14px;
  }

  :deep(.el-form-item__label) {
    padding-bottom: 4px;
    font-size: calc(12px * var(--app-font-scale));
    line-height: 1.3;
  }

  :deep(.el-input__wrapper),
  :deep(.el-select__wrapper) {
    min-height: 34px;
  }

  :deep(.el-alert) {
    padding: 8px 10px;
  }

  :deep(.el-alert__title) {
    font-size: calc(12px * var(--app-font-scale));
    line-height: 1.4;
  }

  :deep(.el-row) {
    row-gap: 12px;
  }

  :deep(.el-col) {
    max-width: 100%;
    flex: 0 0 100%;
  }

  .pillar-row-title {
    font-size: calc(13px * var(--app-font-scale));
  }

  .action-row :deep(.el-button) {
    width: 100%;
    justify-content: center;
  }

  .year-hint-row {
    align-items: stretch;
  }

  .year-hint-row :deep(.el-button),
  .year-select {
    width: 100%;
  }
}

@media (max-width: 480px) {
  :deep(.el-form-item) {
    margin-bottom: 10px;
  }

  :deep(.el-row) {
    row-gap: 8px;
  }

  :deep(.el-input__wrapper),
  :deep(.el-select__wrapper) {
    min-height: 32px;
  }
}


.pillar-row-title {
  color: var(--bazi-primary);
  letter-spacing: 0.04em;
}

.action-row :deep(.el-button),
.year-hint-row :deep(.el-button) {
  min-height: 38px;
}

.year-hint-text {
  color: var(--bazi-text-muted);
}

.candidate-date-text {
  color: var(--bazi-primary-strong);
  font-weight: 800;
}
</style>
