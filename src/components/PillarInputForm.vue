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
          <el-alert :title="`目前四柱：${currentBaZi}`" type="success" :closable="false" show-icon />
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
import { computed, reactive, watch } from "vue";
import type { PillarAnalyzeRequest } from "../types/bazi";

const STORAGE_KEY = "bazi:pillar-input:v1";

const props = defineProps<{
  loading: boolean;
  error: string;
  compact?: boolean;
}>();

const emit = defineEmits<{
  submit: [payload: PillarAnalyzeRequest];
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

function onSubmit() {
  emit("submit", {
    yearPillar: getPillarValue("year"),
    monthPillar: getPillarValue("month"),
    dayPillar: getPillarValue("day"),
    hourPillar: getPillarValue("hour"),
    gender: form.gender,
  });
}

function resetSavedInput() {
  Object.assign(form, defaultForm());
  window.localStorage.removeItem(STORAGE_KEY);
}
</script>

<style scoped>
.pillar-row-title {
  font-weight: 600;
  margin: 4px 0 2px;
}

.action-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

@media (max-width: 768px) {
  :deep(.el-form-item) {
    margin-bottom: 14px;
  }

  :deep(.el-form-item__label) {
    padding-bottom: 4px;
    font-size: 12px;
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
    font-size: 12px;
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
    font-size: 13px;
  }

  .action-row :deep(.el-button) {
    width: 100%;
    justify-content: center;
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
</style>
