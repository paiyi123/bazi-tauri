<template>
  <el-card shadow="hover" class="form-card">
    <template #header>
      <div class="card-heading">
        <div>
          <p class="eyebrow">BaZi Native</p>
          <h2>輸入出生資料</h2>
          <p>使用 Tauri + Rust + tyme4rs 在本機完成排盤，不再依賴外部 API。</p>
        </div>
      </div>
    </template>

    <el-form label-position="top" @submit.prevent="onSubmit">
      <el-row :gutter="12">
        <el-col :span="24">
          <el-form-item label="已儲存人物">
            <el-select
              v-model="selectedRecordId"
              clearable
              class="full-width"
              placeholder="選擇已存資料"
              @change="applySelectedRecord"
            >
              <el-option
                v-for="record in savedRecords"
                :key="record.id"
                :label="`${record.name}（${record.yearEra === 'ROC' ? '民國' : '西元'}${record.year}/${record.month}/${record.day}）`"
                :value="record.id"
              />
            </el-select>
            <div class="storage-label">儲存位置：本機 `localStorage`</div>
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="姓名（儲存用）">
            <el-input v-model="saveName" placeholder="輸入姓名以便儲存" />
          </el-form-item>
        </el-col>

        <el-col :span="12" class="save-button-wrap">
          <el-space wrap>
            <el-button type="success" plain @click="onSaveRecord">新增儲存</el-button>
            <el-button type="warning" plain :disabled="!selectedRecordId" @click="onUpdateRecord">
              更新所選
            </el-button>
            <el-button type="danger" plain :disabled="!selectedRecordId" @click="onDeleteRecord">
              刪除所選
            </el-button>
            <el-button plain @click="onClearLocalRecords">清除全部</el-button>
          </el-space>
        </el-col>

        <el-col :span="12">
          <el-form-item label="曆別">
            <el-select v-model="form.calendarType" class="full-width">
              <el-option label="公曆" value="SOLAR" />
              <el-option label="農曆" value="LUNAR" />
            </el-select>
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="性別">
            <el-select v-model="form.gender" class="full-width">
              <el-option label="男" value="MALE" />
              <el-option label="女" value="FEMALE" />
            </el-select>
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="紀元">
            <el-select v-model="form.yearEra" class="full-width">
              <el-option label="西元" value="AD" />
              <el-option label="民國" value="ROC" />
            </el-select>
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item :label="form.yearEra === 'ROC' ? '年（民國）' : '年（西元）'">
            <el-input-number
              v-model="form.year"
              :min="form.yearEra === 'ROC' ? 1 : 1"
              :max="form.yearEra === 'ROC' ? 300 : 9999"
              controls-position="right"
              class="full-width"
            />
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="月">
            <el-select
              v-if="form.calendarType === 'LUNAR'"
              v-model="selectedLunarMonthKey"
              class="full-width"
            >
              <el-option
                v-for="option in lunarMonthOptions"
                :key="`${option.value}-${option.leap}`"
                :label="option.label"
                :value="`${option.value}-${option.leap}`"
              />
            </el-select>
            <el-input-number
              v-else
              v-model="form.month"
              :min="1"
              :max="12"
              controls-position="right"
              class="full-width"
            />
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="日">
            <el-select v-if="form.calendarType === 'LUNAR'" v-model="form.day" class="full-width">
              <el-option
                v-for="option in lunarDayOptions"
                :key="option.value"
                :label="option.label"
                :value="option.value"
              />
            </el-select>
            <el-input-number
              v-else
              v-model="form.day"
              :min="1"
              :max="31"
              controls-position="right"
              class="full-width"
            />
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="時間輸入方式">
            <el-select v-model="timeInputMode" class="full-width">
              <el-option label="時分" value="HM" />
              <el-option label="時辰" value="SHICHEN" />
            </el-select>
          </el-form-item>
        </el-col>

        <template v-if="timeInputMode === 'HM'">
          <el-col :span="6">
            <el-form-item label="時">
              <el-input-number
                v-model="form.hour"
                :min="0"
                :max="23"
                controls-position="right"
                class="full-width"
              />
            </el-form-item>
          </el-col>
          <el-col :span="6">
            <el-form-item label="分">
              <el-input-number
                v-model="form.minute"
                :min="0"
                :max="59"
                controls-position="right"
                class="full-width"
              />
            </el-form-item>
          </el-col>
          <el-col :span="24">
            <el-alert
              :title="`目前對應時辰：${currentShiChenLabel}`"
              type="info"
              show-icon
              :closable="false"
            />
          </el-col>
        </template>

        <template v-else>
          <el-col :span="12">
            <el-form-item label="時辰">
              <el-select v-model="timePreset" class="full-width">
                <el-option label="日子（00:00–00:59）" value="ZI_EARLY" />
                <el-option label="丑時（01:00–02:59）" value="CHOU" />
                <el-option label="寅時（03:00–04:59）" value="YIN" />
                <el-option label="卯時（05:00–06:59）" value="MAO" />
                <el-option label="辰時（07:00–08:59）" value="CHEN" />
                <el-option label="巳時（09:00–10:59）" value="SI" />
                <el-option label="午時（11:00–12:59）" value="WU" />
                <el-option label="未時（13:00–14:59）" value="WEI" />
                <el-option label="申時（15:00–16:59）" value="SHEN" />
                <el-option label="酉時（17:00–18:59）" value="YOU" />
                <el-option label="戌時（19:00–20:59）" value="XU" />
                <el-option label="亥時（21:00–22:59）" value="HAI" />
                <el-option label="夜子（23:00–23:59）" value="ZI_LATE" />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="對應時間">
              <el-input :model-value="presetTimeLabel" readonly />
            </el-form-item>
          </el-col>
        </template>

        <el-col :span="12">
          <el-form-item label="日柱流派">
            <el-select v-model="form.baziSect" class="full-width">
              <el-option label="晚子時按當天" :value="2" />
              <el-option label="晚子時按明天" :value="1" />
            </el-select>
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="起運流派">
            <el-select v-model="form.yunSect" class="full-width">
              <el-option label="按天數/時辰" :value="1" />
              <el-option label="按分鐘數" :value="2" />
            </el-select>
          </el-form-item>
        </el-col>

        <el-col v-if="leapMonthLabel" :span="24">
          <el-alert :title="`此農曆年有 ${leapMonthLabel}`" type="info" show-icon :closable="false" />
        </el-col>

        <el-col v-if="timeInputMode === 'SHICHEN' && (timePreset === 'ZI_LATE' || timePreset === 'ZI_EARLY')" :span="24">
          <el-alert
            :title="timePreset === 'ZI_LATE' ? '夜子時使用 23:00，並套用日柱流派設定。' : '日子使用 00:00，屬當日。'"
            type="warning"
            show-icon
            :closable="false"
          />
        </el-col>

        <el-col v-if="saveMessage" :span="24">
          <el-alert :title="saveMessage" type="success" show-icon :closable="false" />
        </el-col>

        <el-col v-if="auxiliaryError" :span="24">
          <el-alert :title="auxiliaryError" type="error" show-icon :closable="false" />
        </el-col>

        <el-col v-if="error" :span="24">
          <el-alert :title="error" type="error" show-icon :closable="false" />
        </el-col>

        <el-col :span="24">
          <el-button type="primary" native-type="submit" :loading="loading" class="submit-button">
            開始排盤
          </el-button>
        </el-col>
      </el-row>
    </el-form>
  </el-card>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { getLunarMonthDetail, getLunarYearOptions } from "../services/bazi";
import type {
  BaziRequest,
  BirthRecordResponse,
  LunarDayOption,
  LunarMonthOption,
  TimePreset,
} from "../types/bazi";

const RECORDS_STORAGE_KEY = "bazi:birth-records:v1";
const DRAFT_STORAGE_KEY = "bazi:birth-form-draft:v1";

defineProps<{
  loading: boolean;
  error: string;
}>();

const emit = defineEmits<{
  submit: [payload: BaziRequest];
}>();

const defaultForm = (): BaziRequest => ({
  calendarType: "SOLAR",
  gender: "FEMALE",
  year: 1998,
  yearEra: "AD",
  month: 2,
  day: 4,
  hour: 23,
  minute: 0,
  second: 0,
  baziSect: 2,
  yunSect: 1,
  leapMonth: false,
});

function loadDraft() {
  try {
    const raw = window.localStorage.getItem(DRAFT_STORAGE_KEY);
    return raw ? { ...defaultForm(), ...JSON.parse(raw) } : defaultForm();
  } catch {
    return defaultForm();
  }
}

const form = reactive<BaziRequest>(loadDraft());
const lunarMonthOptions = ref<LunarMonthOption[]>([]);
const lunarDayOptions = ref<LunarDayOption[]>([]);
const selectedLunarMonthKey = ref(`${form.month}-${form.leapMonth}`);
const leapMonthLabel = ref("");
const timeInputMode = ref<"HM" | "SHICHEN">("SHICHEN");
const timePreset = ref<TimePreset>("ZI_LATE");
const saveName = ref("");
const saveMessage = ref("");
const auxiliaryError = ref("");
const savedRecords = ref<BirthRecordResponse[]>([]);
const selectedRecordId = ref<number | null>(null);

const timePresetMap: Record<TimePreset, { hour: number; minute: number; label: string }> = {
  ZI_EARLY: { hour: 0, minute: 0, label: "日子 00:00" },
  CHOU: { hour: 1, minute: 0, label: "丑時 01:00" },
  YIN: { hour: 3, minute: 0, label: "寅時 03:00" },
  MAO: { hour: 5, minute: 0, label: "卯時 05:00" },
  CHEN: { hour: 7, minute: 0, label: "辰時 07:00" },
  SI: { hour: 9, minute: 0, label: "巳時 09:00" },
  WU: { hour: 11, minute: 0, label: "午時 11:00" },
  WEI: { hour: 13, minute: 0, label: "未時 13:00" },
  SHEN: { hour: 15, minute: 0, label: "申時 15:00" },
  YOU: { hour: 17, minute: 0, label: "酉時 17:00" },
  XU: { hour: 19, minute: 0, label: "戌時 19:00" },
  HAI: { hour: 21, minute: 0, label: "亥時 21:00" },
  ZI_LATE: { hour: 23, minute: 0, label: "夜子 23:00" },
};

const presetTimeLabel = computed(() => timePresetMap[timePreset.value].label);

const currentShiChenLabel = computed(() => {
  if (form.hour === 23) return "夜子（23:00–23:59）";
  if (form.hour === 0) return "日子（00:00–00:59）";
  if (form.hour <= 2) return "丑時（01:00–02:59）";
  if (form.hour <= 4) return "寅時（03:00–04:59）";
  if (form.hour <= 6) return "卯時（05:00–06:59）";
  if (form.hour <= 8) return "辰時（07:00–08:59）";
  if (form.hour <= 10) return "巳時（09:00–10:59）";
  if (form.hour <= 12) return "午時（11:00–12:59）";
  if (form.hour <= 14) return "未時（13:00–14:59）";
  if (form.hour <= 16) return "申時（15:00–16:59）";
  if (form.hour <= 18) return "酉時（17:00–18:59）";
  if (form.hour <= 20) return "戌時（19:00–20:59）";
  return "亥時（21:00–22:59）";
});

function persistDraft() {
  window.localStorage.setItem(DRAFT_STORAGE_KEY, JSON.stringify(form));
}

function loadSavedRecords() {
  try {
    const raw = window.localStorage.getItem(RECORDS_STORAGE_KEY);
    savedRecords.value = raw ? (JSON.parse(raw) as BirthRecordResponse[]) : [];
  } catch {
    savedRecords.value = [];
    auxiliaryError.value = "讀取已儲存人物失敗";
  }
}

function persistSavedRecords() {
  window.localStorage.setItem(RECORDS_STORAGE_KEY, JSON.stringify(savedRecords.value));
}

function syncTimePresetToForm() {
  const mapped = timePresetMap[timePreset.value];
  form.hour = mapped.hour;
  if (timeInputMode.value === "SHICHEN") {
    form.minute = mapped.minute;
  }
}

function syncFormToTimePreset() {
  if (form.hour === 23) timePreset.value = "ZI_LATE";
  else if (form.hour === 0) timePreset.value = "ZI_EARLY";
  else if (form.hour <= 2) timePreset.value = "CHOU";
  else if (form.hour <= 4) timePreset.value = "YIN";
  else if (form.hour <= 6) timePreset.value = "MAO";
  else if (form.hour <= 8) timePreset.value = "CHEN";
  else if (form.hour <= 10) timePreset.value = "SI";
  else if (form.hour <= 12) timePreset.value = "WU";
  else if (form.hour <= 14) timePreset.value = "WEI";
  else if (form.hour <= 16) timePreset.value = "SHEN";
  else if (form.hour <= 18) timePreset.value = "YOU";
  else if (form.hour <= 20) timePreset.value = "XU";
  else timePreset.value = "HAI";
}

function currentRecordPayload() {
  if (timeInputMode.value === "SHICHEN") {
    syncTimePresetToForm();
  }
  return {
    name: saveName.value.trim(),
    gender: form.gender,
    calendarType: form.calendarType,
    yearEra: form.yearEra,
    year: form.year,
    month: form.month,
    day: form.day,
    hour: form.hour,
    minute: form.minute,
    leapMonth: form.leapMonth,
    timeLabel: timeInputMode.value === "SHICHEN" ? timePresetMap[timePreset.value].label : currentShiChenLabel.value,
    notes: "",
  };
}

function applyRecord(record: BirthRecordResponse) {
  form.calendarType = record.calendarType;
  form.gender = record.gender;
  form.yearEra = record.yearEra;
  form.year = record.year;
  form.month = record.month;
  form.day = record.day;
  form.hour = record.hour;
  form.minute = record.minute;
  form.leapMonth = record.leapMonth;
  saveName.value = record.name;
  syncFormToTimePreset();
}

function applySelectedRecord() {
  const record = savedRecords.value.find((item) => item.id === selectedRecordId.value);
  if (!record) return;
  applyRecord(record);
}

function onSaveRecord() {
  auxiliaryError.value = "";
  saveMessage.value = "";
  const payload = currentRecordPayload();

  if (!payload.name) {
    saveMessage.value = "請先輸入姓名";
    return;
  }

  const now = new Date().toISOString();
  const nextId = savedRecords.value.reduce((max, item) => Math.max(max, item.id), 0) + 1;
  const record: BirthRecordResponse = {
    ...payload,
    id: nextId,
    createdAt: now,
    updatedAt: now,
  };

  savedRecords.value = [record, ...savedRecords.value];
  persistSavedRecords();
  selectedRecordId.value = record.id;
  saveMessage.value = `已儲存：${record.name}`;
}

function onUpdateRecord() {
  auxiliaryError.value = "";
  saveMessage.value = "";
  const payload = currentRecordPayload();

  if (!selectedRecordId.value) {
    return;
  }
  if (!payload.name) {
    saveMessage.value = "請先輸入姓名";
    return;
  }

  const index = savedRecords.value.findIndex((item) => item.id === selectedRecordId.value);
  if (index === -1) {
    auxiliaryError.value = "找不到要更新的資料";
    return;
  }

  savedRecords.value[index] = {
    ...savedRecords.value[index],
    ...payload,
    updatedAt: new Date().toISOString(),
  };
  savedRecords.value = [...savedRecords.value];
  persistSavedRecords();
  saveMessage.value = `已更新：${payload.name}`;
}

function onDeleteRecord() {
  auxiliaryError.value = "";
  saveMessage.value = "";
  if (!selectedRecordId.value) {
    return;
  }

  const record = savedRecords.value.find((item) => item.id === selectedRecordId.value);
  savedRecords.value = savedRecords.value.filter((item) => item.id !== selectedRecordId.value);
  persistSavedRecords();
  selectedRecordId.value = null;
  saveMessage.value = record ? `已刪除：${record.name}` : "已刪除資料";
}

function onClearLocalRecords() {
  auxiliaryError.value = "";
  saveMessage.value = "";
  if (!savedRecords.value.length) {
    saveMessage.value = "目前沒有可清除的資料";
    return;
  }
  if (!window.confirm("這會刪除本機所有已儲存生日組，且無法復原。是否繼續？")) {
    return;
  }
  savedRecords.value = [];
  persistSavedRecords();
  selectedRecordId.value = null;
  saveMessage.value = "已清除本機生日組";
}

async function refreshLunarMonthOptions() {
  if (form.calendarType !== "LUNAR") {
    lunarMonthOptions.value = [];
    lunarDayOptions.value = [];
    leapMonthLabel.value = "";
    return;
  }

  try {
    auxiliaryError.value = "";
    const data = await getLunarYearOptions(form.year, form.yearEra);
    lunarMonthOptions.value = data.monthOptions;
    leapMonthLabel.value = data.leapMonth
      ? `閏${data.monthOptions.find((option) => option.value === data.leapMonth && option.leap)?.label.replace("閏", "") ?? ""}`
      : "";

    const matched =
      lunarMonthOptions.value.find(
        (option) => option.value === form.month && option.leap === form.leapMonth,
      ) ?? lunarMonthOptions.value[0];

    if (matched) {
      selectedLunarMonthKey.value = `${matched.value}-${matched.leap}`;
      form.month = matched.value;
      form.leapMonth = matched.leap;
      await refreshLunarDayOptions();
    }
  } catch (errorValue) {
    auxiliaryError.value = errorValue instanceof Error ? errorValue.message : "讀取農曆月份失敗";
  }
}

async function refreshLunarDayOptions() {
  if (form.calendarType !== "LUNAR") {
    return;
  }

  try {
    auxiliaryError.value = "";
    const data = await getLunarMonthDetail(form.year, form.yearEra, form.month, form.leapMonth);
    lunarDayOptions.value = data.dayOptions;
    if (form.day > data.dayCount) {
      form.day = data.dayCount;
    }
    if (form.day < 1) {
      form.day = 1;
    }
  } catch (errorValue) {
    auxiliaryError.value = errorValue instanceof Error ? errorValue.message : "讀取農曆日期失敗";
  }
}

watch(
  form,
  () => {
    persistDraft();
  },
  { deep: true },
);

watch(
  () => [form.calendarType, form.year, form.yearEra],
  async () => {
    await refreshLunarMonthOptions();
  },
);

watch(selectedLunarMonthKey, async (value) => {
  const [month, leap] = value.split("-");
  form.month = Number(month);
  form.leapMonth = leap === "true";
  await refreshLunarDayOptions();
});

watch(timePreset, () => {
  if (timeInputMode.value === "SHICHEN") {
    syncTimePresetToForm();
  }
});

watch(timeInputMode, () => {
  if (timeInputMode.value === "SHICHEN") {
    syncFormToTimePreset();
    syncTimePresetToForm();
  }
});

onMounted(async () => {
  loadSavedRecords();
  syncFormToTimePreset();
  await refreshLunarMonthOptions();
});

function onSubmit() {
  if (timeInputMode.value === "SHICHEN") {
    syncTimePresetToForm();
  }
  form.second = 0;
  emit("submit", { ...form });
}
</script>

<style scoped>
.save-button-wrap {
  display: flex;
  align-items: center;
}

.storage-label {
  margin-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

@media (max-width: 768px) {
  :deep(.el-row) {
    row-gap: 12px;
  }

  :deep(.el-col) {
    max-width: 100%;
    flex: 0 0 100%;
  }

  .save-button-wrap {
    align-items: stretch;
  }
}
</style>
