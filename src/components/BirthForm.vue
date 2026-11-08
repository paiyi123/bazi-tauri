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

    <el-form label-position="top" :size="formSize" @submit.prevent="onSubmit">
      <el-row :gutter="12">
        <el-col :span="24">
          <el-form-item label="已儲存人物" :class="{ 'compact-form-item': compact }">
            <div class="record-picker-row">
              <el-select
                v-model="selectedRecordId"
                clearable
                class="record-picker"
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
              <el-button :size="formSize" type="primary" :loading="loading" class="record-picker-submit" @click="onSubmit">
                開始排盤
              </el-button>
            </div>
            <div class="storage-label">儲存位置：本機 `localStorage`</div>
          </el-form-item>
        </el-col>

        <el-col :span="12">
          <el-form-item label="姓名（儲存用）" :class="{ 'compact-form-item': compact }">
            <el-input v-model="saveName" placeholder="輸入姓名以便儲存" />
          </el-form-item>
        </el-col>

        <el-col :span="12" class="save-button-wrap">
          <el-space direction="horizontal" wrap class="save-button-group">
            <el-button :size="formSize" type="success" plain @click="onSaveRecord">新增儲存</el-button>
            <el-button :size="formSize" type="warning" plain :disabled="!selectedRecordId" @click="onUpdateRecord">
              更新所選
            </el-button>
            <el-button :size="formSize" type="danger" plain :disabled="!selectedRecordId" @click="onDeleteRecord">
              刪除所選
            </el-button>
            <el-button :size="formSize" plain @click="onClearLocalRecords">清除全部</el-button>
            <el-button :size="formSize" type="primary" plain :disabled="!savedRecords.length" @click="onOpenShare">
              分享 QR
            </el-button>
            <el-button :size="formSize" type="primary" plain @click="onOpenImport">匯入命例</el-button>
          </el-space>
        </el-col>

        <el-dialog
          v-model="shareDialogVisible"
          title="分享命例"
          width="min(460px, 94vw)"
          append-to-body
          class="record-share-dialog"
        >
          <template v-if="shareStep === 'select'">
            <div class="share-select-toolbar">
              <span>已選 {{ shareSelectedIds.length }} / {{ savedRecords.length }} 筆</span>
              <el-space>
                <el-button size="small" link type="primary" @click="shareSelectedIds = savedRecords.map((item) => item.id)">全選</el-button>
                <el-button size="small" link @click="shareSelectedIds = []">清除</el-button>
              </el-space>
            </div>
            <el-input v-model="shareFilter" size="small" clearable placeholder="搜尋姓名" class="share-filter" />
            <el-checkbox-group v-model="shareSelectedIds" class="share-record-list">
              <el-checkbox v-for="record in filteredShareRecords" :key="record.id" :value="record.id" class="share-record-item">
                {{ recordLabel(record) }}
              </el-checkbox>
            </el-checkbox-group>
            <p class="share-hint">
              一張 QR Code 最多放 {{ QR_MAX_RECORDS }} 筆，選比較多時會分成好幾張，逐張掃描即可。
            </p>
          </template>

          <template v-else>
            <p class="share-name">
              第 {{ sharePage + 1 }} / {{ shareChunks.length }} 張（{{ currentShareChunk?.records.length ?? 0 }} 筆）
            </p>
            <div v-if="shareQrSvg" class="share-qr" v-html="shareQrSvg"></div>
            <el-alert v-else-if="shareError" :title="shareError" type="warning" show-icon :closable="false" />
            <p class="share-names">{{ currentShareChunk?.records.map((item) => item.name).join("、") }}</p>
            <div v-if="shareChunks.length > 1" class="share-pager">
              <el-button size="small" :disabled="sharePage === 0" @click="showSharePage(sharePage - 1)">上一張</el-button>
              <el-button size="small" :disabled="sharePage >= shareChunks.length - 1" @click="showSharePage(sharePage + 1)">
                下一張
              </el-button>
            </div>
            <p class="share-hint">在另一台裝置按「匯入命例」→「掃描 QR Code」，或複製分享碼貼過去。</p>
            <el-input :model-value="currentShareChunk?.code ?? ''" type="textarea" :rows="3" readonly />
          </template>

          <template #footer>
            <template v-if="shareStep === 'select'">
              <el-button @click="shareDialogVisible = false">取消</el-button>
              <el-button type="primary" :disabled="!shareSelectedIds.length" @click="onGenerateShare">
                產生 QR Code（{{ shareSelectedIds.length }} 筆{{ sharePreviewCount > 1 ? `，共 ${sharePreviewCount} 張` : "" }}）
              </el-button>
            </template>
            <template v-else>
              <el-button @click="shareStep = 'select'">重新選擇</el-button>
              <el-button @click="onCopyShareCode(false)">複製這張</el-button>
              <el-button v-if="shareChunks.length > 1" @click="onCopyShareCode(true)">複製全部</el-button>
              <el-button type="primary" @click="shareDialogVisible = false">完成</el-button>
            </template>
          </template>
        </el-dialog>

        <el-dialog
          v-model="importDialogVisible"
          title="匯入命例"
          width="min(420px, 92vw)"
          append-to-body
          class="record-share-dialog"
        >
          <el-button v-if="scanAvailable" type="primary" class="full-width scan-button" :loading="scanning" @click="onScanImport">
            掃描 QR Code
          </el-button>
          <p class="share-hint">{{ scanAvailable ? "或貼上" : "貼上" }}另一台裝置的命例分享碼（BAZI1: 開頭）：</p>
          <el-input v-model="importCode" type="textarea" :rows="3" placeholder='BAZI1:["王小明","F","S","A",1970,11,6,5,0,0,"卯時 05:00",""]' />
          <el-alert v-if="importError" :title="importError" type="error" show-icon :closable="false" class="import-error" />
          <template #footer>
            <el-button @click="importDialogVisible = false">取消</el-button>
            <el-button type="primary" :disabled="!importCode.trim()" @click="onImportCode">匯入</el-button>
          </template>
        </el-dialog>

        <Teleport to="body">
          <div v-if="cameraOpen" class="qr-scan-overlay">
            <p class="qr-scan-tip">將 QR Code 對準框內</p>
            <div class="qr-scan-frame" />
            <button type="button" class="qr-scan-cancel" @click="onCancelScan">取消掃描</button>
          </div>
        </Teleport>

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
          <el-button :size="formSize" type="primary" native-type="submit" :loading="loading" class="submit-button">
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
import { canScanRecordQr, cancelRecordQrScan, getRecordShareQrSvg, scanRecordQr } from "../services/recordShare";
import { QR_MAX_RECORDS, chunkRecordsForShare, decodeRecordsShare, importSharedRecords } from "../utils/recordShare";
import type {
  BaziRequest,
  BirthRecordResponse,
  LunarDayOption,
  LunarMonthOption,
  PrintContext,
  TimePreset,
} from "../types/bazi";

const RECORDS_STORAGE_KEY = "bazi:birth-records:v1";
const DRAFT_STORAGE_KEY = "bazi:birth-form-draft:v1";

const props = defineProps<{
  loading: boolean;
  error: string;
  compact?: boolean;
  prefillDraft?: BaziRequest | null;
}>();

const emit = defineEmits<{
  submit: [payload: BaziRequest];
  printContext: [payload: PrintContext];
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
const formSize = computed(() => (props.compact ? "small" : "default"));

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

async function applyBirthDraft(draft: BaziRequest) {
  form.calendarType = draft.calendarType;
  form.gender = draft.gender;
  form.yearEra = draft.yearEra;
  form.year = draft.year;
  form.month = draft.month;
  form.day = draft.day;
  form.hour = draft.hour;
  form.minute = draft.minute;
  form.second = draft.second;
  form.baziSect = draft.baziSect;
  form.yunSect = draft.yunSect;
  form.leapMonth = draft.leapMonth;
  timeInputMode.value = "SHICHEN";
  syncFormToTimePreset();
  await refreshLunarMonthOptions();
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

const shareDialogVisible = ref(false);
const shareStep = ref<"select" | "qr">("select");
const shareSelectedIds = ref<number[]>([]);
const shareFilter = ref("");
const shareChunks = ref<ReturnType<typeof chunkRecordsForShare>>([]);
const sharePage = ref(0);
const shareQrSvg = ref("");
const shareError = ref("");
const importDialogVisible = ref(false);
const importCode = ref("");
const importError = ref("");
const scanning = ref(false);
const cameraOpen = ref(false);
const scanAvailable = canScanRecordQr();

const filteredShareRecords = computed(() => {
  const keyword = shareFilter.value.trim();
  return keyword ? savedRecords.value.filter((item) => item.name.includes(keyword)) : savedRecords.value;
});
const shareSelectedRecords = computed(() =>
  savedRecords.value.filter((item) => shareSelectedIds.value.includes(item.id)),
);
const sharePreviewCount = computed(() => chunkRecordsForShare(shareSelectedRecords.value).length);
const currentShareChunk = computed(() => shareChunks.value[sharePage.value]);

function recordLabel(record: BirthRecordResponse) {
  return `${record.name}（${record.yearEra === "ROC" ? "民國" : "西元"}${record.year}/${record.month}/${record.day}）`;
}

function onOpenShare() {
  shareStep.value = "select";
  shareFilter.value = "";
  shareSelectedIds.value = selectedRecordId.value ? [selectedRecordId.value] : [];
  shareDialogVisible.value = true;
}

async function showSharePage(page: number) {
  sharePage.value = page;
  shareQrSvg.value = "";
  shareError.value = "";
  const chunk = shareChunks.value[page];
  if (!chunk) return;
  try {
    const svg = await getRecordShareQrSvg(chunk.code);
    if (sharePage.value === page) shareQrSvg.value = svg;
  } catch (errorValue) {
    shareError.value = errorValue instanceof Error ? errorValue.message : String(errorValue);
  }
}

async function onGenerateShare() {
  shareChunks.value = chunkRecordsForShare(shareSelectedRecords.value);
  shareStep.value = "qr";
  await showSharePage(0);
}

async function onCopyShareCode(all: boolean) {
  const text = all ? shareChunks.value.map((chunk) => chunk.code).join("\n") : currentShareChunk.value?.code ?? "";
  try {
    await navigator.clipboard.writeText(text);
    saveMessage.value = all ? `已複製全部 ${shareChunks.value.length} 張的分享碼` : "已複製分享碼";
  } catch {
    saveMessage.value = "無法自動複製，請長按分享碼手動複製";
  }
}

function onOpenImport() {
  importCode.value = "";
  importError.value = "";
  importDialogVisible.value = true;
}

function importFromCode(text: string) {
  importError.value = "";
  try {
    const incoming = decodeRecordsShare(text);
    const result = importSharedRecords(savedRecords.value, incoming);
    if (result.added.length) {
      savedRecords.value = result.records;
      persistSavedRecords();
    }
    const first = result.added[0] ?? result.duplicates[0];
    if (first) {
      selectedRecordId.value = first.id;
      applyRecord(first);
    }
    auxiliaryError.value = "";
    if (incoming.length === 1) {
      saveMessage.value = result.added.length ? `已匯入：${recordLabel(first)}` : `已經有這筆命例：${recordLabel(first)}`;
    } else {
      saveMessage.value = `已匯入 ${result.added.length} 筆${result.duplicates.length ? `，${result.duplicates.length} 筆原本就有` : ""}`;
    }
    importDialogVisible.value = false;
  } catch (errorValue) {
    importError.value = errorValue instanceof Error ? errorValue.message : String(errorValue);
  }
}

function onImportCode() {
  importFromCode(importCode.value);
}

async function onScanImport() {
  importError.value = "";
  scanning.value = true;
  try {
    const content = await scanRecordQr({
      onCameraOpen: () => {
        cameraOpen.value = true;
      },
    });
    importCode.value = content;
    importFromCode(content);
  } catch (errorValue) {
    const message = errorValue instanceof Error ? errorValue.message : String(errorValue);
    importError.value = /cancel/i.test(message) ? "" : `掃描失敗：${message}`;
  } finally {
    cameraOpen.value = false;
    scanning.value = false;
  }
}

function onCancelScan() {
  void cancelRecordQrScan();
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

watch(
  () => form.calendarType,
  (calendarType, previousCalendarType) => {
    if (calendarType !== "LUNAR" || previousCalendarType === "LUNAR") {
      return;
    }
    if (form.yearEra === "AD") {
      form.year = Math.max(1, form.year - 1911);
    }
    form.yearEra = "ROC";
  },
);

watch(
  () => props.prefillDraft,
  async (draft) => {
    if (!draft) {
      return;
    }
    await applyBirthDraft(draft);
  },
  { immediate: true },
);

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
  emit("printContext", {
    source: "birth",
    name: saveName.value.trim() || undefined,
    gender: form.gender,
    calendarType: form.calendarType,
    yearEra: form.yearEra,
    inputText:
      `${form.calendarType === "SOLAR" ? "公曆" : "農曆"} ` +
      `${form.yearEra === "ROC" ? "民國" : "西元"}${form.year}年${form.leapMonth ? "閏" : ""}${form.month}月${form.day}日 ` +
      `${String(form.hour).padStart(2, "0")}:${String(form.minute).padStart(2, "0")}`,
    timeLabel: timeInputMode.value === "SHICHEN" ? timePresetMap[timePreset.value].label : currentShiChenLabel.value,
  });
  emit("submit", { ...form });
}
</script>

<style scoped>
.save-button-wrap {
  display: flex;
  align-items: center;
}

.save-button-group {
  width: 100%;
}

.record-picker-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.record-picker {
  flex: 1 1 auto;
  min-width: 0;
}

.record-picker-submit {
  flex: none;
  min-width: 140px;
}

@media (max-width: 768px) {
  .record-picker-submit {
    min-width: 0;
  }
}

.share-name {
  margin: 0 0 8px;
  font-weight: 600;
  text-align: center;
}

.share-select-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.share-filter {
  margin-bottom: 8px;
}

.share-record-list {
  display: flex;
  flex-direction: column;
  max-height: 45vh;
  overflow-y: auto;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 6px;
  padding: 4px 10px;
}

.share-record-item {
  margin-right: 0;
  height: auto;
  min-height: 30px;
}

.share-names {
  margin: 0 0 8px;
  text-align: center;
  font-size: calc(13px * var(--app-font-scale));
}

.share-pager {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 4px;
}

.share-qr {
  display: flex;
  justify-content: center;
  margin: 0 auto 8px;
  width: min(320px, 80vw);
}

.share-qr :deep(svg) {
  width: 100%;
  height: auto;
}

.share-hint {
  margin: 8px 0;
  font-size: calc(13px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
}

.scan-button {
  margin-bottom: 4px;
}

.import-error {
  margin-top: 8px;
}

.storage-label {
  margin-top: 8px;
  font-size: calc(12px * var(--app-font-scale));
  color: var(--el-text-color-secondary);
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
  :deep(.el-select__wrapper),
  :deep(.el-textarea__inner),
  :deep(.el-input-number) {
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

  .save-button-wrap {
    align-items: stretch;
  }

  .save-button-group {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
  }

  .save-button-group :deep(.el-space__item) {
    margin: 0 !important;
    min-width: 0;
  }

  .save-button-group :deep(.el-button) {
    width: 100%;
    justify-content: center;
    margin: 0;
  }

  .storage-label {
    font-size: calc(11px * var(--app-font-scale));
  }
}

@media (max-width: 480px) {
  :deep(.el-row) {
    row-gap: 8px;
  }

  :deep(.el-form-item) {
    margin-bottom: 10px;
  }

  :deep(.el-input__wrapper),
  :deep(.el-select__wrapper),
  :deep(.el-textarea__inner),
  :deep(.el-input-number) {
    min-height: 32px;
  }

  .save-button-wrap {
    margin-top: -2px;
  }
}


.form-card :deep(.el-card__header) {
  background: linear-gradient(180deg, rgba(255, 247, 234, 0.92), rgba(255, 247, 234, 0));
}

.form-card :deep(.el-form-item__label) {
  color: var(--bazi-primary);
  font-weight: 800;
}

.storage-label {
  color: var(--bazi-text-muted);
}

.save-button-group :deep(.el-button) {
  min-height: 38px;
}
</style>

<style>
/* 掃描 QR Code 時：網頁全透明，只露出掃描遮罩，相機畫面在網頁後面。 */
html.qr-scanning,
html.qr-scanning body {
  background: transparent !important;
}

html.qr-scanning body > *:not(.qr-scan-overlay) {
  visibility: hidden !important;
}

.qr-scan-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 28px;
  padding: 24px;
}

.qr-scan-tip {
  margin: 0;
  padding: 8px 16px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
  font-size: 16px;
}

.qr-scan-frame {
  width: min(70vw, 280px);
  aspect-ratio: 1;
  border: 3px solid rgba(255, 255, 255, 0.9);
  border-radius: 16px;
  box-shadow: 0 0 0 100vmax rgba(0, 0, 0, 0.35);
}

.qr-scan-cancel {
  min-width: 180px;
  padding: 14px 28px;
  border: none;
  border-radius: 999px;
  background: #fff;
  color: #7a2e14;
  font-size: 17px;
  font-weight: 600;
}
</style>
