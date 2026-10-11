//! 陸致極《八字命理學進階教程》《動態分析教程》的日主強弱檢核表。
//!
//! 書中沒有給強弱分數，只有定性的步驟：得令、得地（根的等級與位置）、得生、得助，
//! 加上「無根一定身弱」「母多滅子」「五指傘」等硬性規則。這裡把每一項換成點數，
//! 門檻用兩本書中附有作者強弱結論的命例校準（tests/fixtures/lu-strength-cases.tsv）。
//! 本檢核與量化五行力、課堂三步法互相獨立，只供並列參考。

use serde::Serialize;
use tyme4rs::tyme::sixtycycle::SixtyCycle;
use tyme4rs::tyme::Culture;

const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
const PILLAR_BRANCH_LABELS: [&str; 4] = ["年支", "月支", "日支", "時支"];
const PILLAR_STEM_LABELS: [&str; 4] = ["年干", "月干", "日干", "時干"];
/// 根的位置權重：離日主越近越有力（進階 p119–120 親疏距離：日支、月支、時支、年支）。
const ROOT_POSITION_WEIGHT: [f64; 4] = [0.7, 0.9, 1.0, 0.8];
/// 強根（祿、刃／主氣）、中根（中氣，如衰、陽干長生）、弱根（餘氣，如墓、陰干長生）。
const ROOT_GRADE_WEIGHT: [f64; 3] = [1.0, 0.6, 0.35];
const ROOT_GRADE_LABELS: [&str; 3] = ["強根", "中根", "弱根"];
const ROOT_SCORE_MULTIPLIER: f64 = 2.0;
const STEM_PEER_POINTS: f64 = 0.8;
const STEM_SEAL_POINTS: f64 = 0.6;
const BRANCH_SEAL_POINTS: f64 = 0.4;
const STEM_OPPOSING_POINTS: f64 = -0.6;
const BRANCH_OPPOSING_POINTS: f64 = -0.5;
/// 校準門檻：分數 ≥ 此值判為偏強（77 個無疑義命例中 72 個相符）。
pub const LU_STRENGTH_THRESHOLD: f64 = 1.25;
const SEAL_OVERLOAD_COUNT: usize = 5;
const UMBRELLA_COUNT: usize = 5;
const DOMINANT_COUNT: usize = 7;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LuStrengthAnalysis {
    pub method: String,
    pub verdict: String,
    pub category: String,
    pub score: f64,
    pub threshold: f64,
    pub month_state: String,
    pub same_party_count: usize,
    pub checks: Vec<LuStrengthCheck>,
    pub roots: Vec<LuStrengthRoot>,
    pub flags: Vec<LuStrengthFlag>,
    pub conclusion: String,
    pub sources: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LuStrengthCheck {
    pub item: String,
    pub result: String,
    pub points: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LuStrengthRoot {
    pub pillar: String,
    pub branch: String,
    pub hidden_stem: String,
    pub grade: String,
    pub weight: f64,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LuStrengthFlag {
    pub kind: String,
    pub title: String,
    pub detail: String,
}

fn stem_element(stem: &str) -> &'static str {
    match stem {
        "甲" | "乙" => "木",
        "丙" | "丁" => "火",
        "戊" | "己" => "土",
        "庚" | "辛" => "金",
        _ => "水",
    }
}

fn stem_is_yang(stem: &str) -> bool {
    STEMS.iter().position(|item| *item == stem).unwrap_or(0) % 2 == 0
}

fn hidden_stems(branch: &str) -> &'static [&'static str] {
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
        _ => &["壬", "甲"],
    }
}

fn branch_main_element(branch: &str) -> &'static str {
    stem_element(hidden_stems(branch)[0])
}

fn generates(element: &str) -> &'static str {
    match element {
        "木" => "火",
        "火" => "土",
        "土" => "金",
        "金" => "水",
        _ => "木",
    }
}

fn controls(element: &str) -> &'static str {
    match element {
        "木" => "土",
        "土" => "水",
        "水" => "火",
        "火" => "金",
        _ => "木",
    }
}

/// 五類十神：比劫、印、食傷、財、官殺。
fn family(day: &str, other: &str) -> &'static str {
    if day == other {
        "比劫"
    } else if generates(other) == day {
        "印"
    } else if generates(day) == other {
        "食傷"
    } else if controls(day) == other {
        "財"
    } else {
        "官殺"
    }
}

fn is_clash(left: &str, right: &str) -> bool {
    const PAIRS: [(&str, &str); 6] = [
        ("子", "午"),
        ("丑", "未"),
        ("寅", "申"),
        ("卯", "酉"),
        ("辰", "戌"),
        ("巳", "亥"),
    ];
    PAIRS
        .iter()
        .any(|(a, b)| (*a == left && *b == right) || (*a == right && *b == left))
}

/// 祿、刃（帝旺）所在地支，用於「五指傘」的祿刃判斷。
fn lu_and_ren(stem: &str) -> (&'static str, &'static str) {
    match stem {
        "甲" => ("寅", "卯"),
        "乙" => ("卯", "寅"),
        "丙" => ("巳", "午"),
        "丁" => ("午", "巳"),
        "戊" => ("巳", "午"),
        "己" => ("午", "巳"),
        "庚" => ("申", "酉"),
        "辛" => ("酉", "申"),
        "壬" => ("亥", "子"),
        _ => ("子", "亥"),
    }
}

/// 地支合局改變五行：三會要三字齊全，三合可只有中神加一字（半合）；
/// 都要天干透出化神才算成化（動態 p34–36）。半合不改變月令。
fn combined_elements(stems: &[String; 4], branches: &[String; 4]) -> [Option<(&'static str, String)>; 4] {
    const MEETINGS: [([&str; 3], &str); 4] = [
        (["寅", "卯", "辰"], "木"),
        (["巳", "午", "未"], "火"),
        (["申", "酉", "戌"], "金"),
        (["亥", "子", "丑"], "水"),
    ];
    const TRIADS: [([&str; 3], &str); 4] = [
        (["申", "子", "辰"], "水"),
        (["亥", "卯", "未"], "木"),
        (["寅", "午", "戌"], "火"),
        (["巳", "酉", "丑"], "金"),
    ];
    let mut result: [Option<(&'static str, String)>; 4] = [None, None, None, None];
    let groups = MEETINGS
        .iter()
        .map(|(members, element)| (members, *element, false))
        .chain(TRIADS.iter().map(|(members, element)| (members, *element, true)));
    for (members, element, is_triad) in groups {
        let indexes = (0..4)
            .filter(|index| members.contains(&branches[*index].as_str()))
            .collect::<Vec<_>>();
        let present = members
            .iter()
            .filter(|member| branches.iter().any(|branch| branch == *member))
            .count();
        let full = present == 3;
        let half = is_triad && present >= 2 && branches.iter().any(|branch| branch == members[1]);
        let led = stems.iter().any(|stem| stem_element(stem) == element);
        if !(full || half) || !led {
            continue;
        }
        let shown = members
            .iter()
            .filter(|member| branches.iter().any(|branch| branch == *member))
            .copied()
            .collect::<Vec<_>>()
            .join("");
        let label = format!(
            "{}{}{}",
            shown,
            if full { if is_triad { "三合" } else { "三會" } } else { "半合" },
            element
        );
        for index in indexes {
            if full || index != 1 {
                result[index] = Some((element, label.clone()));
            }
        }
    }
    result
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn analyze_names(pillars: [&str; 4]) -> LuStrengthAnalysis {
    let stems: [String; 4] = pillars.map(|pillar| pillar.chars().next().unwrap_or('甲').to_string());
    let branches: [String; 4] = pillars.map(|pillar| pillar.chars().nth(1).unwrap_or('子').to_string());
    analyze_parts(&stems, &branches)
}

pub fn analyze(natal_cycles: &[SixtyCycle; 4]) -> LuStrengthAnalysis {
    let names = natal_cycles.each_ref().map(|cycle| cycle.get_name());
    analyze_names(names.each_ref().map(|name| name.as_str()))
}

fn analyze_parts(stems: &[String; 4], branches: &[String; 4]) -> LuStrengthAnalysis {
    let day_stem = stems[2].as_str();
    let day = stem_element(day_stem);
    let combined = combined_elements(stems, branches);
    let element_of_branch = |index: usize| -> &str {
        combined[index]
            .as_ref()
            .map(|(element, _)| *element)
            .unwrap_or_else(|| branch_main_element(&branches[index]))
    };

    // 得地：根的等級、位置與受沖。
    let mut roots = Vec::new();
    let mut root_score = 0.0;
    let mut strong_root = false;
    for (index, branch) in branches.iter().enumerate() {
        if let Some((element, label)) = &combined[index] {
            if *element == day {
                let weight = ROOT_POSITION_WEIGHT[index];
                root_score += weight;
                strong_root = true;
                roots.push(LuStrengthRoot {
                    pillar: PILLAR_BRANCH_LABELS[index].to_string(),
                    branch: branch.clone(),
                    hidden_stem: "-".to_string(),
                    grade: ROOT_GRADE_LABELS[0].to_string(),
                    weight: round2(weight),
                    note: format!("{label}，合化成日主同類"),
                });
            }
            continue;
        }
        let hidden = hidden_stems(branch);
        let Some(layer) = hidden.iter().position(|stem| stem_element(stem) == day) else {
            continue;
        };
        let grade = if layer == 0 || (day == "土" && (branch == "巳" || branch == "午")) {
            0
        } else if layer == 1 {
            1
        } else {
            2
        };
        let mut damage = 1.0;
        let mut notes = Vec::new();
        for (other_index, other) in branches.iter().enumerate() {
            if other_index != index && is_clash(branch, other) {
                let adjacent = other_index.abs_diff(index) == 1;
                damage *= if adjacent { 0.5 } else { 0.8 };
                notes.push(format!(
                    "被{}{}{}",
                    PILLAR_BRANCH_LABELS[other_index],
                    other,
                    if adjacent { "緊沖" } else { "遙沖" }
                ));
            }
        }
        if grade == 0 && damage >= 0.75 {
            strong_root = true;
        }
        let weight = ROOT_GRADE_WEIGHT[grade] * ROOT_POSITION_WEIGHT[index] * damage;
        root_score += weight;
        roots.push(LuStrengthRoot {
            pillar: PILLAR_BRANCH_LABELS[index].to_string(),
            branch: branch.clone(),
            hidden_stem: hidden[layer].to_string(),
            grade: ROOT_GRADE_LABELS[grade].to_string(),
            weight: round2(weight),
            note: notes.join("、"),
        });
    }
    let rooted = !roots.is_empty() && root_score > 0.15;

    // 得令：月令對日主的旺相休囚死。
    let month_family = family(day, element_of_branch(1));
    let (month_state, month_points) = match month_family {
        "比劫" => ("旺", 3.0),
        "印" => ("相", 2.0),
        "食傷" => ("休", -1.0),
        "財" => ("囚", -1.5),
        _ => ("死", -2.0),
    };

    // 得生、得助與異方。
    let seal_stem = (0..4).any(|index| index != 2 && family(day, stem_element(&stems[index])) == "印");
    let mut stem_points = 0.0;
    let mut peer_stems = Vec::new();
    let mut seal_stems = Vec::new();
    let mut opposing_stems = Vec::new();
    let mut transformed_killings = Vec::new();
    for index in [0usize, 1, 3] {
        let stem = stems[index].as_str();
        let label = format!("{}{}", PILLAR_STEM_LABELS[index], stem);
        match family(day, stem_element(stem)) {
            "比劫" => {
                stem_points += STEM_PEER_POINTS;
                peer_stems.push(label);
            }
            "印" => {
                stem_points += STEM_SEAL_POINTS * if rooted { 1.0 } else { 0.5 };
                seal_stems.push(label);
            }
            "官殺" if seal_stem => transformed_killings.push(label),
            _ => {
                stem_points += STEM_OPPOSING_POINTS;
                opposing_stems.push(label);
            }
        }
    }
    let mut branch_points = 0.0;
    let mut seal_branches = Vec::new();
    let mut opposing_branches = Vec::new();
    for index in [0usize, 2, 3] {
        let label = format!("{}{}", PILLAR_BRANCH_LABELS[index], branches[index]);
        match family(day, element_of_branch(index)) {
            "印" => {
                branch_points += BRANCH_SEAL_POINTS;
                seal_branches.push(label);
            }
            "比劫" => {}
            _ => {
                branch_points += BRANCH_OPPOSING_POINTS;
                opposing_branches.push(label);
            }
        }
    }
    let score = round2(month_points + ROOT_SCORE_MULTIPLIER * root_score + stem_points + branch_points);

    let families = (0..4)
        .map(|index| family(day, stem_element(&stems[index])))
        .chain((0..4).map(|index| family(day, element_of_branch(index))))
        .collect::<Vec<_>>();
    let same_party_count = families
        .iter()
        .filter(|item| **item == "比劫" || **item == "印")
        .count();
    let seal_count = (0..4)
        .filter(|index| *index != 2 && family(day, stem_element(&stems[*index])) == "印")
        .count()
        + (0..4).filter(|index| family(day, element_of_branch(*index)) == "印").count();

    // 硬性規則與提示。
    let mut flags = Vec::new();
    let mut forced_weak = None;
    if !rooted && peer_stems.is_empty() {
        forced_weak = Some("日主無根，天干也沒有比劫：「無根一定身弱，即使有印也改變不了」（動態 p186）。");
        flags.push(LuStrengthFlag {
            kind: "rootless".to_string(),
            title: "無根".to_string(),
            detail: "身弱的下限。運歲見根反而可能招剋，也忌他干來合（動態 p186–189）。".to_string(),
        });
    } else if !rooted {
        flags.push(LuStrengthFlag {
            kind: "rootless".to_string(),
            title: "無根（天干有比劫）".to_string(),
            detail: format!(
                "地支沒有日主的根，只靠{}幫身。作者說「無根一定身弱」，但書中也有無根而比劫透干、判為身強的命例（動態命例 20、107），此結論只供參考。",
                peer_stems.join("、")
            ),
        });
    }
    if forced_weak.is_none() && seal_count >= SEAL_OVERLOAD_COUNT && !strong_root {
        forced_weak = Some("印星多達五個以上而日主沒有完好的強根：母多滅子，作者論身弱、不另立從印格（進階 p43–44）。");
        flags.push(LuStrengthFlag {
            kind: "seal_overload".to_string(),
            title: "母多滅子".to_string(),
            detail: format!("印星 {seal_count} 個，宜扶日主、制印，不論從印。"),
        });
    }
    let (lu_branch, ren_branch) = lu_and_ren(day_stem);
    let has_lu = branches.iter().any(|branch| branch == lu_branch);
    let has_ren = branches.iter().any(|branch| branch == ren_branch);
    let same_polarity_peer = (0..4).any(|index| {
        index != 2 && stems[index] == day_stem
    });
    let other_polarity_peer = (0..4).any(|index| {
        index != 2
            && stem_element(&stems[index]) == day
            && stem_is_yang(&stems[index]) != stem_is_yang(day_stem)
    });
    let full_umbrella = same_polarity_peer && other_polarity_peer && has_lu && has_ren && seal_count > 0;
    let killing_stems = (0..4)
        .filter(|index| *index != 2 && family(day, stem_element(&stems[*index])) == "官殺")
        .map(|index| format!("{}{}", PILLAR_STEM_LABELS[index], stems[index]))
        .collect::<Vec<_>>();
    if same_party_count >= DOMINANT_COUNT {
        if killing_stems.is_empty() {
            flags.push(LuStrengthFlag {
                kind: "dominant".to_string(),
                title: "專旺候選".to_string(),
                detail: format!(
                    "同方（比劫＋印）佔八字中的 {same_party_count} 個，天干沒有剋神。專旺只能順勢，用印或食傷，不可逆勢（進階 p245–246）。"
                ),
            });
        } else {
            flags.push(LuStrengthFlag {
                kind: "offended_dominance".to_string(),
                title: "犯旺".to_string(),
                detail: format!(
                    "同方 {same_party_count} 個，但{}透出剋神，不能論專旺，反成犯旺（動態 p183）。",
                    killing_stems.join("、")
                ),
            });
        }
    } else if same_party_count >= UMBRELLA_COUNT || full_umbrella {
        flags.push(LuStrengthFlag {
            kind: "umbrella".to_string(),
            title: "五指傘（身強上限）".to_string(),
            detail: if full_umbrella {
                "比、劫、祿、刃、印俱全。再走助身的運歲容易出問題（動態 p179–181）。".to_string()
            } else {
                format!("同方（比劫＋印）{same_party_count} 個，已到身強上限，再走助身的運歲容易出問題（動態 p179–180）。")
            },
        });
    }
    if !rooted && peer_stems.is_empty() && seal_count <= 1 {
        flags.push(LuStrengthFlag {
            kind: "follow_candidate".to_string(),
            title: "從格候選".to_string(),
            detail: "無根、無比劫、印也很少。作者論從的條件很嚴，多主張順勢而不輕論從（動態 p45–55, p77–78）。".to_string(),
        });
    }
    if roots.len() == 1 && peer_stems.is_empty() {
        flags.push(LuStrengthFlag {
            kind: "single_root".to_string(),
            title: "獨根".to_string(),
            detail: format!(
                "只有{}{}一個根，周圍沒有比劫幫扶，偏身弱；獨根不能遭破壞，運歲沖合此根要留意（動態 p184）。",
                roots[0].pillar, roots[0].branch
            ),
        });
    }
    if !transformed_killings.is_empty() {
        flags.push(LuStrengthFlag {
            kind: "killing_via_seal".to_string(),
            title: "官殺經印轉化".to_string(),
            detail: format!(
                "{}有印透出，官殺「兩頭蛇」貪生忘剋，不計為剋身（進階 p45–46）。",
                transformed_killings.join("、")
            ),
        });
    }

    let diff = score - LU_STRENGTH_THRESHOLD;
    let mut verdict = if diff >= 4.0 {
        "很強"
    } else if diff >= 1.5 {
        "強"
    } else if diff >= 0.0 {
        "稍強"
    } else if diff >= -1.5 {
        "稍弱"
    } else if diff >= -4.0 {
        "弱"
    } else {
        "很弱"
    };
    if forced_weak.is_some() && diff >= -1.5 {
        verdict = "弱";
    }
    let category = if verdict.contains('強') { "強" } else { "弱" };

    let checks = vec![
        LuStrengthCheck {
            item: "得令".to_string(),
            result: format!("{}（月令{}為{}）", month_state, branches[1], month_family),
            points: month_points,
            detail: "月令對日主：比劫為旺、印為相、食傷為休、財為囚、官殺為死。".to_string(),
        },
        LuStrengthCheck {
            item: "得地".to_string(),
            result: if roots.is_empty() {
                "無根".to_string()
            } else {
                roots
                    .iter()
                    .map(|root| format!("{}{}{}", root.pillar, root.branch, root.grade))
                    .collect::<Vec<_>>()
                    .join("、")
            },
            points: round2(ROOT_SCORE_MULTIPLIER * root_score),
            detail: "祿刃為強根、中氣為中根、餘氣為弱根；依日支、月支、時支、年支遞減，被沖再打折。".to_string(),
        },
        LuStrengthCheck {
            item: "得助".to_string(),
            result: if peer_stems.is_empty() { "無".to_string() } else { peer_stems.join("、") },
            points: round2(peer_stems.len() as f64 * STEM_PEER_POINTS),
            detail: "天干比劫幫身。".to_string(),
        },
        LuStrengthCheck {
            item: "得生".to_string(),
            result: {
                let items = seal_stems.iter().chain(seal_branches.iter()).cloned().collect::<Vec<_>>();
                let base = if items.is_empty() { "無".to_string() } else { items.join("、") };
                if month_family == "印" {
                    format!("{base}（月令{}的印已在得令計入）", branches[1])
                } else {
                    base
                }
            },
            points: round2(
                seal_stems.len() as f64 * STEM_SEAL_POINTS * if rooted { 1.0 } else { 0.5 }
                    + seal_branches.len() as f64 * BRANCH_SEAL_POINTS,
            ),
            detail: "印生身；無根時印的效果減半（印不能代替根）。".to_string(),
        },
        LuStrengthCheck {
            item: "異方".to_string(),
            result: {
                let items = opposing_stems.iter().chain(opposing_branches.iter()).cloned().collect::<Vec<_>>();
                if items.is_empty() { "無".to_string() } else { items.join("、") }
            },
            points: round2(
                opposing_stems.len() as f64 * STEM_OPPOSING_POINTS
                    + opposing_branches.len() as f64 * BRANCH_OPPOSING_POINTS,
            ),
            detail: "食傷、財、官殺耗身；有印透出時官殺不計。".to_string(),
        },
    ];
    let combined_notes = combined
        .iter()
        .flatten()
        .map(|(_, label)| label.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let conclusion = format!(
        "{}得令為「{}」，{}；合計 {:.2} 分（門檻 {:.2}），判為{}。{}{}",
        day_stem,
        month_state,
        if roots.is_empty() { "地支無根".to_string() } else { format!("有 {} 個根", roots.len()) },
        score,
        LU_STRENGTH_THRESHOLD,
        verdict,
        forced_weak.unwrap_or(""),
        if combined_notes.is_empty() {
            String::new()
        } else {
            format!("合局：{}。", combined_notes.join("、"))
        }
    );

    LuStrengthAnalysis {
        method: "陸致極強弱檢核（得令、得地、得助、得生，加無根、母多滅子、五指傘等硬性規則）".to_string(),
        verdict: verdict.to_string(),
        category: category.to_string(),
        score,
        threshold: LU_STRENGTH_THRESHOLD,
        month_state: month_state.to_string(),
        same_party_count,
        checks,
        roots,
        flags,
        conclusion,
        sources: vec![
            "陸致極《八字命理學進階教程》第一、二、四章".to_string(),
            "陸致極《八字命理學動態分析教程》第六章（強弱視角、五指傘、無根）".to_string(),
        ],
        limitations: vec![
            "書中沒有強弱分數，點數與門檻是用兩本書約 77 個無疑義命例校準的工程估計。".to_string(),
            "未處理拱夾虛神、月令司令深淺、調候與形象（順勢）視角。".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASES: &str = include_str!("../../tests/fixtures/lu-strength-cases.tsv");

    #[test]
    fn book_cases_mostly_agree_with_author_verdicts() {
        let mut total = 0;
        let mut agree = 0;
        let mut misses = Vec::new();
        for line in CASES.lines().skip(1) {
            let cols = line.split('\t').collect::<Vec<_>>();
            if cols.len() < 13 {
                continue;
            }
            let class = cols[10];
            if (class != "強" && class != "弱") || cols[12].trim() == "Y" || cols[8].contains('?') {
                continue;
            }
            let result = analyze_names([cols[5], cols[6], cols[7], cols[8]]);
            total += 1;
            if result.category == class {
                agree += 1;
            } else {
                misses.push(format!("{}{} {} {}", cols[0], cols[1], cols[3], result.score));
            }
        }
        assert!(total >= 70, "expected the clean book cases, got {total}");
        let rate = agree as f64 / total as f64;
        assert!(rate >= 0.9, "agreement {agree}/{total}; misses: {misses:?}");
    }

    #[test]
    fn rootless_day_master_is_weak_even_with_seals() {
        // 古案 戊戌 丙辰 辛丑 戊戌：四柱皆土，母多滅子（進階 p43–44）。
        let result = analyze_names(["戊戌", "丙辰", "辛丑", "戊戌"]);
        assert_eq!(result.category, "弱");
        assert!(result.flags.iter().any(|flag| flag.kind == "seal_overload"));
    }

    #[test]
    fn yue_fei_is_strong() {
        // 岳飛 癸未 乙卯 甲子 己巳：月令卯為帝旺，身強無疑（進階 p42–43）。
        let result = analyze_names(["癸未", "乙卯", "甲子", "己巳"]);
        assert_eq!(result.category, "強");
        assert_eq!(result.month_state, "旺");
    }

    #[test]
    fn umbrella_case_is_flagged() {
        // 命例 51：戊×3、辰×2、午，比刃印俱全而年干甲殺透出 → 犯旺或五指傘提示。
        let result = analyze_names(["甲辰", "戊辰", "戊申", "戊午"]);
        assert!(result
            .flags
            .iter()
            .any(|flag| flag.kind == "offended_dominance" || flag.kind == "umbrella"));
    }
}
