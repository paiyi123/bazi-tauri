import type { TimeCorrectionRequest } from "../types/bazi";

// 出生地（縣市政府所在地附近的經度，真太陽時用）。region "TW" 才套台灣夏令時間表。
export interface BirthPlace {
  name: string;
  group: string;
  longitude: number;
  utcOffsetMinutes: number;
  region?: "TW";
}

export const BIRTH_PLACES: BirthPlace[] = [
  { name: "台北", group: "台灣", longitude: 121.52, utcOffsetMinutes: 480, region: "TW" },
  { name: "新北", group: "台灣", longitude: 121.47, utcOffsetMinutes: 480, region: "TW" },
  { name: "基隆", group: "台灣", longitude: 121.74, utcOffsetMinutes: 480, region: "TW" },
  { name: "桃園", group: "台灣", longitude: 121.3, utcOffsetMinutes: 480, region: "TW" },
  { name: "新竹", group: "台灣", longitude: 120.97, utcOffsetMinutes: 480, region: "TW" },
  { name: "苗栗", group: "台灣", longitude: 120.82, utcOffsetMinutes: 480, region: "TW" },
  { name: "台中", group: "台灣", longitude: 120.68, utcOffsetMinutes: 480, region: "TW" },
  { name: "彰化", group: "台灣", longitude: 120.54, utcOffsetMinutes: 480, region: "TW" },
  { name: "南投", group: "台灣", longitude: 120.69, utcOffsetMinutes: 480, region: "TW" },
  { name: "雲林", group: "台灣", longitude: 120.53, utcOffsetMinutes: 480, region: "TW" },
  { name: "嘉義", group: "台灣", longitude: 120.45, utcOffsetMinutes: 480, region: "TW" },
  { name: "台南", group: "台灣", longitude: 120.21, utcOffsetMinutes: 480, region: "TW" },
  { name: "高雄", group: "台灣", longitude: 120.31, utcOffsetMinutes: 480, region: "TW" },
  { name: "屏東", group: "台灣", longitude: 120.49, utcOffsetMinutes: 480, region: "TW" },
  { name: "宜蘭", group: "台灣", longitude: 121.75, utcOffsetMinutes: 480, region: "TW" },
  { name: "花蓮", group: "台灣", longitude: 121.6, utcOffsetMinutes: 480, region: "TW" },
  { name: "台東", group: "台灣", longitude: 121.15, utcOffsetMinutes: 480, region: "TW" },
  { name: "澎湖", group: "台灣", longitude: 119.57, utcOffsetMinutes: 480, region: "TW" },
  { name: "金門", group: "台灣", longitude: 118.32, utcOffsetMinutes: 480, region: "TW" },
  { name: "馬祖", group: "台灣", longitude: 119.95, utcOffsetMinutes: 480, region: "TW" },
  { name: "香港", group: "其他", longitude: 114.17, utcOffsetMinutes: 480 },
  { name: "澳門", group: "其他", longitude: 113.54, utcOffsetMinutes: 480 },
  { name: "北京", group: "其他", longitude: 116.4, utcOffsetMinutes: 480 },
  { name: "上海", group: "其他", longitude: 121.47, utcOffsetMinutes: 480 },
  { name: "廣州", group: "其他", longitude: 113.26, utcOffsetMinutes: 480 },
  { name: "新加坡", group: "其他", longitude: 103.82, utcOffsetMinutes: 480 },
  { name: "吉隆坡", group: "其他", longitude: 101.69, utcOffsetMinutes: 480 },
  { name: "東京", group: "其他", longitude: 139.69, utcOffsetMinutes: 540 },
];

export const CUSTOM_PLACE = "自訂經度";

/** 新表單的預設：台北、夏令時間自動、不用真太陽時。 */
export function defaultTimeCorrection(): TimeCorrectionRequest {
  return { dst: "auto", region: "TW", placeName: "台北", longitude: 121.52, utcOffsetMinutes: 480, trueSolarTime: false };
}

/** 依地名套用經度、時區與地區；自訂經度時保留原本的經度。 */
export function applyPlace(current: TimeCorrectionRequest, name: string): TimeCorrectionRequest {
  const place = BIRTH_PLACES.find((item) => item.name === name);
  if (!place) return { ...current, placeName: CUSTOM_PLACE, region: null };
  return {
    ...current,
    placeName: place.name,
    longitude: place.longitude,
    utcOffsetMinutes: place.utcOffsetMinutes,
    region: place.region ?? null,
  };
}
