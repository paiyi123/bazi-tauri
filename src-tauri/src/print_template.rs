use std::collections::HashMap;

use chrono::Datelike;
use serde::Deserialize;
use serde_json::Value;

const PRINT_TEMPLATE_HTML: &str = include_str!("../templates/bazi_print_template_v5.html");

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintContext {
    pub source: String,
    pub name: Option<String>,
    pub gender: Option<String>,
    pub calendar_type: Option<String>,
    pub year_era: Option<String>,
    pub input_text: String,
    pub time_label: Option<String>,
    pub note: Option<String>,
}

#[tauri::command]
pub fn render_print_html(result: Value, context: Option<PrintContext>) -> Result<String, String> {
    let values = build_template_values(&result, context.as_ref());
    let mut html = PRINT_TEMPLATE_HTML.to_string();

    for (key, value) in values {
        html = html.replace(&format!("{{{{{key}}}}}"), &value);
    }

    Ok(html)
}

fn build_template_values(result: &Value, context: Option<&PrintContext>) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let mut put = |key: &str, value: String| {
        values.insert(key.to_string(), value);
    };

    let name = context
        .and_then(|item| item.name.as_ref())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| "未填".to_string());
    let note = context
        .and_then(|item| item.note.as_ref())
        .map(|value| value.trim().to_string())
        .unwrap_or_default();

    put("姓名", escape_html(&name));
    put("姓名內容", escape_html(&name));
    put("出生地內容", "未填".to_string());
    put("區域時間", "區域時間調整：不調整".to_string());
    put("經度欄", escape_html(&note));

    let gender = context
        .and_then(|item| item.gender.as_deref())
        .unwrap_or_default();
    let chart_type = match gender {
        "MALE" => "乾造",
        "FEMALE" => "坤造",
        _ => "命造",
    };
    let chart_type_parts = split_chars(chart_type, 2);
    put("造1", chart_type_parts[0].clone());
    put("造2", chart_type_parts[1].clone());

    let pillars = [
        PillarPrintData::from_value(result.get("hourPillar"), result.get("hourStemTenGod"), result.get("hourHiddenStems"), result.get("hourBranchTenGods")),
        PillarPrintData::from_value(result.get("dayPillar"), Some(&Value::String("日主".to_string())), result.get("dayHiddenStems"), result.get("dayBranchTenGods")),
        PillarPrintData::from_value(result.get("monthPillar"), result.get("monthStemTenGod"), result.get("monthHiddenStems"), result.get("monthBranchTenGods")),
        PillarPrintData::from_value(result.get("yearPillar"), result.get("yearStemTenGod"), result.get("yearHiddenStems"), result.get("yearBranchTenGods")),
    ];

    for (prefix, pillar) in [
        ("時", &pillars[0]),
        ("日", &pillars[1]),
        ("月", &pillars[2]),
        ("年", &pillars[3]),
    ] {
        if prefix != "日" {
            let parts = split_chars(&pillar.ten_god, 2);
            put(&format!("{prefix}柱十神1"), escape_html(&parts[0]));
            put(&format!("{prefix}柱十神2"), escape_html(&parts[1]));
        }
        put(&format!("{prefix}柱天干"), escape_html(&pillar.stem));
        put(&format!("{prefix}柱地支"), escape_html(&pillar.branch));
        put(&format!("{prefix}支藏干"), escape_html(&join_reversed(&pillar.hidden_stems, "　")));
        put(&format!("{prefix}支藏干十神"), ten_god_grid_html(&pillar.branch_ten_gods));
    }

    let luck_start = result.get("luckStart").unwrap_or(&Value::Null);
    let transition = value_string(luck_start, "transitionSummary");
    let transition_info = parse_transition_info(&transition);
    put("交運天干1", transition_info.stems[0].clone());
    put("交運天干2", transition_info.stems[1].clone());
    put("交運節氣", escape_html(&transition_info.jie_qi));
    put("交運節後天數", transition_info.after_days);

    let start_summary = value_string(luck_start, "startSummary");
    let start_parts = digit_runs(&start_summary);
    let start_years = start_parts.get(0).cloned().unwrap_or_default();
    let start_months = start_parts.get(1).cloned().unwrap_or_default();
    let start_days = start_parts.get(2).cloned().unwrap_or_default();
    let virtual_age = start_years
        .parse::<i32>()
        .map(|value| (value + 1).to_string())
        .unwrap_or_default();
    put("出生節氣", escape_html(&value_string(luck_start, "birthJieName")));
    put("起運年", start_years);
    put("起運月", start_months);
    put("起運日", start_days);
    put("虛歲", virtual_age);

    let solar = parse_date_time(&value_string(result, "solarDateTime"));
    let lunar = parse_lunar_date_time(&value_string(result, "lunarDateTime"), &solar.year);
    put("國曆年", solar.year);
    put("國曆月", solar.month);
    put("國曆日", solar.day);
    put("農曆年", lunar.year);
    put("農曆閏月", if value_string(result, "lunarDateTime").contains('閏') { "閏".to_string() } else { String::new() });
    put("農曆月", lunar.month);
    put("農曆日", lunar.day);
    put("出生時", solar.hour);
    put("出生分", solar.minute);
    put("時辰", parse_shi_chen(context.and_then(|item| item.time_label.as_deref()), &pillars[0].branch));

    let now = chrono::Local::now();
    put("排盤年", (now.year() - 1911).to_string());
    put("排盤月", now.month().to_string());
    put("排盤日", now.day().to_string());

    let luck_rows = build_luck_rows(result.get("daYun"));
    for index in 0..8 {
        let row = luck_rows.get(index).cloned().unwrap_or_default();
        put(&format!("大運{}_歲", index + 1), row.start_age.map(|age| age.to_string()).unwrap_or_default());
        put(&format!("大運{}_干支", index + 1), vertical_chars_html(&row.gan_zhi));
        put(&format!("流年{}_干支", index + 1), build_annual_cells(&row.annuals, true));
        put(&format!("流年{}_歲數", index + 1), build_annual_cells(&row.annuals, false));
    }

    values
}

#[derive(Debug, Clone, Default)]
struct PillarPrintData {
    ten_god: String,
    stem: String,
    branch: String,
    hidden_stems: Vec<String>,
    branch_ten_gods: Vec<String>,
}

impl PillarPrintData {
    fn from_value(
        pillar: Option<&Value>,
        ten_god: Option<&Value>,
        hidden_stems: Option<&Value>,
        branch_ten_gods: Option<&Value>,
    ) -> Self {
        Self {
            ten_god: ten_god.and_then(Value::as_str).unwrap_or_default().to_string(),
            stem: pillar.and_then(|value| value.get("stem")).and_then(Value::as_str).unwrap_or_default().to_string(),
            branch: pillar.and_then(|value| value.get("branch")).and_then(Value::as_str).unwrap_or_default().to_string(),
            hidden_stems: string_array(hidden_stems),
            branch_ten_gods: string_array(branch_ten_gods),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct LuckPrintRow {
    gan_zhi: String,
    start_age: Option<i32>,
    annuals: Vec<AnnualPrintData>,
}

#[derive(Debug, Clone, Default)]
struct AnnualPrintData {
    gan_zhi: String,
    age: i32,
}

fn build_luck_rows(value: Option<&Value>) -> Vec<LuckPrintRow> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(8)
        .map(|row| {
            let mut annuals = row
                .get("liuNian")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|annual| AnnualPrintData {
                    gan_zhi: value_string(annual, "ganZhi"),
                    age: annual.get("age").and_then(Value::as_i64).unwrap_or_default() as i32,
                })
                .collect::<Vec<_>>();
            let start_age = row.get("startAge").and_then(Value::as_i64).map(|age| age as i32);
            annuals = normalize_annuals(annuals, start_age);

            LuckPrintRow {
                gan_zhi: value_string(row, "ganZhi"),
                start_age,
                annuals,
            }
        })
        .collect()
}

fn normalize_annuals(mut annuals: Vec<AnnualPrintData>, start_age: Option<i32>) -> Vec<AnnualPrintData> {
    let Some(start_age) = start_age else {
        annuals.sort_by(|left, right| right.age.cmp(&left.age));
        annuals.truncate(10);
        return annuals;
    };

    if !annuals.iter().any(|annual| annual.age == start_age) {
        if let Some(next_annual) = annuals.iter().find(|annual| annual.age == start_age + 1) {
            if let Some(previous) = previous_gan_zhi(&next_annual.gan_zhi) {
                annuals.push(AnnualPrintData {
                    gan_zhi: previous,
                    age: start_age,
                });
            }
        }
    }

    let mut filtered = ((start_age)..=(start_age + 9))
        .filter_map(|age| annuals.iter().find(|annual| annual.age == age).cloned())
        .collect::<Vec<_>>();
    filtered.sort_by(|left, right| right.age.cmp(&left.age));

    if filtered.is_empty() {
        annuals.sort_by(|left, right| right.age.cmp(&left.age));
        annuals.truncate(10);
        annuals
    } else {
        filtered
    }
}

fn previous_gan_zhi(value: &str) -> Option<String> {
    const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    const BRANCHES: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

    let mut chars = value.chars();
    let stem = chars.next()?.to_string();
    let branch = chars.next()?.to_string();
    let stem_index = STEMS.iter().position(|item| *item == stem)?;
    let branch_index = BRANCHES.iter().position(|item| *item == branch)?;
    let previous_stem = STEMS[(stem_index + STEMS.len() - 1) % STEMS.len()];
    let previous_branch = BRANCHES[(branch_index + BRANCHES.len() - 1) % BRANCHES.len()];

    Some(format!("{previous_stem}{previous_branch}"))
}

fn build_annual_cells(items: &[AnnualPrintData], use_gan_zhi: bool) -> String {
    let mut cells = items
        .iter()
        .take(10)
        .map(|item| {
            if use_gan_zhi {
                format!("<td><span class=\"annual-ganzhi\">{}</span></td>", vertical_chars_html(&item.gan_zhi))
            } else {
                format!("<td>{}</td>", item.age)
            }
        })
        .collect::<Vec<_>>();
    while cells.len() < 10 {
        cells.push("<td></td>".to_string());
    }
    cells.join("")
}

fn vertical_chars_html(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .map(|ch| format!("<span>{}</span>", escape_html(&ch.to_string())))
        .collect::<Vec<_>>()
        .join("")
}

fn ten_god_grid_html(items: &[String]) -> String {
    let terms = items
        .iter()
        .rev()
        .map(|item| format!("<span class=\"ten-god-term\">{}</span>", vertical_chars_html(item)))
        .collect::<Vec<_>>();

    if terms.is_empty() {
        String::new()
    } else {
        format!("<span class=\"ten-god-grid\">{}</span>", terms.join(""))
    }
}

fn join_reversed(items: &[String], separator: &str) -> String {
    items
        .iter()
        .rev()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(separator)
}

#[derive(Debug, Default)]
struct TransitionInfo {
    stems: [String; 2],
    jie_qi: String,
    after_days: String,
}

fn parse_transition_info(text: &str) -> TransitionInfo {
    let stems = text
        .chars()
        .filter(|ch| "甲乙丙丁戊己庚辛壬癸".contains(*ch))
        .take(2)
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>();
    let after_days = text
        .find('後')
        .map(|index| {
            text[index + "後".len()..]
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
        })
        .unwrap_or_default();
    let jie_qi = text
        .find('後')
        .map(|index| {
            text[..index]
                .chars()
                .rev()
                .take_while(|ch| !"年月日0123456789及每逢甲乙丙丁戊己庚辛壬癸".contains(*ch))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>()
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "節氣".to_string());

    TransitionInfo {
        stems: [
            stems.get(0).cloned().unwrap_or_default(),
            stems.get(1).cloned().unwrap_or_default(),
        ],
        jie_qi,
        after_days,
    }
}

#[derive(Debug, Default)]
struct DateTimeParts {
    year: String,
    month: String,
    day: String,
    hour: String,
    minute: String,
}

fn parse_date_time(text: &str) -> DateTimeParts {
    let mut runs = digit_runs(text);
    if runs.len() < 3 {
        runs = date_runs_from_chinese(text);
    }

    let year = runs
        .get(0)
        .and_then(|value| value.parse::<i32>().ok())
        .map(to_roc_year)
        .unwrap_or_default();

    DateTimeParts {
        year,
        month: strip_leading_zero(runs.get(1).cloned().unwrap_or_default()),
        day: strip_leading_zero(runs.get(2).cloned().unwrap_or_default()),
        hour: runs.get(3).cloned().unwrap_or_default(),
        minute: runs.get(4).cloned().unwrap_or_default(),
    }
}

fn parse_lunar_date_time(text: &str, fallback_year: &str) -> DateTimeParts {
    if text.trim().is_empty() || text.contains("未提供") {
        return DateTimeParts {
            year: fallback_year.to_string(),
            ..DateTimeParts::default()
        };
    }

    let month = text
        .find('年')
        .and_then(|year_pos| {
            let after_year = &text[year_pos + "年".len()..];
            let month_token = after_year
                .chars()
                .take_while(|ch| *ch != ' ' && *ch != '　' && *ch != '(' && *ch != '（')
                .collect::<String>()
                .replace('閏', "")
                .replace('月', "");
            chinese_date_number(&month_token).map(|value| value.to_string())
        })
        .unwrap_or_default();

    let day = text
        .find('（')
        .and_then(|start| {
            text[start + "（".len()..]
                .find('）')
                .map(|end| &text[start + "（".len()..start + "（".len() + end])
        })
        .and_then(chinese_date_number)
        .map(|value| value.to_string())
        .unwrap_or_default();

    if month.is_empty() && day.is_empty() {
        let mut parsed = parse_date_time(text);
        if parsed.year.is_empty() || parsed.year == "0" {
            parsed.year = fallback_year.to_string();
        }
        return parsed;
    }

    DateTimeParts {
        year: fallback_year.to_string(),
        month,
        day,
        hour: String::new(),
        minute: String::new(),
    }
}

fn to_roc_year(year: i32) -> String {
    if year > 1911 {
        (year - 1911).to_string()
    } else {
        year.to_string()
    }
}

fn strip_leading_zero(value: String) -> String {
    if value.is_empty() {
        return value;
    }

    let trimmed = value.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn date_runs_from_chinese(text: &str) -> Vec<String> {
    let mut runs = Vec::new();
    for marker in ['年', '月', '日'] {
        if let Some(pos) = text.find(marker) {
            let before = &text[..pos];
            let token = before
                .chars()
                .rev()
                .take_while(|ch| is_chinese_date_char(*ch))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>();
            if token.is_empty() {
                runs.push(String::new());
            } else {
                runs.push(chinese_date_number(&token).map(|value| value.to_string()).unwrap_or_default());
            }
        }
    }
    runs
}

fn is_chinese_date_char(ch: char) -> bool {
    matches!(ch, '〇' | '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十' | '正' | '冬' | '臘' | '腊' | '初')
}

fn chinese_date_number(text: &str) -> Option<i32> {
    let cleaned = text.trim_start_matches('初');
    if cleaned.is_empty() {
        return None;
    }

    if cleaned.chars().count() >= 3 {
        let mut value = String::new();
        for ch in cleaned.chars() {
            value.push_str(&chinese_digit(ch)?.to_string());
        }
        return value.parse::<i32>().ok();
    }

    if cleaned == "十" {
        return Some(10);
    }
    if let Some(suffix) = cleaned.strip_prefix('十') {
        return Some(10 + chinese_digit(suffix.chars().next()?)?);
    }
    if let Some(prefix) = cleaned.strip_suffix('十') {
        return Some(chinese_digit(prefix.chars().next()?)? * 10);
    }
    if let Some(index) = cleaned.find('十') {
        let chars = cleaned.chars().collect::<Vec<_>>();
        let tens = chinese_digit(chars[index.saturating_sub(1)])? * 10;
        let ones = chars.get(index + 1).and_then(|ch| chinese_digit(*ch)).unwrap_or(0);
        return Some(tens + ones);
    }

    cleaned.chars().next().and_then(chinese_digit)
}

fn chinese_digit(ch: char) -> Option<i32> {
    match ch {
        '〇' | '零' => Some(0),
        '一' | '正' => Some(1),
        '二' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        '冬' => Some(11),
        '臘' | '腊' => Some(12),
        _ => None,
    }
}

fn parse_shi_chen(time_label: Option<&str>, hour_branch: &str) -> String {
    time_label
        .and_then(|label| label.chars().find(|ch| "子丑寅卯辰巳午未申酉戌亥".contains(*ch)))
        .or_else(|| hour_branch.chars().next())
        .map(|ch| ch.to_string())
        .unwrap_or_default()
}

fn digit_runs(text: &str) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }

    runs
}

fn split_chars(text: &str, count: usize) -> Vec<String> {
    let mut parts = text
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .take(count)
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>();
    while parts.len() < count {
        parts.push(String::new());
    }
    parts
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn value_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
