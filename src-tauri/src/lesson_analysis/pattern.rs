use super::*;

pub(super) fn build_pattern(
    facts: &LessonFacts,
    strength: &LessonStrengthAnalysis,
    reference: &LessonModelReference,
) -> LessonPatternAnalysis {
    // 2026-11-04（講義例一）：月令被相鄰的地支刑或沖，餘氣、雜氣被打掉，只剩本氣，
    // 取格只看本氣（例一戌月被未刑，時干透丁也不取正官格，取本氣戊＝偏印格）。
    let month_knocks = adjacent_knocks(facts, 1);
    let usable = facts
        .transparencies
        .iter()
        .filter(|item| month_knocks.is_empty() || item.hidden_stem_layer == "本氣")
        .collect::<Vec<_>>();
    let transparent = usable
        .iter()
        .copied()
        .filter(|item| !item.visible_locations.is_empty())
        .collect::<Vec<_>>();
    let source = if transparent.is_empty() {
        usable
    } else {
        transparent
    };
    let dropped_by_knock = if month_knocks.is_empty() {
        Vec::new()
    } else {
        facts
            .transparencies
            .iter()
            .filter(|item| item.hidden_stem_layer != "本氣" && !item.visible_locations.is_empty())
            .map(|item| format!("{}（{}，透{}）", item.hidden_stem, item.ten_god, item.visible_locations.join("、")))
            .collect::<Vec<_>>()
    };
    let multiple = source.len() > 1 && source.iter().all(|item| !item.visible_locations.is_empty());
    let ordinary_candidates = source
        .iter()
        .enumerate()
        .map(|(index, item)| LessonPatternCandidate {
            pattern: pattern_name(&item.ten_god),
            ten_god: item.ten_god.clone(),
            month_hidden_stem: item.hidden_stem.clone(),
            hidden_stem_layer: item.hidden_stem_layer.clone(),
            transparent_locations: item.visible_locations.clone(),
            selection: if multiple {
                "candidate"
            } else if index == 0 {
                "selected"
            } else {
                "fallback_candidate"
            }
            .to_string(),
            evidence_ids: vec![item.evidence_id.clone()],
        })
        .collect::<Vec<_>>();
    let ordinary_result = if multiple {
        None
    } else {
        ordinary_candidates.first().map(|item| item.pattern.clone())
    };
    // 刑沖後不取的透干仍列出：講義 26 章也有月令被刑、仍取透干的例子（己未 甲戌 壬子 丁未 取財格），
    // 兩說並存，讓使用者自己比較。
    let mut ordinary_candidates = ordinary_candidates;
    if !month_knocks.is_empty() {
        ordinary_candidates.extend(
            facts
                .transparencies
                .iter()
                .filter(|item| item.hidden_stem_layer != "本氣" && !item.visible_locations.is_empty())
                .map(|item| LessonPatternCandidate {
                    pattern: pattern_name(&item.ten_god),
                    ten_god: item.ten_god.clone(),
                    month_hidden_stem: item.hidden_stem.clone(),
                    hidden_stem_layer: item.hidden_stem_layer.clone(),
                    transparent_locations: item.visible_locations.clone(),
                    selection: "knocked_candidate".to_string(),
                    evidence_ids: vec![item.evidence_id.clone()],
                }),
        );
    }
    let rootless_trigger = facts.roots.is_empty()
        && strength.support_stem_count == 0
        && strength.support_branch_count == 0;
    let extreme_trigger = matches!(strength.classification.as_str(), "旺極候選" | "衰極候選");
    let mut reasons = Vec::new();
    if extreme_trigger {
        reasons.push(format!(
            "強弱初判為{}，須另審特殊格資格。",
            strength.classification
        ));
    }
    if rootless_trigger {
        reasons.push(
            "日主無同五行藏干根，且表層天干、地支本氣均無可見印比，須另審是否能從。".to_string(),
        );
    }
    if reference.special_pattern.is_some() || reference.special_category.is_some() {
        reasons.push("App 既有特殊格只作 model_reference；不據此新增或排除課派成格。".to_string());
    }
    let mut evidence = ordinary_candidates
        .iter()
        .filter(|candidate| candidate.selection != "knocked_candidate")
        .enumerate()
        .map(|(index, candidate)| LessonEvidence {
            id: format!("pattern-rule-{}", index + 1),
            kind: "lesson_rule".to_string(),
            pillar: Some("月支".to_string()),
            hidden_stem_layer: Some(candidate.hidden_stem_layer.clone()),
            interaction_id: None,
            description: if candidate.transparent_locations.is_empty() {
                format!(
                    "月令藏干未透，按{}順序保留{}。",
                    candidate.hidden_stem_layer, candidate.pattern
                )
            } else {
                format!(
                    "月令{}透於{}，保留{}。",
                    candidate.month_hidden_stem,
                    candidate.transparent_locations.join("、"),
                    candidate.pattern
                )
            },
            source_date: Some("2026-07-29".to_string()),
        })
        .collect::<Vec<_>>();
    if !month_knocks.is_empty() {
        evidence.push(LessonEvidence {
            id: "pattern-month-knocked".to_string(),
            kind: "lesson_rule".to_string(),
            pillar: Some("月支".to_string()),
            hidden_stem_layer: Some("本氣".to_string()),
            interaction_id: None,
            description: format!(
                "月令{}被{}，刑沖後只剩本氣，取格只看本氣{}。",
                facts.month_command.branch,
                month_knocks.join("、"),
                if dropped_by_knock.is_empty() {
                    String::new()
                } else {
                    format!("；餘氣透出的{}不取", dropped_by_knock.join("、"))
                }
            ),
            source_date: Some("2026-11-04".to_string()),
        });
    }
    evidence.push(LessonEvidence {
        id: "pattern-model-reference".to_string(),
        kind: "model_reference".to_string(),
        pillar: None,
        hidden_stem_layer: None,
        interaction_id: None,
        description: format!(
            "App 原格局為{}；此值只供比較，不餵回課堂候選。",
            reference.ge_ju
        ),
        source_date: None,
    });
    let conclusion = if multiple {
        format!(
            "月令有多個合格透干，保留{}，不以工程排序冒充老師唯一取格。",
            ordinary_candidates
                .iter()
                .map(|item| item.pattern.as_str())
                .collect::<Vec<_>>()
                .join("、")
        )
    } else if let (Some(result), false) = (&ordinary_result, month_knocks.is_empty()) {
        format!(
            "普通格候選為{}（月令{}被{}，刑沖後只剩本氣，取格看本氣{}）。",
            result,
            facts.month_command.branch,
            month_knocks.join("、"),
            if dropped_by_knock.is_empty() {
                String::new()
            } else {
                format!("，不取{}", dropped_by_knock.join("、"))
            }
        )
    } else if let Some(result) = &ordinary_result {
        format!("普通格候選為{}。", result)
    } else {
        "普通格依據不足。".to_string()
    };

    LessonPatternAnalysis {
        title: "第一步：格局判定".to_string(),
        conclusion,
        status: if multiple { "conditional" } else { "candidate" }.to_string(),
        ordinary_result,
        ordinary_candidates,
        special_review: LessonSpecialReview {
            required: extreme_trigger || rootless_trigger,
            reasons,
            model_reference_pattern: reference.special_pattern.clone(),
            model_reference_category: reference.special_category.clone(),
            specialized_use_versions: vec![
                LessonSpecialUseVersion {
                    source_date: "2026-07-08".to_string(),
                    use_direction: "旺神".to_string(),
                    favorable_direction: "生旺神與洩旺神".to_string(),
                    limitation: "課次用語未與後課統一，不作唯一順位。".to_string(),
                },
                LessonSpecialUseVersion {
                    source_date: "2026-09-02".to_string(),
                    use_direction: "食傷洩秀".to_string(),
                    favorable_direction: "印比扶旺".to_string(),
                    limitation: "需先確認專旺成格與破格控制元素。".to_string(),
                },
            ],
            follow_child_peer_exception:
                "從兒以食傷為用時，比劫可生食傷；不得套用所有從弱一律忌印比。".to_string(),
        },
        evidence,
        limitations: vec![
            "普通格候選不覆蓋 App geJu。".to_string(),
            "首版不新增從格、專旺或化氣的泛化成格閾值。".to_string(),
        ],
    }
}
