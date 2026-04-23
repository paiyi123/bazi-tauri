export type CalendarType = "SOLAR" | "LUNAR";
export type Gender = "MALE" | "FEMALE";
export type YearEra = "AD" | "ROC";
export type TimePreset =
  | "ZI_EARLY"
  | "CHOU"
  | "YIN"
  | "MAO"
  | "CHEN"
  | "SI"
  | "WU"
  | "WEI"
  | "SHEN"
  | "YOU"
  | "XU"
  | "HAI"
  | "ZI_LATE";

export interface BaziRequest {
  calendarType: CalendarType;
  gender: Gender;
  year: number;
  yearEra: YearEra;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
  baziSect: number;
  yunSect: number;
  leapMonth: boolean;
}

export interface PillarAnalyzeRequest {
  yearPillar: string;
  monthPillar: string;
  dayPillar: string;
  hourPillar: string;
  gender?: Gender;
  selectedGregorianYear?: number | null;
}

export interface Pillar {
  ganZhi: string;
  stem: string;
  branch: string;
  naYin: string;
  wuXing: string;
  diShi: string;
  xun: string;
  xunKong: string;
}

export interface LuckStart {
  forward: boolean | null;
  startYear: number | null;
  startMonth: number | null;
  startDay: number | null;
  startHour: number | null;
  startSolar: string | null;
  startSummary?: string | null;
  transitionSummary?: string | null;
  birthJieName?: string | null;
  birthJieSolar?: string | null;
  birthJieDayOrdinal?: number | null;
  transitionSummaryExperimental?: string | null;
}

export interface LiuNian {
  index: number;
  year: number;
  age: number;
  ganZhi: string;
  stemTenGod?: string;
  branchHiddenStems?: string[];
  branchTenGods?: string[];
  xun: string;
  xunKong: string;
  liuYue?: LiuYue[];
}

export interface LiuYue {
  index: number;
  jieQi?: string;
  jieQiDate?: string;
  month: string;
  ganZhi: string;
  stemTenGod?: string;
  branchHiddenStems?: string[];
  branchTenGods?: string[];
  xun: string;
  xunKong: string;
}

export interface DaYun {
  index: number;
  startAge: number | null;
  endAge: number | null;
  startYear: number | null;
  endYear: number | null;
  ganZhi: string;
  stemTenGod?: string;
  branchHiddenStems?: string[];
  branchTenGods?: string[];
  xun: string;
  xunKong: string;
  liuNian?: LiuNian[];
}

export interface BirthRecordRequest {
  name: string;
  gender: Gender;
  calendarType: CalendarType;
  yearEra: YearEra;
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  leapMonth: boolean;
  timeLabel?: string;
  notes?: string;
}

export interface BirthRecordResponse extends BirthRecordRequest {
  id: number;
  createdAt: string;
  updatedAt: string;
}

export interface LuckPreviewPillar {
  key: string;
  label: string;
  ganZhi: string;
  stemTenGod?: string;
  branchHiddenStems?: string[];
  branchTenGods?: string[];
  xun?: string;
  xunKong?: string;
  isCurrentYear?: boolean;
}

export interface QuantModelSubScore {
  item: string;
  hiddenStem?: string;
  tenGod?: string;
  ratio?: number;
  rawScore?: number;
  positionAdjustedScore?: number;
  interaction?: string;
  adjustmentScore?: number;
  finalContribution?: number;
  note?: string;
}

export interface QuantModelInteraction {
  scope: string;
  type: string;
  target: string;
  outcome: string;
  pillars: string;
  detail: string;
}

export interface QuantModelPillarScore {
  pillar: string;
  target: string;
  tenGod: string;
  baseScore: number;
  positionWeight: number;
  finalScore: number;
  details: string;
  category?: string;
  combineNote?: string;
  rawScore?: number;
  positionAdjustedScore?: number;
  clashAdjustment?: number;
  bonusScore?: number;
  rows?: QuantModelSubScore[];
}

export interface QuantAnnualLuckScore {
  index: number;
  year: number;
  age: number;
  ganZhi: string;
  annualStemTenGod?: string;
  annualBranchHiddenStems?: string[];
  annualBranchTenGods?: string[];
  activeLuckLabel: string;
  activeLuckScore: number;
  fullLuckScore: number;
  annualStemScore: number;
  annualBranchScore: number;
  annualTotalScore: number;
  combinedScore: number;
  fullCombinedScore: number;
  impactRatio: number;
  fullImpactRatio: number;
  effectiveNatalScore: number;
  fullEffectiveNatalScore: number;
  tendency: string;
  fullTendency: string;
  overviewLines?: string[];
  interactionLines?: string[];
  scoringLines?: string[];
  details: string;
}

export interface QuantLuckScore {
  index: number;
  ganZhi: string;
  startAge: number | null;
  endAge: number | null;
  stemTenGod?: string;
  branchHiddenStems?: string[];
  branchTenGods?: string[];
  firstHalfLabel: string;
  firstHalfScore: number;
  secondHalfLabel: string;
  secondHalfScore: number;
  stemScore: number;
  branchScore: number;
  totalScore: number;
  impactRatio: number;
  effectiveNatalScore: number;
  tendency: string;
  overviewLines?: string[];
  interactionLines?: string[];
  scoringLines?: string[];
  details: string;
  annualScores?: QuantAnnualLuckScore[];
}

export interface QuantYongShen {
  methodSummary: string;
  strengthBasis: string;
  printPresence: string;
  favorableTenGods: string[];
  favorableElements: string[];
  conditionalTenGods: string[];
  conditionalElements: string[];
  unfavorableTenGods: string[];
  unfavorableElements: string[];
  cautionTenGods: string[];
  cautionElements: string[];
  process: string;
  conclusion: string;
}

export interface QuantCongPattern {
  methodSummary: string;
  boundaryScore: number;
  trueBoundaryScore: number;
  pattern: string;
  authenticity: string;
  subtype: string;
  dominantFamily: string;
  primaryUseGods: string[];
  secondaryUseGods: string[];
  avoidGods: string[];
  riskNote: string;
  process: string;
  conclusion: string;
}

export interface QuantShaYin {
  methodSummary: string;
  stemPatternFound: boolean;
  stemTransformed: boolean;
  stemChain?: string;
  stemSourceNegativeScore?: number;
  stemSealSupportScore?: number;
  stemAdjustedTotalScore?: number;
  branchPatternFound: boolean;
  branchTransformed: boolean;
  branchChain?: string;
  branchSourceNegativeScore?: number;
  branchSealSupportScore?: number;
  branchAdjustedTotalScore?: number;
  process: string;
  conclusion: string;
}

export interface QuantModelResponse {
  dayMaster: string;
  summary: string;
  note: string;
  stemScoreTotal: number;
  branchScoreTotal: number;
  totalScore: number;
  strengthLabel: string;
  yongShen?: QuantYongShen;
  congPattern?: QuantCongPattern;
  shaYin?: QuantShaYin;
  interactions?: QuantModelInteraction[];
  stemScores: QuantModelPillarScore[];
  branchScores: QuantModelPillarScore[];
  luckScores?: QuantLuckScore[];
}

export interface ShenShaMatch {
  name: string;
  basis: string;
  matchedPillars: string[];
}

export interface FourPillarShenSha {
  note: string;
  year: string[];
  month: string[];
  day: string[];
  hour: string[];
  matches: ShenShaMatch[];
}

export interface DirectPillarYearHint {
  candidateYears: number[];
  selectedYear?: number | null;
  candidates?: DirectPillarBirthCandidate[];
  note: string;
}

export interface DirectPillarBirthCandidate {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
  solarDateTime: string;
  label: string;
}

export interface BaziResponse {
  inputCalendarType: string;
  inputDateTime: string;
  solarDateTime: string;
  lunarDateTime: string;
  baZi: string;
  dayMaster: string;
  geJu: string;
  geJuBasis: string;
  taiYuan: string;
  mingGong: string;
  shenGong: string;
  shenSha?: FourPillarShenSha;
  directPillarYearHint?: DirectPillarYearHint;
  yearPillar: Pillar;
  monthPillar: Pillar;
  dayPillar: Pillar;
  hourPillar: Pillar;
  yearHiddenStems: string[];
  monthHiddenStems: string[];
  dayHiddenStems: string[];
  hourHiddenStems: string[];
  yearStemTenGod: string;
  monthStemTenGod: string;
  dayStemTenGod: string;
  hourStemTenGod: string;
  yearBranchTenGods: string[];
  monthBranchTenGods: string[];
  dayBranchTenGods: string[];
  hourBranchTenGods: string[];
  luckStart: LuckStart;
  daYun: DaYun[];
  quantModel?: QuantModelResponse;
}

export interface LunarMonthOption {
  value: number;
  label: string;
  leap: boolean;
}

export interface LunarDayOption {
  value: number;
  label: string;
}

export interface LunarYearOptionsResponse {
  year: number;
  normalizedYear: number;
  yearEra: YearEra;
  leapMonth: number | null;
  monthOptions: LunarMonthOption[];
}

export interface LunarMonthDetailResponse {
  year: number;
  normalizedYear: number;
  yearEra: YearEra;
  month: number;
  leapMonth: boolean;
  dayCount: number;
  dayOptions: LunarDayOption[];
}

export interface PrintContext {
  source: "birth" | "pillars";
  name?: string;
  gender?: Gender;
  calendarType?: CalendarType | null;
  yearEra?: YearEra | null;
  inputText: string;
  timeLabel?: string;
  note?: string;
}
