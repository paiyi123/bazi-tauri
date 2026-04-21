import { invoke } from "@tauri-apps/api/core";
import {
  browserFixtureAnalyzePillars,
  browserFixtureCalculateBazi,
  browserFixtureGetLunarMonthDetail,
  browserFixtureGetLunarYearOptions,
} from "../debug/browser-fixtures";
import type {
  BaziRequest,
  BaziResponse,
  LunarMonthDetailResponse,
  LunarYearOptionsResponse,
  PillarAnalyzeRequest,
  YearEra,
} from "../types/bazi";

let browserDebugWarningShown = false;

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function shouldUseBrowserFixtures() {
  return import.meta.env.DEV && !isTauriRuntime();
}

function warnBrowserFixtureUsage() {
  if (browserDebugWarningShown) {
    return;
  }
  browserDebugWarningShown = true;
  console.warn(
    "[bazi] Tauri runtime not detected. Using browser debug fixtures instead of Rust invoke.",
  );
}

export async function calculateBazi(payload: BaziRequest): Promise<BaziResponse> {
  if (shouldUseBrowserFixtures()) {
    warnBrowserFixtureUsage();
    return browserFixtureCalculateBazi(payload);
  }
  return invoke<BaziResponse>("calculate_bazi", { request: payload });
}

export async function getLunarYearOptions(
  year: number,
  yearEra: YearEra,
): Promise<LunarYearOptionsResponse> {
  if (shouldUseBrowserFixtures()) {
    warnBrowserFixtureUsage();
    return browserFixtureGetLunarYearOptions(year, yearEra);
  }
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
  if (shouldUseBrowserFixtures()) {
    warnBrowserFixtureUsage();
    return browserFixtureGetLunarMonthDetail(year, yearEra, month, leapMonth);
  }
  return invoke<LunarMonthDetailResponse>("get_lunar_month_detail", {
    request: { year, yearEra, month, leapMonth },
  });
}

export async function analyzePillars(
  payload: PillarAnalyzeRequest,
): Promise<BaziResponse> {
  if (shouldUseBrowserFixtures()) {
    warnBrowserFixtureUsage();
    return browserFixtureAnalyzePillars(payload);
  }
  return invoke<BaziResponse>("analyze_pillars", { request: payload });
}
