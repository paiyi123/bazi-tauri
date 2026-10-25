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
  inferGregorianYears?: boolean;
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

export interface QuantSpecialPattern {
  primaryPattern?: string | null;
  category: string;
  authenticity: string;
  normalPatternReference: string;
  specialUseGods: string[];
  specialAvoidGods: string[];
  process: string;
  conclusion: string;
  rejectionReasons: string[];
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

export interface QuantLessonInference {
  ruleId: string;
  title: string;
  status: string;
  sourceVerification: string;
  algorithmStatus: "candidate" | "verified" | "implemented" | "blocked";
  matchStatus: "matched" | "not_matched" | "not_evaluated" | "blocked";
  outputMode: "annotation_only" | "documentation_only";
  fixtureScope: "none" | "source_case_only" | "rule_validation";
  agreementStatus: "agree" | "partial" | "conflict" | "not_comparable";
  fallbackPolicy: "preserve_normal_outputs";
  appReference: {
    geJu: string;
    strengthLabel: string;
    specialPattern: string;
  };
  lessonCandidate?: {
    pattern: string;
    strength: string;
    useElements: string[];
  };
  sensitiveClaimsPolicy: "excluded";
  conclusion: string;
  evidence: string[];
  rejectedAlternatives: string[];
}

export interface QuantLessonContent {
  title: string;
  status: string;
  analysisOrder: string[];
  futureRuleNotice: string;
  inferences: QuantLessonInference[];
  threeStepAnalysis?: LessonThreeStepAnalysis | null;
}

export interface LessonEvidence { id: string; kind: string; pillar: string | null; hiddenStemLayer: string | null; interactionId: string | null; description: string; sourceDate: string | null; }
export interface LessonSource { date: string; ruleIds: string[]; }
export interface LessonHiddenStemFact { stem: string; element: string; tenGod: string; layer: string; }
export interface LessonPillarFact { pillar: string; ganZhi: string; stem: string; stemElement: string; stemTenGod: string; branch: string; branchMainElement: string; hiddenStems: LessonHiddenStemFact[]; }
export interface LessonFacts { title: string; conclusion: string; pillars: LessonPillarFact[]; dayMaster: { stem: string; element: string }; monthCommand: { branch: string; mainStem: string; element: string; tenGod: string }; transparencies: Array<{ hiddenStem: string; hiddenStemLayer: string; tenGod: string; visibleLocations: string[]; evidenceId: string }>; roots: Array<{ pillar: string; branch: string; hiddenStem: string; hiddenStemLayer: string; element: string; tenGod: string; rootKind: string; evidenceId: string }>; tenGodDistribution: Array<{ family: string; element: string; surfaceLocations: string[]; hiddenLocations: string[]; surfaceEvidenceIds: string[]; hiddenEvidenceIds: string[] }>; evidence: LessonEvidence[]; limitations: string[]; }
export interface LessonPatternCandidate { pattern: string; tenGod: string; monthHiddenStem: string; hiddenStemLayer: string; transparentLocations: string[]; selection: string; evidenceIds: string[]; }
export interface LessonPatternAnalysis { title: string; conclusion: string; status: string; ordinaryResult: string | null; ordinaryCandidates: LessonPatternCandidate[]; specialReview: { required: boolean; reasons: string[]; modelReferencePattern: string | null; modelReferenceCategory: string | null; specializedUseVersions: Array<{ sourceDate: string; useDirection: string; favorableDirection: string; limitation: string }>; followChildPeerException: string }; evidence: LessonEvidence[]; limitations: string[]; }
export interface LessonInteractionEvidence { id: string; scope: string; interactionType: string; target: string; outcome: string; pillars: string; detail: string; assessment: string; }
export interface LessonStrengthCheck { item: string; hit: boolean; rule: string; finding: string; sourceDate: string; }
export interface LessonStrengthAnalysis { title: string; conclusion: string; status: string; classification: string; sameSeasonElement: boolean; sealInSeason: boolean; supportStemCount: number; supportBranchCount: number; samePartyCount: number; opposingPartyCount: number; monthCommandSameParty: boolean; momentumProxy: boolean; rootCount: number; observationCount: number; samePartyCharacters: string[]; opposingPartyCharacters: string[]; checks: LessonStrengthCheck[]; verdictReason: string; pressureFamiliesSurface: string[]; pressureFamiliesHidden: string[]; interactions: LessonInteractionEvidence[]; evidence: LessonEvidence[]; limitations: string[]; }
export interface LessonDirection { family: string; element: string; role: string; reason: string; }
export interface LessonPreferencePlan { ruleId: string; title: string; applicability: string; condition: string; useDirections: LessonDirection[]; favorableDirections: LessonDirection[]; unfavorableDirections: LessonDirection[]; conditionalDirections: LessonDirection[]; evidenceIds: string[]; rejections: string[]; sources: string[]; }
export interface LessonPreferenceAnalysis { title: string; conclusion: string; status: string; plans: LessonPreferencePlan[]; noPlanReasons: string[]; bridgePaths: Array<{ conflictFrom: string; intermediary: string; conflictTo: string; path: string[]; actualNodes: string[]; missingNodes: string[]; status: string; explanation: string }>; medicineNotes: Array<{ protectedPlanRuleId: string; protectedFamily: string; protectedElement: string; hindranceFamily: string; medicineFamily: string | null; status: string; explanation: string }>; seasonalCompatibility: Array<{ planRuleId: string | null; season: string; monthBranches: string[]; status: string; element: string | null; explanation: string }>; evidence: LessonEvidence[]; limitations: string[]; }
export interface LessonComparisonItem { appValue: string; lessonValue: string; agreementStatus: string; note: string; }
// 天干生剋合化（2026-10-07、10-14）
export interface LessonStemCombination { id: string; stems: string; positions: string[]; element: string; scope: string; kind: 'blocked' | 'bound' | 'contested' | 'transformed' | string; kindLabel: string; direction: string | null; boundStems: string[]; verdict: 'good' | 'bad' | 'mixed' | 'neutral' | string; verdictLabel: string; reason: string; notes: string[]; needsConfirmation: boolean; sourceDate: string; }
export interface LessonStemRelation { id: string; from: string; to: string; relation: string; distance: string; status: 'active' | 'weak' | 'ignored' | string; statusLabel: string; reason: string; sourceDate: string; }
export interface LessonStemInteractionAnalysis { title: string; conclusion: string; status: string; combinations: LessonStemCombination[]; relations: LessonStemRelation[]; limitations: string[]; sources: string[]; }
export interface LessonBranchClash { id: string; branches: string; positions: string[]; distance: string; forceLabel: string; winner: string | null; loser: string | null; outcome: 'decisive' | 'both_damaged' | 'storage' | string; outcomeLabel: string; verdict: 'good' | 'bad' | 'mixed' | 'neutral' | string; verdictLabel: string; reason: string; notes: string[]; sourceDate: string; }
export interface LessonBranchCombination { id: string; branches: string; positions: string[]; kind: 'generating' | 'controlling' | string; kindLabel: string; adjacent: boolean; verdict: 'good' | 'bad' | 'mixed' | 'neutral' | string; verdictLabel: string; reason: string; clashEffect: string | null; notes: string[]; sourceDate: string; }
export interface LessonBranchGathering { id: string; kind: 'three_combination' | 'three_meeting' | 'half_combination' | 'incomplete_half' | 'earth_full' | string; kindLabel: string; branches: string; positions: string[]; element: string; formed: boolean; forceLabel: string; verdict: 'good' | 'bad' | 'mixed' | 'neutral' | string; verdictLabel: string; reason: string; notes: string[]; sourceDate: string; }
export interface LessonBranchPunishment { id: string; kind: 'punishment' | 'self_punishment' | 'harm' | string; kindLabel: string; branches: string; positions: string[]; relation: string; verdict: 'good' | 'bad' | 'mixed' | 'neutral' | string; verdictLabel: string; reason: string; notes: string[]; sourceDate: string; }
export interface LessonBranchInteractionAnalysis { title: string; conclusion: string; status: string; clashes: LessonBranchClash[]; combinations: LessonBranchCombination[]; gatherings?: LessonBranchGathering[]; punishments?: LessonBranchPunishment[]; limitations: string[]; sources: string[]; }
export interface LessonPillarRelation { id: string; pillar: string; ganZhi: string; kind: string; kindLabel: string; hiddenCombination: string | null; hiddenCombinationPure: boolean; stemFavor: string; verdict: 'good' | 'bad' | 'mixed' | 'neutral' | string; verdictLabel: string; reason: string; notes: string[]; sourceDate: string; }
export interface LessonPillarRelationAnalysis { title: string; conclusion: string; relations: LessonPillarRelation[]; limitations: string[]; sources: string[]; }
export interface LessonInteractionStrengthReview { title: string; conclusion: string; samePartyCount: number; opposingPartyCount: number; momentum: boolean; classification: string; changed: boolean; changes: string[]; limitations: string[]; sources: string[]; }
export interface LessonThreeStepAnalysis { version: string; status: string; facts: LessonFacts; pattern: LessonPatternAnalysis; strength: LessonStrengthAnalysis; preferences: LessonPreferenceAnalysis; stemInteractions?: LessonStemInteractionAnalysis; branchInteractions?: LessonBranchInteractionAnalysis; pillarRelations?: LessonPillarRelationAnalysis; interactionStrength?: LessonInteractionStrengthReview; comparison: { pattern: LessonComparisonItem; strength: LessonComparisonItem; preferences: LessonComparisonItem }; sources: LessonSource[]; limitations: string[]; }

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
  specialPattern?: QuantSpecialPattern;
  shaYin?: QuantShaYin;
  lessonContent?: QuantLessonContent;
  luStrength?: LuStrengthAnalysis;
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
  gender?: Gender;
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

export interface LuStrengthCheck {
  item: string;
  result: string;
  points: number;
  detail: string;
}

export interface LuStrengthRoot {
  pillar: string;
  branch: string;
  hiddenStem: string;
  grade: string;
  weight: number;
  note: string;
}

export interface LuStrengthFlag {
  kind: string;
  title: string;
  detail: string;
}

export interface LuStrengthAnalysis {
  method: string;
  verdict: string;
  category: string;
  score: number;
  threshold: number;
  monthState: string;
  samePartyCount: number;
  checks: LuStrengthCheck[];
  roots: LuStrengthRoot[];
  flags: LuStrengthFlag[];
  conclusion: string;
  sources: string[];
  limitations: string[];
}
