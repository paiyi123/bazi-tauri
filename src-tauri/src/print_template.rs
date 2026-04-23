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
        put(&format!("{prefix}支藏干"), escape_html(&pillar.hidden_stems.join("　")));
        put(&format!("{prefix}支藏干十神"), pillar.branch_ten_gods.iter().map(|item| escape_html(item)).collect::<Vec<_>>().join("<br>"));
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
    let lunar = parse_date_time(&value_string(result, "lunarDateTime"));
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
        put(&format!("大運{}_歲", index + 1), row.start_age.unwrap_or_default());
        put(&format!("大運{}_干支", index + 1), escape_html(&row.gan_zhi));
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
    start_age: Option<String>,
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
            annuals.sort_by(|left, right| right.age.cmp(&left.age));
            annuals.truncate(10);

            LuckPrintRow {
                gan_zhi: value_string(row, "ganZhi"),
                start_age: row.get("startAge").and_then(Value::as_i64).map(|age| age.to_string()),
                annuals,
            }
        })
        .collect()
}

fn build_annual_cells(items: &[AnnualPrintData], use_gan_zhi: bool) -> String {
    let mut cells = items
        .iter()
        .take(10)
        .map(|item| {
            if use_gan_zhi {
                format!("<td>{}</td>", escape_html(&item.gan_zhi))
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
    let runs = digit_runs(text);
    DateTimeParts {
        year: runs.get(0).cloned().unwrap_or_default(),
        month: runs.get(1).cloned().unwrap_or_default(),
        day: runs.get(2).cloned().unwrap_or_default(),
        hour: runs.get(3).cloned().unwrap_or_default(),
        minute: runs.get(4).cloned().unwrap_or_default(),
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
