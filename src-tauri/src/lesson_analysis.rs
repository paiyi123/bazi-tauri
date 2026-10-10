mod dto;
mod facts;
mod pattern;
mod preferences;
mod strength;

pub use dto::*;
use facts::build_facts;
use pattern::build_pattern;
use preferences::build_preferences;
use strength::build_strength;

use crate::bazi::QuantModelInteraction;
use std::collections::BTreeSet;
use tyme4rs::tyme::culture::Element;
use tyme4rs::tyme::enums::HideHeavenStemType;
use tyme4rs::tyme::sixtycycle::{HeavenStem, SixtyCycle};
use tyme4rs::tyme::Culture;

const PILLAR_LABELS: [&str; 4] = ["年柱", "月柱", "日柱", "時柱"];
const STEM_LOCATION_LABELS: [&str; 4] = ["年干", "月干", "日干", "時干"];
const BRANCH_LOCATION_LABELS: [&str; 4] = ["年支", "月支", "日支", "時支"];
const FAMILY_ORDER: [&str; 5] = ["印", "比劫", "食傷", "財", "官殺"];
const PRESSURE_FAMILIES: [&str; 3] = ["食傷", "財", "官殺"];

pub(crate) fn analyze(
    natal_cycles: &[SixtyCycle; 4],
    reference: &LessonModelReference,
    interactions: &[QuantModelInteraction],
) -> LessonThreeStepAnalysis {
    let facts = build_facts(natal_cycles);
    let strength = build_strength(&facts, interactions);
    let pattern = build_pattern(&facts, &strength, reference);
    let preferences = build_preferences(&facts, &strength);
    let status = if strength.classification == "依據不足" {
        "insufficient"
    } else if strength.status == "conditional"
        || pattern.special_review.required
        || preferences.plans.len() > 1
    {
        "conditional"
    } else {
        "candidate"
    };
    let comparison = build_comparison(&pattern, &strength, &preferences, reference);

    LessonThreeStepAnalysis {
        version: "classroom-2026-09-v1".to_string(),
        status: status.to_string(),
        facts,
        pattern,
        strength,
        preferences,
        comparison,
        sources: lesson_sources(),
        limitations: vec![
            "本結果只重現本課派首版條件分支，不代表命理預測能力獲得驗證。".to_string(),
            "得勢採用 2026/07/22 的工程暫定門檻；互動後有效力量未重新計分。".to_string(),
            "工程候選可能與老師的精確來源命例結論不同；既有 source_case_only 註解仍分開保留，不以四柱硬編改判。".to_string(),
            "不輸出疾病、死亡、自傷、事故、婚姻、投資或人格等現實事件斷語。".to_string(),
        ],
    }
}

fn build_comparison(
    pattern: &LessonPatternAnalysis,
    strength: &LessonStrengthAnalysis,
    preferences: &LessonPreferenceAnalysis,
    reference: &LessonModelReference,
) -> LessonComparison {
    let lesson_pattern = pattern.ordinary_result.clone().unwrap_or_else(|| {
        pattern
            .ordinary_candidates
            .iter()
            .map(|item| item.pattern.clone())
            .collect::<Vec<_>>()
            .join("、")
    });
    let lesson_directions = preferences
        .plans
        .iter()
        .flat_map(|plan| plan.use_directions.iter().map(|item| item.element.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    LessonComparison {
        pattern: LessonComparisonItem {
            agreement_status: agreement(&reference.ge_ju, &lesson_pattern),
            app_value: reference.ge_ju.clone(),
            lesson_value: lesson_pattern,
            note: "App geJu 與課堂普通格候選逐項比較；直輸入口的標籤不當作課堂格名。".to_string(),
        },
        strength: LessonComparisonItem {
            agreement_status: match (
                app_strength_category(&reference.strength_label),
                lesson_strength_category(&strength.classification),
            ) {
                (Some(app), Some(lesson)) if app == lesson => "agree",
                (Some(_), Some(_)) => "conflict",
                _ => "not_comparable",
            }
            .to_string(),
            app_value: reference.strength_label.clone(),
            lesson_value: strength.classification.clone(),
            note: "App 量化強弱只供比較，不參與課堂三項觀察。".to_string(),
        },
        preferences: LessonComparisonItem {
            agreement_status: if reference
                .use_elements
                .iter()
                .any(|item| lesson_directions.contains(item))
            {
                "partial"
            } else if reference.use_elements.is_empty() && lesson_directions.is_empty() {
                "not_comparable"
            } else {
                "conflict"
            }
            .to_string(),
            app_value: reference.use_elements.join("、"),
            lesson_value: lesson_directions.join("、"),
            note: "僅比較元素方向；不把跨方案角色合併成新的正式用神。".to_string(),
        },
    }
}

fn app_strength_category(value: &str) -> Option<bool> {
    match value {
        "偏強" => Some(true),
        "偏弱" => Some(false),
        _ => None,
    }
}

fn lesson_strength_category(value: &str) -> Option<bool> {
    match value {
        "強候選" | "旺候選" | "旺極候選" => Some(true),
        "弱候選" | "衰極候選" => Some(false),
        _ => None,
    }
}

fn pressure_families(facts: &LessonFacts, surface: bool) -> Vec<String> {
    PRESSURE_FAMILIES
        .iter()
        .filter(|family| {
            let distribution = family_distribution(facts, family);
            if surface {
                !distribution.surface_locations.is_empty()
            } else {
                !distribution.hidden_locations.is_empty()
            }
        })
        .map(|item| (*item).to_string())
        .collect()
}

fn family_distribution<'a>(facts: &'a LessonFacts, family: &str) -> &'a LessonFamilyDistribution {
    facts
        .ten_god_distribution
        .iter()
        .find(|item| item.family == family)
        .expect("all five ten-god families are built")
}

fn family_evidence_ids(facts: &LessonFacts, families: &[&str]) -> Vec<String> {
    families
        .iter()
        .flat_map(|family| {
            let distribution = family_distribution(facts, family);
            distribution
                .surface_evidence_ids
                .iter()
                .chain(distribution.hidden_evidence_ids.iter())
                .cloned()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn family_for_element(day: &Element, target: &Element) -> &'static str {
    if target == day {
        "比劫"
    } else if target == &day.get_reinforced() {
        "印"
    } else if target == &day.get_reinforce() {
        "食傷"
    } else if target == &day.get_restrain() {
        "財"
    } else {
        "官殺"
    }
}

fn family_element(day: &Element, family: &str) -> Element {
    match family {
        "印" => day.get_reinforced(),
        "比劫" => day.clone(),
        "食傷" => day.get_reinforce(),
        "財" => day.get_restrain(),
        "官殺" => day.get_restrained(),
        _ => day.clone(),
    }
}

fn ten_god_name(day: &HeavenStem, target: &HeavenStem) -> String {
    traditional(&day.get_ten_star(target.clone()).get_name())
}

fn traditional(value: &str) -> String {
    value
        .replace('伤', "傷")
        .replace('财', "財")
        .replace('杀', "殺")
}

fn element_name(element: &Element) -> String {
    element.get_name()
}

fn hidden_layer(layer: HideHeavenStemType) -> &'static str {
    match layer {
        HideHeavenStemType::MAIN => "本氣",
        HideHeavenStemType::MIDDLE => "中氣",
        HideHeavenStemType::RESIDUAL => "餘氣",
    }
}

fn pattern_name(ten_god: &str) -> String {
    match ten_god {
        "比肩" => "建祿格".to_string(),
        "劫財" => "羊刃格".to_string(),
        _ => format!("{}格", ten_god),
    }
}

fn agreement(app: &str, lesson: &str) -> String {
    if app == lesson {
        "agree"
    } else if app == "直輸四柱分析" || app.is_empty() || lesson.is_empty() {
        "not_comparable"
    } else {
        "conflict"
    }
    .to_string()
}

fn lesson_sources() -> Vec<LessonSource> {
    vec![
        LessonSource {
            date: "2026-07-08".to_string(),
            rule_ids: vec!["specialized-use-version".to_string()],
        },
        LessonSource {
            date: "2026-07-22".to_string(),
            rule_ids: vec!["day-master-three-observations".to_string()],
        },
        LessonSource {
            date: "2026-07-29".to_string(),
            rule_ids: vec!["ordinary-pattern-and-interaction-evidence".to_string()],
        },
        LessonSource {
            date: "2026-08-12".to_string(),
            rule_ids: vec!["medicine-protects-selected-role".to_string()],
        },
        LessonSource {
            date: "2026-08-19".to_string(),
            rule_ids: vec!["seasonal-follows-support-balance".to_string()],
        },
        LessonSource {
            date: "2026-08-26".to_string(),
            rule_ids: vec![
                "ordinary-preference-branches".to_string(),
                "five-bridge-paths".to_string(),
            ],
        },
        LessonSource {
            date: "2026-09-02".to_string(),
            rule_ids: vec![
                "specialized-use-version".to_string(),
                "follow-child-peer-exception".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests;
