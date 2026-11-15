//! 出生時間校正與交界提醒：
//!
//! - 台灣夏令時間（日光節約時間）與日治時期的 UTC+9：當時的鐘錶時間比標準時間快 1 小時，排盤前要扣回來。
//!   期間依 IANA 時區資料庫 Asia/Taipei（2018 年依中央氣象局資料修訂；1946、1947 年的起訖和氣象局舊表略有出入）。
//! - 真太陽時：依出生地經度（每差 1 度 4 分鐘）加均時差。子平各家看法不同，預設不採用，只算出來提示。
//! - 交界提醒：出生時間離「節」（換月柱，立春另換年柱）很近、離時辰交界很近、夜子時（日柱看流派）、
//!   校正前後四柱不同時，都列出來讓使用者自己確認出生時間。

use serde::{Deserialize, Serialize};
use tyme4rs::tyme::eightchar::provider::{
    DefaultEightCharProvider, EightCharProvider, LunarSect2EightCharProvider,
};
use tyme4rs::tyme::eightchar::EightChar;
use tyme4rs::tyme::solar::{SolarTerm, SolarTime};
use tyme4rs::tyme::{Culture, Tyme};

/// 前端送來的校正設定；整個欄位沒送時不做任何校正（舊紀錄、測試都維持原行為）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TimeCorrectionRequest {
    /// "auto"：出生地在台灣時依夏令時間表自動扣 1 小時（日治時期 UTC+9 也扣）；"on"：一律扣 1 小時；"off"：不扣。
    pub dst: Option<String>,
    /// "TW" 表示出生地在台灣（套用台灣夏令時間表）。
    pub region: Option<String>,
    pub place_name: Option<String>,
    /// 出生地東經（度）。
    pub longitude: Option<f64>,
    /// 出生地標準時區（分鐘，台灣、香港、中國 +480）。
    pub utc_offset_minutes: Option<i32>,
    /// 是否改用真太陽時排盤。
    pub true_solar_time: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeCorrectionDto {
    /// 使用者輸入的鐘錶時間（公曆）。
    pub clock_time: String,
    /// 扣掉夏令時間後的標準時間。
    pub standard_time: String,
    /// 實際排盤用的時間。
    pub used_time: String,
    pub place_name: Option<String>,
    pub longitude: Option<f64>,
    /// 夏令時間／時區扣掉的分鐘（負數）。
    pub dst_minutes: i32,
    pub dst_label: Option<String>,
    pub true_solar_applied: bool,
    /// 真太陽時（有經度就算，不一定採用）。
    pub true_solar_time: Option<String>,
    pub longitude_minutes: Option<f64>,
    pub equation_minutes: Option<f64>,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BirthBoundaryWarningDto {
    /// jie / hour / zi / correction / dst
    pub kind: String,
    /// warn / info
    pub level: String,
    pub title: String,
    pub detail: String,
}

/// 台灣夏令時間（含首尾兩天，日期為當地日期）：(年, 起月, 起日, 迄月, 迄日)。
const TAIWAN_DST: [(isize, usize, usize, usize, usize); 17] = [
    (1946, 5, 15, 9, 30),
    (1947, 4, 15, 10, 31),
    (1948, 5, 1, 9, 30),
    (1949, 5, 1, 9, 30),
    (1950, 5, 1, 9, 30),
    (1951, 5, 1, 9, 30),
    (1952, 3, 1, 10, 31),
    (1953, 4, 1, 10, 31),
    (1954, 4, 1, 10, 31),
    (1955, 4, 1, 9, 30),
    (1956, 4, 1, 9, 30),
    (1957, 4, 1, 9, 30),
    (1958, 4, 1, 9, 30),
    (1959, 4, 1, 9, 30),
    (1960, 6, 1, 9, 30),
    (1961, 6, 1, 9, 30),
    (1974, 4, 1, 9, 30),
];
const TAIWAN_DST_EXTRA: [(isize, usize, usize, usize, usize); 2] = [(1975, 4, 1, 9, 30), (1979, 7, 1, 9, 30)];

fn ymd(time: &SolarTime) -> (isize, usize, usize) {
    (time.get_year(), time.get_month(), time.get_day())
}

/// 台灣當時鐘錶比標準時間（東經 120 度）快多少分鐘，以及說明。
fn taiwan_clock_offset(clock: &SolarTime) -> Option<(i32, String, bool)> {
    let (year, month, day) = ymd(clock);
    let date = (year, month, day);
    // 日治時期 1937-10-01 起到 1945-09-20 用中央標準時（UTC+9）。
    if date >= (1937, 10, 1) && date <= (1945, 9, 20) {
        return Some((60, "日治時期中央標準時（UTC+9，1937/10/1–1945/9/20）".to_string(), false));
    }
    for (y, sm, sd, em, ed) in TAIWAN_DST.iter().chain(TAIWAN_DST_EXTRA.iter()) {
        if *y != year {
            continue;
        }
        if (month, day) >= (*sm, *sd) && (month, day) <= (*em, *ed) {
            // 結束那天 23 點起的一小時，撥回標準時間後又走了一次，分不出是哪一個。
            let ambiguous = (month, day) == (*em, *ed) && clock.get_hour() == 23;
            return Some((
                60,
                format!("台灣夏令時間（{}/{}/{}–{}/{}，鐘錶快 1 小時）", y, sm, sd, em, ed),
                ambiguous,
            ));
        }
    }
    None
}

/// 均時差（分鐘，真太陽時 − 平太陽時），NOAA 近似公式，誤差約半分鐘。
fn equation_of_time(time: &SolarTime) -> f64 {
    let day_of_year = time.get_solar_day().get_index_in_year() as f64 + 1.0;
    let hour = time.get_hour() as f64 + time.get_minute() as f64 / 60.0;
    let gamma = 2.0 * std::f64::consts::PI / 365.0 * (day_of_year - 1.0 + (hour - 12.0) / 24.0);
    229.18
        * (0.000075 + 0.001868 * gamma.cos() - 0.032077 * gamma.sin()
            - 0.014615 * (2.0 * gamma).cos()
            - 0.040849 * (2.0 * gamma).sin())
}

pub fn format_time(time: &SolarTime) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        time.get_year(),
        time.get_month(),
        time.get_day(),
        time.get_hour(),
        time.get_minute()
    )
}

fn shift_minutes(time: &SolarTime, minutes: f64) -> SolarTime {
    time.next((minutes * 60.0).round() as isize)
}

/// 依設定把輸入的鐘錶時間換成排盤用的時間。沒有設定時原樣回傳。
pub fn correct(
    clock: SolarTime,
    request: Option<&TimeCorrectionRequest>,
) -> (SolarTime, Option<TimeCorrectionDto>) {
    let Some(request) = request else {
        return (clock, None);
    };
    let mut lines = Vec::new();
    let dst_mode = request.dst.as_deref().unwrap_or("auto");
    let in_taiwan = request.region.as_deref() == Some("TW");
    let taiwan = taiwan_clock_offset(&clock);
    let (dst_minutes, dst_label) = match dst_mode {
        "on" => (-60, Some("手動設定：出生時間是夏令時間，扣 1 小時".to_string())),
        "off" => (0, None),
        _ if in_taiwan => match &taiwan {
            Some((minutes, label, _)) => (-minutes, Some(label.clone())),
            None => (0, None),
        },
        _ => (0, None),
    };
    let standard = shift_minutes(&clock, dst_minutes as f64);
    if let Some(label) = &dst_label {
        lines.push(format!(
            "{}：鐘錶 {} → 標準時間 {}。",
            label,
            format_time(&clock),
            format_time(&standard)
        ));
    }
    let offset = request.utc_offset_minutes.unwrap_or(480) as f64;
    let (true_solar, longitude_minutes, equation_minutes) = match request.longitude {
        Some(longitude) => {
            let longitude_minutes = (longitude - offset / 60.0 * 15.0) * 4.0;
            let equation = equation_of_time(&standard);
            (
                Some(shift_minutes(&standard, longitude_minutes + equation)),
                Some((longitude_minutes * 10.0).round() / 10.0),
                Some((equation * 10.0).round() / 10.0),
            )
        }
        None => (None, None, None),
    };
    if let (Some(solar), Some(lm), Some(eq)) = (&true_solar, longitude_minutes, equation_minutes) {
        lines.push(format!(
            "真太陽時 {}（經度差 {:+.1} 分、均時差 {:+.1} 分）{}。",
            format_time(solar),
            lm,
            eq,
            if request.true_solar_time {
                "，排盤採用真太陽時"
            } else {
                "，未採用（排盤用標準時間）"
            }
        ));
    }
    let used = match (&true_solar, request.true_solar_time) {
        (Some(solar), true) => *solar,
        _ => standard,
    };
    (
        used,
        Some(TimeCorrectionDto {
            clock_time: format_time(&clock),
            standard_time: format_time(&standard),
            used_time: format_time(&used),
            place_name: request.place_name.clone(),
            longitude: request.longitude,
            dst_minutes,
            dst_label,
            true_solar_applied: request.true_solar_time && true_solar.is_some(),
            true_solar_time: true_solar.as_ref().map(format_time),
            longitude_minutes,
            equation_minutes,
            lines,
        }),
    )
}

fn eight_char_at(time: &SolarTime, bazi_sect: u8) -> EightChar {
    let hour = time.get_lunar_hour();
    if bazi_sect == 2 {
        LunarSect2EightCharProvider::new().get_eight_char(hour)
    } else {
        DefaultEightCharProvider::new().get_eight_char(hour)
    }
}

fn pillar_names(eight_char: &EightChar) -> [String; 4] {
    [
        eight_char.get_year().get_name(),
        eight_char.get_month().get_name(),
        eight_char.get_day().get_name(),
        eight_char.get_hour().get_name(),
    ]
}

fn duration_text(seconds: isize) -> String {
    let minutes = (seconds.abs() + 30) / 60;
    if minutes >= 60 * 24 {
        format!("{} 天 {} 小時", minutes / (60 * 24), minutes % (60 * 24) / 60)
    } else if minutes >= 60 {
        format!("{} 小時 {} 分", minutes / 60, minutes % 60)
    } else {
        format!("{} 分鐘", minutes)
    }
}

fn changed_pillars(base: &[String; 4], other: &[String; 4]) -> Vec<String> {
    const LABELS: [&str; 4] = ["年柱", "月柱", "日柱", "時柱"];
    (0..4)
        .filter(|index| base[*index] != other[*index])
        .map(|index| format!("{}{}→{}", LABELS[index], base[index], other[index]))
        .collect()
}

/// 離「節」（換月）多近：3 小時內 warn、1 天內 info。
const JIE_WARN_SECONDS: isize = 3 * 3600;
const JIE_INFO_SECONDS: isize = 86400;
/// 離時辰交界 15 分鐘內提醒。
const HOUR_EDGE_MINUTES: isize = 15;

pub fn boundary_warnings(
    clock: &SolarTime,
    used: &SolarTime,
    bazi_sect: u8,
    correction: Option<&TimeCorrectionDto>,
    request: Option<&TimeCorrectionRequest>,
) -> Vec<BirthBoundaryWarningDto> {
    let mut warnings = Vec::new();
    let base = pillar_names(&eight_char_at(used, bazi_sect));

    // 1. 節氣交界：找出生前後最近的「節」。
    let mut term = used.get_term();
    if !term.is_jie() {
        term = term.next(-1);
    }
    let mut previous: SolarTerm = term.clone();
    if used.is_before(previous.get_julian_day().get_solar_time()) {
        previous = previous.next(-2);
    }
    let next = previous.next(2);
    for (jie, after_birth) in [(previous, false), (next, true)] {
        let jie_time = jie.get_julian_day().get_solar_time();
        let gap = if after_birth {
            jie_time.subtract(*used)
        } else {
            used.subtract(jie_time)
        };
        if gap < 0 || gap > JIE_INFO_SECONDS {
            continue;
        }
        let other_side = if after_birth { jie_time.next(60) } else { jie_time.next(-60) };
        // 跨節只換年柱（立春）和月柱；日柱、時柱是因為時間不同，不算在這裡。
        let mut changes = changed_pillars(&base, &pillar_names(&eight_char_at(&other_side, bazi_sect)));
        changes.retain(|item| item.starts_with("年柱") || item.starts_with("月柱"));
        let name = jie.get_name().replace('惊', "驚").replace('蛰', "蟄").replace('种', "種");
        warnings.push(BirthBoundaryWarningDto {
            kind: "jie".to_string(),
            level: if gap <= JIE_WARN_SECONDS { "warn" } else { "info" }.to_string(),
            title: format!(
                "出生在{}{} {}",
                name,
                if after_birth { "前" } else { "後" },
                duration_text(gap)
            ),
            detail: format!(
                "{}交節時刻 {}，出生時間 {}。出生時間如果有 {} 以上的誤差，就會跨到{}，{}。",
                name,
                format_time(&jie_time),
                format_time(used),
                duration_text(gap),
                if after_birth { "下一個月" } else { "上一個月" },
                if changes.is_empty() {
                    "但年柱、月柱不變".to_string()
                } else {
                    format!("會變成：{}", changes.join("、"))
                }
            ),
        });
    }

    // 2. 時辰交界：時辰從奇數點開始（23、1、3…）。
    let minute_of_day = used.get_hour() as isize * 60 + used.get_minute() as isize;
    let into_slot = (minute_of_day + 60).rem_euclid(120);
    let (edge_gap, edge_shift) = if into_slot < HOUR_EDGE_MINUTES {
        (into_slot, -(into_slot + 1))
    } else if 120 - into_slot <= HOUR_EDGE_MINUTES {
        (120 - into_slot, 120 - into_slot + 1)
    } else {
        (-1, 0)
    };
    if edge_gap >= 0 {
        let other = used.next(edge_shift * 60);
        let changes = changed_pillars(&base, &pillar_names(&eight_char_at(&other, bazi_sect)));
        if !changes.is_empty() {
            warnings.push(BirthBoundaryWarningDto {
                kind: "hour".to_string(),
                level: "warn".to_string(),
                title: format!("離時辰交界只有 {} 分鐘", edge_gap.max(1)),
                detail: format!(
                    "排盤時間 {}，再{} {} 分鐘就換時辰：{}。請確認出生時間是否精確。",
                    format_time(used),
                    if edge_shift < 0 { "早" } else { "晚" },
                    edge_gap.max(1),
                    changes.join("、")
                ),
            });
        }
    }

    // 3. 夜子時：日柱依「日柱流派」設定。
    if used.get_hour() == 23 {
        let other_sect = if bazi_sect == 2 { 1 } else { 2 };
        let changes = changed_pillars(&base, &pillar_names(&eight_char_at(used, other_sect)));
        if !changes.is_empty() {
            warnings.push(BirthBoundaryWarningDto {
                kind: "zi".to_string(),
                level: "info".to_string(),
                title: "夜子時（23 點）".to_string(),
                detail: format!(
                    "目前設定「晚子時{}」；另一派（{}）的四柱是：{}。",
                    if bazi_sect == 2 { "按當天" } else { "按明天" },
                    if bazi_sect == 2 { "按明天" } else { "按當天" },
                    changes.join("、")
                ),
            });
        }
    }

    // 4. 校正前後四柱不同。
    if let Some(correction) = correction {
        let clock_pillars = pillar_names(&eight_char_at(clock, bazi_sect));
        let changes = changed_pillars(&clock_pillars, &base);
        if !changes.is_empty() {
            warnings.push(BirthBoundaryWarningDto {
                kind: "correction".to_string(),
                level: "warn".to_string(),
                title: "時間校正改變了四柱".to_string(),
                detail: format!(
                    "照鐘錶時間 {} 排是 {}；校正後（{}）是 {}：{}。",
                    correction.clock_time,
                    clock_pillars.join(" "),
                    correction.used_time,
                    base.join(" "),
                    changes.join("、")
                ),
            });
        }
        // 真太陽時沒採用、但採用的話四柱會不同。
        if let (false, Some(text)) = (correction.true_solar_applied, &correction.true_solar_time) {
            if let Some(request) = request {
                if let Some(longitude) = request.longitude {
                    let offset = request.utc_offset_minutes.unwrap_or(480) as f64;
                    let standard = shift_minutes(clock, correction.dst_minutes as f64);
                    let solar = shift_minutes(
                        &standard,
                        (longitude - offset / 60.0 * 15.0) * 4.0 + equation_of_time(&standard),
                    );
                    let solar_changes =
                        changed_pillars(&base, &pillar_names(&eight_char_at(&solar, bazi_sect)));
                    if !solar_changes.is_empty() {
                        warnings.push(BirthBoundaryWarningDto {
                            kind: "true_solar".to_string(),
                            level: "info".to_string(),
                            title: "改用真太陽時四柱會不同".to_string(),
                            detail: format!(
                                "真太陽時是 {}，若採用：{}。子平各家對真太陽時看法不同，App 預設不採用，可在輸入表單打開。",
                                text,
                                solar_changes.join("、")
                            ),
                        });
                    }
                }
            }
        }
    }

    // 5. 出生在台灣夏令時間期間但沒有扣（或沒有設定出生地）。
    let dst_applied = correction.map(|item| item.dst_minutes != 0).unwrap_or(false);
    let taiwan_or_unknown = request
        .map(|item| item.region.as_deref() == Some("TW"))
        .unwrap_or(true);
    if !dst_applied && taiwan_or_unknown {
        if let Some((_, label, _)) = taiwan_clock_offset(clock) {
            let shifted = clock.next(-3600);
            let changes = changed_pillars(
                &pillar_names(&eight_char_at(clock, bazi_sect)),
                &pillar_names(&eight_char_at(&shifted, bazi_sect)),
            );
            warnings.push(BirthBoundaryWarningDto {
                kind: "dst".to_string(),
                level: if changes.is_empty() { "info" } else { "warn" }.to_string(),
                title: "出生日期在台灣夏令時間期間".to_string(),
                detail: format!(
                    "{}。如果出生時間是當時的鐘錶時間，要扣 1 小時{}；請在輸入表單把「夏令時間」設為自動。",
                    label,
                    if changes.is_empty() {
                        "（這張命盤扣了四柱也不變）".to_string()
                    } else {
                        format!("，四柱會變成：{}", changes.join("、"))
                    }
                ),
            });
        }
    }
    if let Some((_, _, true)) = taiwan_clock_offset(clock) {
        if dst_applied {
            warnings.push(BirthBoundaryWarningDto {
                kind: "dst".to_string(),
                level: "warn".to_string(),
                title: "夏令時間結束當晚 23 點".to_string(),
                detail: "這天夜裡鐘錶在 24 點撥回 23 點，23 點這一小時走了兩次，分不出是夏令時間還是標準時間，請兩種都排來比較。".to_string(),
            });
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tw(y: isize, m: usize, d: usize, h: usize, mi: usize) -> SolarTime {
        SolarTime::from_ymd_hms(y, m, d, h, mi, 0)
    }

    fn taipei() -> TimeCorrectionRequest {
        TimeCorrectionRequest {
            dst: Some("auto".to_string()),
            region: Some("TW".to_string()),
            place_name: Some("台北".to_string()),
            longitude: Some(121.52),
            utc_offset_minutes: Some(480),
            true_solar_time: false,
        }
    }

    #[test]
    fn taiwan_dst_is_removed_automatically() {
        let (used, dto) = correct(tw(1974, 7, 10, 8, 30), Some(&taipei()));
        assert_eq!(format_time(&used), "1974-07-10 07:30");
        let dto = dto.unwrap();
        assert_eq!(dto.dst_minutes, -60);
        assert!(dto.dst_label.unwrap().contains("1974"));
        // 不在夏令時間期間就不扣
        let (used, _) = correct(tw(1974, 10, 2, 8, 30), Some(&taipei()));
        assert_eq!(format_time(&used), "1974-10-02 08:30");
        // 日治時期 UTC+9
        let (used, _) = correct(tw(1940, 3, 1, 8, 0), Some(&taipei()));
        assert_eq!(format_time(&used), "1940-03-01 07:00");
        // 沒有設定就原樣
        let (used, dto) = correct(tw(1974, 7, 10, 8, 30), None);
        assert_eq!(format_time(&used), "1974-07-10 08:30");
        assert!(dto.is_none());
    }

    #[test]
    fn true_solar_time_uses_longitude_and_equation_of_time() {
        let mut request = taipei();
        request.true_solar_time = true;
        // 台北 121.52°E：經度差 +6.1 分；11 月初均時差約 +16 分。
        let (used, dto) = correct(tw(2000, 11, 3, 12, 0), Some(&request));
        let dto = dto.unwrap();
        assert!(dto.true_solar_applied);
        assert!((dto.longitude_minutes.unwrap() - 6.1).abs() < 0.1);
        assert!((dto.equation_minutes.unwrap() - 16.4).abs() < 0.6);
        assert_eq!(format_time(&used), "2000-11-03 12:22");
    }

    #[test]
    fn warns_near_jie_and_hour_boundary() {
        // 2026 立春 02-04 04:02（北京時間）；出生 03:30 → 立春前 32 分鐘，年柱月柱都會變。
        let birth = tw(2026, 2, 4, 3, 30);
        let warnings = boundary_warnings(&birth, &birth, 2, None, None);
        let jie = warnings.iter().find(|item| item.kind == "jie").expect("jie warning");
        assert_eq!(jie.level, "warn");
        assert!(jie.title.contains("立春前"));
        assert!(jie.detail.contains("年柱") && jie.detail.contains("月柱"));
        // 08:55 離巳時 5 分鐘
        let near_hour = tw(2026, 6, 15, 8, 55);
        let warnings = boundary_warnings(&near_hour, &near_hour, 2, None, None);
        assert!(warnings.iter().any(|item| item.kind == "hour"));
        // 平常時間沒有提醒
        let calm = tw(2026, 6, 20, 10, 0);
        assert!(boundary_warnings(&calm, &calm, 2, None, None).is_empty());
    }

    #[test]
    fn dst_period_without_correction_is_flagged() {
        let clock = tw(1979, 8, 1, 7, 10);
        let warnings = boundary_warnings(&clock, &clock, 2, None, None);
        let dst = warnings.iter().find(|item| item.kind == "dst").expect("dst warning");
        assert_eq!(dst.level, "warn");
        assert!(dst.detail.contains("時柱"));
    }
}
