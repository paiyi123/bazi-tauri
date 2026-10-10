use super::*;

fn chart(names: [&str; 4]) -> [SixtyCycle; 4] {
    names.map(SixtyCycle::from_name)
}

fn reference() -> LessonModelReference {
    LessonModelReference {
        ge_ju: "App格局".to_string(),
        strength_label: "偏強".to_string(),
        use_elements: vec!["木".to_string()],
        special_pattern: None,
        special_category: None,
    }
}

#[test]
fn three_step_facts_keep_hidden_layers_and_exclude_day_stem_transparency() {
    let result = analyze(&chart(["戊申", "丙辰", "庚午", "癸未"]), &reference(), &[]);

    assert_eq!(result.version, "classroom-2026-09-v1");
    assert_eq!(result.facts.day_master.stem, "庚");
    assert_eq!(result.facts.month_command.main_stem, "戊");
    assert_eq!(
        result.facts.pillars[1]
            .hidden_stems
            .iter()
            .map(|item| (item.stem.as_str(), item.layer.as_str()))
            .collect::<Vec<_>>(),
        vec![("戊", "本氣"), ("乙", "中氣"), ("癸", "餘氣")]
    );
    assert_eq!(result.pattern.ordinary_result, None);
    assert_eq!(
        result
            .pattern
            .ordinary_candidates
            .iter()
            .map(|item| (item.pattern.as_str(), item.transparent_locations.as_slice()))
            .collect::<Vec<_>>(),
        vec![
            ("偏印格", ["年干".to_string()].as_slice()),
            ("傷官格", ["時干".to_string()].as_slice()),
        ]
    );
    assert!(!result.pattern.conclusion.contains("直輸四柱分析"));
}

#[test]
fn strength_separates_same_season_from_seal_and_keeps_support_boundary_literal() {
    let same = analyze(&chart(["壬子", "辛亥", "壬子", "辛亥"]), &reference(), &[]);
    assert!(same.strength.same_season_element);
    assert!(!same.strength.seal_in_season);
    assert_eq!(same.strength.support_stem_count, 3);
    assert!(same.strength.momentum_proxy);
    assert_eq!(same.strength.classification, "旺極候選");
    assert!(same
        .strength
        .pressure_families_hidden
        .contains(&"食傷".to_string()));

    let seal = analyze(&chart(["甲寅", "丁亥", "甲午", "丙寅"]), &reference(), &[]);
    assert!(!seal.strength.same_season_element);
    assert!(seal.strength.seal_in_season);
    assert_eq!(seal.strength.support_stem_count, 1);
    assert!(seal.strength.support_branch_count >= 2);
    assert!(seal.strength.momentum_proxy);
}

#[test]
fn support_proxy_is_false_at_one_and_true_at_two_visible_support_stems() {
    let one = analyze(&chart(["甲午", "丁丑", "甲午", "丙戌"]), &reference(), &[]);
    let two = analyze(&chart(["甲午", "丁丑", "甲午", "甲戌"]), &reference(), &[]);

    assert_eq!(one.strength.support_stem_count, 1);
    assert_eq!(one.strength.support_branch_count, 0);
    assert!(!one.strength.momentum_proxy);
    assert_eq!(two.strength.support_stem_count, 2);
    assert_eq!(two.strength.support_branch_count, 0);
    assert!(two.strength.momentum_proxy);
}

#[test]
fn no_transparency_falls_back_in_month_hidden_stem_order() {
    let result = analyze(&chart(["甲子", "丁丑", "丙寅", "壬辰"]), &reference(), &[]);

    assert_eq!(result.pattern.ordinary_candidates.len(), 3);
    assert!(result
        .pattern
        .ordinary_candidates
        .iter()
        .all(|candidate| candidate.transparent_locations.is_empty()));
    assert_eq!(
        result.pattern.ordinary_candidates[0].month_hidden_stem,
        "己"
    );
    assert_eq!(
        result.pattern.ordinary_candidates[0].hidden_stem_layer,
        "本氣"
    );
    assert_eq!(result.pattern.ordinary_candidates[0].selection, "selected");
    assert_eq!(
        result.pattern.ordinary_result,
        Some(result.pattern.ordinary_candidates[0].pattern.clone())
    );
}

#[test]
fn zero_observations_need_all_three_surface_pressure_families_for_extreme_weak() {
    let partial = analyze(&chart(["戊戌", "己丑", "甲午", "丁巳"]), &reference(), &[]);
    assert_eq!(partial.strength.observation_count, 0);
    assert_eq!(partial.strength.classification, "依據不足");
    assert_eq!(partial.strength.status, "insufficient");

    let full = analyze(&chart(["丙戌", "己丑", "甲午", "辛酉"]), &reference(), &[]);
    assert_eq!(full.strength.observation_count, 0);
    assert_eq!(
        full.strength.pressure_families_surface,
        vec!["食傷", "財", "官殺"]
    );
    assert_eq!(full.strength.classification, "衰極候選");
}

#[test]
fn interactions_preserve_positions_and_make_strength_conditional_without_pruning_roots() {
    let interaction = QuantModelInteraction {
        scope: "地支".to_string(),
        r#type: "沖".to_string(),
        target: "寅申".to_string(),
        outcome: "互沖".to_string(),
        pillars: "月支、日支".to_string(),
        detail: "原模型互動證據".to_string(),
    };
    let without = analyze(&chart(["丙子", "丙申", "庚寅", "丁亥"]), &reference(), &[]);
    let with = analyze(
        &chart(["丙子", "丙申", "庚寅", "丁亥"]),
        &reference(),
        &[interaction],
    );

    assert_eq!(with.strength.status, "conditional");
    assert_eq!(with.strength.interactions[0].pillars, "月支、日支");
    assert_eq!(with.strength.interactions[0].target, "寅申");
    assert_eq!(with.strength.root_count, without.strength.root_count);
    assert_eq!(
        with.strength.classification,
        without.strength.classification
    );
}

#[test]
fn references_change_comparison_but_not_classroom_core() {
    let cycles = chart(["丁卯", "丙午", "己酉", "癸酉"]);
    let a = analyze(&cycles, &reference(), &[]);
    let b = analyze(
        &cycles,
        &LessonModelReference {
            ge_ju: "完全不同".to_string(),
            strength_label: "偏弱".to_string(),
            use_elements: vec!["水".to_string()],
            special_pattern: Some("既有特殊格".to_string()),
            special_category: Some("從格/專旺".to_string()),
        },
        &[],
    );

    assert_eq!(
        serde_json::to_value(a.facts).unwrap(),
        serde_json::to_value(b.facts).unwrap()
    );
    assert_eq!(
        serde_json::to_value(a.strength).unwrap(),
        serde_json::to_value(b.strength).unwrap()
    );
    assert_eq!(
        serde_json::to_value(a.preferences).unwrap(),
        serde_json::to_value(b.preferences).unwrap()
    );
    assert_ne!(
        serde_json::to_value(a.comparison).unwrap(),
        serde_json::to_value(b.comparison).unwrap()
    );
}

#[test]
fn strength_comparison_requires_recognized_categories_on_both_sides() {
    let weak_chart = chart(["甲寅", "癸酉", "壬午", "癸卯"]);
    let unrecognized_app = analyze(
        &weak_chart,
        &LessonModelReference {
            strength_label: "依據不足".to_string(),
            ..reference()
        },
        &[],
    );
    assert_eq!(unrecognized_app.strength.classification, "弱候選");
    assert_eq!(
        unrecognized_app.comparison.strength.agreement_status,
        "not_comparable"
    );

    let insufficient_lesson = analyze(
        &chart(["戊戌", "己丑", "甲午", "丁巳"]),
        &LessonModelReference {
            strength_label: "偏弱".to_string(),
            ..reference()
        },
        &[],
    );
    assert_eq!(insufficient_lesson.strength.classification, "依據不足");
    assert_eq!(
        insufficient_lesson.comparison.strength.agreement_status,
        "not_comparable"
    );
}

#[test]
fn strong_chart_without_a_detailed_branch_gets_a_guarded_fuyi_fallback() {
    let result = analyze(&chart(["甲寅", "甲寅", "甲寅", "甲寅"]), &reference(), &[]);
    let fallback = result
        .preferences
        .plans
        .iter()
        .find(|plan| plan.rule_id == "ordinary-strong-fuyi-fallback")
        .expect("a recognized strong category must not leave the ordinary direction blank");

    assert_eq!(result.strength.classification, "旺極候選");
    assert!(result.pattern.special_review.required);
    assert!(fallback.condition.contains("若按普通格論"));
    assert!(fallback.condition.contains("特殊格資格排除後"));
    assert!(fallback.use_directions.is_empty());
    assert!(fallback.favorable_directions.is_empty());
    assert_eq!(
        fallback
            .conditional_directions
            .iter()
            .map(|direction| direction.family.as_str())
            .collect::<Vec<_>>(),
        vec!["食傷", "財", "官殺"]
    );
    assert!(fallback
        .rejections
        .iter()
        .any(|gap| gap.contains("未命中細分分支")));
}

#[test]
fn seasonal_assessment_stays_plan_specific_for_competing_roles() {
    let result = analyze(&chart(["戊申", "丙午", "庚午", "癸未"]), &reference(), &[]);
    let notes = serde_json::to_value(&result.preferences.seasonal_compatibility).unwrap();
    let notes = notes
        .as_array()
        .expect("seasonal notes should serialize as rows");
    let officer = notes
        .iter()
        .find(|note| note["planRuleId"] == "strong-peer-use-officer")
        .expect("officer plan needs its own seasonal assessment");
    let output = notes
        .iter()
        .find(|note| note["planRuleId"] == "strong-peer-use-output")
        .expect("output plan needs its own seasonal assessment");

    assert_eq!(officer["element"], "水");
    assert_eq!(officer["status"], "unresolved");
    assert_eq!(output["element"], "水");
    assert_eq!(output["status"], "conditional_compatible");
    assert_ne!(officer["explanation"], output["explanation"]);
}

#[test]
fn competing_plans_stay_separate_and_each_has_an_ordinary_pattern_guard() {
    let result = analyze(&chart(["丁卯", "丙午", "己酉", "癸酉"]), &reference(), &[]);

    assert!(result.preferences.plans.len() >= 2);
    assert!(
        result
            .preferences
            .plans
            .iter()
            .all(|plan| plan.applicability == "conditional"
                && plan.condition.contains("若按普通格論"))
    );
    assert!(result
        .preferences
        .plans
        .windows(2)
        .all(|pair| pair[0].rule_id <= pair[1].rule_id));
    assert!(result
        .preferences
        .medicine_notes
        .iter()
        .all(|note| !note.protected_plan_rule_id.is_empty() && !note.protected_family.is_empty()));
}

#[test]
fn preference_evidence_ids_resolve_to_real_chart_fact_evidence() {
    let result = analyze(&chart(["戊申", "丙辰", "庚午", "癸未"]), &reference(), &[]);
    let fact_ids = result
        .facts
        .evidence
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    assert!(result.preferences.plans.iter().all(|plan| {
        !plan.evidence_ids.is_empty()
            && plan
                .evidence_ids
                .iter()
                .all(|evidence_id| fact_ids.contains(evidence_id.as_str()))
    }));
}

#[test]
fn all_eight_ordinary_preference_branches_are_reachable_from_chart_facts() {
    let mut rule_ids = BTreeSet::new();
    for names in [
        ["丁卯", "丙午", "己酉", "癸酉"],
        ["戊申", "丙辰", "庚午", "癸未"],
        ["丙戌", "己丑", "甲午", "辛酉"],
    ] {
        rule_ids.extend(
            analyze(&chart(names), &reference(), &[])
                .preferences
                .plans
                .into_iter()
                .map(|plan| plan.rule_id),
        );
    }

    assert_eq!(
        rule_ids,
        [
            "strong-hidden-officer-visible-wealth",
            "strong-hidden-wealth-visible-kill",
            "strong-peer-use-officer",
            "strong-peer-use-output",
            "strong-seal-use-wealth",
            "weak-officer-use-seal",
            "weak-output-use-seal",
            "weak-wealth-use-peer",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    );
}

#[test]
fn wealth_hidden_kill_visible_and_officer_hidden_wealth_visible_use_distinct_evidence() {
    let kill = analyze(&chart(["戊申", "丙辰", "庚午", "癸未"]), &reference(), &[]);
    let kill_plan = kill
        .preferences
        .plans
        .iter()
        .find(|plan| plan.rule_id == "strong-hidden-wealth-visible-kill")
        .expect("kill plan should use hidden wealth and visible rooted kill");
    assert!(kill_plan
        .evidence_ids
        .contains(&"fact-hidden-2-2".to_string()));
    assert!(kill_plan.evidence_ids.contains(&"fact-stem-2".to_string()));

    let wealth = analyze(&chart(["壬子", "癸亥", "壬申", "丙午"]), &reference(), &[]);
    let wealth_plan = wealth
        .preferences
        .plans
        .iter()
        .find(|plan| plan.rule_id == "strong-hidden-officer-visible-wealth")
        .expect("wealth plan should use hidden officer and visible wealth");
    assert!(wealth_plan
        .evidence_ids
        .contains(&"fact-hidden-3-3".to_string()));
    assert!(wealth_plan
        .evidence_ids
        .contains(&"fact-stem-4".to_string()));
}

#[test]
fn special_guards_include_both_specialized_versions_and_follow_child_exception() {
    let result = analyze(&chart(["壬子", "辛亥", "壬子", "辛亥"]), &reference(), &[]);

    assert!(result.pattern.special_review.required);
    assert_eq!(
        result
            .pattern
            .special_review
            .specialized_use_versions
            .iter()
            .map(|item| item.source_date.as_str())
            .collect::<Vec<_>>(),
        vec!["2026-07-08", "2026-09-02"]
    );
    assert!(result.sources.iter().any(|source| {
        source.date == "2026-07-08"
            && source.rule_ids == vec!["specialized-use-version".to_string()]
    }));
    assert!(result
        .pattern
        .special_review
        .follow_child_peer_exception
        .contains("比劫"));
    assert!(result
        .preferences
        .plans
        .iter()
        .all(|plan| plan.condition.contains("若按普通格論")));
}

#[test]
fn rootless_special_review_still_requires_no_visible_branch_support() {
    let result = analyze(&chart(["丙子", "丙子", "甲午", "丙戌"]), &reference(), &[]);

    assert!(result.facts.roots.is_empty());
    assert_eq!(result.strength.support_stem_count, 0);
    assert_eq!(result.strength.support_branch_count, 2);
    assert!(!result.pattern.special_review.required);
}

#[test]
fn bridge_contract_has_five_paths_and_winter_never_invents_fire_direction() {
    let result = analyze(&chart(["甲午", "丙子", "庚寅", "丁亥"]), &reference(), &[]);

    assert_eq!(result.preferences.bridge_paths.len(), 5);
    assert!(result
        .preferences
        .bridge_paths
        .iter()
        .all(|path| path.path.len() == 3));
    let fire_is_use_or_favorable = result.preferences.plans.iter().any(|plan| {
        plan.use_directions
            .iter()
            .chain(plan.favorable_directions.iter())
            .any(|direction| direction.element == "火")
    });
    assert!(!fire_is_use_or_favorable);
    assert!(result
        .preferences
        .seasonal_compatibility
        .iter()
        .all(|note| note.plan_rule_id.is_some()
            && note.element.as_deref() == Some("火")
            && note.status != "conditional_compatible"));
    assert!(result
        .preferences
        .seasonal_compatibility
        .iter()
        .any(|note| note.status == "conflict"));

    let missing_bridge = analyze(&chart(["壬子", "丙午", "壬子", "丙午"]), &reference(), &[]);
    let water_to_fire = missing_bridge
        .preferences
        .bridge_paths
        .iter()
        .find(|path| path.conflict_from == "水" && path.conflict_to == "火")
        .expect("water-controls-fire bridge should be one of the five mappings");
    assert_eq!(water_to_fire.intermediary, "木");
    assert_eq!(water_to_fire.status, "missing_intermediary");
    assert_eq!(water_to_fire.actual_nodes, vec!["水", "火"]);
    assert_eq!(water_to_fire.missing_nodes, vec!["木"]);
}
