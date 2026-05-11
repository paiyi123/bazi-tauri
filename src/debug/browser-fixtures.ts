import type {
  BaziRequest,
  BaziResponse,
  DaYun,
  LiuNian,
  LiuYue,
  LunarMonthDetailResponse,
  LunarYearOptionsResponse,
  Pillar,
  PillarAnalyzeRequest,
  QuantAnnualLuckScore,
  QuantLuckScore,
  QuantModelResponse,
  YearEra,
} from "../types/bazi";

function createPillar(
  ganZhi: string,
  naYin: string,
  wuXing: string,
  diShi: string,
  xun: string,
  xunKong: string,
): Pillar {
  return {
    ganZhi,
    stem: ganZhi.charAt(0),
    branch: ganZhi.charAt(1),
    naYin,
    wuXing,
    diShi,
    xun,
    xunKong,
  };
}

function makeLiuYue(index: number, month: string, ganZhi: string, jieQi: string, jieQiDate: string): LiuYue {
  return {
    index,
    month,
    ganZhi,
    jieQi,
    jieQiDate,
    stemTenGod: ["偏印", "正印", "比肩", "食神"][index % 4],
    branchHiddenStems: index % 2 === 0 ? ["甲", "丙"] : ["辛", "癸"],
    branchTenGods: index % 2 === 0 ? ["七殺", "偏財"] : ["正官", "正印"],
    xun: "甲子",
    xunKong: index % 2 === 0 ? "戌亥" : "申酉",
  };
}

function makeLiuNian(index: number, year: number, age: number, ganZhi: string): LiuNian {
  const monthSeeds = [
    ["正", "丙寅", "立春", "2/4"],
    ["二", "丁卯", "驚蟄", "3/5"],
    ["三", "戊辰", "清明", "4/4"],
    ["四", "己巳", "立夏", "5/5"],
    ["五", "庚午", "芒種", "6/5"],
    ["六", "辛未", "小暑", "7/7"],
    ["七", "壬申", "立秋", "8/7"],
    ["八", "癸酉", "白露", "9/7"],
    ["九", "甲戌", "寒露", "10/8"],
    ["十", "乙亥", "立冬", "11/7"],
    ["冬", "丙子", "大雪", "12/7"],
    ["臘", "丁丑", "小寒", "1/5"],
  ] as const;

  return {
    index,
    year,
    age,
    ganZhi,
    stemTenGod: index % 2 === 0 ? "偏財" : "正官",
    branchHiddenStems: index % 2 === 0 ? ["戊", "辛"] : ["甲", "壬"],
    branchTenGods: index % 2 === 0 ? ["比肩", "傷官"] : ["食神", "偏印"],
    xun: "甲子",
    xunKong: index % 2 === 0 ? "戌亥" : "申酉",
    liuYue: monthSeeds.map(([month, monthGanZhi, jieQi, jieQiDate], monthIndex) =>
      makeLiuYue(monthIndex, month, monthGanZhi, jieQi, jieQiDate),
    ),
  };
}

function makeQuantAnnualScore(index: number, year: number, age: number, ganZhi: string): QuantAnnualLuckScore {
  return {
    index,
    year,
    age,
    ganZhi,
    annualStemTenGod: index % 2 === 0 ? "偏財" : "正官",
    annualBranchHiddenStems: ["戊", "辛"],
    annualBranchTenGods: ["比肩", "傷官"],
    activeLuckLabel: "當值半運",
    activeLuckScore: 18 + index,
    fullLuckScore: 24 + index,
    annualStemScore: 10 + index,
    annualBranchScore: 12 + index,
    annualTotalScore: 22 + index,
    combinedScore: 28 + index,
    fullCombinedScore: 34 + index,
    impactRatio: 0.72 + index * 0.03,
    fullImpactRatio: 0.95 + index * 0.02,
    effectiveNatalScore: 56 + index,
    fullEffectiveNatalScore: 62 + index,
    tendency: index % 2 === 0 ? "偏旺" : "平衡",
    fullTendency: index % 2 === 0 ? "偏旺" : "中和",
    overviewLines: ["歲運並臨影響明顯", "對原局財官結構有增強效果"],
    interactionLines: ["年.局：合土", "年.運：相沖後轉入財勢"],
    scoringLines: ["流年干 10 分", "流年支 12 分", "合參後 28 分"],
    details: "瀏覽器 debug fixture：這裡是供前端除錯用的年度量化摘要。",
  };
}

function makeQuantLuckScore(index: number, ganZhi: string, startAge: number, endAge: number): QuantLuckScore {
  return {
    index,
    ganZhi,
    startAge,
    endAge,
    stemTenGod: index % 2 === 0 ? "偏財" : "七殺",
    branchHiddenStems: ["戊", "辛"],
    branchTenGods: ["比肩", "傷官"],
    firstHalfLabel: "前五年(干)",
    firstHalfScore: 18 + index,
    secondHalfLabel: "後五年(支)",
    secondHalfScore: 14 + index,
    stemScore: 18 + index,
    branchScore: 14 + index,
    totalScore: 32 + index,
    impactRatio: 0.86 + index * 0.05,
    effectiveNatalScore: 58 + index * 2,
    tendency: index % 2 === 0 ? "偏旺" : "轉強",
    overviewLines: ["大運主軸轉入財官運", "與原局月令形成明顯互動"],
    interactionLines: ["運.局：合土", "運.局：相沖"],
    scoringLines: ["前五年 18 分", "後五年 14 分", "整柱 32 分"],
    details: "瀏覽器 debug fixture：這裡是供前端除錯用的大運量化摘要。",
    annualScores: Array.from({ length: 10 }, (_, annualIndex) =>
      makeQuantAnnualScore(
        annualIndex,
        2024 + index * 10 + annualIndex,
        startAge + annualIndex,
        ["甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥"][annualIndex % 10],
      ),
    ),
  };
}

const DAYUN_SEEDS = ["辛酉", "壬戌", "癸亥", "甲子", "乙丑", "丙寅", "丁卯", "戊辰"] as const;
const LIUNIAN_SEEDS = ["甲寅", "乙卯", "丙辰", "丁巳", "戊午", "己未", "庚申", "辛酉", "壬戌", "癸亥"] as const;

function createQuantModel(): QuantModelResponse {
  return {
    dayMaster: "戊",
    summary: "日主偏旺，以木水為先用，火土宜節制。",
    note: "這是瀏覽器 debug fixture，用於在非 Tauri 環境驗證互動與版型。",
    stemScoreTotal: 42,
    branchScoreTotal: 58,
    totalScore: 100,
    strengthLabel: "偏旺",
    yongShen: {
      methodSummary: "以月令、通根、透干與歲運合參判定。",
      strengthBasis: "月令火土旺，日主得令得根。",
      printPresence: "印星有力但不過旺。",
      favorableTenGods: ["正官", "七殺", "正財"],
      favorableElements: ["木", "水"],
      conditionalTenGods: ["食神"],
      conditionalElements: ["金"],
      unfavorableTenGods: ["比肩", "劫財", "偏印"],
      unfavorableElements: ["火", "土"],
      cautionTenGods: ["傷官"],
      cautionElements: ["金"],
      process: "命局火土偏盛，宜取木疏土、取水潤燥。",
      conclusion: "用神木水，次取金，不喜火土再增。",
    },
    congPattern: {
      methodSummary: "以真從與假從界線比較。",
      boundaryScore: 18,
      trueBoundaryScore: 28,
      pattern: "不從",
      authenticity: "非從格",
      subtype: "身旺常格",
      dominantFamily: "比印",
      primaryUseGods: ["木", "水"],
      secondaryUseGods: ["金"],
      avoidGods: ["火", "土"],
      riskNote: "再遇強火土之運，燥土偏枯。",
      process: "雖比印偏強，但財官仍可用，未達真從條件。",
      conclusion: "不從，以抑扶法取用。",
    },
    shaYin: {
      methodSummary: "檢查七殺與印星之鏈路是否可轉化。",
      stemPatternFound: true,
      stemTransformed: true,
      stemChain: "甲木七殺 -> 壬水印",
      stemSourceNegativeScore: 12,
      stemSealSupportScore: 16,
      stemAdjustedTotalScore: 8,
      branchPatternFound: true,
      branchTransformed: false,
      branchChain: "寅木七殺 -> 子水印",
      branchSourceNegativeScore: 10,
      branchSealSupportScore: 6,
      branchAdjustedTotalScore: 4,
      process: "天干印透，能制衡七殺；地支支援稍弱。",
      conclusion: "有殺印相生傾向，但仍需看運勢配合。",
    },
    interactions: [
      {
        scope: "天干",
        type: "合",
        target: "甲己",
        outcome: "合土",
        pillars: "流年 - 日柱",
        detail: "瀏覽器 fixture：流年天干與日干形成合。",
      },
      {
        scope: "地支",
        type: "沖",
        target: "子午",
        outcome: "氣勢擾動",
        pillars: "大運 - 月柱",
        detail: "瀏覽器 fixture：大運地支與月支形成相沖。",
      },
    ],
    stemScores: [
      {
        pillar: "年干",
        target: "甲",
        tenGod: "七殺",
        baseScore: 10,
        positionWeight: 1,
        finalScore: 12,
        details: "年干七殺透出，受月令制衡。",
        combineNote: "甲己合土",
        rawScore: 10,
        positionAdjustedScore: 11,
        clashAdjustment: -1,
        bonusScore: 2,
        rows: [{ item: "甲己合土", interaction: "甲己合土", adjustmentScore: 2 }],
      },
      {
        pillar: "月干",
        target: "丙",
        tenGod: "偏印",
        baseScore: 14,
        positionWeight: 1.2,
        finalScore: 16,
        details: "月干印星得令，火勢偏旺。",
        rawScore: 14,
        positionAdjustedScore: 16,
        bonusScore: 0,
      },
    ],
    branchScores: [
      {
        pillar: "月支",
        target: "午",
        tenGod: "偏印",
        baseScore: 18,
        positionWeight: 1.4,
        finalScore: 20,
        details: "月令火旺，日主得勢。",
        combineNote: "寅午半合火",
        positionAdjustedScore: 20,
        bonusScore: 2,
        rows: [
          {
            item: "丁火透干",
            hiddenStem: "丁",
            tenGod: "正印",
            ratio: 0.7,
            rawScore: 12,
            positionAdjustedScore: 14,
            interaction: "半合",
            adjustmentScore: 2,
            finalContribution: 14,
            note: "印星得月令",
          },
        ],
      },
      {
        pillar: "日支",
        target: "子",
        tenGod: "正財",
        baseScore: 10,
        positionWeight: 1,
        finalScore: 11,
        details: "日支財星為用，但受火土夾制。",
        positionAdjustedScore: 10,
        bonusScore: 1,
      },
    ],
    luckScores: DAYUN_SEEDS.map((ganZhi, index) => makeQuantLuckScore(index, ganZhi, 28 + index * 10, 37 + index * 10)),
  };
}

function createDaYun(): DaYun[] {
  return DAYUN_SEEDS.map((ganZhi, index) => ({
    index,
    startAge: 28 + index * 10,
    endAge: 37 + index * 10,
    startYear: 2024 + index * 10,
    endYear: 2033 + index * 10,
    ganZhi,
    stemTenGod: ["傷官", "偏財", "正財", "七殺", "正官", "偏印", "正印", "比肩"][index % 8],
    branchHiddenStems: index % 2 === 0 ? ["辛"] : ["戊", "辛", "丁"],
    branchTenGods: index % 2 === 0 ? ["傷官"] : ["比肩", "傷官", "正印"],
    xun: "甲子",
    xunKong: index % 2 === 0 ? "戌亥" : "申酉",
    liuNian: Array.from({ length: 10 }, (_, annualIndex) =>
      makeLiuNian(
        annualIndex,
        2024 + index * 10 + annualIndex,
        28 + index * 10 + annualIndex,
        LIUNIAN_SEEDS[annualIndex % LIUNIAN_SEEDS.length],
      ),
    ),
  }));
}

function buildResponse(overrides?: Partial<BaziResponse>): BaziResponse {
  return {
    inputCalendarType: "公曆",
    inputDateTime: "1998-02-04 23:00",
    solarDateTime: "1998-02-04 23:00",
    lunarDateTime: "戊寅年 正月初八 子時",
    baZi: "甲寅 丙午 戊子 壬子",
    dayMaster: "戊土",
    geJu: "身旺格",
    geJuBasis: "月令火旺，日主得令得根。",
    taiYuan: "丁酉",
    mingGong: "壬子",
    shenGong: "甲寅",
    directPillarYearHint: undefined,
    yearPillar: createPillar("甲寅", "大溪水", "木", "長生", "甲子", "子丑"),
    monthPillar: createPillar("丙午", "天河水", "火", "帝旺", "甲子", "戌亥"),
    dayPillar: createPillar("戊子", "霹靂火", "土", "胎", "甲子", "午未"),
    hourPillar: createPillar("壬子", "桑柘木", "水", "胎", "甲子", "午未"),
    yearHiddenStems: ["甲", "丙", "戊"],
    monthHiddenStems: ["丁", "己"],
    dayHiddenStems: ["癸"],
    hourHiddenStems: ["癸"],
    yearStemTenGod: "七殺",
    monthStemTenGod: "偏印",
    dayStemTenGod: "日元",
    hourStemTenGod: "偏財",
    yearBranchTenGods: ["七殺", "偏印", "比肩"],
    monthBranchTenGods: ["正印", "劫財"],
    dayBranchTenGods: ["正財"],
    hourBranchTenGods: ["正財"],
    luckStart: {
      forward: true,
      startYear: 2,
      startMonth: 8,
      startDay: 12,
      startHour: 0,
      startSolar: "2000-10-16 00:00",
      startSummary: "2年8月12日後起運",
      transitionSummary: "約 2024 年交入辛酉大運",
      birthJieName: "立春",
      birthJieSolar: "1998-02-04 05:57",
      birthJieDayOrdinal: 35,
      transitionSummaryExperimental: "研究值：2000-10-16 起運",
    },
    daYun: createDaYun(),
    quantModel: createQuantModel(),
    ...overrides,
  };
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export function browserFixtureCalculateBazi(request: BaziRequest): Promise<BaziResponse> {
  const calendarLabel = request.calendarType === "LUNAR" ? "農曆" : "公曆";
  return Promise.resolve(
    clone(
      buildResponse({
        inputCalendarType: calendarLabel,
        inputDateTime: `${request.year}/${request.month}/${request.day} ${String(request.hour).padStart(2, "0")}:${String(request.minute).padStart(2, "0")}`,
      }),
    ),
  );
}

export function browserFixtureAnalyzePillars(request: PillarAnalyzeRequest): Promise<BaziResponse> {
  const candidates = [
    {
      year: request.selectedGregorianYear ?? 1998,
      month: 2,
      day: 4,
      hour: 23,
      minute: 30,
      second: 0,
      solarDateTime: `${String(request.selectedGregorianYear ?? 1998).padStart(4, "0")}-02-04 23:30:00`,
      label: `${String(request.selectedGregorianYear ?? 1998).padStart(4, "0")}-02-04 23:30`,
    },
    {
      year: (request.selectedGregorianYear ?? 1998) - 60,
      month: 2,
      day: 5,
      hour: 23,
      minute: 30,
      second: 0,
      solarDateTime: `${String((request.selectedGregorianYear ?? 1998) - 60).padStart(4, "0")}-02-05 23:30:00`,
      label: `${String((request.selectedGregorianYear ?? 1998) - 60).padStart(4, "0")}-02-05 23:30`,
    },
  ];
  return Promise.resolve(
    clone(
      buildResponse({
        inputCalendarType: "四柱直輸",
        inputDateTime: `${request.yearPillar} ${request.monthPillar} ${request.dayPillar} ${request.hourPillar}`,
        baZi: `${request.yearPillar} ${request.monthPillar} ${request.dayPillar} ${request.hourPillar}`,
        directPillarYearHint: request.inferGregorianYears
          ? {
              candidateYears: candidates.map((candidate) => candidate.year),
              selectedYear: request.selectedGregorianYear ?? null,
              candidates,
              note: "瀏覽器 debug fixture：回推最近兩個可能公曆生日時間。",
            }
          : undefined,
        yearPillar: createPillar(request.yearPillar, "海中金", "木", "長生", "甲子", "子丑"),
        monthPillar: createPillar(request.monthPillar, "爐中火", "火", "帝旺", "甲子", "戌亥"),
        dayPillar: createPillar(request.dayPillar, "大林木", "土", "胎", "甲子", "午未"),
        hourPillar: createPillar(request.hourPillar, "路旁土", "水", "胎", "甲子", "午未"),
      }),
    ),
  );
}

export function browserFixtureGetLunarYearOptions(year: number, yearEra: YearEra): Promise<LunarYearOptionsResponse> {
  return Promise.resolve({
    year,
    normalizedYear: yearEra === "ROC" ? year + 1911 : year,
    yearEra,
    leapMonth: 6,
    monthOptions: [
      { value: 1, label: "正月", leap: false },
      { value: 2, label: "二月", leap: false },
      { value: 3, label: "三月", leap: false },
      { value: 4, label: "四月", leap: false },
      { value: 5, label: "五月", leap: false },
      { value: 6, label: "六月", leap: false },
      { value: 6, label: "閏六月", leap: true },
      { value: 7, label: "七月", leap: false },
      { value: 8, label: "八月", leap: false },
      { value: 9, label: "九月", leap: false },
      { value: 10, label: "十月", leap: false },
      { value: 11, label: "冬月", leap: false },
      { value: 12, label: "臘月", leap: false },
    ],
  });
}

export function browserFixtureGetLunarMonthDetail(
  year: number,
  yearEra: YearEra,
  month: number,
  leapMonth: boolean,
): Promise<LunarMonthDetailResponse> {
  return Promise.resolve({
    year,
    normalizedYear: yearEra === "ROC" ? year + 1911 : year,
    yearEra,
    month,
    leapMonth,
    dayCount: 30,
    dayOptions: Array.from({ length: 30 }, (_, index) => ({
      value: index + 1,
      label: `初${index + 1}`,
    })),
  });
}
