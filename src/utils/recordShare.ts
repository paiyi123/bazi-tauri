import type { BirthRecordRequest, BirthRecordResponse } from "../types/bazi";

// 命例分享碼：「BAZI1:」加上一個精簡的 JSON 陣列，可放進 QR Code，也能直接複製貼上。
// 欄位順序：[姓名, 性別 M/F, 曆別 S/L, 紀元 A/R, 年, 月, 日, 時, 分, 閏月 0/1, 時辰標籤, 備註]
export const RECORD_SHARE_PREFIX = "BAZI1:";

function recordFields(record: BirthRecordRequest) {
  return [
    record.name,
    record.gender === "MALE" ? "M" : "F",
    record.calendarType === "LUNAR" ? "L" : "S",
    record.yearEra === "ROC" ? "R" : "A",
    record.year,
    record.month,
    record.day,
    record.hour,
    record.minute,
    record.leapMonth ? 1 : 0,
    record.timeLabel ?? "",
    record.notes ?? "",
  ];
}

// 單筆：BAZI1:[...]；多筆：BAZI1:[[...],[...]]
export function encodeRecordShare(record: BirthRecordRequest): string {
  return RECORD_SHARE_PREFIX + JSON.stringify(recordFields(record));
}

export function encodeRecordsShare(records: BirthRecordRequest[]): string {
  if (records.length === 1) return encodeRecordShare(records[0]);
  return RECORD_SHARE_PREFIX + JSON.stringify(records.map(recordFields));
}

// 一張 QR Code 最多放幾筆、幾個位元組（手機對著螢幕還能穩定掃到的大小）。
export const QR_MAX_RECORDS = 10;
export const QR_MAX_BYTES = 900;

const utf8Length = (text: string) => new TextEncoder().encode(text).length;

// 把選到的命例分成好幾張 QR Code，每張都能單獨掃描匯入。
export function chunkRecordsForShare(
  records: BirthRecordRequest[],
  maxRecords = QR_MAX_RECORDS,
  maxBytes = QR_MAX_BYTES,
): { records: BirthRecordRequest[]; code: string }[] {
  const chunks: { records: BirthRecordRequest[]; code: string }[] = [];
  let current: BirthRecordRequest[] = [];
  for (const record of records) {
    const candidate = [...current, record];
    if (current.length && (candidate.length > maxRecords || utf8Length(encodeRecordsShare(candidate)) > maxBytes)) {
      chunks.push({ records: current, code: encodeRecordsShare(current) });
      current = [record];
    } else {
      current = candidate;
    }
  }
  if (current.length) chunks.push({ records: current, code: encodeRecordsShare(current) });
  return chunks;
}

function isInt(value: unknown, min: number, max: number): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= min && value <= max;
}

// 文字裡可以有好幾段分享碼（例如一次複製了好幾張 QR Code 的內容）。
function parseShareJson(text: string): unknown[][] {
  const parts = text.split(RECORD_SHARE_PREFIX).slice(1);
  if (!parts.length) {
    throw new Error("這不是命例分享碼（應以 BAZI1: 開頭）");
  }
  return parts.map((part) => {
    let fields: unknown;
    try {
      fields = JSON.parse(part.trim());
    } catch {
      throw new Error("命例分享碼不完整，請重新掃描或複製");
    }
    if (!Array.isArray(fields) || fields.length < 1) {
      throw new Error("命例分享碼欄位不足");
    }
    return fields;
  });
}

function fieldsToRecord(fields: unknown): BirthRecordRequest {
  if (!Array.isArray(fields) || fields.length < 10) {
    throw new Error("命例分享碼欄位不足");
  }

  const [name, gender, calendar, era, year, month, day, hour, minute, leap, timeLabel, notes] = fields;
  if (typeof name !== "string" || !name.trim()) throw new Error("命例分享碼缺少姓名");
  if (gender !== "M" && gender !== "F") throw new Error("命例分享碼的性別不正確");
  if (calendar !== "S" && calendar !== "L") throw new Error("命例分享碼的曆別不正確");
  if (era !== "A" && era !== "R") throw new Error("命例分享碼的紀元不正確");
  if (!isInt(year, 1, 9999)) throw new Error("命例分享碼的年份不正確");
  if (!isInt(month, 1, 12)) throw new Error("命例分享碼的月份不正確");
  if (!isInt(day, 1, 31)) throw new Error("命例分享碼的日期不正確");
  if (!isInt(hour, 0, 23)) throw new Error("命例分享碼的時間不正確");
  if (!isInt(minute, 0, 59)) throw new Error("命例分享碼的分鐘不正確");
  if (leap !== 0 && leap !== 1) throw new Error("命例分享碼的閏月欄位不正確");

  return {
    name: name.trim(),
    gender: gender === "M" ? "MALE" : "FEMALE",
    calendarType: calendar === "L" ? "LUNAR" : "SOLAR",
    yearEra: era === "R" ? "ROC" : "AD",
    year,
    month,
    day,
    hour,
    minute,
    leapMonth: leap === 1,
    timeLabel: typeof timeLabel === "string" ? timeLabel : "",
    notes: typeof notes === "string" ? notes : "",
  };
}

// 解析分享碼（單筆或多筆）；格式不對時丟出中文錯誤訊息。
export function decodeRecordsShare(text: string): BirthRecordRequest[] {
  const items = parseShareJson(text).flatMap((fields) => (Array.isArray(fields[0]) ? fields : [fields]));
  return items.map((item, index) => {
    try {
      return fieldsToRecord(item);
    } catch (error) {
      if (items.length === 1) throw error;
      throw new Error(`第 ${index + 1} 筆：${error instanceof Error ? error.message : String(error)}`);
    }
  });
}

export function decodeRecordShare(text: string): BirthRecordRequest {
  return decodeRecordsShare(text)[0];
}

// 出生資料（不含姓名、備註）完全相同就視為同一筆命例。
export function sameBirthData(a: BirthRecordRequest, b: BirthRecordRequest): boolean {
  return (
    a.gender === b.gender &&
    a.calendarType === b.calendarType &&
    a.yearEra === b.yearEra &&
    a.year === b.year &&
    a.month === b.month &&
    a.day === b.day &&
    a.hour === b.hour &&
    a.minute === b.minute &&
    a.leapMonth === b.leapMonth
  );
}

export type RecordImportResult =
  | { kind: "duplicate"; record: BirthRecordResponse; records: BirthRecordResponse[] }
  | { kind: "added"; record: BirthRecordResponse; records: BirthRecordResponse[] };

// 把匯入的命例放進清單：同名且出生資料相同就不重複新增，其餘一律新增一筆。
export function importSharedRecord(
  records: BirthRecordResponse[],
  incoming: BirthRecordRequest,
  now: string = new Date().toISOString(),
): RecordImportResult {
  const existing = records.find((item) => item.name === incoming.name && sameBirthData(item, incoming));
  if (existing) {
    return { kind: "duplicate", record: existing, records };
  }
  const nextId = records.reduce((max, item) => Math.max(max, item.id), 0) + 1;
  const record: BirthRecordResponse = { ...incoming, id: nextId, createdAt: now, updatedAt: now };
  return { kind: "added", record, records: [record, ...records] };
}

// 一次匯入多筆：回傳新清單，以及新增和已存在的命例。
export function importSharedRecords(
  records: BirthRecordResponse[],
  incoming: BirthRecordRequest[],
  now: string = new Date().toISOString(),
) {
  let list = records;
  const added: BirthRecordResponse[] = [];
  const duplicates: BirthRecordResponse[] = [];
  for (const item of incoming) {
    const result = importSharedRecord(list, item, now);
    list = result.records;
    (result.kind === "added" ? added : duplicates).push(result.record);
  }
  return { records: list, added, duplicates };
}
