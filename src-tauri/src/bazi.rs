use serde::{Deserialize, Serialize};
use std::array;
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

    Ok(finalize_bazi_response(BaziResponse {
        input_calendar_type: match request.calendar_type {
            CalendarType::Solar => "公曆".to_string(),
            CalendarType::Lunar => "農曆".to_string(),
        },
        input_date_time: format_input(&request),
        solar_date_time: format_solar_time(solar_time),
        lunar_date_time: format_lunar_time(lunar_hour.clone()),
        ba_zi: trad(&eight_char.to_string()),
        day_master: trad(&day_master.get_name()),
        ge_ju: "待移植".to_string(),
        ge_ju_basis: "Rust 版已補上排盤、四柱直輸與相容量化；原 Java 細部格局公式仍可再逐步收斂。"
            .to_string(),
        tai_yuan: trad(&eight_char.get_fetal_origin().get_name()),
        ming_gong: trad(&eight_char.get_own_sign().get_name()),
        shen_gong: trad(&eight_char.get_body_sign().get_name()),
        year_pillar: build_pillar(eight_char.get_year(), day_master.clone()),
        month_pillar: build_pillar(eight_char.get_month(), day_master.clone()),
        day_pillar: build_pillar(eight_char.get_day(), day_master.clone()),
        hour_pillar: build_pillar(eight_char.get_hour(), day_master.clone()),
        year_hidden_stems,
        month_hidden_stems,
        day_hidden_stems,
        hour_hidden_stems,
        year_stem_ten_god: ten_god_name(
            day_master.clone(),
            eight_char.get_year().get_heaven_stem(),
        ),
        month_stem_ten_god: ten_god_name(
            day_master.clone(),
            eight_char.get_month().get_heaven_stem(),
        ),
        day_stem_ten_god: "日元".to_string(),
        hour_stem_ten_god: ten_god_name(day_master, eight_char.get_hour().get_heaven_stem()),
        year_branch_ten_gods,
        month_branch_ten_gods,
        day_branch_ten_gods,
        hour_branch_ten_gods,
        luck_start,
        da_yun,
        quant_model,
    }))
}

#[tauri::command]
pub fn analyze_pillars(request: PillarAnalyzeRequest) -> Result<BaziResponse, String> {
    let year_cycle = parse_direct_pillar(&request.year_pillar)?;
    let month_cycle = parse_direct_pillar(&request.month_pillar)?;
    let day_cycle = parse_direct_pillar(&request.day_pillar)?;
    let hour_cycle = parse_direct_pillar(&request.hour_pillar)?;
    let day_master = day_cycle.get_heaven_stem();
    let gender = request.gender.unwrap_or(Gender::Female);
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

    let year_hidden_stems = hidden_stems(year_cycle.get_earth_branch());
    let month_hidden_stems = hidden_stems(month_cycle.get_earth_branch());
    let day_hidden_stems = hidden_stems(day_cycle.get_earth_branch());
    let hour_hidden_stems = hidden_stems(hour_cycle.get_earth_branch());

    Ok(finalize_bazi_response(BaziResponse {
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
    }))
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
        assert!(android_response
            .da_yun
            .iter()
            .any(|luck| luck
                .liu_nian
                .iter()
                .any(|annual| !annual.liu_yue.is_empty())));
        assert!(android_response.quant_model.is_some());
        assert!(android_response
            .quant_model
            .as_ref()
            .and_then(|quant_model| quant_model.luck_scores.as_ref())
            .is_some_and(|luck_scores| luck_scores
                .iter()
                .any(|score| score
                    .annual_scores
                    .as_ref()
                    .is_some_and(|annual_scores| !annual_scores.is_empty()))));
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

    Ok(LuckStartDto {
        forward: Some(forward),
        start_year: Some(info.get_year_count() as i32),
        start_month: Some(info.get_month_count() as i32),
        start_day: Some(info.get_day_count() as i32),
        start_hour: Some(info.get_hour_count() as i32),
        start_solar: Some(format_solar_time(start_time)),
        start_summary: Some(format!(
            "{}年{}個月{}天{}小時",
            info.get_year_count(),
            info.get_month_count(),
            info.get_day_count(),
            info.get_hour_count()
        )),
        transition_summary: Some(format!("約於 {} 起運。", format_solar_time(start_time))),
        birth_jie_name: Some(trad(&birth_prev_jie.get_name())),
        birth_jie_solar: Some(format_solar_time(birth_prev_jie_time)),
        birth_jie_day_ordinal: Some(birth_prev_jie_days.max(0)),
        transition_summary_experimental: Some(format!(
            "依目前起運流派推算，出生後約 {} 上大運。",
            format_solar_time(start_time)
        )),
    })
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
    let first_start_year = info.get_end_time().get_year() as i32;
    let first_start_age = first_start_year - birth_time.get_year() as i32 + 1;

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
    format_month_day(term.get_julian_day().get_solar_time())
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
    let day_master_name = trad(&day_master.get_name());
    let day_element = day_master.get_element();
    let mut interactions = Vec::new();
    let mut stem_adjustments = [0.0_f64; 4];
    let mut branch_adjustments = [0.0_f64; 4];
    let mut stem_notes: [Vec<String>; 4] = array::from_fn(|_| Vec::new());
    let mut branch_notes: [Vec<String>; 4] = array::from_fn(|_| Vec::new());
    let mut stem_combine_notes: [Vec<String>; 4] = array::from_fn(|_| Vec::new());
    let mut branch_combine_notes: [Vec<String>; 4] = array::from_fn(|_| Vec::new());

    let stems = natal_cycles.clone().map(|cycle| cycle.get_heaven_stem());
    let branches = natal_cycles.clone().map(|cycle| cycle.get_earth_branch());

    for i in 0..4 {
        for j in (i + 1)..4 {
            if let Some(element) = stems[i].combine(stems[j].clone()) {
                let adjustment =
                    round1(element_relation_score(day_element.clone(), element.clone()) * 2.0);
                let each = round1(adjustment / 2.0);
                stem_adjustments[i] += each;
                stem_adjustments[j] += each;
                let outcome = trad(&element.get_name());
                interactions.push(QuantModelInteraction {
                    scope: "天干".to_string(),
                    r#type: "五合".to_string(),
                    target: format!(
                        "{}{}合",
                        trad(&stems[i].get_name()),
                        trad(&stems[j].get_name())
                    ),
                    outcome: format!("化{}", outcome),
                    pillars: format!("{}、{}", STEM_PILLAR_LABELS[i], STEM_PILLAR_LABELS[j]),
                    detail: if adjustment >= 0.0 {
                        format!(
                            "{}與{}干合化{}，對日主{}偏扶助。",
                            STEM_PILLAR_LABELS[i], STEM_PILLAR_LABELS[j], outcome, day_master_name
                        )
                    } else {
                        format!(
                            "{}與{}干合化{}，對日主{}偏消耗。",
                            STEM_PILLAR_LABELS[i], STEM_PILLAR_LABELS[j], outcome, day_master_name
                        )
                    },
                });
                stem_notes[i].push(format!(
                    "{}與{}干合，{} {:+}",
                    STEM_PILLAR_LABELS[i], STEM_PILLAR_LABELS[j], outcome, each
                ));
                stem_notes[j].push(format!(
                    "{}與{}干合，{} {:+}",
                    STEM_PILLAR_LABELS[i], STEM_PILLAR_LABELS[j], outcome, each
                ));
                stem_combine_notes[i].push(outcome.clone());
                stem_combine_notes[j].push(outcome);
            }

            if let Some(element) = branches[i].combine(branches[j].clone()) {
                let adjustment =
                    round1(element_relation_score(day_element.clone(), element.clone()) * 2.4);
                let each = round1(adjustment / 2.0);
                branch_adjustments[i] += each;
                branch_adjustments[j] += each;
                let outcome = trad(&element.get_name());
                interactions.push(QuantModelInteraction {
                    scope: "地支".to_string(),
                    r#type: "六合".to_string(),
                    target: format!(
                        "{}{}合",
                        trad(&branches[i].get_name()),
                        trad(&branches[j].get_name())
                    ),
                    outcome: format!("化{}", outcome),
                    pillars: format!("{}、{}", BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j]),
                    detail: format!(
                        "{}與{}成六合，化{}。",
                        BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], outcome
                    ),
                });
                branch_notes[i].push(format!(
                    "{}與{}六合化{} {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], outcome, each
                ));
                branch_notes[j].push(format!(
                    "{}與{}六合化{} {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], outcome, each
                ));
                branch_combine_notes[i].push(outcome.clone());
                branch_combine_notes[j].push(outcome);
            }

            if branches[i].get_opposite() == branches[j] {
                let each = -1.4;
                branch_adjustments[i] += each;
                branch_adjustments[j] += each;
                interactions.push(QuantModelInteraction {
                    scope: "地支".to_string(),
                    r#type: "沖".to_string(),
                    target: format!(
                        "{}{}沖",
                        trad(&branches[i].get_name()),
                        trad(&branches[j].get_name())
                    ),
                    outcome: "氣勢擾動".to_string(),
                    pillars: format!("{}、{}", BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j]),
                    detail: format!(
                        "{}與{}相沖，命局穩定度下降。",
                        BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j]
                    ),
                });
                branch_notes[i].push(format!(
                    "{}與{}相沖 {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], each
                ));
                branch_notes[j].push(format!(
                    "{}與{}相沖 {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], each
                ));
            }

            if branches[i].get_harm() == branches[j] {
                let each = -0.8;
                branch_adjustments[i] += each;
                branch_adjustments[j] += each;
                interactions.push(QuantModelInteraction {
                    scope: "地支".to_string(),
                    r#type: "害".to_string(),
                    target: format!(
                        "{}{}害",
                        trad(&branches[i].get_name()),
                        trad(&branches[j].get_name())
                    ),
                    outcome: "暗耗".to_string(),
                    pillars: format!("{}、{}", BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j]),
                    detail: format!(
                        "{}與{}成害，易出現暗耗或牽制。",
                        BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j]
                    ),
                });
                branch_notes[i].push(format!(
                    "{}與{}相害 {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], each
                ));
                branch_notes[j].push(format!(
                    "{}與{}相害 {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], each
                ));
            }

            if let Some(kind) = branch_penalty_kind(branches[i].clone(), branches[j].clone()) {
                let each = -0.7;
                branch_adjustments[i] += each;
                branch_adjustments[j] += each;
                interactions.push(QuantModelInteraction {
                    scope: "地支".to_string(),
                    r#type: "刑".to_string(),
                    target: format!(
                        "{}{}刑",
                        trad(&branches[i].get_name()),
                        trad(&branches[j].get_name())
                    ),
                    outcome: "牽制".to_string(),
                    pillars: format!("{}、{}", BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j]),
                    detail: format!(
                        "{}與{}形成{}。",
                        BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], kind
                    ),
                });
                branch_notes[i].push(format!(
                    "{}與{}{} {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], kind, each
                ));
                branch_notes[j].push(format!(
                    "{}與{}{} {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], kind, each
                ));
            }

            if let Some(kind) = branch_break_kind(branches[i].clone(), branches[j].clone()) {
                let each = -0.6;
                branch_adjustments[i] += each;
                branch_adjustments[j] += each;
                interactions.push(QuantModelInteraction {
                    scope: "地支".to_string(),
                    r#type: "破".to_string(),
                    target: format!(
                        "{}{}破",
                        trad(&branches[i].get_name()),
                        trad(&branches[j].get_name())
                    ),
                    outcome: "破耗".to_string(),
                    pillars: format!("{}、{}", BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j]),
                    detail: format!(
                        "{}與{}形成{}。",
                        BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], kind
                    ),
                });
                branch_notes[i].push(format!(
                    "{}與{}{} {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], kind, each
                ));
                branch_notes[j].push(format!(
                    "{}與{}{} {:+}",
                    BRANCH_PILLAR_LABELS[i], BRANCH_PILLAR_LABELS[j], kind, each
                ));
            }
        }
    }

    let stem_weights = [0.9, 1.25, 1.05, 0.95];
    let branch_weights = [1.0, 1.35, 1.1, 0.95];

    let mut stem_scores = Vec::new();
    let mut branch_scores = Vec::new();

    for index in 0..4 {
        let stem = stems[index].clone();
        let ten_god = if index == 2 {
            "日元".to_string()
        } else {
            ten_god_name(day_master.clone(), stem.clone())
        };
        let raw_score = round1(10.0 * ten_god_score(&ten_god));
        let position_adjusted_score = round1(raw_score * stem_weights[index]);
        let final_score = round1(position_adjusted_score + stem_adjustments[index]);
        let details = if stem_notes[index].is_empty() {
            format!(
                "{}{}以{}入局。",
                STEM_PILLAR_LABELS[index],
                trad(&stem.get_name()),
                ten_god
            )
        } else {
            stem_notes[index].join("；")
        };
        let sub_rows = if stem_notes[index].is_empty() {
            None
        } else {
            Some(
                stem_notes[index]
                    .iter()
                    .map(|line| QuantModelSubScore {
                        item: STEM_PILLAR_LABELS[index].to_string(),
                        hidden_stem: None,
                        ten_god: Some(ten_god.clone()),
                        ratio: None,
                        raw_score: Some(raw_score),
                        position_adjusted_score: Some(position_adjusted_score),
                        interaction: Some(line.clone()),
                        adjustment_score: Some(if stem_adjustments[index] == 0.0 {
                            0.0
                        } else {
                            round1(stem_adjustments[index] / stem_notes[index].len() as f64)
                        }),
                        final_contribution: Some(final_score),
                        note: None,
                    })
                    .collect(),
            )
        };

        stem_scores.push(QuantModelPillarScore {
            pillar: STEM_PILLAR_LABELS[index].to_string(),
            target: trad(&stem.get_name()),
            ten_god,
            base_score: raw_score,
            position_weight: stem_weights[index],
            final_score,
            details,
            category: Some("stem".to_string()),
            combine_note: if stem_combine_notes[index].is_empty() {
                None
            } else {
                Some(stem_combine_notes[index].join("、"))
            },
            raw_score: Some(raw_score),
            position_adjusted_score: Some(position_adjusted_score),
            clash_adjustment: Some(round1(stem_adjustments[index].min(0.0))),
            bonus_score: Some(round1(stem_adjustments[index].max(0.0))),
            rows: sub_rows,
        });
    }

    for index in 0..4 {
        let branch = branches[index].clone();
        let hidden = branch.get_hide_heaven_stems();
        let ratios = hidden_ratios(hidden.len());
        let base_unit = 14.0;
        let mut rows = Vec::new();
        let mut raw_sum = 0.0;
        let mut pos_sum = 0.0;
        let mut final_sum = 0.0;

        for (row_index, hidden_stem) in hidden.into_iter().enumerate() {
            let stem = hidden_stem.get_heaven_stem();
            let ten_god = ten_god_name(day_master.clone(), stem.clone());
            let ratio = ratios[row_index];
            let raw_score = round1(base_unit * ratio * ten_god_score(&ten_god));
            let position_adjusted_score = round1(raw_score * branch_weights[index]);
            let adjustment_score = round1(branch_adjustments[index] * ratio);
            let final_contribution = round1(position_adjusted_score + adjustment_score);
            raw_sum += raw_score;
            pos_sum += position_adjusted_score;
            final_sum += final_contribution;
            rows.push(QuantModelSubScore {
                item: BRANCH_PILLAR_LABELS[index].to_string(),
                hidden_stem: Some(trad(&stem.get_name())),
                ten_god: Some(ten_god),
                ratio: Some(ratio),
                raw_score: Some(raw_score),
                position_adjusted_score: Some(position_adjusted_score),
                interaction: Some(if branch_notes[index].is_empty() {
                    "原局".to_string()
                } else {
                    branch_notes[index].join("；")
                }),
                adjustment_score: Some(adjustment_score),
                final_contribution: Some(final_contribution),
                note: Some(format!(
                    "{}藏干按 {:.0}% 比重計入。",
                    trad(&stem.get_name()),
                    ratio * 100.0
                )),
            });
        }

        branch_scores.push(QuantModelPillarScore {
            pillar: BRANCH_PILLAR_LABELS[index].to_string(),
            target: trad(&branch.get_name()),
            ten_god: "藏干綜合".to_string(),
            base_score: round1(raw_sum),
            position_weight: branch_weights[index],
            final_score: round1(final_sum),
            details: if branch_notes[index].is_empty() {
                format!(
                    "{}{}以藏干分布計分。",
                    BRANCH_PILLAR_LABELS[index],
                    trad(&branch.get_name())
                )
            } else {
                branch_notes[index].join("；")
            },
            category: Some("branch".to_string()),
            combine_note: if branch_combine_notes[index].is_empty() {
                None
            } else {
                Some(branch_combine_notes[index].join("、"))
            },
            raw_score: Some(round1(raw_sum)),
            position_adjusted_score: Some(round1(pos_sum)),
            clash_adjustment: Some(round1(branch_adjustments[index].min(0.0))),
            bonus_score: Some(round1(branch_adjustments[index].max(0.0))),
            rows: Some(rows),
        });
    }

    let stem_score_total = round1(stem_scores.iter().map(|item| item.final_score).sum());
    let branch_score_total = round1(branch_scores.iter().map(|item| item.final_score).sum());
    let total_score = round1(stem_score_total + branch_score_total);
    let strength_label = strength_label(total_score);
    let yong_shen = Some(build_yong_shen(
        day_master.clone(),
        total_score,
        stem_score_total,
        branch_score_total,
    ));
    let prefer_positive = total_score <= 0.0;
    let luck_scores = if da_yun.is_empty() {
        None
    } else {
        Some(build_luck_scores(
            day_master.clone(),
            total_score,
            prefer_positive,
            da_yun,
            &branches,
            &stems,
        ))
    };

    QuantModelResponse {
        day_master: day_master_name.clone(),
        summary: format!(
            "日主{}，天干 {:.1} 分、地支 {:.1} 分，命局 {:.1} 分，判定{}。",
            day_master_name, stem_score_total, branch_score_total, total_score, strength_label
        ),
        note:
            "此為 Rust 桌面版相容量化，用於補齊 UI 書表與四柱直輸流程，後續仍可向原 Java 規則細化。"
                .to_string(),
        stem_score_total,
        branch_score_total,
        total_score,
        strength_label,
        yong_shen,
        cong_pattern: None,
        sha_yin: None,
        interactions: Some(interactions),
        stem_scores,
        branch_scores,
        luck_scores,
    }
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
