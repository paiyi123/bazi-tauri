import { invoke } from "@tauri-apps/api/core";
import type {
  BaziRequest,
  BaziResponse,
  LunarMonthDetailResponse,
  LunarYearOptionsResponse,
  PillarAnalyzeRequest,
  YearEra,
} from "../types/bazi";

export async function calculateBazi(payload: BaziRequest): Promise<BaziResponse> {
  return invoke<BaziResponse>("calculate_bazi", { request: payload });
}

export async function getLunarYearOptions(
  year: number,
  yearEra: YearEra,
): Promise<LunarYearOptionsResponse> {
  return invoke<LunarYearOptionsResponse>("get_lunar_year_options", {
    request: { year, yearEra },
  });
}

export async function getLunarMonthDetail(
  year: number,
  yearEra: YearEra,
  month: number,
  leapMonth: boolean,
): Promise<LunarMonthDetailResponse> {
  return invoke<LunarMonthDetailResponse>("get_lunar_month_detail", {
    request: { year, yearEra, month, leapMonth },
  });
}

export async function analyzePillars(
  payload: PillarAnalyzeRequest,
): Promise<BaziResponse> {
  return invoke<BaziResponse>("analyze_pillars", { request: payload });
}
