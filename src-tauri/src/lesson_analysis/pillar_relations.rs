//! 同柱干支生剋合的作用變化（草綠色講義第二十五章，第 42–46 頁）。
//!
//! 書明說：六十甲子上下關係分五類（天干生地支、地支生天干、天干剋地支、地支剋天干、同類），
//! 各十二組；同柱干支相合（暗合）十二組，其中甲午、丁亥、戊子、己亥、辛巳、壬午六組較純粹。
//! 判斷原則：日主所需之五行喜其通根乘旺，忌被剋敗、合壞；所忌之五行喜其虛浮無根、遭剋或坐絕。
//! 日柱干支上下之合若不入外格，以生合、剋合論。
//!
//! 推論：逐柱把天干的上下關係套進上述原則標示吉凶；天干生地支、天干剋地支只算「稍有耗損」。

use super::branch_gatherings::{favor_code, favor_label};
use super::stem_interactions::Favor;
use super::*;

const SOURCE: &str = "講義第25章";

const HIDDEN_COMBINATIONS: [(&str, &str, bool); 12] = [
    ("甲午", "甲木合午中己土", true),
    ("乙巳", "乙木合巳中庚金", false),
    ("丙戌", "丙火合戌中辛金", false),
    ("丁亥", "丁火合亥中壬水", true),
    ("戊子", "戊土合子中癸水", true),
    ("戊辰", "戊土合辰中癸水", false),
    ("己亥", "己土合亥中甲木", true),
    ("庚辰", "庚金合辰中乙木", false),
    ("辛巳", "辛金合巳中丙火", true),
    ("壬午", "壬水合午中丁火", true),
    ("壬戌", "壬水合戌中丁火", false),
    ("癸巳", "癸水合巳中戊土", false),
];

fn relation_kind(stem: &Element, branch: &Element) -> (&'static str, &'static str) {
    if stem == branch {
        ("same", "干支同類")
    } else if stem.get_reinforce() == *branch {
        ("stem_generates_branch", "天干生地支")
    } else if branch.get_reinforce() == *stem {
        ("branch_generates_stem", "地支生天干")
    } else if stem.get_restrain() == *branch {
        ("stem_controls_branch", "天干剋地支")
    } else {
        ("branch_controls_stem", "地支剋天干")
    }
}

pub(super) fn classify_pillar_relations(
    cycles: &[SixtyCycle; 4],
    favor: &dyn Fn(&str) -> Favor,
) -> Vec<LessonPillarRelation> {
    (0..4)
        .map(|index| {
            let cycle = &cycles[index];
            let stem = cycle.get_heaven_stem();
            let branch = cycle.get_earth_branch();
            let stem_element = stem.get_element();
            let branch_element = branch.get_element();
            let gan_zhi = cycle.get_name();
            let (kind, kind_label) = relation_kind(&stem_element, &branch_element);
            let stem_name = element_name(&stem_element);
            let branch_name = element_name(&branch_element);
            let stem_favor = favor(&stem_name);
            let effect = match kind {
                "same" => format!("上下比助，{}力加強（六十甲子中以干支同類之氣最專）", stem_name),
                "stem_generates_branch" => format!("{}生{}，{}加強、{}被洩減力", stem_name, branch_name, branch_name, stem_name),
                "branch_generates_stem" => format!("{}生{}，{}加強、{}被洩減力", branch_name, stem_name, stem_name, branch_name),
                "stem_controls_branch" => format!("{}剋{}，{}減弱，{}也因剋而損力", stem_name, branch_name, branch_name, stem_name),
                _ => format!("{}剋{}，{}坐剋（截腳）減弱，{}也因剋而損力", branch_name, stem_name, stem_name, branch_name),
            };
            let hidden = HIDDEN_COMBINATIONS.iter().find(|(name, _, _)| *name == gan_zhi.as_str());
            let mut notes = Vec::new();
            let is_day = index == 2;
            let (verdict, verdict_label) = if is_day {
                let label = match kind {
                    "same" | "branch_generates_stem" => "日主自坐得生扶",
                    "branch_controls_stem" => "日主坐剋",
                    _ => "日主坐下耗洩",
                };
                (Favor::Neutral, label.to_string())
            } else {
                match (stem_favor, kind) {
                    (Favor::Good, "same" | "branch_generates_stem") => (Favor::Good, "喜神通根乘旺".to_string()),
                    (Favor::Good, "branch_controls_stem") => (Favor::Bad, "喜神坐剋受損".to_string()),
                    (Favor::Bad, "same" | "branch_generates_stem") => (Favor::Bad, "忌神通根乘旺".to_string()),
                    (Favor::Bad, "branch_controls_stem") => (Favor::Good, "忌神坐剋減力".to_string()),
                    (Favor::Good, _) => (Favor::Neutral, "喜神稍有耗損".to_string()),
                    (Favor::Bad, _) => (Favor::Neutral, "忌神稍有耗損".to_string()),
                    (Favor::Mixed, _) => (Favor::Mixed, "依方案而定".to_string()),
                    (Favor::Neutral, _) => (Favor::Neutral, "未列喜忌".to_string()),
                }
            };
            let mut verdict = verdict;
            let mut verdict_label = verdict_label;
            if let Some((_, text, pure)) = hidden {
                if is_day {
                    let mode = match kind {
                        "stem_generates_branch" | "branch_generates_stem" => "生合",
                        "same" => "同類之合",
                        _ => "剋合",
                    };
                    notes.push(format!(
                        "日柱干支上下相合（{}）：不入外格時以生合、剋合論，依本氣屬{}（講義第 46 頁；生剋歸類為推論）。",
                        text, mode
                    ));
                } else {
                    notes.push(format!(
                        "干支暗合：{}{}；天干與支中藏干互相合絆，喜則有合壞之虞，忌則減其作用（講義第 46 頁）。",
                        text,
                        if *pure { "，屬六組較純粹者，影響絕對" } else { "" }
                    ));
                    // 暗合絆住的是天干與支中那一干兩邊（26 章地支例二十：壬午「絆住午中丁火」）。
                    let hidden_stem: String = text.chars().nth(5).map(String::from).unwrap_or_default();
                    let hidden_element = element_name(&HeavenStem::from_name(&hidden_stem).get_element());
                    let bound = [stem_favor, favor(&hidden_element)]
                        .into_iter()
                        .map(|item| match item {
                            Favor::Good => Favor::Bad,
                            Favor::Bad => Favor::Good,
                            other => other,
                        })
                        .collect::<Vec<_>>();
                    let good = bound.contains(&Favor::Good);
                    let bad = bound.contains(&Favor::Bad);
                    if *pure && (good || bad) {
                        if good && bad {
                            verdict = Favor::Mixed;
                            verdict_label = "暗合互絆，喜忌皆有".to_string();
                        } else if bad {
                            verdict = Favor::Bad;
                            verdict_label = format!("暗合絆住喜神（{}或{}）", stem_name, hidden_element);
                        } else {
                            verdict = Favor::Good;
                            verdict_label = "暗合絆住忌神".to_string();
                        }
                    }
                }
            }
            let branch_kind = ["寅", "申", "巳", "亥"].contains(&branch.get_name().as_str());
            let storage = ["辰", "戌", "丑", "未"].contains(&branch.get_name().as_str());
            if branch_kind {
                notes.push("四生之地藏干力量：本氣最大，其次長生，再次寄土；透出天干者最大（講義第 45 頁）。".to_string());
            } else if storage {
                notes.push("四墓之地藏干力量：土星本氣最大，其次餘氣，再次庫神；透出天干者最大（講義第 45 頁）。".to_string());
            }
            LessonPillarRelation {
                id: format!("pillar-relation-{}", index + 1),
                pillar: PILLAR_LABELS[index].to_string(),
                gan_zhi: gan_zhi.clone(),
                kind: kind.to_string(),
                kind_label: kind_label.to_string(),
                hidden_combination: hidden.map(|(_, text, _)| text.to_string()),
                hidden_combination_pure: hidden.map(|(_, _, pure)| *pure).unwrap_or(false),
                stem_favor: favor_label(stem_favor).to_string(),
                verdict: favor_code(verdict).to_string(),
                verdict_label,
                reason: format!("{}：{}。", gan_zhi, effect),
                notes,
                source_date: SOURCE.to_string(),
            }
        })
        .collect()
}

pub(super) fn build_pillar_relations(
    natal_cycles: &[SixtyCycle; 4],
    preferences: &LessonPreferenceAnalysis,
) -> LessonPillarRelationAnalysis {
    let favor = |element: &str| super::stem_interactions::element_favor(preferences, element);
    let relations = classify_pillar_relations(natal_cycles, &favor);
    let parts = relations
        .iter()
        .map(|item| {
            format!(
                "{}{}{}",
                item.gan_zhi,
                item.kind_label,
                item.hidden_combination
                    .as_ref()
                    .map(|_| "・暗合")
                    .unwrap_or("")
            )
        })
        .collect::<Vec<_>>();
    LessonPillarRelationAnalysis {
        title: "同柱干支".to_string(),
        conclusion: format!("{}。", parts.join("；")),
        relations,
        limitations: vec![
            "講義說同柱力量要配合時令、藏干與整體結構綜合論斷，沒有固定公式；這裡只標上下關係與喜忌方向，不改強弱與量化分數。".to_string(),
            "日柱的天干就是日主，不另判吉凶，只標自坐得生扶或坐剋。".to_string(),
        ],
        sources: vec![SOURCE.to_string()],
    }
}
