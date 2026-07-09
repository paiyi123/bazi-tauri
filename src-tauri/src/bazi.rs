use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use tyme4rs::tyme::culture::Element;
use tyme4rs::tyme::eightchar::provider::{
    ChildLimitProvider, DefaultEightCharProvider, EightCharProvider, LunarSect1ChildLimitProvider,
    LunarSect2ChildLimitProvider, LunarSect2EightCharProvider,
};
use tyme4rs::tyme::eightchar::EightChar;
use tyme4rs::tyme::enums::{Gender as TymeGender, YinYang};
use tyme4rs::tyme::lunar::{LunarHour, LunarMonth, LunarYear};
use tyme4rs::tyme::sixtycycle::{
    EarthBranch, HeavenStem, SixtyCycle, SixtyCycleYear, SIXTY_CYCLE_NAMES,
};
use tyme4rs::tyme::solar::{SolarTerm, SolarTime};
use tyme4rs::tyme::{Culture, Tyme};

use crate::shen_sha::{analyze_shen_sha, FourPillarShenShaDto};

const MONTH_LABELS: [&str; 12] = [
    "正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "冬月", "臘月",
];
const DAY_LABELS: [&str; 30] = [
    "初一", "初二", "初三", "初四", "初五", "初六", "初七", "初八", "初九", "初十", "十一", "十二",
    "十三", "十四", "十五", "十六", "十七", "十八", "十九", "二十", "廿一", "廿二", "廿三", "廿四",
    "廿五", "廿六", "廿七", "廿八", "廿九", "三十",
];
const STEM_PILLAR_LABELS: [&str; 4] = ["年干", "月干", "日干", "時干"];
const BRANCH_PILLAR_LABELS: [&str; 4] = ["年支", "月支", "日支", "時支"];
const DIRECT_PILLAR_LUCK_COUNT: i32 = 8;
const LIU_YUE_JIE_QI: [&str; 12] = [
    "立春", "惊蛰", "清明", "立夏", "芒种", "小暑", "立秋", "白露", "寒露", "立冬", "大雪", "小寒",
];
const LIU_YUE_BRANCHES: [&str; 12] = [
    "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥", "子", "丑",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalendarType {
    Solar,
    Lunar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YearEra {
    Ad,
    Roc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaziRequest {
    pub calendar_type: CalendarType,
    pub gender: Gender,
    pub year: i32,
    pub year_era: YearEra,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub bazi_sect: u8,
    pub yun_sect: u8,
    pub leap_month: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PillarAnalyzeRequest {
    pub year_pillar: String,
    pub month_pillar: String,
    pub day_pillar: String,
    pub hour_pillar: String,
    pub gender: Option<Gender>,
    pub selected_gregorian_year: Option<i32>,
    pub infer_gregorian_years: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarYearOptionsQuery {
    pub year: i32,
    pub year_era: YearEra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarMonthDetailQuery {
    pub year: i32,
    pub year_era: YearEra,
    pub month: u32,
    pub leap_month: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarMonthOption {
    pub value: u32,
    pub label: String,
    pub leap: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarDayOption {
    pub value: u32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarYearOptionsResponse {
    pub year: i32,
    pub normalized_year: i32,
    pub year_era: YearEra,
    pub leap_month: Option<u32>,
    pub month_options: Vec<LunarMonthOption>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarMonthDetailResponse {
    pub year: i32,
    pub normalized_year: i32,
    pub year_era: YearEra,
    pub month: u32,
    pub leap_month: bool,
    pub day_count: u32,
    pub day_options: Vec<LunarDayOption>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillarDto {
    pub gan_zhi: String,
    pub stem: String,
    pub branch: String,
    pub na_yin: String,
    pub wu_xing: String,
    pub di_shi: String,
    pub xun: String,
    pub xun_kong: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LuckStartDto {
    pub forward: Option<bool>,
    pub start_year: Option<i32>,
    pub start_month: Option<i32>,
    pub start_day: Option<i32>,
    pub start_hour: Option<i32>,
    pub start_solar: Option<String>,
    pub start_summary: Option<String>,
    pub transition_summary: Option<String>,
    pub birth_jie_name: Option<String>,
    pub birth_jie_solar: Option<String>,
    pub birth_jie_day_ordinal: Option<i32>,
    pub transition_summary_experimental: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectPillarBirthCandidateDto {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub solar_date_time: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectPillarYearHintDto {
    pub candidate_years: Vec<i32>,
    pub selected_year: Option<i32>,
    pub candidates: Vec<DirectPillarBirthCandidateDto>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiuNianDto {
    pub index: i32,
    pub year: i32,
    pub age: i32,
    pub gan_zhi: String,
    pub stem_ten_god: String,
    pub branch_hidden_stems: Vec<String>,
    pub branch_ten_gods: Vec<String>,
    pub xun: String,
    pub xun_kong: String,
    pub liu_yue: Vec<LiuYueDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiuYueDto {
    pub index: i32,
    pub jie_qi: Option<String>,
    pub jie_qi_date: Option<String>,
    pub month: String,
    pub gan_zhi: String,
    pub stem_ten_god: String,
    pub branch_hidden_stems: Vec<String>,
    pub branch_ten_gods: Vec<String>,
    pub xun: String,
    pub xun_kong: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaYunDto {
    pub index: i32,
    pub start_age: Option<i32>,
    pub end_age: Option<i32>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub gan_zhi: String,
    pub stem_ten_god: String,
    pub branch_hidden_stems: Vec<String>,
    pub branch_ten_gods: Vec<String>,
    pub xun: String,
    pub xun_kong: String,
    pub liu_nian: Vec<LiuNianDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantModelSubScore {
    pub item: String,
    pub hidden_stem: Option<String>,
    pub ten_god: Option<String>,
    pub ratio: Option<f64>,
    pub raw_score: Option<f64>,
    pub position_adjusted_score: Option<f64>,
    pub interaction: Option<String>,
    pub adjustment_score: Option<f64>,
    pub final_contribution: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantModelInteraction {
    pub scope: String,
    pub r#type: String,
    pub target: String,
    pub outcome: String,
    pub pillars: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantModelPillarScore {
    pub pillar: String,
    pub target: String,
    pub ten_god: String,
    pub base_score: f64,
    pub position_weight: f64,
    pub final_score: f64,
    pub details: String,
    pub category: Option<String>,
    pub combine_note: Option<String>,
    pub raw_score: Option<f64>,
    pub position_adjusted_score: Option<f64>,
    pub clash_adjustment: Option<f64>,
    pub bonus_score: Option<f64>,
    pub rows: Option<Vec<QuantModelSubScore>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantAnnualLuckScore {
    pub index: i32,
    pub year: i32,
    pub age: i32,
    pub gan_zhi: String,
    pub annual_stem_ten_god: String,
    pub annual_branch_hidden_stems: Vec<String>,
    pub annual_branch_ten_gods: Vec<String>,
    pub active_luck_label: String,
    pub active_luck_score: f64,
    pub full_luck_score: f64,
    pub annual_stem_score: f64,
    pub annual_branch_score: f64,
    pub annual_total_score: f64,
    pub combined_score: f64,
    pub full_combined_score: f64,
    pub impact_ratio: f64,
    pub full_impact_ratio: f64,
    pub effective_natal_score: f64,
    pub full_effective_natal_score: f64,
    pub tendency: String,
    pub full_tendency: String,
    pub overview_lines: Option<Vec<String>>,
    pub interaction_lines: Option<Vec<String>>,
    pub scoring_lines: Option<Vec<String>>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantLuckScore {
    pub index: i32,
    pub gan_zhi: String,
    pub start_age: Option<i32>,
    pub end_age: Option<i32>,
    pub stem_ten_god: String,
    pub branch_hidden_stems: Vec<String>,
    pub branch_ten_gods: Vec<String>,
    pub first_half_label: String,
    pub first_half_score: f64,
    pub second_half_label: String,
    pub second_half_score: f64,
    pub stem_score: f64,
    pub branch_score: f64,
    pub total_score: f64,
    pub impact_ratio: f64,
    pub effective_natal_score: f64,
    pub tendency: String,
    pub overview_lines: Option<Vec<String>>,
    pub interaction_lines: Option<Vec<String>>,
    pub scoring_lines: Option<Vec<String>>,
    pub details: String,
    pub annual_scores: Option<Vec<QuantAnnualLuckScore>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantYongShen {
    pub method_summary: String,
    pub strength_basis: String,
    pub print_presence: String,
    pub favorable_ten_gods: Vec<String>,
    pub favorable_elements: Vec<String>,
    pub conditional_ten_gods: Vec<String>,
    pub conditional_elements: Vec<String>,
    pub unfavorable_ten_gods: Vec<String>,
    pub unfavorable_elements: Vec<String>,
    pub caution_ten_gods: Vec<String>,
    pub caution_elements: Vec<String>,
    pub process: String,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantCongPattern {
    pub method_summary: String,
    pub boundary_score: f64,
    pub true_boundary_score: f64,
    pub pattern: String,
    pub authenticity: String,
    pub subtype: String,
    pub dominant_family: String,
    pub primary_use_gods: Vec<String>,
    pub secondary_use_gods: Vec<String>,
    pub avoid_gods: Vec<String>,
    pub risk_note: String,
    pub process: String,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantShaYin {
    pub method_summary: String,
    pub stem_pattern_found: bool,
    pub stem_transformed: bool,
    pub stem_chain: Option<String>,
    pub stem_source_negative_score: Option<f64>,
    pub stem_seal_support_score: Option<f64>,
    pub stem_adjusted_total_score: Option<f64>,
    pub branch_pattern_found: bool,
    pub branch_transformed: bool,
    pub branch_chain: Option<String>,
    pub branch_source_negative_score: Option<f64>,
    pub branch_seal_support_score: Option<f64>,
    pub branch_adjusted_total_score: Option<f64>,
    pub process: String,
    pub conclusion: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantModelResponse {
    pub day_master: String,
    pub summary: String,
    pub note: String,
    pub stem_score_total: f64,
    pub branch_score_total: f64,
    pub total_score: f64,
    pub strength_label: String,
    pub yong_shen: Option<QuantYongShen>,
    pub cong_pattern: Option<QuantCongPattern>,
    pub sha_yin: Option<QuantShaYin>,
    pub interactions: Option<Vec<QuantModelInteraction>>,
    pub stem_scores: Vec<QuantModelPillarScore>,
    pub branch_scores: Vec<QuantModelPillarScore>,
    pub luck_scores: Option<Vec<QuantLuckScore>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaziResponse {
    pub input_calendar_type: String,
    pub input_date_time: String,
    pub solar_date_time: String,
    pub lunar_date_time: String,
    pub ba_zi: String,
    pub day_master: String,
    pub ge_ju: String,
    pub ge_ju_basis: String,
    pub tai_yuan: String,
    pub ming_gong: String,
    pub shen_gong: String,
    pub shen_sha: Option<FourPillarShenShaDto>,
    pub direct_pillar_year_hint: Option<DirectPillarYearHintDto>,
    pub year_pillar: PillarDto,
    pub month_pillar: PillarDto,
    pub day_pillar: PillarDto,
    pub hour_pillar: PillarDto,
    pub year_hidden_stems: Vec<String>,
    pub month_hidden_stems: Vec<String>,
    pub day_hidden_stems: Vec<String>,
    pub hour_hidden_stems: Vec<String>,
    pub year_stem_ten_god: String,
    pub month_stem_ten_god: String,
    pub day_stem_ten_god: String,
    pub hour_stem_ten_god: String,
    pub year_branch_ten_gods: Vec<String>,
    pub month_branch_ten_gods: Vec<String>,
    pub day_branch_ten_gods: Vec<String>,
    pub hour_branch_ten_gods: Vec<String>,
    pub luck_start: LuckStartDto,
    pub da_yun: Vec<DaYunDto>,
    pub quant_model: Option<QuantModelResponse>,
}

#[derive(Debug, Clone)]
struct GeJuResult {
    ge_ju: String,
    basis: String,
}

#[derive(Debug, Clone, Copy)]
struct VisibleStemTenGod<'a> {
    pillar_label: &'static str,
    stem: &'a str,
    branch: &'a str,
    ten_god: &'a str,
}

impl<'a> VisibleStemTenGod<'a> {
    fn new(pillar_label: &'static str, stem: &'a str, branch: &'a str, ten_god: &'a str) -> Self {
        Self {
            pillar_label,
            stem,
            branch,
            ten_god,
        }
    }
}

#[derive(Debug, Clone)]
struct GeJuCandidate<'a> {
    hidden_index: usize,
    stem: &'a str,
    ten_god: &'a str,
    chart_element_score: i32,
    transparent_stems: Vec<VisibleStemTenGod<'a>>,
}

#[tauri::command]
pub fn get_lunar_year_options(
    request: LunarYearOptionsQuery,
) -> Result<LunarYearOptionsResponse, String> {
    let normalized_year = normalize_year(request.year, &request.year_era)?;
    let lunar_year = LunarYear::from_year(normalized_year as isize);
    let leap_month = lunar_year.get_leap_month();

    let month_options = lunar_year
        .get_months()
        .into_iter()
        .map(|month| LunarMonthOption {
            value: month.get_month() as u32,
            label: if month.is_leap() {
                format!("閏{}", MONTH_LABELS[month.get_month() - 1])
            } else {
                MONTH_LABELS[month.get_month() - 1].to_string()
            },
            leap: month.is_leap(),
        })
        .collect();

    Ok(LunarYearOptionsResponse {
        year: request.year,
        normalized_year,
        year_era: request.year_era,
        leap_month: if leap_month == 0 {
            None
        } else {
            Some(leap_month as u32)
        },
        month_options,
    })
}

#[tauri::command]
pub fn get_lunar_month_detail(
    request: LunarMonthDetailQuery,
) -> Result<LunarMonthDetailResponse, String> {
    let normalized_year = normalize_year(request.year, &request.year_era)?;
    let month_value = if request.leap_month {
        -(request.month as isize)
    } else {
        request.month as isize
    };
    let lunar_month = LunarMonth::new(normalized_year as isize, month_value)?;
    let day_count = lunar_month.get_day_count() as u32;
    let day_options = (1..=day_count)
        .map(|day| LunarDayOption {
            value: day,
            label: DAY_LABELS[(day - 1) as usize].to_string(),
        })
        .collect();

    Ok(LunarMonthDetailResponse {
        year: request.year,
        normalized_year,
        year_era: request.year_era,
        month: request.month,
        leap_month: request.leap_month,
        day_count,
        day_options,
    })
}

#[tauri::command]
pub fn calculate_bazi(request: BaziRequest) -> Result<BaziResponse, String> {
    let normalized_year = normalize_year(request.year, &request.year_era)?;
    let solar_time = create_solar_time(&request, normalized_year)?;
    let lunar_hour = solar_time.get_lunar_hour();
    let eight_char = create_eight_char(request.bazi_sect, lunar_hour.clone());
    let day_master = eight_char.get_day().get_heaven_stem();

    let year_hidden_stems = hidden_stems(eight_char.get_year().get_earth_branch());
    let month_hidden_stems = hidden_stems(eight_char.get_month().get_earth_branch());
    let day_hidden_stems = hidden_stems(eight_char.get_day().get_earth_branch());
    let hour_hidden_stems = hidden_stems(eight_char.get_hour().get_earth_branch());

    let year_branch_ten_gods =
        hidden_ten_gods(day_master.clone(), eight_char.get_year().get_earth_branch());
    let month_branch_ten_gods = hidden_ten_gods(
        day_master.clone(),
        eight_char.get_month().get_earth_branch(),
    );
    let day_branch_ten_gods =
        hidden_ten_gods(day_master.clone(), eight_char.get_day().get_earth_branch());
    let hour_branch_ten_gods =
        hidden_ten_gods(day_master.clone(), eight_char.get_hour().get_earth_branch());
    let year_stem_ten_god =
        ten_god_name(day_master.clone(), eight_char.get_year().get_heaven_stem());
    let month_stem_ten_god =
        ten_god_name(day_master.clone(), eight_char.get_month().get_heaven_stem());
    let day_stem_analysis_ten_god = ten_god_name(day_master.clone(), day_master.clone());
    let hour_stem_ten_god =
        ten_god_name(day_master.clone(), eight_char.get_hour().get_heaven_stem());
    let year_stem_name = trad(&eight_char.get_year().get_heaven_stem().get_name());
    let month_stem_name = trad(&eight_char.get_month().get_heaven_stem().get_name());
    let day_stem_name = trad(&day_master.get_name());
    let hour_stem_name = trad(&eight_char.get_hour().get_heaven_stem().get_name());
    let year_branch_name = trad(&eight_char.get_year().get_earth_branch().get_name());
    let month_pillar_branch_name = trad(&eight_char.get_month().get_earth_branch().get_name());
    let day_branch_name = trad(&eight_char.get_day().get_earth_branch().get_name());
    let hour_branch_name = trad(&eight_char.get_hour().get_earth_branch().get_name());
    let month_branch_name = trad(&eight_char.get_month().get_earth_branch().get_name());
    let ge_ju_result = analyze_ge_ju(
        &month_branch_name,
        &month_hidden_stems,
        &month_branch_ten_gods,
        [
            VisibleStemTenGod::new(
                "年干",
                &year_stem_name,
                &year_branch_name,
                &year_stem_ten_god,
            ),
            VisibleStemTenGod::new(
                "月干",
                &month_stem_name,
                &month_pillar_branch_name,
                &month_stem_ten_god,
            ),
            VisibleStemTenGod::new(
                "日干",
                &day_stem_name,
                &day_branch_name,
                &day_stem_analysis_ten_god,
            ),
            VisibleStemTenGod::new(
                "時干",
                &hour_stem_name,
                &hour_branch_name,
                &hour_stem_ten_god,
            ),
        ],
    );

    let luck_start = build_luck_start(&request, solar_time, &eight_char)?;
    let da_yun = build_da_yun(&request, solar_time, &eight_char, day_master.clone())?;
    let natal_cycles = [
        eight_char.get_year(),
        eight_char.get_month(),
        eight_char.get_day(),
        eight_char.get_hour(),
    ];
    let quant_model = Some(build_quant_model(
        day_master.clone(),
        &natal_cycles,
        &da_yun,
    ));

    let mut response = BaziResponse {
        input_calendar_type: match request.calendar_type {
            CalendarType::Solar => "公曆".to_string(),
            CalendarType::Lunar => "農曆".to_string(),
        },
        input_date_time: format_input(&request),
        solar_date_time: format_solar_time(solar_time),
        lunar_date_time: format_lunar_time(lunar_hour.clone()),
        ba_zi: trad(&eight_char.to_string()),
        day_master: trad(&day_master.get_name()),
        ge_ju: ge_ju_result.ge_ju,
        ge_ju_basis: ge_ju_result.basis,
        tai_yuan: trad(&eight_char.get_fetal_origin().get_name()),
        ming_gong: trad(&eight_char.get_own_sign().get_name()),
        shen_gong: trad(&eight_char.get_body_sign().get_name()),
        shen_sha: None,
        direct_pillar_year_hint: None,
        year_pillar: build_pillar(eight_char.get_year(), day_master.clone()),
        month_pillar: build_pillar(eight_char.get_month(), day_master.clone()),
        day_pillar: build_pillar(eight_char.get_day(), day_master.clone()),
        hour_pillar: build_pillar(eight_char.get_hour(), day_master.clone()),
        year_hidden_stems,
        month_hidden_stems,
        day_hidden_stems,
        hour_hidden_stems,
        year_stem_ten_god,
        month_stem_ten_god,
        day_stem_ten_god: "日元".to_string(),
        hour_stem_ten_god,
        year_branch_ten_gods,
        month_branch_ten_gods,
        day_branch_ten_gods,
        hour_branch_ten_gods,
        luck_start,
        da_yun,
        quant_model,
    };
    response.shen_sha = Some(analyze_shen_sha(&response));
    Ok(finalize_bazi_response(response))
}

#[tauri::command]
pub fn analyze_pillars(request: PillarAnalyzeRequest) -> Result<BaziResponse, String> {
    let year_cycle = parse_direct_pillar(&request.year_pillar)?;
    let month_cycle = parse_direct_pillar(&request.month_pillar)?;
    let day_cycle = parse_direct_pillar(&request.day_pillar)?;
    let hour_cycle = parse_direct_pillar(&request.hour_pillar)?;
    let day_master = day_cycle.get_heaven_stem();
    let gender = request.gender.clone().unwrap_or(Gender::Female);
    let forward = is_forward_direct(year_cycle.get_heaven_stem(), &gender);
    let da_yun = build_direct_da_yun(month_cycle.clone(), day_master.clone(), forward);
    let natal_cycles = [
        year_cycle.clone(),
        month_cycle.clone(),
        day_cycle.clone(),
        hour_cycle.clone(),
    ];
    let quant_model = Some(build_quant_model(
        day_master.clone(),
        &natal_cycles,
        &da_yun,
    ));
    let direct_pillar_year_hint = if request.infer_gregorian_years.unwrap_or(false) {
        Some(resolve_recent_matching_solar_years(&request)?)
    } else {
        None
    };

    let year_hidden_stems = hidden_stems(year_cycle.get_earth_branch());
    let month_hidden_stems = hidden_stems(month_cycle.get_earth_branch());
    let day_hidden_stems = hidden_stems(day_cycle.get_earth_branch());
    let hour_hidden_stems = hidden_stems(hour_cycle.get_earth_branch());

    let mut response = BaziResponse {
        input_calendar_type: "直接輸入四柱".to_string(),
        input_date_time: format!(
            "{} {} {} {}",
            trad(&year_cycle.get_name()),
            trad(&month_cycle.get_name()),
            trad(&day_cycle.get_name()),
            trad(&hour_cycle.get_name())
        ),
        solar_date_time: "未提供".to_string(),
        lunar_date_time: "未提供".to_string(),
        ba_zi: format!(
            "{} {} {} {}",
            trad(&year_cycle.get_name()),
            trad(&month_cycle.get_name()),
            trad(&day_cycle.get_name()),
            trad(&hour_cycle.get_name())
        ),
        day_master: trad(&day_master.get_name()),
        ge_ju: "直輸四柱分析".to_string(),
        ge_ju_basis: "以直接輸入四柱進行拆解、順逆排運與相容量化分析。".to_string(),
        tai_yuan: "未提供".to_string(),
        ming_gong: "未提供".to_string(),
        shen_gong: "未提供".to_string(),
        shen_sha: None,
        direct_pillar_year_hint,
        year_pillar: build_pillar(year_cycle.clone(), day_master.clone()),
        month_pillar: build_pillar(month_cycle.clone(), day_master.clone()),
        day_pillar: build_pillar(day_cycle.clone(), day_master.clone()),
        hour_pillar: build_pillar(hour_cycle.clone(), day_master.clone()),
        year_hidden_stems: year_hidden_stems.clone(),
        month_hidden_stems: month_hidden_stems.clone(),
        day_hidden_stems: day_hidden_stems.clone(),
        hour_hidden_stems: hour_hidden_stems.clone(),
        year_stem_ten_god: ten_god_name(day_master.clone(), year_cycle.get_heaven_stem()),
        month_stem_ten_god: ten_god_name(day_master.clone(), month_cycle.get_heaven_stem()),
        day_stem_ten_god: "日元".to_string(),
        hour_stem_ten_god: ten_god_name(day_master.clone(), hour_cycle.get_heaven_stem()),
        year_branch_ten_gods: hidden_ten_gods(day_master.clone(), year_cycle.get_earth_branch()),
        month_branch_ten_gods: hidden_ten_gods(day_master.clone(), month_cycle.get_earth_branch()),
        day_branch_ten_gods: hidden_ten_gods(day_master.clone(), day_cycle.get_earth_branch()),
        hour_branch_ten_gods: hidden_ten_gods(day_master.clone(), hour_cycle.get_earth_branch()),
        luck_start: LuckStartDto {
            forward: Some(forward),
            start_year: None,
            start_month: None,
            start_day: None,
            start_hour: None,
            start_solar: None,
            start_summary: Some("四柱直輸未含出生日期，僅提供順逆排與大運序列。".to_string()),
            transition_summary: Some("未提供出生日期，無法推算交脫大運時間。".to_string()),
            birth_jie_name: None,
            birth_jie_solar: None,
            birth_jie_day_ordinal: None,
            transition_summary_experimental: Some(
                "若需起運歲數與年份，請改用出生資料排盤模式。".to_string(),
            ),
        },
        da_yun,
        quant_model,
    };
    response.shen_sha = Some(analyze_shen_sha(&response));
    Ok(finalize_bazi_response(response))
}

fn finalize_bazi_response(response: BaziResponse) -> BaziResponse {
    #[cfg(target_os = "android")]
    {
        let optimized = optimize_response_for_android(response);
        if let Ok(serialized) = serde_json::to_string(&optimized) {
            eprintln!("android bazi response bytes={}", serialized.len());
        }
        optimized
    }

    #[cfg(not(target_os = "android"))]
    {
        response
    }
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn optimize_response_for_android(response: BaziResponse) -> BaziResponse {
    response
}

fn resolve_recent_matching_solar_years(
    request: &PillarAnalyzeRequest,
) -> Result<DirectPillarYearHintDto, String> {
    let mut candidates = Vec::new();
    let current_year = Local::now().year();

    for year in build_candidate_gregorian_years(&request.year_pillar, current_year) {
        if let Some(candidate) = find_matching_solar_in_year(year, request) {
            candidates.push(candidate);
            if candidates.len() >= 2 {
                break;
            }
        }
    }

    let candidate_years = candidates.iter().map(|candidate| candidate.year).collect();
    let note = if candidates.is_empty() {
        "未在最近 180 年內找到四柱完全吻合的公曆年份。".to_string()
    } else {
        "依四柱完整匹配回推最近兩個可能公曆生日時間；若生日落在立春前後，年份可能落在前後一年邊界。"
            .to_string()
    };

    Ok(DirectPillarYearHintDto {
        candidate_years,
        selected_year: request.selected_gregorian_year,
        candidates,
        note,
    })
}

fn build_candidate_gregorian_years(year_pillar: &str, current_year: i32) -> Vec<i32> {
    let normalized = year_pillar.trim();
    let Some(cycle_index) = SIXTY_CYCLE_NAMES
        .iter()
        .position(|item| *item == normalized)
    else {
        return vec![];
    };

    let base_year = 1984 + cycle_index as i32;
    let mut latest_cycle_year = base_year;
    while latest_cycle_year + 60 <= current_year {
        latest_cycle_year += 60;
    }

    let mut years = Vec::new();
    let mut cycle_year = latest_cycle_year;
    while cycle_year >= current_year - 180 {
        if cycle_year + 1 <= current_year {
            push_unique_year(&mut years, cycle_year + 1);
        }
        if cycle_year <= current_year {
            push_unique_year(&mut years, cycle_year);
        }
        cycle_year -= 60;
    }
    years.sort_by(|a, b| b.cmp(a));
    years
}

fn push_unique_year(years: &mut Vec<i32>, year: i32) {
    if !years.contains(&year) {
        years.push(year);
    }
}

fn find_matching_solar_in_year(
    year: i32,
    request: &PillarAnalyzeRequest,
) -> Option<DirectPillarBirthCandidateDto> {
    for month in 1..=12u32 {
        let day_count = days_in_month(year, month)?;
        for day in 1..=day_count {
            for hour in [0u32, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23] {
                if let Some(candidate) = matches_four_pillars(year, month, day, hour, request) {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn matches_four_pillars(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    request: &PillarAnalyzeRequest,
) -> Option<DirectPillarBirthCandidateDto> {
    let solar_time = SolarTime::new(
        year as isize,
        month as usize,
        day as usize,
        hour as usize,
        30,
        0,
    )
    .ok()?;
    let eight_char = create_eight_char(2, solar_time.get_lunar_hour());

    if trad(&eight_char.get_year().get_name()) == request.year_pillar
        && trad(&eight_char.get_month().get_name()) == request.month_pillar
        && trad(&eight_char.get_day().get_name()) == request.day_pillar
        && trad(&eight_char.get_hour().get_name()) == request.hour_pillar
    {
        return Some(DirectPillarBirthCandidateDto {
            year,
            month,
            day,
            hour,
            minute: 30,
            second: 0,
            solar_date_time: format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hour, 30, 0
            ),
            label: format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hour, 30),
        });
    }
    None
}

fn days_in_month(year: i32, month: u32) -> Option<u32> {
    let start = NaiveDate::from_ymd_opt(year, month, 1)?;
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    Some((next - start).num_days() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> BaziRequest {
        BaziRequest {
            calendar_type: CalendarType::Solar,
            gender: Gender::Female,
            year: 1998,
            year_era: YearEra::Ad,
            month: 2,
            day: 4,
            hour: 23,
            minute: 0,
            second: 0,
            bazi_sect: 2,
            yun_sect: 1,
            leap_month: false,
        }
    }

    fn reported_lunar_request() -> BaziRequest {
        BaziRequest {
            calendar_type: CalendarType::Lunar,
            gender: Gender::Male,
            year: 68,
            year_era: YearEra::Roc,
            month: 10,
            day: 28,
            hour: 1,
            minute: 0,
            second: 0,
            bazi_sect: 2,
            yun_sect: 1,
            leap_month: false,
        }
    }

    #[test]
    fn android_response_keeps_full_luck_and_quant_data() {
        let response = calculate_bazi(sample_request()).expect("sample chart should build");
        let full_size = serde_json::to_vec(&response)
            .expect("response should serialize")
            .len();
        let android_response = optimize_response_for_android(response);
        let android_size = serde_json::to_vec(&android_response)
            .expect("android response should serialize")
            .len();

        println!("full_size={full_size}, android_size={android_size}");

        assert_eq!(android_size, full_size);
        assert!(!android_response.da_yun.is_empty());
        assert!(android_response.da_yun.iter().any(|luck| luck
            .liu_nian
            .iter()
            .any(|annual| !annual.liu_yue.is_empty())));
        assert!(android_response.quant_model.is_some());
        assert!(android_response
            .quant_model
            .as_ref()
            .and_then(|quant_model| quant_model.luck_scores.as_ref())
            .is_some_and(|luck_scores| luck_scores.iter().any(|score| score
                .annual_scores
                .as_ref()
                .is_some_and(|annual_scores| !annual_scores.is_empty()))));
        let quant_model = android_response
            .quant_model
            .as_ref()
            .expect("quant model should exist");
        assert!(quant_model.cong_pattern.is_some());
        assert!(quant_model.sha_yin.is_some());
        assert!(!quant_model.stem_scores.is_empty());
        assert_eq!(quant_model.branch_scores.len(), 4);
    }

    #[test]
    fn reported_lunar_chart_builds() {
        let response =
            calculate_bazi(reported_lunar_request()).expect("reported lunar chart should build");
        let bytes = serde_json::to_vec(&response)
            .expect("response should serialize")
            .len();
        println!("reported_lunar_chart_bytes={bytes}");
        assert!(!response.da_yun.is_empty());
    }

    #[test]
    fn roc_0071_11_22_luck_starts_at_virtual_age_6() {
        let response = calculate_bazi(BaziRequest {
            calendar_type: CalendarType::Solar,
            gender: Gender::Female,
            year: 71,
            year_era: YearEra::Roc,
            month: 11,
            day: 22,
            hour: 11,
            minute: 0,
            second: 0,
            bazi_sect: 2,
            yun_sect: 1,
            leap_month: false,
        })
        .expect("chart should build");

        let starts = response
            .da_yun
            .iter()
            .take(2)
            .map(|row| row.start_age)
            .collect::<Vec<_>>();
        assert_eq!(starts, vec![Some(6), Some(16)]);
    }

    #[test]
    fn roc_0088_02_12_male_luck_starts_at_virtual_age_3() {
        let response = calculate_bazi(BaziRequest {
            calendar_type: CalendarType::Solar,
            gender: Gender::Male,
            year: 88,
            year_era: YearEra::Roc,
            month: 2,
            day: 12,
            hour: 20,
            minute: 0,
            second: 0,
            bazi_sect: 2,
            yun_sect: 1,
            leap_month: false,
        })
        .expect("chart should build");

        assert_eq!(response.luck_start.forward, Some(false));
        assert_eq!(
            response.luck_start.start_solar.as_deref(),
            Some("2001-11-12 20:00:00")
        );
        assert_eq!(response.luck_start.start_year, Some(2));
        assert_eq!(response.luck_start.start_month, Some(9));
        assert_eq!(response.luck_start.start_day, Some(0));

        let starts = response
            .da_yun
            .iter()
            .take(2)
            .map(|row| row.start_age)
            .collect::<Vec<_>>();
        assert_eq!(starts, vec![Some(3), Some(13)]);
    }

    #[test]
    fn calculated_chart_uses_ge_ju_analyzer() {
        let response = calculate_bazi(sample_request()).expect("sample chart should build");

        assert_ne!(response.ge_ju, "待移植");
        assert!(!response.ge_ju.is_empty());
        assert!(response.ge_ju.ends_with('格'));
        assert!(response.ge_ju_basis.contains("以月令十神"));
    }

    #[test]
    fn ge_ju_prefers_month_hidden_ten_god_that_appears_in_stems() {
        let response = calculate_bazi(BaziRequest {
            calendar_type: CalendarType::Solar,
            gender: Gender::Female,
            year: 1997,
            year_era: YearEra::Ad,
            month: 10,
            day: 28,
            hour: 18,
            minute: 0,
            second: 0,
            bazi_sect: 2,
            yun_sect: 1,
            leap_month: false,
        })
        .expect("chart should build");

        assert_eq!(response.ba_zi, "丁丑 庚戌 癸卯 辛酉");
        assert_eq!(response.month_branch_ten_gods, vec!["正官", "偏印", "偏財"]);
        assert_eq!(response.hour_stem_ten_god, "偏印");
        assert_eq!(response.ge_ju, "偏印格");
        assert!(response
            .ge_ju_basis
            .contains("月支戌藏戊(正官)、辛(偏印)、丁(偏財)"));
        assert!(response.ge_ju_basis.contains("辛金透於時干"));
        assert!(response.ge_ju_basis.contains("丁火透於年干"));
        assert!(response.ge_ju_basis.contains("戊土未透"));
        assert!(response.ge_ju_basis.contains("辛金坐酉祿，得祿有根"));
        assert!(response.ge_ju_basis.contains("丁火坐丑濕土，火氣受晦"));
        assert!(response.ge_ju_basis.contains("辛金偏印較丁火偏財有力"));
        assert!(response.ge_ju_basis.contains("取較強之「偏印」為格"));
    }

    #[test]
    fn ge_ju_explains_multiple_transparent_month_hidden_candidates_by_strength() {
        let result = analyze_ge_ju(
            "戌",
            &["戊".to_string(), "辛".to_string(), "丁".to_string()],
            &["正官".to_string(), "偏印".to_string(), "偏財".to_string()],
            [
                VisibleStemTenGod::new("年干", "辛", "酉", "偏印"),
                VisibleStemTenGod::new("月干", "丁", "丑", "偏財"),
                VisibleStemTenGod::new("日干", "癸", "卯", "日元"),
                VisibleStemTenGod::new("時干", "乙", "未", "食神"),
            ],
        );

        assert_eq!(result.ge_ju, "偏印格");
        assert!(result.basis.contains("辛金透於年干、丁火透於月干"));
        assert!(result.basis.contains("辛金坐酉祿，得祿有根"));
        assert!(result.basis.contains("丁火坐丑濕土，火氣受晦"));
        assert!(result.basis.contains("辛金偏印較丁火偏財有力"));
    }

    #[test]
    fn ge_ju_can_choose_lower_layer_candidate_when_whole_chart_element_is_stronger() {
        let result = analyze_ge_ju(
            "戌",
            &["戊".to_string(), "辛".to_string(), "丁".to_string()],
            &["正官".to_string(), "偏印".to_string(), "偏財".to_string()],
            [
                VisibleStemTenGod::new("年干", "丁", "午", "偏財"),
                VisibleStemTenGod::new("月干", "丙", "午", "正財"),
                VisibleStemTenGod::new("日干", "癸", "巳", "日元"),
                VisibleStemTenGod::new("時干", "辛", "亥", "偏印"),
            ],
        );

        assert_eq!(result.ge_ju, "偏財格");
        assert!(result.basis.contains("丁火透於年干"));
        assert!(result.basis.contains("辛金透於時干"));
        assert!(result.basis.contains("丁火偏財較辛金偏印有力"));
        assert!(!result.basis.contains("若全盤火勢旺"));
        assert!(!result.basis.contains("也可能"));
    }

    #[test]
    fn ge_ju_strength_rules_are_general_not_single_case() {
        assert_eq!(stem_branch_root_score("丙", "辰"), -10);
        assert_eq!(stem_branch_root_score("丁", "丑"), -10);
        assert!(stem_branch_strength_description("丙", "辰").contains("丙火坐辰濕土"));
        assert!(stem_branch_strength_description("丁", "丑").contains("丁火坐丑濕土"));

        let visible_stems = [
            VisibleStemTenGod::new("年干", "丁", "午", "偏財"),
            VisibleStemTenGod::new("月干", "丙", "午", "正財"),
            VisibleStemTenGod::new("日干", "癸", "巳", "日元"),
            VisibleStemTenGod::new("時干", "辛", "亥", "偏印"),
        ];
        assert!(
            chart_element_strength_score('火', &visible_stems)
                > chart_element_strength_score('金', &visible_stems)
        );
    }

    #[test]
    fn pillar_analysis_adds_common_shen_sha() {
        let response = analyze_pillars(PillarAnalyzeRequest {
            year_pillar: "甲子".to_string(),
            month_pillar: "乙丑".to_string(),
            day_pillar: "丙寅".to_string(),
            hour_pillar: "丁卯".to_string(),
            gender: Some(Gender::Female),
            selected_gregorian_year: None,
            infer_gregorian_years: Some(true),
        })
        .expect("pillar analysis should succeed");

        let shen_sha = response.shen_sha.expect("shen sha should exist");
        assert!(shen_sha.year.contains(&"將星".to_string()));
        assert!(shen_sha.month.contains(&"天乙貴人".to_string()));
        assert!(shen_sha.day.contains(&"驛馬".to_string()));
        assert!(shen_sha.hour.contains(&"桃花".to_string()));
        assert!(shen_sha.matches.len() >= 4);
        let year_hint = response
            .direct_pillar_year_hint
            .expect("direct pillar year hint should exist");
        assert!(!year_hint.note.is_empty());
    }
}

fn normalize_year(year: i32, year_era: &YearEra) -> Result<i32, String> {
    let normalized = match year_era {
        YearEra::Ad => year,
        YearEra::Roc => year + 1911,
    };

    if !(1..=9999).contains(&normalized) {
        Err("年份超出 tyme4rs 可處理範圍".to_string())
    } else {
        Ok(normalized)
    }
}

fn create_solar_time(request: &BaziRequest, normalized_year: i32) -> Result<SolarTime, String> {
    let second = request.second as usize;
    match request.calendar_type {
        CalendarType::Solar => SolarTime::new(
            normalized_year as isize,
            request.month as usize,
            request.day as usize,
            request.hour as usize,
            request.minute as usize,
            second,
        ),
        CalendarType::Lunar => {
            let month = if request.leap_month {
                -(request.month as isize)
            } else {
                request.month as isize
            };
            let lunar_hour = LunarHour::new(
                normalized_year as isize,
                month,
                request.day as usize,
                request.hour as usize,
                request.minute as usize,
                second,
            )?;
            Ok(lunar_hour.get_solar_time())
        }
    }
}

fn create_eight_char(bazi_sect: u8, lunar_hour: LunarHour) -> EightChar {
    if bazi_sect == 2 {
        LunarSect2EightCharProvider::new().get_eight_char(lunar_hour)
    } else {
        DefaultEightCharProvider::new().get_eight_char(lunar_hour)
    }
}

fn build_pillar(pillar: SixtyCycle, day_master: HeavenStem) -> PillarDto {
    let stem = pillar.get_heaven_stem();
    let branch = pillar.get_earth_branch();
    PillarDto {
        gan_zhi: trad(&pillar.get_name()),
        stem: trad(&stem.get_name()),
        branch: trad(&branch.get_name()),
        na_yin: trad(&pillar.get_sound().get_name()),
        wu_xing: format!(
            "{}{}",
            trad(&stem.get_element().get_name()),
            trad(&branch.get_element().get_name())
        ),
        di_shi: trad(&day_master.get_terrain(branch.clone()).get_name()),
        xun: format!("{}旬", trad(&pillar.get_ten().get_name())),
        xun_kong: pillar
            .get_extra_earth_branches()
            .into_iter()
            .map(|item| trad(&item.get_name()))
            .collect::<Vec<_>>()
            .join(""),
    }
}

fn hidden_stems(branch: EarthBranch) -> Vec<String> {
    branch
        .get_hide_heaven_stems()
        .into_iter()
        .map(|item| trad(&item.get_name()))
        .collect()
}

fn hidden_ten_gods(day_master: HeavenStem, branch: EarthBranch) -> Vec<String> {
    branch
        .get_hide_heaven_stems()
        .into_iter()
        .map(|item| ten_god_name(day_master.clone(), item.get_heaven_stem()))
        .collect()
}

fn ten_god_name(day_master: HeavenStem, target: HeavenStem) -> String {
    trad(&day_master.get_ten_star(target).get_name())
}

fn analyze_ge_ju(
    month_branch: &str,
    month_hidden_stems: &[String],
    month_branch_ten_gods: &[String],
    visible_stems: [VisibleStemTenGod; 4],
) -> GeJuResult {
    let candidates =
        build_ge_ju_candidates(month_hidden_stems, month_branch_ten_gods, visible_stems);
    let Some(dominant_candidate) = pick_dominant_ge_ju_candidate(&candidates) else {
        return GeJuResult {
            ge_ju: "未定格".to_string(),
            basis: "無法從月令十神取得有效資料".to_string(),
        };
    };

    let pattern = ge_ju_pattern_name(dominant_candidate.ten_god);
    let basis = build_ge_ju_basis(month_branch, &candidates, dominant_candidate, pattern);

    GeJuResult {
        ge_ju: pattern.to_string(),
        basis,
    }
}

fn build_ge_ju_candidates<'a>(
    month_hidden_stems: &'a [String],
    month_branch_ten_gods: &'a [String],
    visible_stems: [VisibleStemTenGod<'a>; 4],
) -> Vec<GeJuCandidate<'a>> {
    month_hidden_stems
        .iter()
        .zip(month_branch_ten_gods.iter())
        .enumerate()
        .filter_map(|(hidden_index, (stem, ten_god))| {
            if !is_supported_ge_ju_ten_god(ten_god) {
                return None;
            }

            let transparent_stems = visible_stems
                .iter()
                .copied()
                .filter(|visible| {
                    visible.pillar_label != "日干"
                        && visible.stem == stem.as_str()
                        && visible.ten_god == ten_god.as_str()
                })
                .collect::<Vec<_>>();

            Some(GeJuCandidate {
                hidden_index,
                stem,
                ten_god,
                chart_element_score: stem_element(stem)
                    .map(|element| chart_element_strength_score(element, &visible_stems))
                    .unwrap_or(0),
                transparent_stems,
            })
        })
        .collect()
}

fn pick_dominant_ge_ju_candidate<'a>(
    candidates: &'a [GeJuCandidate<'a>],
) -> Option<&'a GeJuCandidate<'a>> {
    candidates
        .iter()
        .filter(|candidate| !candidate.transparent_stems.is_empty())
        .max_by_key(|candidate| {
            (
                ge_ju_candidate_strength_score(candidate),
                -(candidate.hidden_index as i32),
                -candidate
                    .transparent_stems
                    .iter()
                    .map(|visible| stem_pillar_rank(visible.pillar_label) as i32)
                    .min()
                    .unwrap_or(99),
            )
        })
        .or_else(|| {
            candidates
                .iter()
                .min_by_key(|candidate| candidate.hidden_index)
        })
}

fn build_ge_ju_basis(
    month_branch: &str,
    candidates: &[GeJuCandidate],
    dominant_candidate: &GeJuCandidate,
    pattern: &str,
) -> String {
    let hidden_summary = candidates
        .iter()
        .map(|candidate| format!("{}({})", candidate.stem, candidate.ten_god))
        .collect::<Vec<_>>()
        .join("、");
    let transparent_candidates = candidates
        .iter()
        .filter(|candidate| !candidate.transparent_stems.is_empty())
        .collect::<Vec<_>>();
    let transparent_summary = transparent_candidates
        .iter()
        .map(|candidate| {
            let positions = candidate
                .transparent_stems
                .iter()
                .map(|visible| visible.pillar_label)
                .collect::<Vec<_>>()
                .join("、");
            format!("{}透於{}", stem_with_element(candidate.stem), positions)
        })
        .collect::<Vec<_>>()
        .join("、");
    let non_transparent_summary = {
        let non_transparent = candidates
            .iter()
            .filter(|candidate| candidate.transparent_stems.is_empty())
            .map(|candidate| stem_with_element(candidate.stem))
            .collect::<Vec<_>>();
        if non_transparent.is_empty() {
            "其餘月令藏干皆有透出".to_string()
        } else {
            format!("{}未透", non_transparent.join("、"))
        }
    };
    let decision = if transparent_candidates.is_empty() {
        format!(
            "月令藏干皆未透，故回取{}「{}」為主，判為「{}」",
            qi_layer_name(dominant_candidate.hidden_index),
            dominant_candidate.ten_god,
            pattern
        )
    } else if transparent_candidates.len() == 1 {
        format!(
            "故取透干之「{}」為格，判為「{}」",
            dominant_candidate.ten_god, pattern
        )
    } else {
        let strength_reason = build_multiple_transparency_strength_reason(
            dominant_candidate,
            &transparent_candidates,
        );
        format!(
            "月令藏干有多個透干，{}，故取較強之「{}」為格，判為「{}」",
            strength_reason, dominant_candidate.ten_god, pattern
        )
    };

    if transparent_candidates.is_empty() {
        format!(
            "以月令十神分析：月支{}藏{}。{}。",
            month_branch, hidden_summary, decision
        )
    } else {
        format!(
            "以月令十神分析：月支{}藏{}。其中{}，{}，{}。",
            month_branch, hidden_summary, transparent_summary, non_transparent_summary, decision
        )
    }
}

fn build_multiple_transparency_strength_reason(
    dominant_candidate: &GeJuCandidate,
    transparent_candidates: &[&GeJuCandidate],
) -> String {
    let weaker_transparent = transparent_candidates
        .iter()
        .filter(|candidate| candidate.ten_god != dominant_candidate.ten_god)
        .map(|candidate| {
            format!(
                "{}，{}",
                strongest_transparency_strength_description(candidate),
                candidate_with_ten_god(candidate)
            )
        })
        .collect::<Vec<_>>()
        .join("；");

    if weaker_transparent.is_empty() {
        return format!(
            "{}，透出位置與根氣較有力",
            strongest_transparency_strength_description(dominant_candidate)
        );
    }

    format!(
        "{}；{}；{}較{}有力",
        strongest_transparency_strength_description(dominant_candidate),
        weaker_transparent
            .split('；')
            .collect::<Vec<_>>()
            .join("；"),
        candidate_with_ten_god(dominant_candidate),
        transparent_candidates
            .iter()
            .filter(|candidate| candidate.ten_god != dominant_candidate.ten_god)
            .map(|candidate| candidate_with_ten_god(candidate))
            .collect::<Vec<_>>()
            .join("、")
    )
}

fn ge_ju_candidate_strength_score(candidate: &GeJuCandidate) -> i32 {
    let strongest_root_score = candidate
        .transparent_stems
        .iter()
        .map(|visible| stem_branch_root_score(visible.stem, visible.branch))
        .max()
        .unwrap_or(0);
    let qi_score = match candidate.hidden_index {
        0 => 12,
        1 => 8,
        _ => 4,
    };
    let position_score = candidate
        .transparent_stems
        .iter()
        .map(|visible| match visible.pillar_label {
            "月干" => 6,
            "時干" => 4,
            "年干" => 2,
            _ => 0,
        })
        .max()
        .unwrap_or(0);

    strongest_root_score + qi_score + position_score + candidate.chart_element_score
}

fn strongest_transparency_strength_description(candidate: &GeJuCandidate) -> String {
    candidate
        .transparent_stems
        .iter()
        .max_by_key(|visible| stem_branch_root_score(visible.stem, visible.branch))
        .map(|visible| stem_branch_strength_description(visible.stem, visible.branch))
        .unwrap_or_else(|| format!("{}未見明顯根氣", stem_with_element(candidate.stem)))
}

fn candidate_with_ten_god(candidate: &GeJuCandidate) -> String {
    format!("{}{}", stem_with_element(candidate.stem), candidate.ten_god)
}

fn stem_branch_root_score(stem: &str, branch: &str) -> i32 {
    if is_stem_lu_branch(stem, branch) {
        return 60;
    }
    if stem_element(stem) == Some('火') && is_wet_earth_branch(branch) {
        return -10;
    }
    if branch_hidden_stems(branch).contains(&stem) {
        return 36;
    }
    if stem_element(stem) == branch_main_element(branch) {
        return 22;
    }
    0
}

fn stem_branch_strength_description(stem: &str, branch: &str) -> String {
    if is_stem_lu_branch(stem, branch) {
        return format!("{}坐{}祿，得祿有根", stem_with_element(stem), branch);
    }
    if stem_element(stem) == Some('火') && is_wet_earth_branch(branch) {
        return format!("{}坐{}濕土，火氣受晦", stem_with_element(stem), branch);
    }
    if branch_hidden_stems(branch).contains(&stem) {
        return format!("{}坐{}，地支藏根", stem_with_element(stem), branch);
    }
    if stem_element(stem) == branch_main_element(branch) {
        return format!("{}坐{}，同氣有根", stem_with_element(stem), branch);
    }
    format!("{}坐{}，根氣較平", stem_with_element(stem), branch)
}

fn chart_element_strength_score(element: char, visible_stems: &[VisibleStemTenGod]) -> i32 {
    visible_stems
        .iter()
        .map(|visible| {
            let stem_score = if stem_element(visible.stem) == Some(element) {
                10
            } else {
                0
            };
            let branch_score = if branch_main_element(visible.branch) == Some(element) {
                8
            } else if branch_hidden_stems(visible.branch)
                .iter()
                .any(|hidden| stem_element(hidden) == Some(element))
            {
                4
            } else {
                0
            };
            stem_score + branch_score
        })
        .sum()
}

fn is_wet_earth_branch(branch: &str) -> bool {
    matches!(branch, "丑" | "辰")
}

fn branch_hidden_stems(branch: &str) -> &'static [&'static str] {
    match branch {
        "子" => &["癸"],
        "丑" => &["己", "癸", "辛"],
        "寅" => &["甲", "丙", "戊"],
        "卯" => &["乙"],
        "辰" => &["戊", "乙", "癸"],
        "巳" => &["丙", "戊", "庚"],
        "午" => &["丁", "己"],
        "未" => &["己", "丁", "乙"],
        "申" => &["庚", "壬", "戊"],
        "酉" => &["辛"],
        "戌" => &["戊", "辛", "丁"],
        "亥" => &["壬", "甲"],
        _ => &[],
    }
}

fn is_stem_lu_branch(stem: &str, branch: &str) -> bool {
    matches!(
        (stem, branch),
        ("甲", "寅")
            | ("乙", "卯")
            | ("丙", "巳")
            | ("丁", "午")
            | ("戊", "巳")
            | ("己", "午")
            | ("庚", "申")
            | ("辛", "酉")
            | ("壬", "亥")
            | ("癸", "子")
    )
}

fn stem_element(stem: &str) -> Option<char> {
    stem.chars().next().and_then(|item| match item {
        '甲' | '乙' => Some('木'),
        '丙' | '丁' => Some('火'),
        '戊' | '己' => Some('土'),
        '庚' | '辛' => Some('金'),
        '壬' | '癸' => Some('水'),
        _ => None,
    })
}

fn branch_main_element(branch: &str) -> Option<char> {
    branch.chars().next().and_then(|item| match item {
        '寅' | '卯' => Some('木'),
        '巳' | '午' => Some('火'),
        '辰' | '戌' | '丑' | '未' => Some('土'),
        '申' | '酉' => Some('金'),
        '亥' | '子' => Some('水'),
        _ => None,
    })
}

fn stem_pillar_rank(pillar_label: &str) -> u8 {
    match pillar_label {
        "月干" => 0,
        "時干" => 1,
        "年干" => 2,
        _ => 9,
    }
}

fn qi_layer_name(hidden_index: usize) -> &'static str {
    match hidden_index {
        0 => "本氣",
        1 => "中氣",
        _ => "餘氣",
    }
}

fn stem_with_element(stem: &str) -> String {
    let element = stem
        .chars()
        .next()
        .and_then(stem_element_name)
        .unwrap_or("");
    format!("{}{}", stem, element)
}

fn stem_element_name(stem: char) -> Option<&'static str> {
    match stem {
        '甲' | '乙' => Some("木"),
        '丙' | '丁' => Some("火"),
        '戊' | '己' => Some("土"),
        '庚' | '辛' => Some("金"),
        '壬' | '癸' => Some("水"),
        _ => None,
    }
}

fn is_supported_ge_ju_ten_god(ten_god: &str) -> bool {
    matches!(
        ten_god,
        "比肩" | "劫財" | "食神" | "傷官" | "偏財" | "正財" | "七殺" | "正官" | "偏印" | "正印"
    )
}

fn ge_ju_pattern_name(ten_god: &str) -> &str {
    match ten_god {
        "比肩" => "建祿格",
        "劫財" => "羊刃格",
        "食神" => "食神格",
        "傷官" => "傷官格",
        "偏財" => "偏財格",
        "正財" => "正財格",
        "七殺" => "七殺格",
        "正官" => "正官格",
        "偏印" => "偏印格",
        "正印" => "正印格",
        _ => ten_god,
    }
}

fn build_luck_start(
    request: &BaziRequest,
    birth_time: SolarTime,
    eight_char: &EightChar,
) -> Result<LuckStartDto, String> {
    let gender = to_tyme_gender(&request.gender);
    let forward = is_forward(eight_char, gender);
    let mut term = birth_time.get_term();
    if !term.is_jie() {
        term = term.next(-1);
    }
    if forward {
        term = term.next(2);
    }

    let info = child_limit_info(request.yun_sect, birth_time, term)?;
    let start_time = info.get_end_time();
    let birth_prev_jie = previous_jie(birth_time);
    let birth_prev_jie_time = birth_prev_jie.get_julian_day().get_solar_time();
    let birth_prev_jie_days =
        (birth_time.subtract(birth_prev_jie_time) as f64 / 86400.0).round() as i32;
    let normalized_offset = normalize_luck_offset(
        info.get_year_count() as i32,
        info.get_month_count() as i32,
        info.get_day_count() as i32,
        info.get_hour_count() as i32,
    );
    let normalized_offset_text = format_luck_offset(&normalized_offset);
    let birth_year_gan_zhi = trad(&eight_char.get_year().get_name());
    let birth_jie_name = trad(&birth_prev_jie.get_name());
    let birth_jie_day_ordinal = birth_prev_jie_days.max(0);
    let transition_schedule = derive_transition_schedule(
        eight_char.get_year().get_heaven_stem(),
        birth_prev_jie.clone(),
        birth_jie_day_ordinal,
        normalized_offset,
    );

    Ok(LuckStartDto {
        forward: Some(forward),
        start_year: Some(info.get_year_count() as i32),
        start_month: Some(info.get_month_count() as i32),
        start_day: Some(info.get_day_count() as i32),
        start_hour: Some(info.get_hour_count() as i32),
        start_solar: Some(format_solar_time(start_time)),
        start_summary: Some(format!(
            "出生為{}年{}後{}日，出生後{}上大運",
            birth_year_gan_zhi, birth_jie_name, birth_jie_day_ordinal, normalized_offset_text
        )),
        transition_summary: Some(format!(
            "每逢{}年及{}年{}後{}日交脫大運",
            transition_schedule.primary_year_stem,
            transition_schedule.secondary_year_stem,
            transition_schedule.term_name,
            transition_schedule.day_ordinal
        )),
        birth_jie_name: Some(birth_jie_name.clone()),
        birth_jie_solar: Some(format_solar_time(birth_prev_jie_time)),
        birth_jie_day_ordinal: Some(birth_jie_day_ordinal),
        transition_summary_experimental: Some(format!(
            "陽曆{}上大運（30日進位推算；原始位移 {}年{}月{}日{}時；交脫{}年/{}年 {}後{}日）",
            format_solar_time(start_time),
            info.get_year_count(),
            info.get_month_count(),
            info.get_day_count(),
            info.get_hour_count(),
            transition_schedule.primary_year_stem,
            transition_schedule.secondary_year_stem,
            transition_schedule.term_name,
            transition_schedule.day_ordinal
        )),
    })
}

#[derive(Debug, Clone, Copy)]
struct NormalizedLuckOffset {
    years: i32,
    months: i32,
    days: i32,
}

fn normalize_luck_offset(years: i32, months: i32, days: i32, hours: i32) -> NormalizedLuckOffset {
    let mut total_days = days.max(0);
    if hours >= 24 {
        total_days += hours / 24;
    }

    let mut total_months = months.max(0) + total_days / 30;
    let normalized_days = total_days % 30;
    let normalized_years = years.max(0) + total_months / 12;
    total_months %= 12;

    NormalizedLuckOffset {
        years: normalized_years,
        months: total_months,
        days: normalized_days,
    }
}

fn format_luck_offset(offset: &NormalizedLuckOffset) -> String {
    format!("{}年{}月{}日", offset.years, offset.months, offset.days)
}

#[derive(Debug, Clone)]
struct TransitionSchedule {
    primary_year_stem: String,
    secondary_year_stem: String,
    term_name: String,
    day_ordinal: i32,
}

fn derive_transition_schedule(
    birth_year_stem: HeavenStem,
    birth_jie: SolarTerm,
    birth_jie_day_ordinal: i32,
    offset: NormalizedLuckOffset,
) -> TransitionSchedule {
    let mut target_term = birth_jie;
    let mut crossed_lichun_years = 0;
    for _ in 0..offset.months.max(0) {
        target_term = next_jie(target_term, &mut crossed_lichun_years);
    }

    let mut target_day_ordinal = birth_jie_day_ordinal.max(0) + offset.days.max(0);
    while target_day_ordinal > 30 {
        target_day_ordinal -= 30;
        target_term = next_jie(target_term, &mut crossed_lichun_years);
    }

    let effective_years = offset.years.max(0) + crossed_lichun_years;
    let primary_year_stem = birth_year_stem.next(effective_years as isize);
    let secondary_year_stem = primary_year_stem.next(5);

    TransitionSchedule {
        primary_year_stem: trad(&primary_year_stem.get_name()),
        secondary_year_stem: trad(&secondary_year_stem.get_name()),
        term_name: trad(&target_term.get_name()),
        day_ordinal: target_day_ordinal,
    }
}

fn next_jie(term: SolarTerm, crossed_lichun_years: &mut i32) -> SolarTerm {
    let next_term = term.next(2);
    if next_term.get_name() == "立春" {
        *crossed_lichun_years += 1;
    }
    next_term
}

fn build_da_yun(
    request: &BaziRequest,
    birth_time: SolarTime,
    eight_char: &EightChar,
    day_master: HeavenStem,
) -> Result<Vec<DaYunDto>, String> {
    let gender = to_tyme_gender(&request.gender);
    let forward = is_forward(eight_char, gender);
    let mut term = birth_time.get_term();
    if !term.is_jie() {
        term = term.next(-1);
    }
    if forward {
        term = term.next(2);
    }

    let info = child_limit_info(request.yun_sect, birth_time, term)?;
    let first_start_time = info.get_end_time();
    let first_start_year = first_start_time.get_year() as i32;
    let first_start_age = display_luck_start_age(virtual_age_at(first_start_time, birth_time));

    let mut cycles = Vec::new();
    for index in 0..9 {
        let sequence = index as isize + 1;
        let pillar = eight_char
            .get_month()
            .next(if forward { sequence } else { -sequence });
        let branch = pillar.get_earth_branch();
        let start_year = first_start_year + index * 10;
        let start_age = first_start_age + index * 10;

        let mut years = Vec::new();
        for year_index in 0..11 {
            let year = start_year + year_index;
            let cycle_year = SixtyCycleYear::from_year(year as isize).get_sixty_cycle();
            let annual_branch = cycle_year.get_earth_branch();
            years.push(LiuNianDto {
                index: year_index,
                year,
                age: start_age + year_index,
                gan_zhi: trad(&cycle_year.get_name()),
                stem_ten_god: ten_god_name(day_master.clone(), cycle_year.get_heaven_stem()),
                branch_hidden_stems: hidden_stems(annual_branch.clone()),
                branch_ten_gods: hidden_ten_gods(day_master.clone(), annual_branch),
                xun: format!("{}旬", trad(&cycle_year.get_ten().get_name())),
                xun_kong: cycle_year
                    .get_extra_earth_branches()
                    .into_iter()
                    .map(|item| trad(&item.get_name()))
                    .collect::<Vec<_>>()
                    .join(""),
                liu_yue: build_liu_yue(year, day_master.clone()),
            });
        }

        cycles.push(DaYunDto {
            index: index + 1,
            start_age: Some(start_age),
            end_age: Some(start_age + 9),
            start_year: Some(start_year),
            end_year: Some(start_year + 9),
            gan_zhi: trad(&pillar.get_name()),
            stem_ten_god: ten_god_name(day_master.clone(), pillar.get_heaven_stem()),
            branch_hidden_stems: hidden_stems(branch.clone()),
            branch_ten_gods: hidden_ten_gods(day_master.clone(), branch),
            xun: format!("{}旬", trad(&pillar.get_ten().get_name())),
            xun_kong: pillar
                .get_extra_earth_branches()
                .into_iter()
                .map(|item| trad(&item.get_name()))
                .collect::<Vec<_>>()
                .join(""),
            liu_nian: years,
        });
    }

    Ok(cycles)
}

fn virtual_age_at(target_time: SolarTime, birth_time: SolarTime) -> i32 {
    target_time.get_year() as i32 - birth_time.get_year() as i32 + 1
}

fn display_luck_start_age(raw_start_age: i32) -> i32 {
    raw_start_age
}

fn build_direct_da_yun(
    month_pillar: SixtyCycle,
    day_master: HeavenStem,
    forward: bool,
) -> Vec<DaYunDto> {
    let mut rows = Vec::new();
    for index in 0..DIRECT_PILLAR_LUCK_COUNT {
        let step = index as isize + 1;
        let pillar = month_pillar.next(if forward { step } else { -step });
        let branch = pillar.get_earth_branch();
        rows.push(DaYunDto {
            index: index + 1,
            start_age: None,
            end_age: None,
            start_year: None,
            end_year: None,
            gan_zhi: trad(&pillar.get_name()),
            stem_ten_god: ten_god_name(day_master.clone(), pillar.get_heaven_stem()),
            branch_hidden_stems: hidden_stems(branch.clone()),
            branch_ten_gods: hidden_ten_gods(day_master.clone(), branch),
            xun: format!("{}旬", trad(&pillar.get_ten().get_name())),
            xun_kong: pillar
                .get_extra_earth_branches()
                .into_iter()
                .map(|item| trad(&item.get_name()))
                .collect::<Vec<_>>()
                .join(""),
            liu_nian: Vec::new(),
        });
    }
    rows
}

fn child_limit_info(
    yun_sect: u8,
    birth_time: SolarTime,
    term: SolarTerm,
) -> Result<tyme4rs::tyme::eightchar::ChildLimitInfo, String> {
    Ok(match yun_sect {
        2 => LunarSect2ChildLimitProvider::new().get_info(birth_time, term),
        _ => LunarSect1ChildLimitProvider::new().get_info(birth_time, term),
    })
}

fn build_liu_yue(year: i32, day_master: HeavenStem) -> Vec<LiuYueDto> {
    let year_cycle = SixtyCycleYear::from_year(year as isize).get_sixty_cycle();
    let first_month_stem = first_month_stem(year_cycle.get_heaven_stem());

    LIU_YUE_BRANCHES
        .iter()
        .enumerate()
        .map(|(index, branch_name)| {
            let branch = EarthBranch::from_name(branch_name);
            let stem = first_month_stem.next(index as isize);
            let cycle_name = format!("{}{}", stem.get_name(), branch.get_name());
            let cycle = SixtyCycle::from_name(&cycle_name);
            let jie_qi = trad(LIU_YUE_JIE_QI[index]);
            let jie_qi_date = Some(resolve_liu_yue_jie_qi(year, index));

            LiuYueDto {
                index: index as i32,
                jie_qi: Some(jie_qi),
                jie_qi_date,
                month: MONTH_LABELS[index].to_string(),
                gan_zhi: trad(&cycle.get_name()),
                stem_ten_god: ten_god_name(day_master.clone(), stem),
                branch_hidden_stems: hidden_stems(branch.clone()),
                branch_ten_gods: hidden_ten_gods(day_master.clone(), branch),
                xun: format!("{}旬", trad(&cycle.get_ten().get_name())),
                xun_kong: cycle
                    .get_extra_earth_branches()
                    .into_iter()
                    .map(|item| trad(&item.get_name()))
                    .collect::<Vec<_>>()
                    .join(""),
            }
        })
        .collect()
}

fn first_month_stem(year_stem: HeavenStem) -> HeavenStem {
    match year_stem.get_name().as_str() {
        "甲" | "己" => HeavenStem::from_name("丙"),
        "乙" | "庚" => HeavenStem::from_name("戊"),
        "丙" | "辛" => HeavenStem::from_name("庚"),
        "丁" | "壬" => HeavenStem::from_name("壬"),
        _ => HeavenStem::from_name("甲"),
    }
}

fn resolve_liu_yue_jie_qi(year: i32, index: usize) -> String {
    let target_year = if index == LIU_YUE_JIE_QI.len() - 1 {
        year + 1
    } else {
        year
    };
    let term = SolarTerm::from_name(target_year as isize, LIU_YUE_JIE_QI[index]);
    format_month_day_with_hour(term.get_julian_day().get_solar_time())
}

fn previous_jie(birth_time: SolarTime) -> SolarTerm {
    let mut term = birth_time.get_term();
    if term.is_jie() {
        let term_time = term.get_julian_day().get_solar_time();
        if birth_time.is_before(term_time) {
            term = term.next(-2);
        }
    } else {
        term = term.next(-1);
    }
    term
}

fn is_forward(eight_char: &EightChar, gender: TymeGender) -> bool {
    let yang = eight_char.get_year().get_heaven_stem().get_yin_yang() == YinYang::YANG;
    let man = gender == TymeGender::MAN;
    (yang && man) || (!yang && !man)
}

fn is_forward_direct(year_stem: HeavenStem, gender: &Gender) -> bool {
    let yang = year_stem.get_yin_yang() == YinYang::YANG;
    matches!(gender, Gender::Male) == yang
}

fn to_tyme_gender(gender: &Gender) -> TymeGender {
    match gender {
        Gender::Male => TymeGender::MAN,
        Gender::Female => TymeGender::WOMAN,
    }
}

fn parse_direct_pillar(input: &str) -> Result<SixtyCycle, String> {
    let value = input.trim();
    if value.chars().count() != 2 {
        return Err(format!("四柱格式錯誤：{}，每柱必須為兩個字。", input));
    }
    if !SIXTY_CYCLE_NAMES.contains(&value) {
        return Err(format!("四柱不屬於有效六十甲子：{}", input));
    }
    Ok(SixtyCycle::from_name(value))
}

fn format_input(request: &BaziRequest) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        request.year, request.month, request.day, request.hour, request.minute, request.second
    )
}

fn format_solar_time(time: SolarTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        time.get_year(),
        time.get_month(),
        time.get_day(),
        time.get_hour(),
        time.get_minute(),
        time.get_second()
    )
}

fn format_month_day(time: SolarTime) -> String {
    format!("{}/{}", time.get_month(), time.get_day())
}

fn format_month_day_with_hour(time: SolarTime) -> String {
    format!(
        "{}/{} {}",
        time.get_month(),
        time.get_day(),
        hour_branch_label(time.get_hour() as i32)
    )
}

fn hour_branch_label(hour: i32) -> &'static str {
    match hour.rem_euclid(24) {
        23 | 0 => "子",
        1 | 2 => "丑",
        3 | 4 => "寅",
        5 | 6 => "卯",
        7 | 8 => "辰",
        9 | 10 => "巳",
        11 | 12 => "午",
        13 | 14 => "未",
        15 | 16 => "申",
        17 | 18 => "酉",
        19 | 20 => "戌",
        _ => "亥",
    }
}

fn format_lunar_time(hour: LunarHour) -> String {
    let lunar_day = hour.get_lunar_day();
    let lunar_month = lunar_day.get_lunar_month();
    format!(
        "農曆{}年{}{} {:02}:{:02}:{:02}",
        trad(&lunar_month.get_lunar_year().get_sixty_cycle().get_name()),
        if lunar_month.is_leap() { "閏" } else { "" },
        MONTH_LABELS[lunar_month.get_month() - 1],
        hour.get_hour(),
        hour.get_minute(),
        hour.get_second()
    ) + &format!("（{}）", DAY_LABELS[lunar_day.get_day() - 1])
}

fn build_quant_model(
    day_master: HeavenStem,
    natal_cycles: &[SixtyCycle; 4],
    da_yun: &[DaYunDto],
) -> QuantModelResponse {
    crate::quant_model::build_quant_model_response(day_master, natal_cycles, da_yun)
}

fn build_luck_scores(
    day_master: HeavenStem,
    natal_total: f64,
    prefer_positive: bool,
    da_yun: &[DaYunDto],
    natal_branches: &[EarthBranch; 4],
    natal_stems: &[HeavenStem; 4],
) -> Vec<QuantLuckScore> {
    da_yun
        .iter()
        .map(|row| {
            let cycle = SixtyCycle::from_name(&row.gan_zhi);
            let stem = cycle.get_heaven_stem();
            let branch = cycle.get_earth_branch();
            let stem_ten_god = ten_god_name(day_master.clone(), stem.clone());
            let stem_score = round1(8.0 * ten_god_score(&stem_ten_god));
            let branch_score = round1(branch_total_score(day_master.clone(), branch.clone(), 10.0));
            let total_score = round1(stem_score + branch_score);
            let interaction_lines = luck_interactions(
                stem.clone(),
                branch.clone(),
                natal_stems,
                natal_branches,
                &row.gan_zhi,
            );
            let details = {
                let mut lines = vec![
                    format!(
                        "運干 {} 為 {}，計 {:.1} 分。",
                        trad(&stem.get_name()),
                        stem_ten_god,
                        stem_score
                    ),
                    format!(
                        "運支 {} 藏干 {}，計 {:.1} 分。",
                        trad(&branch.get_name()),
                        row.branch_hidden_stems.join("、"),
                        branch_score
                    ),
                ];
                lines.extend(interaction_lines.clone());
                lines.join("；")
            };

            let annual_scores = if row.liu_nian.is_empty() {
                None
            } else {
                Some(build_annual_scores(
                    day_master.clone(),
                    natal_total,
                    prefer_positive,
                    stem_score,
                    branch_score,
                    total_score,
                    &row.gan_zhi,
                    &row.liu_nian,
                ))
            };

            QuantLuckScore {
                index: row.index,
                gan_zhi: row.gan_zhi.clone(),
                start_age: row.start_age,
                end_age: row.end_age,
                stem_ten_god: row.stem_ten_god.clone(),
                branch_hidden_stems: row.branch_hidden_stems.clone(),
                branch_ten_gods: row.branch_ten_gods.clone(),
                first_half_label: "前五年干氣".to_string(),
                first_half_score: stem_score,
                second_half_label: "後五年支氣".to_string(),
                second_half_score: branch_score,
                stem_score,
                branch_score,
                total_score,
                impact_ratio: ratio(total_score, natal_total),
                effective_natal_score: round1(natal_total + total_score),
                tendency: tendency_label(prefer_positive, total_score),
                overview_lines: Some(vec![
                    format!("前五年干氣：{:.1}", stem_score),
                    format!("後五年支氣：{:.1}", branch_score),
                    format!("整柱參考：{:.1}", total_score),
                    format!("R值：{:.2}", ratio(total_score, natal_total)),
                    format!("作用後命局：{:.1}", round1(natal_total + total_score)),
                    format!("傾向：{}", tendency_label(prefer_positive, total_score)),
                ]),
                interaction_lines: if interaction_lines.is_empty() {
                    None
                } else {
                    Some(interaction_lines.clone())
                },
                scoring_lines: Some(vec![
                    format!("運干十神 {} => {:.1}", stem_ten_god, stem_score),
                    format!("運支藏干總計 => {:.1}", branch_score),
                ]),
                details,
                annual_scores,
            }
        })
        .collect()
}

fn build_annual_scores(
    day_master: HeavenStem,
    natal_total: f64,
    prefer_positive: bool,
    active_stem_score: f64,
    active_branch_score: f64,
    full_luck_score: f64,
    luck_gan_zhi: &str,
    liu_nian: &[LiuNianDto],
) -> Vec<QuantAnnualLuckScore> {
    liu_nian
        .iter()
        .map(|row| {
            let cycle = SixtyCycle::from_name(&row.gan_zhi);
            let stem = cycle.get_heaven_stem();
            let branch = cycle.get_earth_branch();
            let annual_stem_ten_god = ten_god_name(day_master.clone(), stem.clone());
            let annual_stem_score = round1(6.0 * ten_god_score(&annual_stem_ten_god));
            let annual_branch_score =
                round1(branch_total_score(day_master.clone(), branch.clone(), 8.0));
            let annual_total_score = round1(annual_stem_score + annual_branch_score);
            let first_half = row.index <= 4;
            let active_luck_label = if first_half {
                format!("當值運干（{}）", &luck_gan_zhi[..3.min(luck_gan_zhi.len())])
            } else {
                "當值運支".to_string()
            };
            let active_luck_score = if first_half {
                active_stem_score
            } else {
                active_branch_score
            };
            let combined_score = round1(active_luck_score + annual_total_score);
            let full_combined_score = round1(full_luck_score + annual_total_score);
            let impact_ratio = ratio(combined_score, natal_total);
            let full_impact_ratio = ratio(full_combined_score, natal_total);
            let effective_natal_score = round1(natal_total + combined_score);
            let full_effective_natal_score = round1(natal_total + full_combined_score);
            let tendency = tendency_label(prefer_positive, combined_score);
            let full_tendency = tendency_label(prefer_positive, full_combined_score);

            QuantAnnualLuckScore {
                index: row.index,
                year: row.year,
                age: row.age,
                gan_zhi: row.gan_zhi.clone(),
                annual_stem_ten_god,
                annual_branch_hidden_stems: row.branch_hidden_stems.clone(),
                annual_branch_ten_gods: row.branch_ten_gods.clone(),
                active_luck_label,
                active_luck_score,
                full_luck_score,
                annual_stem_score,
                annual_branch_score,
                annual_total_score,
                combined_score,
                full_combined_score,
                impact_ratio,
                full_impact_ratio,
                effective_natal_score,
                full_effective_natal_score,
                tendency: tendency.clone(),
                full_tendency: full_tendency.clone(),
                overview_lines: Some(vec![
                    format!("流年干：{:.1}", annual_stem_score),
                    format!("流年支：{:.1}", annual_branch_score),
                    format!("流年整柱：{:.1}", annual_total_score),
                    format!("半運合參：{:.1}", combined_score),
                    format!("整運合參：{:.1}", full_combined_score),
                ]),
                interaction_lines: Some(vec![
                    format!("當值分數：{:.1}", active_luck_score),
                    format!("整運分數：{:.1}", full_luck_score),
                ]),
                scoring_lines: Some(vec![
                    format!("R值（半運）：{:.2}", impact_ratio),
                    format!("R值（整運）：{:.2}", full_impact_ratio),
                    format!("作用後命局（半運）：{:.1}", effective_natal_score),
                    format!("作用後命局（整運）：{:.1}", full_effective_natal_score),
                ]),
                details: format!(
                    "{}年 {}，半運傾向 {}，整運傾向 {}。",
                    row.year, row.gan_zhi, tendency, full_tendency
                ),
            }
        })
        .collect()
}

fn build_yong_shen(
    day_master: HeavenStem,
    total_score: f64,
    stem_score_total: f64,
    branch_score_total: f64,
) -> QuantYongShen {
    let day_element = day_master.get_element();
    let peer = trad(&day_element.get_name());
    let resource = trad(&producing_element(day_element.clone()).get_name());
    let output = trad(&generated_element(day_element.clone()).get_name());
    let wealth = trad(&controlled_element(day_element.clone()).get_name());
    let officer = trad(&controller_element(day_element.clone()).get_name());
    let strong = total_score > 6.0;

    if strong {
        QuantYongShen {
            method_summary: "以印比增減與地支藏干估分的相容量化法".to_string(),
            strength_basis: format!(
                "命局總分 {:.1}，天干 {:.1}、地支 {:.1}，日主偏強。",
                total_score, stem_score_total, branch_score_total
            ),
            print_presence: format!("{}、{} 類力量已足，宜泄宜制。", peer, resource),
            favorable_ten_gods: vec!["食神", "傷官", "偏財", "正財", "七殺", "正官"]
                .into_iter()
                .map(String::from)
                .collect(),
            favorable_elements: vec![output.clone(), wealth.clone(), officer.clone()],
            conditional_ten_gods: vec!["偏印", "正印"].into_iter().map(String::from).collect(),
            conditional_elements: vec![resource.clone()],
            unfavorable_ten_gods: vec!["比肩", "劫財", "偏印", "正印"]
                .into_iter()
                .map(String::from)
                .collect(),
            unfavorable_elements: vec![peer.clone(), resource.clone()],
            caution_ten_gods: vec!["比肩", "劫財"].into_iter().map(String::from).collect(),
            caution_elements: vec![peer.clone()],
            process: format!(
                "日主{}得印比較多，量化後偏強，故取食傷、財、官殺作為平衡方向。",
                trad(&day_master.get_name())
            ),
            conclusion: format!(
                "首取{}，次取{}與{}，少再加重{}、{}。",
                output, wealth, officer, peer, resource
            ),
        }
    } else {
        QuantYongShen {
            method_summary: "以印比增減與地支藏干估分的相容量化法".to_string(),
            strength_basis: format!(
                "命局總分 {:.1}，天干 {:.1}、地支 {:.1}，日主偏弱。",
                total_score, stem_score_total, branch_score_total
            ),
            print_presence: format!("{}、{} 類力量偏少，宜先扶身。", peer, resource),
            favorable_ten_gods: vec!["比肩", "劫財", "偏印", "正印"]
                .into_iter()
                .map(String::from)
                .collect(),
            favorable_elements: vec![peer.clone(), resource.clone()],
            conditional_ten_gods: vec!["食神", "傷官"].into_iter().map(String::from).collect(),
            conditional_elements: vec![output.clone()],
            unfavorable_ten_gods: vec!["偏財", "正財", "七殺", "正官"]
                .into_iter()
                .map(String::from)
                .collect(),
            unfavorable_elements: vec![wealth.clone(), officer.clone()],
            caution_ten_gods: vec!["食神", "傷官"].into_iter().map(String::from).collect(),
            caution_elements: vec![output.clone()],
            process: format!(
                "日主{}在量化上偏弱，宜先取印比補身，再視情況少量借食傷通關。",
                trad(&day_master.get_name())
            ),
            conclusion: format!(
                "首取{}、{}，次取{}，慎用{}與{}。",
                peer, resource, output, wealth, officer
            ),
        }
    }
}

fn luck_interactions(
    stem: HeavenStem,
    branch: EarthBranch,
    natal_stems: &[HeavenStem; 4],
    natal_branches: &[EarthBranch; 4],
    gan_zhi: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    for (index, natal_stem) in natal_stems.iter().enumerate() {
        if stem.combine(natal_stem.clone()).is_some() {
            lines.push(format!(
                "{} 與 {} 干合。",
                gan_zhi, STEM_PILLAR_LABELS[index]
            ));
        }
    }
    for (index, natal_branch) in natal_branches.iter().enumerate() {
        if branch.combine(natal_branch.clone()).is_some() {
            lines.push(format!(
                "{} 與 {} 六合。",
                gan_zhi, BRANCH_PILLAR_LABELS[index]
            ));
        }
        if branch.get_opposite() == *natal_branch {
            lines.push(format!(
                "{} 與 {} 相沖。",
                gan_zhi, BRANCH_PILLAR_LABELS[index]
            ));
        }
        if branch.get_harm() == *natal_branch {
            lines.push(format!(
                "{} 與 {} 相害。",
                gan_zhi, BRANCH_PILLAR_LABELS[index]
            ));
        }
    }
    lines
}

fn branch_total_score(day_master: HeavenStem, branch: EarthBranch, base_unit: f64) -> f64 {
    let hidden = branch.get_hide_heaven_stems();
    let ratios = hidden_ratios(hidden.len());
    hidden
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let ten_god = ten_god_name(day_master.clone(), item.get_heaven_stem());
            base_unit * ratios[index] * ten_god_score(&ten_god)
        })
        .sum()
}

fn ten_god_score(ten_god: &str) -> f64 {
    match ten_god {
        "比肩" => 1.0,
        "劫財" => 0.9,
        "偏印" => 1.15,
        "正印" => 1.2,
        "食神" => -0.75,
        "傷官" => -0.9,
        "偏財" => -1.0,
        "正財" => -1.0,
        "七殺" => -1.15,
        "正官" => -1.05,
        "日元" => 0.8,
        _ => 0.0,
    }
}

fn hidden_ratios(count: usize) -> Vec<f64> {
    match count {
        1 => vec![1.0],
        2 => vec![0.7, 0.3],
        _ => vec![0.6, 0.3, 0.1],
    }
}

fn strength_label(total_score: f64) -> String {
    if total_score >= 18.0 {
        "偏強".to_string()
    } else if total_score >= 6.0 {
        "中和偏強".to_string()
    } else if total_score <= -18.0 {
        "偏弱".to_string()
    } else if total_score <= -6.0 {
        "中和偏弱".to_string()
    } else {
        "中和".to_string()
    }
}

fn tendency_label(prefer_positive: bool, score: f64) -> String {
    let preferred = if prefer_positive { score } else { -score };
    if preferred >= 8.0 {
        "偏吉".to_string()
    } else if preferred >= 2.0 {
        "可用".to_string()
    } else if preferred <= -8.0 {
        "偏壓".to_string()
    } else if preferred <= -2.0 {
        "需防".to_string()
    } else {
        "平".to_string()
    }
}

fn ratio(delta: f64, natal_total: f64) -> f64 {
    let base = natal_total.abs();
    if base < 0.1 {
        0.0
    } else {
        round2(delta / base)
    }
}

fn element_relation_score(day_element: Element, target_element: Element) -> f64 {
    if day_element == target_element {
        1.0
    } else if generates(target_element.clone(), day_element.clone()) {
        1.1
    } else if generates(day_element.clone(), target_element.clone()) {
        -0.8
    } else if controls(day_element.clone(), target_element.clone()) {
        -1.0
    } else if controls(target_element, day_element) {
        -1.15
    } else {
        0.0
    }
}

fn generates(from: Element, to: Element) -> bool {
    (from.get_index() + 1) % 5 == to.get_index()
}

fn controls(from: Element, to: Element) -> bool {
    (from.get_index() + 2) % 5 == to.get_index()
}

fn producing_element(day_element: Element) -> Element {
    Element::from_index(day_element.get_index() as isize - 1)
}

fn generated_element(day_element: Element) -> Element {
    Element::from_index(day_element.get_index() as isize + 1)
}

fn controlled_element(day_element: Element) -> Element {
    Element::from_index(day_element.get_index() as isize + 2)
}

fn controller_element(day_element: Element) -> Element {
    Element::from_index(day_element.get_index() as isize - 2)
}

fn branch_penalty_kind(a: EarthBranch, b: EarthBranch) -> Option<&'static str> {
    let first = a.get_name();
    let second = b.get_name();
    let pair = (first.as_str(), second.as_str());
    if first == second && matches!(first.as_str(), "辰" | "午" | "酉" | "亥") {
        return Some("自刑");
    }
    if unordered_pair(pair, ("子", "卯")) {
        return Some("無禮之刑");
    }
    if unordered_pair(pair, ("寅", "巳"))
        || unordered_pair(pair, ("寅", "申"))
        || unordered_pair(pair, ("巳", "申"))
    {
        return Some("無恩之刑");
    }
    if unordered_pair(pair, ("丑", "未"))
        || unordered_pair(pair, ("丑", "戌"))
        || unordered_pair(pair, ("未", "戌"))
    {
        return Some("恃勢之刑");
    }
    None
}

fn branch_break_kind(a: EarthBranch, b: EarthBranch) -> Option<&'static str> {
    let first = a.get_name();
    let second = b.get_name();
    let pair = (first.as_str(), second.as_str());
    if unordered_pair(pair, ("子", "酉"))
        || unordered_pair(pair, ("丑", "辰"))
        || unordered_pair(pair, ("寅", "亥"))
        || unordered_pair(pair, ("卯", "午"))
        || unordered_pair(pair, ("申", "巳"))
        || unordered_pair(pair, ("未", "戌"))
    {
        Some("六破")
    } else {
        None
    }
}

fn unordered_pair(pair: (&str, &str), target: (&str, &str)) -> bool {
    (pair.0 == target.0 && pair.1 == target.1) || (pair.0 == target.1 && pair.1 == target.0)
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn trad(input: &str) -> String {
    let replacements = [
        ("农", "農"),
        ("历", "曆"),
        ("闰", "閏"),
        ("伤", "傷"),
        ("财", "財"),
        ("处", "處"),
        ("杀", "殺"),
        ("惊", "驚"),
        ("种", "種"),
        ("长", "長"),
        ("临", "臨"),
        ("绝", "絕"),
        ("养", "養"),
        ("术", "術"),
        ("满", "滿"),
        ("谷", "穀"),
        ("涧", "澗"),
        ("钗", "釵"),
        ("炉", "爐"),
        ("蜡", "蠟"),
        ("杨", "楊"),
        ("驿", "驛"),
        ("穷", "窮"),
        ("凤", "鳳"),
        ("会", "會"),
        ("气", "氣"),
        ("阴", "陰"),
        ("阳", "陽"),
    ];

    let mut output = input.to_string();
    for (from, to) in replacements {
        output = output.replace(from, to);
    }
    output
}
